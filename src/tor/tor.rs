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

use std::collections::{BTreeMap, BTreeSet};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use arti_client::config::pt::TransportConfigBuilder;
use lazy_static::lazy_static;
use futures::task::SpawnExt;

use arti_client::{TorClient, TorClientConfig};
use arti_client::config::{BridgeConfigBuilder, TorClientConfigBuilder};
use fs_mistrust::Mistrust;
use grin_util::secp::SecretKey;
use ed25519_dalek::hazmat::ExpandedSecretKey;
use curve25519_dalek::digest::Digest;
use sha2::Sha512;
use tokio::time::{sleep, sleep_until};
use tor_config::CfgPath;
use tor_rtcompat::tokio::TokioNativeTlsRuntime;
use tor_rtcompat::Runtime;
use tor_hsrproxy::OnionServiceReverseProxy;
use tor_hsrproxy::config::{Encapsulation, ProxyAction, ProxyPattern, ProxyRule, TargetAddr, ProxyConfigBuilder};
use tor_hsservice::config::OnionServiceConfigBuilder;
use tor_hsservice::{HsIdKeypairSpecifier, HsIdPublicKeySpecifier, HsNickname, RunningOnionService};
use tor_keymgr::{ArtiNativeKeystore, KeyMgrBuilder, KeystoreSelector};
use tor_llcrypto::pk::ed25519::ExpandedKeypair;
use tor_hscrypto::pk::{HsIdKey, HsIdKeypair};
use arti_hyper::ArtiHttpConnector;
use hyper::{Body, Uri};
use tls_api::{TlsConnector as TlsConnectorTrait, TlsConnectorBuilder};

// On aarch64-apple-darwin targets there is an issue with the native and rustls
// tls implementation so this makes it fall back to the openssl variant.
//
// https://gitlab.torproject.org/tpo/core/arti/-/issues/715
#[cfg(not(all(target_vendor = "apple", target_arch = "aarch64")))]
use tls_api_native_tls::TlsConnector;
#[cfg(all(target_vendor = "apple", target_arch = "aarch64"))]
use tls_api_openssl::TlsConnector;

use crate::tor::TorConfig;

lazy_static! {
    /// Static thread-aware state of [`Node`] to be updated from separate thread.
    static ref TOR_SERVER_STATE: Arc<Tor> = Arc::new(Tor::default());
}

/// Tor server to use as SOCKS proxy for requests and to launch Onion services.
pub struct Tor {
    /// Mapping of running Onion services identifiers to proxy.
    running_services: Arc<RwLock<BTreeMap<String,
        (Arc<RunningOnionService>, Arc<OnionServiceReverseProxy>)>>>,
    /// Starting Onion services identifiers.
    starting_services: Arc<RwLock<BTreeSet<String>>>,
    /// Failed Onion services identifiers.
    failed_services: Arc<RwLock<BTreeSet<String>>>
}

impl Default for Tor {
    fn default() -> Self {
        Self {
            running_services: Arc::new(RwLock::new(BTreeMap::new())),
            starting_services: Arc::new(RwLock::new(BTreeSet::new())),
            failed_services: Arc::new(RwLock::new(BTreeSet::new()))
        }
    }
}

impl Tor {
    async fn build_client(runtime: TokioNativeTlsRuntime)
                          -> (TorClient<TokioNativeTlsRuntime>, TorClientConfig) {
        // Create Tor client config.
        let mut builder =
            TorClientConfigBuilder::from_directories(TorConfig::state_path(),
                                                     TorConfig::cache_path());
        // Setup bridges.
        let bridge = TorConfig::get_bridge();
        if let Some(b) = bridge {
            match b {
                super::TorBridge::Snowflake(path) => Self::build_snowflake(&mut builder, path),
                super::TorBridge::Obfs4(path) => Self::build_obfs4(&mut builder, path),
            }
        }
        // Setup address filter.
        builder.address_filter().allow_onion_addrs(true);
        // Create connected Tor client from config.
        let config = builder.build().unwrap();
        (TorClient::with_runtime(runtime)
             .config(config.clone())
             .create_bootstrapped()
             .await
             .unwrap(), config)
    }

    /// Send post request using Tor.
    pub async fn post(body: String, url: String) -> Option<String> {
        // Create client.
        let runtime = TokioNativeTlsRuntime::create().unwrap();
        let (client, _) = Self::build_client(runtime).await;
        // Create http tor-powered client to post data.
        let tls_connector = TlsConnector::builder().unwrap().build().unwrap();
        let tor_connector = ArtiHttpConnector::new(client, tls_connector);
        let http = hyper::Client::builder().build::<_, Body>(tor_connector);
        // Create request.
        let req = hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(url)
            .body(Body::from(body))
            .unwrap();
        // Send request.
        let mut resp = None;
        match http.request(req).await {
            Ok(r) => {
                match hyper::body::to_bytes(r).await {
                    Ok(raw) => {
                        resp = Some(String::from_utf8_lossy(&raw).to_string())
                    },
                    Err(_) => {},
                }
            },
            Err(_) => {},
        }
        resp
    }

    /// Check if Onion service is starting.
    pub fn is_service_starting(id: &String) -> bool {
        let r_services = TOR_SERVER_STATE.starting_services.read().unwrap();
        r_services.contains(id)
    }

    /// Check if Onion service is running.
    pub fn is_service_running(id: &String) -> bool {
        let r_services = TOR_SERVER_STATE.running_services.read().unwrap();
        r_services.contains_key(id)
    }

    /// Check if Onion service failed on start.
    pub fn is_service_failed(id: &String) -> bool {
        let r_services = TOR_SERVER_STATE.failed_services.read().unwrap();
        r_services.contains(id)
    }

    /// Stop running Onion service.
    pub fn stop_service(id: &String) {
        let mut w_services = TOR_SERVER_STATE.running_services.write().unwrap();
        if let Some((svc, proxy)) = w_services.remove(id) {
            proxy.shutdown();
            drop(svc);
        }
    }

    /// Start Onion service from listening local port and [`SecretKey`].
    pub fn start_service(port: u16, key: SecretKey, id: &String) {
        // Check if service is already running.
        if Self::is_service_running(id) {
            return;
        } else {
            // Save starting service.
            let mut w_services = TOR_SERVER_STATE.starting_services.write().unwrap();
            w_services.insert(id.clone());
            // Remove service from failed.
            let mut w_services = TOR_SERVER_STATE.failed_services.write().unwrap();
            w_services.remove(id);
        }

        let service_id = id.clone();
        let runtime = TokioNativeTlsRuntime::create().unwrap();
        let runtime_client = runtime.clone();
        runtime.spawn(async move {
            let (client, config) = Self::build_client(runtime_client.clone()).await;
            // Add service key to keystore.
            let hs_nickname = HsNickname::new(service_id.clone()).unwrap();
            Self::add_service_key(config.fs_mistrust(), &key, &hs_nickname);
            // Launch Onion service.
            let service_config = OnionServiceConfigBuilder::default()
                .nickname(hs_nickname.clone())
                .build()
                .unwrap();
            let (service, request) = client.launch_onion_service(service_config).unwrap();

            // Check service availability.
            let service_check = service.clone();
            std::thread::spawn(move || {
                let runtime = TokioNativeTlsRuntime::create().unwrap();
                let runtime_client = runtime.clone();
                runtime.spawn(async move {
                    loop {
                        // Create client.
                        let (client, _) = Self::build_client(runtime_client.clone()).await;

                        // Create http tor-powered client to ping service.
                        let tls_connector = TlsConnector::builder().unwrap().build().unwrap();
                        let tor_connector = ArtiHttpConnector::new(client, tls_connector);
                        let http = hyper::Client::builder().build::<_, Body>(tor_connector);

                        let url = format!("http://{}", service_check.onion_name().unwrap().to_string());
                        match http.get(Uri::from_str(url.as_str()).unwrap()).await {
                            Ok(_) => {
                                // Remove service from starting.
                                let mut w_services = TOR_SERVER_STATE.starting_services.write().unwrap();
                                w_services.remove(&service_id);

                                println!("success");
                            },
                            Err(e) => {
                                // Put service to starting.
                                let mut w_services = TOR_SERVER_STATE.starting_services.write().unwrap();
                                w_services.insert(service_id.clone());

                                println!("err: {}", e);
                            },
                        }
                        sleep(Duration::from_millis(5000)).await;
                    }
                }).unwrap();
            });

            // Launch service proxy.
            let addr = SocketAddr::new(IpAddr::from(Ipv4Addr::LOCALHOST), port);
            tokio::spawn(
                Self::run_service_proxy(addr, client, service.clone(), request, hs_nickname.clone())
            ).await.unwrap();
        }).unwrap();
    }

    /// Launch Onion service proxy.
    async fn run_service_proxy<R, S>(
        addr: SocketAddr,
        client: TorClient<R>,
        service: Arc<RunningOnionService>,
        request: S,
        nickname: HsNickname
    )
        where
            R: Runtime,
            S: futures::Stream<Item = tor_hsservice::RendRequest> + Unpin + Send + 'static,
    {
        let id = nickname.to_string();
        let runtime = client.runtime().clone();

        // Setup proxy to forward request from Tor address to local address.
        let proxy_rule = ProxyRule::new(
            ProxyPattern::one_port(80).unwrap(),
            ProxyAction::Forward(Encapsulation::Simple, TargetAddr::Inet(addr)),
        );
        let mut proxy_cfg_builder = ProxyConfigBuilder::default();
        proxy_cfg_builder.set_proxy_ports(vec![proxy_rule]);
        let proxy = OnionServiceReverseProxy::new(proxy_cfg_builder.build().unwrap());

        // Save running service.
        let mut w_services = TOR_SERVER_STATE.running_services.write().unwrap();
        w_services.insert(id.clone(), (service.clone(), proxy.clone()));

        // Start proxy for launched service.
        client
            .runtime()
            .spawn(async move {
                match proxy
                    .handle_requests(runtime, nickname.clone(), request)
                    .await {
                    Ok(()) => {
                        // Remove service from running.
                        let mut w_services =
                            TOR_SERVER_STATE.running_services.write().unwrap();
                        w_services.remove(&id);
                    }
                    Err(e) => {
                        // Remove service from running.
                        let mut w_services =
                            TOR_SERVER_STATE.running_services.write().unwrap();
                        w_services.remove(&id);
                        // Save failed service.
                        let mut w_services =
                            TOR_SERVER_STATE.failed_services.write().unwrap();
                        w_services.insert(id);
                    }
                }
            }).unwrap();
    }

    /// Save Onion service key to keystore.
    fn add_service_key(mistrust: &Mistrust, key: &SecretKey, hs_nickname: &HsNickname) {
        let arti_store =
            ArtiNativeKeystore::from_path_and_mistrust(TorConfig::keystore_path(), &mistrust)
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

    fn build_snowflake(builder: &mut TorClientConfigBuilder, bin_path: String) {
        let mut bridges = vec![];
        // Add a single bridge to the list of bridges, from a bridge line.
        // This line comes from https://gitlab.torproject.org/tpo/applications/tor-browser-build/-/blob/main/projects/common/bridges_list.snowflake.txt
        // this is a real bridge line you can use as-is, after making sure it's still up to date with
        // above link.
        const BRIDGE1_LINE : &str = "Bridge snowflake 192.0.2.3:80 2B280B23E1107BB62ABFC40DDCC8824814F80A72 fingerprint=2B280B23E1107BB62ABFC40DDCC8824814F80A72 url=https://snowflake-broker.torproject.net.global.prod.fastly.net/ front=cdn.sstatic.net ice=stun:stun.l.google.com:19302,stun:stun.antisip.com:3478,stun:stun.bluesip.net:3478,stun:stun.dus.net:3478,stun:stun.epygi.com:3478,stun:stun.sonetel.com:3478,stun:stun.uls.co.za:3478,stun:stun.voipgate.com:3478,stun:stun.voys.nl:3478 utls-imitate=hellorandomizedalpn";
        let bridge_1: BridgeConfigBuilder = BRIDGE1_LINE.parse().unwrap();
        bridges.push(bridge_1);

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
        bridges.push(bridge2_builder);

        // Set bridges to client config builder.
        builder.bridges().set_bridges(bridges);

        // Now configure an snowflake transport. (Requires the "pt-client" feature)
        let mut transport = TransportConfigBuilder::default();
        transport
            .protocols(vec!["snowflake".parse().unwrap()])
            // this might be named differently on some systems, this should work on Debian, but Archlinux is known to use `snowflake-pt-client` instead for instance.
            .path(CfgPath::new("snowflake-client".into()))
            .run_on_startup(true);
        builder.bridges().set_transports(vec![transport]);
    }

    fn build_obfs4(builder: &mut TorClientConfigBuilder, bin_path: String) {
        // This bridge line is made up for demonstration, and won't work.
        const BRIDGE1_LINE : &str = "Bridge obfs4 192.0.2.55:38114 316E643333645F6D79216558614D3931657A5F5F cert=YXJlIGZyZXF1ZW50bHkgZnVsbCBvZiBsaXR0bGUgbWVzc2FnZXMgeW91IGNhbiBmaW5kLg iat-mode=0";
        let bridge_1: BridgeConfigBuilder = BRIDGE1_LINE.parse().unwrap();
        // This is where we pass `BRIDGE1_LINE` into the BridgeConfigBuilder.
        builder.bridges().bridges().push(bridge_1);

        // Add a second bridge, built by hand.  This way is harder.
        // This bridge is made up for demonstration, and won't work.
        let mut bridge2_builder = BridgeConfigBuilder::default();
        bridge2_builder
            .transport("obfs4")
            .push_setting("iat-mode", "1")
            .push_setting(
                "cert",
                "YnV0IHNvbWV0aW1lcyB0aGV5IGFyZSByYW5kb20u8x9aQG/0cIIcx0ItBcTqiSXotQne+Q"
            );
        bridge2_builder.set_addrs(vec!["198.51.100.25:443".parse().unwrap()]);
        bridge2_builder.set_ids(vec!["7DD62766BF2052432051D7B7E08A22F7E34A4543".parse().unwrap()]);
        // Now insert the second bridge into our config builder.
        builder.bridges().bridges().push(bridge2_builder);

        // Now configure an obfs4 transport. (Requires the "pt-client" feature)
        let mut transport = TransportConfigBuilder::default();
        transport
            .protocols(vec!["obfs4".parse().unwrap()])
            // Specify either the name or the absolute path of pluggable transport client binary, this
            // may differ from system to system.
            .path(CfgPath::new("/usr/bin/obfs4proxy".into()))
            .run_on_startup(true);
        builder.bridges().transports().push(transport);
    }
}