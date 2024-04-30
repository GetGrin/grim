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
use std::sync::{Arc, RwLock};
use futures::executor::block_on;
use lazy_static::lazy_static;
use futures::task::SpawnExt;

use arti_client::{TorClient, TorClientConfig};
use arti_client::config::TorClientConfigBuilder;
use fs_mistrust::Mistrust;
use grin_util::secp::SecretKey;
use ed25519_dalek::hazmat::ExpandedSecretKey;
use curve25519_dalek::digest::Digest;
use sha2::Sha512;
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
use futures::TryFutureExt;
use hyper::Body;
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
    /// [`TorClient`] used for connections with configuration.
    client: Arc<RwLock<(TorClient<TokioNativeTlsRuntime>, TorClientConfig)>>,

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
        // Create Tor client config.
        let mut builder =
            TorClientConfigBuilder::from_directories(TorConfig::state_path(),
                                                     TorConfig::cache_path());
        builder.address_filter().allow_onion_addrs(true);

        // Create connected Tor client from config.
        let runtime = TokioNativeTlsRuntime::create().unwrap();
        let config = builder.build().unwrap();
        let client = TorClient::with_runtime(runtime)
            .config(config.clone())
            .create_unbootstrapped()
            .unwrap();
        Self {
            client: Arc::new(RwLock::new((client, config))),
            running_services: Arc::new(RwLock::new(BTreeMap::new())),
            starting_services: Arc::new(RwLock::new(BTreeSet::new())),
            failed_services: Arc::new(RwLock::new(BTreeSet::new()))
        }
    }
}

impl Tor {
    /// Send post request using Tor.
    pub async fn post(body: String, url: String) -> Option<String> {
        // Bootstrap client.
        let client_config = TOR_SERVER_STATE.client.read().unwrap();
        let client = client_config.0.clone();
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
        let client_config = TOR_SERVER_STATE.client.read().unwrap();
        let client = client_config.0.clone();
        let config = client_config.1.clone();
        client.clone().runtime().spawn(async move {
            // Add service key to keystore.
            let hs_nickname = HsNickname::new(service_id.clone()).unwrap();
            Self::add_service_key(config.fs_mistrust(), &key, &hs_nickname);

            // Bootstrap client and launch Onion service.
            client.bootstrap().await.unwrap();
            let service_config = OnionServiceConfigBuilder::default()
                .nickname(hs_nickname.clone())
                .build()
                .unwrap();
            let (service, request) = client.launch_onion_service(service_config).unwrap();

            // Launch service proxy.
            let addr = SocketAddr::new(IpAddr::from(Ipv4Addr::LOCALHOST), port);
            tokio::spawn(
                Self::run_service_proxy(addr, client, service.clone(), request, hs_nickname.clone())
            ).await.unwrap();

            println!(
                "Onion service {} launched at: {}",
                hs_nickname,
                service.onion_name().unwrap().to_string()
            );
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
 
        // Remove service from starting.
        let mut w_services = TOR_SERVER_STATE.starting_services.write().unwrap();
        w_services.remove(&id);

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

                        println!("Onion service {} stopped.", nickname);
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

                        eprintln!("Onion service {} exited with an error: {}", nickname, e);
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
}