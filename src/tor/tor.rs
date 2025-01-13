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

use arti_client::config::pt::TransportConfigBuilder;
use futures::task::SpawnExt;
use lazy_static::lazy_static;
use std::collections::{BTreeMap, BTreeSet};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use std::{fs, thread};
use std::time::Duration;

use arti_client::config::{CfgPath, TorClientConfigBuilder};
use arti_client::{TorClient, TorClientConfig};
use curve25519_dalek::digest::Digest;
use ed25519_dalek::hazmat::ExpandedSecretKey;
use fs_mistrust::Mistrust;
use grin_util::secp::SecretKey;
use hyper::{Body, Uri};
use parking_lot::RwLock;
use sha2::Sha512;
use tls_api_native_tls::TlsConnector;
use tls_api::{TlsConnector as TlsConnectorTrait, TlsConnectorBuilder};
use tokio::time::sleep;
use tor_hscrypto::pk::{HsIdKey, HsIdKeypair};
use tor_hsrproxy::config::{
    Encapsulation, ProxyAction, ProxyConfigBuilder, ProxyPattern, ProxyRule, TargetAddr,
};
use tor_hsrproxy::OnionServiceReverseProxy;
use tor_hsservice::config::OnionServiceConfigBuilder;
use tor_hsservice::{
    HsIdKeypairSpecifier, HsIdPublicKeySpecifier, HsNickname, RunningOnionService,
};
use tor_keymgr::{ArtiNativeKeystore, KeyMgrBuilder, KeystoreSelector};
use tor_llcrypto::pk::ed25519::ExpandedKeypair;
use tor_rtcompat::tokio::TokioNativeTlsRuntime;
use tor_rtcompat::Runtime;

use crate::tor::http::ArtiHttpConnector;
use crate::tor::TorConfig;

lazy_static! {
    /// Static thread-aware state of [`Node`] to be updated from separate thread.
    static ref TOR_SERVER_STATE: Arc<Tor> = Arc::new(Tor::default());
}

/// Tor server to use as SOCKS proxy for requests and to launch Onion services.
pub struct Tor {
    /// Tor client and config.
    client_config: Arc<RwLock<(TorClient<TokioNativeTlsRuntime>, TorClientConfig)>>,
    /// Mapping of running Onion services identifiers to proxy.
    running_services:
        Arc<RwLock<BTreeMap<String, (Arc<RunningOnionService>, Arc<OnionServiceReverseProxy>)>>>,
    /// Starting Onion services identifiers.
    starting_services: Arc<RwLock<BTreeSet<String>>>,
    /// Failed Onion services identifiers.
    failed_services: Arc<RwLock<BTreeSet<String>>>,
    /// Checking Onion services identifiers.
    checking_services: Arc<RwLock<BTreeSet<String>>>,
}

impl Default for Tor {
    fn default() -> Self {
        // Cleanup keys, state and cache on start.
        fs::remove_dir_all(TorConfig::keystore_path()).unwrap_or_default();
        fs::remove_dir_all(TorConfig::state_path()).unwrap_or_default();
        fs::remove_dir_all(TorConfig::cache_path()).unwrap_or_default();
        // Create Tor client.
        let runtime = TokioNativeTlsRuntime::create().unwrap();
        let config = Self::build_config();
        let client = TorClient::with_runtime(runtime)
            .config(config.clone())
            .create_unbootstrapped().unwrap();
        Self {
            running_services: Arc::new(RwLock::new(BTreeMap::new())),
            starting_services: Arc::new(RwLock::new(BTreeSet::new())),
            failed_services: Arc::new(RwLock::new(BTreeSet::new())),
            checking_services: Arc::new(RwLock::new(BTreeSet::new())),
            client_config: Arc::new(RwLock::new((client, config))),
        }
    }
}

impl Tor {
    /// Create Tor client configuration.
    fn build_config() -> TorClientConfig {
        // Create Tor client config.
        let mut builder = TorClientConfigBuilder::from_directories(
            TorConfig::state_path(),
            TorConfig::cache_path(),
        );
        builder.address_filter().allow_onion_addrs(true);
        // Setup bridges.
        let bridge = TorConfig::get_bridge();
        if let Some(b) = bridge {
            match b {
                super::TorBridge::Snowflake(path, conn) => {
                    Self::build_snowflake(&mut builder, path, conn)
                }
                super::TorBridge::Obfs4(path, conn) => Self::build_obfs4(&mut builder, path, conn),
            }
        }
        // Create config.
        let config = builder.build().unwrap();
        config
    }

    /// Recreate Tor client with configuration.
    pub fn rebuild_client() {
        let config = Self::build_config();
        let r_client = TOR_SERVER_STATE.client_config.read();
        r_client
            .0
            .reconfigure(&config, tor_config::Reconfigure::AllOrNothing)
            .unwrap();
    }

    /// Send post request using Tor.
    pub async fn post(body: String, url: String) -> Option<String> {
        // Bootstrap client.
        let (client, _) = Self::client_config();
        client.bootstrap().await.unwrap();
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
            Ok(r) => match hyper::body::to_bytes(r).await {
                Ok(raw) => resp = Some(String::from_utf8_lossy(&raw).to_string()),
                Err(_) => {}
            },
            Err(_) => {}
        }
        resp
    }

    fn client_config() -> (TorClient<TokioNativeTlsRuntime>, TorClientConfig) {
        let r_client_config = TOR_SERVER_STATE.client_config.read();
        r_client_config.clone()
    }

    /// Check if Onion service is starting.
    pub fn is_service_starting(id: &String) -> bool {
        let r_services = TOR_SERVER_STATE.starting_services.read();
        r_services.contains(id)
    }

    /// Check if Onion service is running.
    pub fn is_service_running(id: &String) -> bool {
        let r_services = TOR_SERVER_STATE.running_services.read();
        r_services.contains_key(id)
    }

    /// Check if Onion service failed on start.
    pub fn is_service_failed(id: &String) -> bool {
        let r_services = TOR_SERVER_STATE.failed_services.read();
        r_services.contains(id)
    }

    /// Check if Onion service is checking.
    pub fn is_service_checking(id: &String) -> bool {
        let r_services = TOR_SERVER_STATE.checking_services.read();
        r_services.contains(id)
    }

    // Restart Onion service.
    pub fn restart_service(port: u16, key: SecretKey, id: &String) {
        Self::stop_service(id);
        Self::rebuild_client();
        Self::start_service(port, key, id)
    }

    /// Stop running Onion service.
    pub fn stop_service(id: &String) {
        let mut w_services = TOR_SERVER_STATE.running_services.write();
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
            let mut w_services = TOR_SERVER_STATE.starting_services.write();
            w_services.insert(id.clone());
            // Remove service from failed.
            let mut w_services = TOR_SERVER_STATE.failed_services.write();
            w_services.remove(id);
        }

        let service_id = id.clone();
        thread::spawn(move || {
            let on_error = |service_id: String| {
                // Remove service from starting.
                let mut w_services = TOR_SERVER_STATE.starting_services.write();
                w_services.remove(&service_id);
                // Save failed service.
                let mut w_services = TOR_SERVER_STATE.failed_services.write();
                w_services.insert(service_id);
            };

            let (client, config) = Self::client_config();
            let client_thread = client.clone();
            client
                .runtime()
                .spawn(async move {
                    // Add service key to keystore.
                    let hs_nickname = HsNickname::new(service_id.clone()).unwrap();
                    if let Err(_) = Self::add_service_key(config.fs_mistrust(), &key, &hs_nickname)
                    {
                        on_error(service_id);
                        return;
                    }
                    // Bootstrap client.
                    client_thread.bootstrap().await.unwrap();
                    // Launch Onion service.
                    let service_config = OnionServiceConfigBuilder::default()
                        .nickname(hs_nickname.clone())
                        .build()
                        .unwrap();
                    if let Ok((service, request)) =
                        client_thread.launch_onion_service(service_config)
                    {
                        // Launch service proxy.
                        let addr = SocketAddr::new(IpAddr::from(Ipv4Addr::LOCALHOST), port);
                        tokio::spawn(Self::run_service_proxy(
                            addr,
                            client_thread.clone(),
                            service.clone(),
                            request,
                            hs_nickname.clone(),
                        ))
                        .await
                        .unwrap();

                        // Check service availability if not checking.
                        if Self::is_service_checking(&service_id) {
                            return;
                        }
                        let client_check = client_thread.clone();
                        let url = format!("http://{}/", service.onion_name().unwrap().to_string());
                        thread::spawn(move || {
                            // Wait 1 second to start.
                            thread::sleep(Duration::from_millis(1000));
                            let runtime = client_thread.runtime();
                            // Put service to checking.
                            {
                                let mut w_services = TOR_SERVER_STATE.checking_services.write();
                                w_services.insert(service_id.clone());
                            }
                            runtime
                                .spawn(async move {
                                    let tls_connector =
                                        TlsConnector::builder().unwrap().build().unwrap();
                                    let tor_connector =
                                        ArtiHttpConnector::new(client_check.clone(), tls_connector);
                                    let http =
                                        hyper::Client::builder().build::<_, Body>(tor_connector);

                                    const MAX_ERRORS: i32 = 3;
                                    let mut errors_count = 0;
                                    loop {
                                        if !Self::is_service_running(&service_id) {
                                            // Remove service from checking.
                                            let mut w_services =
                                                TOR_SERVER_STATE.checking_services.write();
                                            w_services.remove(&service_id);
                                            break;
                                        }
                                        // Send request.
                                        let duration = match http
                                            .get(Uri::from_str(url.clone().as_str()).unwrap())
                                            .await
                                        {
                                            Ok(_) => {
                                                // Remove service from starting.
                                                let mut w_services =
                                                    TOR_SERVER_STATE.starting_services.write();
                                                w_services.remove(&service_id);
                                                // Check again after 50 seconds.
                                                Duration::from_millis(50000)
                                            }
                                            Err(_) => {
                                                // Restart service on 3rd error.
                                                errors_count += 1;
                                                if errors_count == MAX_ERRORS {
                                                    errors_count = 0;
                                                    let key = key.clone();
                                                    let service_id = service_id.clone();
                                                    thread::spawn(move || {
                                                        Self::restart_service(
                                                            port,
                                                            key,
                                                            &service_id,
                                                        );
                                                    });
                                                }
                                                Duration::from_millis(5000)
                                            }
                                        };
                                        // Wait to check service again.
                                        sleep(duration).await;
                                    }
                                })
                                .unwrap();
                        });
                        return;
                    }
                    on_error(service_id);
                })
                .unwrap();
        });
    }

    /// Launch Onion service proxy.
    async fn run_service_proxy<R, S>(
        addr: SocketAddr,
        client: TorClient<R>,
        service: Arc<RunningOnionService>,
        request: S,
        nickname: HsNickname,
    ) where
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
        let mut w_services = TOR_SERVER_STATE.running_services.write();
        w_services.insert(id.clone(), (service.clone(), proxy.clone()));

        // Start proxy for launched service.
        client
            .runtime()
            .spawn(async move {
                match proxy
                    .handle_requests(runtime, nickname.clone(), request)
                    .await
                {
                    Ok(()) => {
                        // Remove service from running.
                        let mut w_services = TOR_SERVER_STATE.running_services.write();
                        w_services.remove(&id);
                    }
                    Err(_) => {
                        // Remove service from running.
                        let mut w_services = TOR_SERVER_STATE.running_services.write();
                        w_services.remove(&id);
                        // Save failed service.
                        let mut w_services = TOR_SERVER_STATE.failed_services.write();
                        w_services.insert(id);
                    }
                }
            })
            .unwrap();
    }

    /// Save Onion service key to keystore.
    fn add_service_key(
        mistrust: &Mistrust,
        key: &SecretKey,
        hs_nickname: &HsNickname,
    ) -> tor_keymgr::Result<()> {
        let arti_store =
            ArtiNativeKeystore::from_path_and_mistrust(TorConfig::keystore_path(), mistrust)?;

        let key_manager = KeyMgrBuilder::default()
            .primary_store(Box::new(arti_store))
            .build()
            .unwrap();

        let expanded_sk =
            ExpandedSecretKey::from_bytes(Sha512::default().chain_update(key).finalize().as_ref());

        let mut sk_bytes = [0_u8; 64];
        sk_bytes[0..32].copy_from_slice(&expanded_sk.scalar.to_bytes());
        sk_bytes[32..64].copy_from_slice(&expanded_sk.hash_prefix);
        let expanded_kp = ExpandedKeypair::from_secret_key_bytes(sk_bytes).unwrap();

        key_manager.insert(
            HsIdKey::from(expanded_kp.public().clone()),
            &HsIdPublicKeySpecifier::new(hs_nickname.clone()),
            KeystoreSelector::Primary,
            true
        )?;

        key_manager.insert(
            HsIdKeypair::from(expanded_kp),
            &HsIdKeypairSpecifier::new(hs_nickname.clone()),
            KeystoreSelector::Primary,
            true
        )?;
        Ok(())
    }

    fn build_snowflake(builder: &mut TorClientConfigBuilder, bin_path: String, conn_line: String) {
        let bridge_line = format!("Bridge {}", conn_line);
        if let Ok(bridge) = bridge_line.parse() {
            builder.bridges().bridges().push(bridge);
        }

        // Now configure a snowflake transport. (Requires the "pt-client" feature)
        let mut transport = TransportConfigBuilder::default();
        transport
            .protocols(vec!["snowflake".parse().unwrap()])
            // this might be named differently on some systems, this should work on Debian,
            // but Archlinux is known to use `snowflake-pt-client` instead for instance.
            .path(CfgPath::new(bin_path.into()))
            .run_on_startup(true);
        builder.bridges().set_transports(vec![transport]);
    }

    fn build_obfs4(builder: &mut TorClientConfigBuilder, bin_path: String, conn_line: String) {
        let bridge_line = format!("Bridge {}", conn_line);
        if let Ok(bridge) = bridge_line.parse() {
            builder.bridges().bridges().push(bridge);
        }

        // Now configure an obfs4 transport. (Requires the "pt-client" feature)
        let mut transport = TransportConfigBuilder::default();
        transport
            .protocols(vec!["obfs4".parse().unwrap()])
            // Specify either the name or the absolute path of pluggable transport client binary,
            // this may differ from system to system.
            .path(CfgPath::new(bin_path.into()))
            .run_on_startup(true);
        builder.bridges().transports().push(transport);
    }
}
