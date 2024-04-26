// Copyright 2024 The Grim Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use lazy_static::lazy_static;
use futures::task::SpawnExt;
use tokio::task::JoinHandle;
use anyhow::Result;
use tokio::time::sleep;

use arti::socks::run_socks_proxy;
use arti_client::{TorClient, TorClientConfig};
use arti_client::config::pt::TransportConfigBuilder;
use arti_client::config::{BridgeConfigBuilder, TorClientConfigBuilder};
use fs_mistrust::Mistrust;
use grin_util::secp::SecretKey;
use grin_wallet_util::OnionV3Address;
use ed25519_dalek::hazmat::ExpandedSecretKey;
use curve25519_dalek::digest::Digest;
use sha2::Sha512;
use tor_config::{CfgPath, Listen};
use tor_rtcompat::{BlockOn, PreferredRuntime, Runtime};
use tor_hsrproxy::OnionServiceReverseProxy;
use tor_hsrproxy::config::{Encapsulation, ProxyAction, ProxyPattern, ProxyRule, TargetAddr, ProxyConfigBuilder};
use tor_hsservice::config::OnionServiceConfigBuilder;
use tor_hsservice::{HsIdKeypairSpecifier, HsIdPublicKeySpecifier, HsNickname};
use tor_keymgr::{ArtiNativeKeystore, KeyMgrBuilder, KeystoreSelector};
use tor_llcrypto::pk::ed25519::ExpandedKeypair;
use tor_hscrypto::pk::{HsIdKey, HsIdKeypair};

use crate::tor::TorServerConfig;

lazy_static! {
    /// Static thread-aware state of [`Node`] to be updated from separate thread.
    static ref TOR_SERVER_STATE: Arc<TorServer> = Arc::new(TorServer::default());
}

/// Tor server to use as SOCKS proxy for requests and to launch Onion services.
pub struct TorServer {
    /// Running Tor client.
    client: Arc<RwLock<Option<TorClient<PreferredRuntime>>>>,
    /// Running Tor client configuration.
    config: Arc<RwLock<Option<TorClientConfig>>>,

    /// Flag to check if server is running.
    running: AtomicBool,
    /// Flag to check if server is starting.
    starting: AtomicBool,
    /// Flag to check if server needs to stop.
    stopping: AtomicBool,

    /// Flag to check if error happened.
    error: AtomicBool,

    /// Mapping of running Onion services identifiers to proxy.
    running_services: Arc<RwLock<HashMap<String, Arc<OnionServiceReverseProxy>>>>
}

impl Default for TorServer {
    fn default() -> Self {
        Self {
            running: AtomicBool::new(false),
            starting: AtomicBool::new(false),
            stopping: AtomicBool::new(false),
            error: AtomicBool::new(false),
            client: Arc::new(RwLock::new(None)),
            running_services: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(None)),
        }
    }
}

impl TorServer {
    /// Check if server is running.
    pub fn is_running() -> bool {
        TOR_SERVER_STATE.running.load(Ordering::Relaxed)
    }

    /// Check if server is running.
    pub fn is_starting() -> bool {
        TOR_SERVER_STATE.starting.load(Ordering::Relaxed)
    }

    /// Check if server is stopping.
    pub fn is_stopping() -> bool {
        TOR_SERVER_STATE.stopping.load(Ordering::Relaxed)
    }

    /// Check if server has error.
    pub fn has_error() -> bool {
        TOR_SERVER_STATE.error.load(Ordering::Relaxed)
    }

    /// Stop the server.
    pub fn stop() {
        TOR_SERVER_STATE.stopping.store(true, Ordering::Relaxed);
    }

    /// Start or restart the server if already running.
    pub fn start() {
        if Self::is_running() {
            Self::stop();
        }

        thread::spawn(|| {
            while Self::is_stopping() {
                thread::sleep(Duration::from_millis(1000));
            }
            TOR_SERVER_STATE.starting.store(true, Ordering::Relaxed);
            TOR_SERVER_STATE.error.store(false, Ordering::Relaxed);

            // Check if Tor client is already running.
            if TOR_SERVER_STATE.client.read().unwrap().is_some() {
                let r_client = TOR_SERVER_STATE.client.read().unwrap();
                let client = r_client.as_ref().unwrap().clone();
                let runtime = client.runtime().clone();
                let _ = runtime.clone().block_on(Self::launch_socks_proxy(runtime, client));
            } else {
                // Create Tor client config to connect.
                let mut builder =
                    TorClientConfigBuilder::from_directories(TorServerConfig::state_path(),
                                                             TorServerConfig::cache_path());
                builder.address_filter().allow_onion_addrs(true);

                // Setup Snowflake bridges.
                Self::setup_bridges(&mut builder);

                // Create Tor client from config.
                if let Ok(config) = builder.build() {
                    let mut w_config = TOR_SERVER_STATE.config.write().unwrap();
                    *w_config = Some(config.clone());

                    // Restart server on connection timeout.
                    thread::spawn(|| {
                        thread::sleep(Duration::from_millis(30000));
                        let r_client = TOR_SERVER_STATE.client.read().unwrap();
                        if r_client.is_none() {
                            Self::start();
                        }
                    });
                    // Create Tor client.
                    let runtime = PreferredRuntime::current().unwrap();
                    match TorClient::with_runtime(runtime.clone())
                        .config(config)
                        .bootstrap_behavior(arti_client::BootstrapBehavior::OnDemand)
                        .create_unbootstrapped() {
                        Ok(tor_client) => {
                            let mut w_client = TOR_SERVER_STATE.client.write().unwrap();
                            if w_client.is_some() {
                                return;
                            }
                            *w_client = Some(tor_client.clone());
                            let _ = runtime.clone().block_on(
                                // Launch SOCKS proxy server.
                                Self::launch_socks_proxy(runtime, tor_client)
                            );
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                            TOR_SERVER_STATE.starting.store(false, Ordering::Relaxed);
                            TOR_SERVER_STATE.error.store(true, Ordering::Relaxed);
                        }
                    }
                } else {
                    TOR_SERVER_STATE.starting.store(false, Ordering::Relaxed);
                    TOR_SERVER_STATE.error.store(true, Ordering::Relaxed);
                }
            }
        });
    }

    /// Launch SOCKS proxy server to send connections.
    async fn launch_socks_proxy<R: Runtime>(runtime: R, tor_client: TorClient<R>) -> Result<()> {
        let proxy_handle: JoinHandle<Result<()>> = tokio::spawn(
            run_socks_proxy(
                runtime,
                tor_client,
                Listen::new_localhost(TorServerConfig::socks_port()),
            )
        );

        // Setup server state flags.
        TOR_SERVER_STATE.starting.store(false, Ordering::Relaxed);
        TOR_SERVER_STATE.running.store(true, Ordering::Relaxed);

        loop {
            if Self::is_stopping() || proxy_handle.is_finished() {
                proxy_handle.abort();
                TOR_SERVER_STATE.stopping.store(false, Ordering::Relaxed);
                TOR_SERVER_STATE.running.store(false, Ordering::Relaxed);
                return Ok(());
            }
            sleep(Duration::from_millis(3000)).await;
        }
    }

    /// Check if Onion service is running.
    pub fn is_service_running(id: &String) -> bool {
        let r_services = TOR_SERVER_STATE.running_services.read().unwrap();
        r_services.contains_key(id)
    }

    /// Stop running Onion service.
    pub fn stop_service(id: &String) {
        let mut w_services = TOR_SERVER_STATE.running_services.write().unwrap();
        if let Some(proxy) = w_services.remove(id) {
            proxy.shutdown();
        }
    }

    /// Run Onion service from listening local address, secret key and identifier.
    pub fn run_service(addr: SocketAddr, key: SecretKey, id: &String) {
        // Check if service is already running.
        if Self::is_service_running(id) {
            return;
        }

        let hs_nickname = HsNickname::new(id.clone()).unwrap();
        let service_config = OnionServiceConfigBuilder::default()
            .nickname(hs_nickname.clone())
            .build()
            .unwrap();
        let r_client = TOR_SERVER_STATE.client.read().unwrap();
        let client = r_client.clone().unwrap();

        // Add service key to keystore.
        let r_config = TOR_SERVER_STATE.config.read().unwrap();
        let config = r_config.clone().unwrap();
        Self::add_service_key(config.fs_mistrust(), &key, &hs_nickname);

        // Launch Onion service.
        let (_, request) = client.launch_onion_service(service_config).unwrap();

        // Setup proxy to forward request from Tor address to local address.
        let proxy_rule = ProxyRule::new(
            ProxyPattern::one_port(80).unwrap(),
            ProxyAction::Forward(Encapsulation::Simple, TargetAddr::Inet(addr)),
        );
        let mut proxy_cfg_builder = ProxyConfigBuilder::default();
        proxy_cfg_builder.set_proxy_ports(vec![proxy_rule]);
        let proxy = OnionServiceReverseProxy::new(proxy_cfg_builder.build().unwrap());

        // Launch proxy at client runtime.
        let proxy_service = proxy.clone();
        let runtime = client.runtime().clone();
        let nickname = hs_nickname.clone();
        client
            .runtime()
            .spawn(async move {
                // Launch proxy for launched service.
                match proxy_service.handle_requests(runtime, nickname.clone(), request).await {
                    Ok(()) => {
                        eprintln!("Onion service {} stopped.", nickname);
                    }
                    Err(e) => {
                        eprintln!("Onion service {} exited with an error: {}", nickname, e);
                    }
                }
            }).unwrap();

        // Save running service.
        let mut w_services = TOR_SERVER_STATE.running_services.write().unwrap();
        w_services.insert(id.clone(), proxy);

        let onion_addr = OnionV3Address::from_private(&key.0).unwrap();
        eprintln!("Onion service {} launched at {}", hs_nickname, onion_addr.to_ov3_str());
    }

    /// Add Onion service key to keystore.
    fn add_service_key(mistrust: &Mistrust, key: &SecretKey, hs_nickname: &HsNickname) {
        let mut client_config_builder = TorClientConfigBuilder::from_directories(
            TorServerConfig::state_path(),
            TorServerConfig::cache_path()
        );
        client_config_builder
            .address_filter()
            .allow_onion_addrs(true);
        let arti_store =
            ArtiNativeKeystore::from_path_and_mistrust(TorServerConfig::keystore_path(), &mistrust)
                .unwrap();

        let key_manager = KeyMgrBuilder::default()
            .default_store(Box::new(arti_store))
            .build()
            .unwrap();

        let expanded_sk = ExpandedSecretKey::from_bytes(
            Sha512::default()
                .chain_update(key)
                .finalize()
                .as_ref(),
        );

        let mut sk_bytes = [0_u8; 64];
        sk_bytes[0..32].copy_from_slice(&expanded_sk.scalar.to_bytes());
        sk_bytes[32..64].copy_from_slice(&expanded_sk.hash_prefix);
        let expanded_kp = ExpandedKeypair::from_secret_key_bytes(sk_bytes).unwrap();

        key_manager
            .insert(
                HsIdKey::from(expanded_kp.public().clone()),
                &HsIdPublicKeySpecifier::new(hs_nickname.clone()),
                KeystoreSelector::Default,
            )
            .unwrap();

        key_manager
            .insert(
                HsIdKeypair::from(expanded_kp),
                &HsIdKeypairSpecifier::new(hs_nickname.clone()),
                KeystoreSelector::Default,
            )
            .unwrap();
    }

    /// Setup Tor Snowflake bridges.
    fn setup_bridges(builder: &mut TorClientConfigBuilder) {
        // Add a single bridge to the list of bridges, from a bridge line.
        // This line comes from https://gitlab.torproject.org/tpo/applications/tor-browser-build/-/blob/main/projects/common/bridges_list.snowflake.txt
        // this is a real bridge line you can use as-is, after making sure it's still up to date with
        // above link.
        const BRIDGE1_LINE: &str = "Bridge snowflake 192.0.2.3:80 2B280B23E1107BB62ABFC40DDCC8824814F80A72 fingerprint=2B280B23E1107BB62ABFC40DDCC8824814F80A72 url=https://snowflake-broker.torproject.net.global.prod.fastly.net/ front=cdn.sstatic.net ice=stun:stun.l.google.com:19302,stun:stun.antisip.com:3478,stun:stun.bluesip.net:3478,stun:stun.dus.net:3478,stun:stun.epygi.com:3478,stun:stun.sonetel.com:3478,stun:stun.uls.co.za:3478,stun:stun.voipgate.com:3478,stun:stun.voys.nl:3478 utls-imitate=hellorandomizedalpn";
        let bridge_1: BridgeConfigBuilder = BRIDGE1_LINE.parse().unwrap();
        builder.bridges().bridges().push(bridge_1);

        // Add a second bridge, built by hand. We use the 2nd bridge line from above, but modify some
        // parameters to use AMP Cache instead of Fastly as a signaling channel. The difference in
        // configuration is detailed in
        // https://gitlab.torproject.org/tpo/anti-censorship/pluggable-transports/snowflake/-/tree/main/client#amp-cache
        let mut bridge2_builder = BridgeConfigBuilder::default();
        bridge2_builder
            .transport("snowflake")
            .push_setting(
                "fingerprint",
                "8838024498816A039FCBBAB14E6F40A0843051FA"
            )
            .push_setting("url", "https://snowflake-broker.torproject.net/")
            .push_setting("ampcache", "https://cdn.ampproject.org/")
            .push_setting("front", "www.google.com")
            .push_setting(
                "ice",
                "stun:stun.l.google.com:19302,stun:stun.antisip.com:3478,stun:stun.bluesip.net:3478,stun:stun.dus.net:3478,stun:stun.epygi.com:3478,stun:stun.sonetel.net:3478,stun:stun.uls.co.za:3478,stun:stun.voipgate.com:3478,stun:stun.voys.nl:3478",
            )
            .push_setting("utls-imitate", "hellorandomizedalpn");
        bridge2_builder.set_addrs(vec!["192.0.2.4:80".parse().unwrap()]);
        bridge2_builder.set_ids(vec!["8838024498816A039FCBBAB14E6F40A0843051FA".parse().unwrap()]);
        // Now insert the second bridge into our config builder.
        builder.bridges().bridges().push(bridge2_builder);

        // Now configure an snowflake transport. (Requires the "pt-client" feature)
        let mut transport = TransportConfigBuilder::default();
        transport
            .protocols(vec!["snowflake".parse().unwrap()])
            // this might be named differently on some systems, this should work on Debian,
            // but Archlinux is known to use `snowflake-pt-client` instead for instance.
            .path(CfgPath::new("snowflake-client".into()))
            .run_on_startup(true);
        builder.bridges().transports().push(transport);
    }
}