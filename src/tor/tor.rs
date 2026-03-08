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
use arti_client::config::{CfgPath, TorClientConfigBuilder};
use arti_client::{TorClient, TorClientConfig};
use curve25519_dalek::digest::Digest;
use ed25519_dalek::hazmat::ExpandedSecretKey;
use fs_mistrust::Mistrust;
use futures::task::SpawnExt;
use grin_util::secp::SecretKey;
use http_body_util::{BodyExt, Full};
use lazy_static::lazy_static;
use parking_lot::RwLock;
use sha2::Sha512;
use std::collections::{BTreeMap, BTreeSet};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use std::{fs, thread};
use bytes::Bytes;
use log::error;
use safelog::DisplayRedacted;
use tls_api::{TlsConnector as TlsConnectorTrait, TlsConnectorBuilder};
use tls_api_native_tls::TlsConnector;
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

use crate::http::HttpClient;
use crate::tor::http::ArtiHttpConnector;
use crate::tor::{TorBridge, TorConfig, TorProxy};

lazy_static! {
    /// Static thread-aware state of [`Node`] to be updated from separate thread.
    static ref TOR_SERVER_STATE: Arc<Tor> = Arc::new(Tor::default());
}

/// Tor server to use as SOCKS proxy for requests and to launch Onion services.
pub struct Tor {
    /// Tor client and config.
    client_config: Arc<RwLock<(TorClient<TokioNativeTlsRuntime>, TorClientConfig)>>,
    /// Client to check services availability.
    check_client: Arc<RwLock<
        Option<hyper_tor::Client<ArtiHttpConnector<TokioNativeTlsRuntime, TlsConnector>>>
    >>,
    /// Mapping of running Onion services identifiers to proxy.
    run: Arc<RwLock<
        BTreeMap<String, (u16, SecretKey, Arc<RunningOnionService>, Arc<OnionServiceReverseProxy>)>
    >>,
    /// Starting Onion services identifiers.
    start: Arc<RwLock<BTreeSet<String>>>,
    /// Failed Onion services identifiers.
    fail: Arc<RwLock<BTreeSet<String>>>,
    /// Checking Onion services identifiers.
    check: Arc<RwLock<BTreeSet<String>>>,
}

impl Default for Tor {
    fn default() -> Self {
        // Extract webtunnel bridge binary.
        if !fs::exists(TorConfig::webtunnel_path()).unwrap_or(true) {
            let webtunnel = include_bytes!(concat!(env!("OUT_DIR"), "/tor/webtunnel"));
            if !webtunnel.is_empty() {
                fs::write(TorConfig::webtunnel_path(), webtunnel).unwrap_or_default();
            }
        }
        // Create Tor client.
        let runtime = TokioNativeTlsRuntime::create().unwrap();
        let config = Self::build_config(true);
        let client = TorClient::with_runtime(runtime)
            .config(config.clone())
            .create_unbootstrapped()
            .unwrap();
        Self {
            client_config: Arc::new(RwLock::new((client, config))),
            check_client: Arc::new(RwLock::new(None)),
            run: Arc::new(RwLock::new(BTreeMap::new())),
            start: Arc::new(RwLock::new(BTreeSet::new())),
            fail: Arc::new(RwLock::new(BTreeSet::new())),
            check: Arc::new(RwLock::new(BTreeSet::new())),
        }
    }
}

impl Tor {
    /// Create Tor client configuration.
    fn build_config(clean: bool) -> TorClientConfig {
        // Cleanup keys, state and cache.
        if clean {
            fs::remove_dir_all(TorConfig::keystore_path()).unwrap_or_default();
            fs::remove_dir_all(TorConfig::state_path()).unwrap_or_default();
            fs::remove_dir_all(TorConfig::cache_path()).unwrap_or_default();
        }
        // Create Tor client config.
        let mut builder = TorClientConfigBuilder::from_directories(
            TorConfig::state_path(),
            TorConfig::cache_path(),
        );
        builder.address_filter().allow_onion_addrs(true);
        // Setup bridges.
        let bridge = TorConfig::get_bridge();
        if let Some(b) = bridge {
            Self::build_bridge(&mut builder, b);
        }
        // Create config.
        let config = builder.build().unwrap();
        config
    }

    /// Send post request using Tor.
    pub async fn post(body: String, url: String) -> Option<String> {
        if let Some(proxy) = TorConfig::get_proxy() {
            let req: hyper::Request<Full<Bytes>> = hyper::Request::builder()
                .method(hyper::Method::POST)
                .uri(url)
                .body(Full::from(body))
                .unwrap();
            let res = match proxy {
                TorProxy::SOCKS5(url) => {
                    HttpClient::send_socks_proxy(url, req).await
                }
                TorProxy::HTTP(url) => {
                    HttpClient::send_http_proxy(url, req).await
                }
            };
            match res {
                Ok(res) => {
                    match res.into_body().collect().await {
                        Ok(r) => {
                            let body = r.to_bytes().into();
                            match String::from_utf8(body) {
                                Ok(r) => Some(r),
                                Err(e) => {
                                    error!("Tor: POST with proxy, response to string error: {}", e);
                                    None
                                }
                            }
                        }
                        Err(e) => {
                            error!("Tor: POST with proxy, response parse error: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    error!("Tor: POST failed with proxy: {}", e);
                    None
                }
            }
        } else {
            if let Some(b) = TorConfig::get_bridge() {
                if !fs::exists(b.binary_path()).unwrap() {
                    return None;
                }
            }
            // Bootstrap client.
            let (client, _) = Self::client_config();
            client.bootstrap().await.unwrap_or_default();
            // Create http tor-powered client to post data.
            let tls_conn = TlsConnector::builder().unwrap().build().unwrap();
            let conn = ArtiHttpConnector::new(client, tls_conn);
            let http = hyper_tor::Client::builder().build::<_, hyper_tor::Body>(conn);
            // Create request.
            let req = hyper_tor::Request::builder()
                .method(hyper_tor::Method::POST)
                .uri(url)
                .body(hyper_tor::Body::from(body))
                .unwrap();
            // Send request.
            let mut resp = None;
            match http.request(req).await {
                Ok(r) => match hyper_tor::body::to_bytes(r).await {
                    Ok(raw) => resp = Some(String::from_utf8_lossy(&raw).to_string()),
                    Err(e) => {
                        error!("Tor: POST response parse error: {}", e);
                    }
                },
                Err(e) => {
                    error!("Tor: POST failed: {}", e);
                }
            }
            resp
        }
    }

    fn client_config() -> (TorClient<TokioNativeTlsRuntime>, TorClientConfig) {
        let r_client_config = TOR_SERVER_STATE.client_config.read();
        r_client_config.clone()
    }

    /// Check if Onion service is starting.
    pub fn is_service_starting(id: &String) -> bool {
        let r_services = TOR_SERVER_STATE.start.read();
        r_services.contains(id)
    }

    /// Check if Onion service is running.
    pub fn is_service_running(id: &String) -> bool {
        let r_services = TOR_SERVER_STATE.run.read();
        r_services.contains_key(id)
    }

    /// Check if Onion service failed on start.
    pub fn is_service_failed(id: &String) -> bool {
        let r_services = TOR_SERVER_STATE.fail.read();
        r_services.contains(id)
    }

    /// Check if Onion service is checking.
    pub fn is_service_checking(id: &String) -> bool {
        let r_services = TOR_SERVER_STATE.check.read();
        r_services.contains(id)
    }

    /// Restart running Onion services.
    pub fn restart_services() {
        // Stop all services saving port key for relaunch.
        let service_ids = {
            let r_services = TOR_SERVER_STATE.run.read().clone();
            r_services.keys().map(|s| s.to_string()).collect::<Vec<String>>()
        };
        let mut services: BTreeMap<String, (u16, SecretKey)> = BTreeMap::new();
        for id in service_ids.clone() {
            if let Some(res) = Self::stop_service(&id) {
                services.insert(id, res);
            }
        }
        // Reconfigure client.
        let config = Self::build_config(false);
        let r_client = TOR_SERVER_STATE.client_config.read();
        r_client.0.reconfigure(&config, tor_config::Reconfigure::WarnOnFailures).unwrap();
        // Start services.
        for id in services.keys() {
            let (port, key) = services.get(id).unwrap();
            Self::start_service(port.clone(), key.clone(), &id);
        }
    }

    /// Stop running Onion service returning port and key.
    pub fn stop_service(id: &String) -> Option<(u16, SecretKey)> {
        {
            // Remove service from starting.
            let mut w_services = TOR_SERVER_STATE.start.write();
            w_services.remove(id);
        }
        let mut w_services = TOR_SERVER_STATE.run.write();
        if let Some((port, key, svc, proxy)) = w_services.remove(id) {
            proxy.shutdown();
            drop(proxy);
            drop(svc);
            return Some((port, key));
        }
        None
    }

    /// Start Onion service from listening local port and [`SecretKey`].
    pub fn start_service(port: u16, key: SecretKey, id: &String) {
        // Check if service is already running.
        if Self::is_service_running(id) || Self::is_service_starting(id) {
            return;
        } else {
            // Save starting service.
            let mut w_services = TOR_SERVER_STATE.start.write();
            w_services.insert(id.clone());
            // Remove service from failed.
            let mut w_services = TOR_SERVER_STATE.fail.write();
            w_services.remove(id);
        }

        let service_id = id.clone();
        thread::spawn(move || {
            let on_error = |service_id: String| {
                // Remove service from starting.
                let mut w_services = TOR_SERVER_STATE.start.write();
                w_services.remove(&service_id);
                // Save failed service.
                let mut w_services = TOR_SERVER_STATE.fail.write();
                w_services.insert(service_id);
            };

            // Check bridge binary existence and permissions.
            if let Some(bridge) = TorConfig::get_bridge() {
                if !fs::exists(bridge.binary_path()).unwrap() {
                    on_error(service_id);
                    return;
                }
                // Add execute permission for Unix.
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(bridge.binary_path())
                        .unwrap()
                        .permissions();
                    let mode = perms.mode() | 0o100;
                    perms.set_mode(mode);
                    fs::set_permissions(bridge.binary_path(), perms).unwrap_or_default();
                }
            }

            let (client, config) = Self::client_config();
            client
                .runtime()
                .spawn(async move {
                    // Add service key to keystore.
                    let hs = HsNickname::new(service_id.clone()).unwrap();
                    if let Err(_) = Self::add_service_key(config.fs_mistrust(), &key, &hs) {
                        on_error(service_id);
                        return;
                    }
                    let (c, _) = Self::client_config();
                    // Bootstrap client.
                    if c.bootstrap_status().as_frac() == 0.0 {
                        if let Err(_) = c.bootstrap().await {
                            on_error(service_id);
                            return;
                        }
                        // Launch first time after delay.
                        thread::sleep(Duration::from_millis(10000));
                    }
                    // Launch Onion service.
                    let service_config = OnionServiceConfigBuilder::default()
                        .nickname(hs.clone())
                        .build()
                        .unwrap();
                    if let Ok(res) = c.launch_onion_service(service_config) {
                        if let Some((service, request)) = res {
                            let onion_addr = service.onion_address();
                            // Launch service proxy.
                            let addr = SocketAddr::new(IpAddr::from(Ipv4Addr::LOCALHOST), port);
                            let proxy = tokio::spawn(Self::run_service_proxy(
                                addr,
                                request,
                                hs.clone(),
                            )).await.unwrap();
                            // Save running service.
                            let mut w_services = TOR_SERVER_STATE.run.write();
                            let id = service_id.clone();
                            w_services.insert(id, (port, key.clone(), service, proxy));
                            // Check service availability.
                            let addr = onion_addr.unwrap().display_unredacted().to_string();
                            let url = format!("http://{}/", addr);
                            if !Self::is_service_checking(&service_id) {
                                Self::check_service(service_id, url, port, key)
                            }
                            return;
                        }
                    }
                    on_error(service_id);
                })
                .unwrap();
        });
    }

    /// Check service availability.
    fn check_service(service_id: String,
                     url: String,
                     port: u16,
                     key: SecretKey) {
        {
            let mut w_services = TOR_SERVER_STATE.check.write();
            w_services.insert(service_id.clone());
        }
        let (client, _) = Self::client_config();
        thread::spawn(move || {
            // Wait 5 sec before check.
            thread::sleep(Duration::from_millis(5000));
            // Request client setup.
            if TOR_SERVER_STATE.check_client.read().is_none() {
                let (client_check, _) = Self::client_config();
                let tls_conn = TlsConnector::builder().unwrap().build().unwrap();
                let conn = ArtiHttpConnector::new(client_check, tls_conn);
                let http = hyper_tor::Client::builder().build::<_, hyper_tor::Body>(conn);
                let mut w_client = TOR_SERVER_STATE.check_client.write();
                *w_client = Some(http);
            }
            let r_client = TOR_SERVER_STATE.check_client.read();
            let http = r_client.clone().unwrap();
            let runtime = client.runtime();
            runtime
                .spawn(async move {
                    const MAX_ERRORS: i32 = 16;
                    let mut errors_count = 0;
                    loop {
                        // Check if service is running.
                        fn is_running(service_id: &String) -> bool {
                            let running = Tor::is_service_running(service_id);
                            if !running {
                                // Remove service from checking.
                                let mut w_services =
                                    TOR_SERVER_STATE.check.write();
                                w_services.remove(service_id);
                            }
                            running
                        }
                        if !is_running(&service_id) {
                            break;
                        }
                        let duration = {
                            let mut on_error = |service_id: &String| -> bool {
                                if !is_running(service_id) {
                                    return true;
                                }
                                // Restart service after maximum amount of errors.
                                errors_count += 1;
                                if errors_count == MAX_ERRORS {
                                    // Remove service from checking.
                                    let mut w_services =
                                        TOR_SERVER_STATE.check.write();
                                    w_services.remove(service_id);
                                    // Remove service from starting.
                                    let mut w_services = TOR_SERVER_STATE.start.write();
                                    w_services.remove(service_id);
                                    // Restart service.
                                    let key = key.clone();
                                    let id = service_id.clone();
                                    thread::spawn(move || {
                                        Self::stop_service(&id);
                                        Self::start_service(port, key, &id);
                                    });
                                    return true;
                                }
                                false
                            };
                            // Send request.
                            let uri = hyper_tor::Uri::from_str(url.clone().as_str()).unwrap();
                            let check = http.get(uri);
                            // Check with timeout of 20s.
                            match tokio::time::timeout(Duration::from_millis(20000), check).await {
                                Ok(resp) => {
                                    match resp {
                                        Ok(_) => {
                                            if !is_running(&service_id) {
                                                break;
                                            }
                                            // Remove service from starting.
                                            let mut w_services = TOR_SERVER_STATE.start.write();
                                            w_services.remove(&service_id);
                                            errors_count = 0;
                                            // Check again after 20s.
                                            Duration::from_millis(20000)
                                        }
                                        Err(e) => {
                                            if on_error(&service_id) {
                                                break;
                                            }
                                            error!("Tor check failed: {} for {}, errors: {}/{}",
                                                e, service_id, errors_count, MAX_ERRORS);
                                            // Check again after 5s.
                                            Duration::from_millis(5000)
                                        }
                                    }
                                }
                                Err(_) => {
                                    if on_error(&service_id) {
                                        break;
                                    }
                                    error!("Tor check for {} timeout, errors: {}/{}",
                                        &service_id, errors_count, MAX_ERRORS);
                                    // Check again after 5s.
                                    Duration::from_millis(5000)
                                }
                            }
                        };
                        // Wait to check service again.
                        thread::sleep(duration);
                    }
                })
                .unwrap();
        });
    }

    /// Launch Onion service proxy.
    async fn run_service_proxy<S>(
        addr: SocketAddr,
        request: S,
        nickname: HsNickname,
    ) -> Arc<OnionServiceReverseProxy> where
        S: futures::Stream<Item = tor_hsservice::RendRequest> + Unpin + Send + 'static,
    {
        let id = nickname.to_string();
        let (client, _) = Self::client_config();
        let runtime = client.runtime().clone();

        // Setup proxy to forward request from Tor address to local address.
        let proxy_rule = ProxyRule::new(
            ProxyPattern::one_port(80).unwrap(),
            ProxyAction::Forward(Encapsulation::Simple, TargetAddr::Inet(addr)),
        );
        let mut proxy_cfg_builder = ProxyConfigBuilder::default();
        proxy_cfg_builder.set_proxy_ports(vec![proxy_rule]);
        let proxy = OnionServiceReverseProxy::new(proxy_cfg_builder.build().unwrap());

        // Remove service from failed.
        let mut w_services = TOR_SERVER_STATE.fail.write();
        w_services.remove(&id);

        // Start proxy for launched service.
        let p = proxy.clone();
        client
            .runtime()
            .spawn(async move {
                match p.handle_requests(runtime, nickname.clone(), request).await {
                    Ok(()) => {
                        // Remove service from running.
                        let mut w_services = TOR_SERVER_STATE.run.write();
                        w_services.remove(&id);
                    }
                    Err(_) => {
                        if Self::is_service_running(&id) {
                            // Remove service from running.
                            let mut w_services = TOR_SERVER_STATE.run.write();
                            w_services.remove(&id);
                            // Save failed service.
                            let mut w_services = TOR_SERVER_STATE.fail.write();
                            w_services.insert(id);
                        }
                    }
                }
            })
            .unwrap();
        proxy
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

    fn build_bridge(builder: &mut TorClientConfigBuilder, bridge: TorBridge) {
        let bridge_line = format!("Bridge {}", bridge.connection_line());
        if let Ok(bridge) = bridge_line.parse() {
            builder.bridges().bridges().push(bridge);
        }

        // Now configure bridge transport. (Requires the "pt-client" feature)
        let mut transport = TransportConfigBuilder::default();
        transport
            .protocols(vec![bridge.protocol_name().parse().unwrap()])
            // Specify either the name or the absolute path of pluggable transport client binary,
            // this may differ from system to system.
            .path(CfgPath::new(bridge.binary_path().into()))
            .run_on_startup(true);
        builder.bridges().transports().push(transport);
    }
}
