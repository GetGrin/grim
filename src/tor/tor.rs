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

use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use lazy_static::lazy_static;
use arti::socks::run_socks_proxy;
use arti_client::{TorClient, TorClientConfig};
use arti_client::config::pt::{TransportConfigBuilder};
use arti_client::config::{BridgeConfigBuilder, TorClientConfigBuilder, StorageConfigBuilder};
use futures::task::SpawnExt;
use tokio::task::JoinHandle;
use anyhow::{Result};
use tokio::time::sleep;
use tor_config::{CfgPath, Listen};
use tor_rtcompat::{BlockOn, Runtime};
use tor_rtcompat::tokio::TokioNativeTlsRuntime;

use crate::tor::TorServerConfig;

lazy_static! {
    /// Static thread-aware state of [`Node`] to be updated from separate thread.
    static ref TOR_SERVER_STATE: Arc<TorServer> = Arc::new(TorServer::default());
}

/// Tor SOCKS proxy server.
pub struct TorServer {
    /// Flag to check if server is running.
    running: AtomicBool,
    /// Flag to check if server is starting.
    starting: AtomicBool,
    /// Flag to check if server needs to stop.
    stopping: AtomicBool,
    /// Flag to check if error happened.
    error: AtomicBool,
    /// Tor client to use for proxy.
    client: Arc<RwLock<Option<TorClient<TokioNativeTlsRuntime>>>>
}

impl Default for TorServer {
    fn default() -> Self {
        Self {
            running: AtomicBool::new(false),
            starting: AtomicBool::new(false),
            stopping: AtomicBool::new(false),
            error: AtomicBool::new(false),
            client: Arc::new(RwLock::new(None)),
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
                let runtime = TokioNativeTlsRuntime::create().unwrap();
                let _ = runtime.clone().block_on(
                    Self::launch_socks_proxy(runtime, r_client.as_ref().unwrap().clone())
                );
            } else {
                // Create Tor client config to connect.
                let mut builder = TorClientConfig::builder();

                // Setup Snowflake bridges.
                Self::setup_bridges(&mut builder);

                // Create Tor client from config.
                if let Ok(config) = builder.build() {
                    // Restart server on connection timeout.
                    thread::spawn(|| {
                        thread::sleep(Duration::from_millis(30000));
                        let r_client = TOR_SERVER_STATE.client.read().unwrap();
                        if r_client.is_none() {
                            Self::start();
                        }
                    });
                    // Create Tor client.
                    let runtime = TokioNativeTlsRuntime::create().unwrap();
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

    /// Launch SOCKS proxy server.
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