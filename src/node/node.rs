// Copyright 2023 The Grim Developers
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

use std::{fs, thread};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use lazy_static::lazy_static;
use parking_lot::RwLock;
use futures::channel::oneshot;

use grin_chain::SyncStatus;
use grin_core::global;
use grin_core::global::ChainTypes;
use grin_p2p::msg::PeerAddrs;
use grin_p2p::Seeding;
use grin_servers::{Server, ServerStats, StratumServerConfig, StratumStats};
use grin_servers::common::types::Error;

use crate::node::{NodeConfig, NodeError, PeersConfig};
use crate::node::stratum::{StratumStopState, StratumServer};

lazy_static! {
    /// Static thread-aware state of [`Node`] to be updated from separate thread.
    static ref NODE_STATE: Arc<Node> = Arc::new(Node::default());
}

/// Provides [`Server`] control, holds current status and statistics.
pub struct Node {
    /// Node [`Server`] statistics information.
    stats: Arc<RwLock<Option<ServerStats>>>,

    /// [`StratumServer`] statistics information.
    stratum_stats: Arc<grin_util::RwLock<StratumStats>>,
    /// Flag to start [`StratumServer`].
    start_stratum_needed: AtomicBool,
    /// State to stop [`StratumServer`] from outside.
    stratum_stop_state: Arc<StratumStopState>,

    /// Indicator if node [`Server`] is starting.
    starting: AtomicBool,
    /// Flag to stop the [`Server`] and start it again.
    restart_needed: AtomicBool,
    /// Flag to stop the [`Server`].
    stop_needed: AtomicBool,
    /// Flag to check if app exit is needed after [`Server`] stop.
    exit_after_stop: AtomicBool,
    /// Flag to reset peers data and restart the [`Server`].
    reset_peers: AtomicBool,

    /// An error occurred on [`Server`] start.
    error: Arc<RwLock<Option<Error>>>
}

impl Default for Node {
    fn default() -> Self {
        Self {
            stats: Arc::new(RwLock::new(None)),
            stratum_stats: Arc::new(grin_util::RwLock::new(StratumStats::default())),
            stratum_stop_state: Arc::new(StratumStopState::default()),
            starting: AtomicBool::new(false),
            restart_needed: AtomicBool::new(false),
            stop_needed: AtomicBool::new(false),
            exit_after_stop: AtomicBool::new(false),
            start_stratum_needed: AtomicBool::new(false),
            error: Arc::new(RwLock::new(None)),
            reset_peers: AtomicBool::new(false),
        }
    }
}

impl Node {
    /// Delay for thread to update the stats.
    pub const STATS_UPDATE_DELAY: Duration = Duration::from_millis(1000);

    /// Default Mainnet DNS Seeds
    pub const MAINNET_DNS_SEEDS: &'static[&'static str] = &[
        "mainnet.seed.grin.lesceller.com",
        "grinseed.revcore.net",
        "mainnet-seed.grinnode.live",
        "mainnet.grin.punksec.de",
        "grinnode.30-r.com",
        "grincoin.org"
    ];

    /// Stop the [`Server`] and setup exit flag after if needed.
    pub fn stop(exit_after_stop: bool) {
        NODE_STATE.stop_needed.store(true, Ordering::Relaxed);
        NODE_STATE.exit_after_stop.store(exit_after_stop, Ordering::Relaxed);
    }

    /// Request to start the [`Node`].
    pub fn start() {
        if !Self::is_running() {
            Self::start_server_thread();
        }
    }

    /// Request to restart the [`Node`].
    pub fn restart() {
        if Self::is_running() {
            NODE_STATE.restart_needed.store(true, Ordering::Relaxed);
        } else {
            Node::start();
        }
    }

    /// Request to start [`StratumServer`].
    pub fn start_stratum() {
        NODE_STATE.start_stratum_needed.store(true, Ordering::Relaxed);
    }

    /// Check if [`StratumServer`] is starting.
    pub fn is_stratum_starting() -> bool {
        NODE_STATE.start_stratum_needed.load(Ordering::Relaxed)
    }

    /// Get [`StratumServer`] statistics.
    pub fn get_stratum_stats() -> StratumStats {
        NODE_STATE.stratum_stats.read().clone()
    }

    /// Stop [`StratumServer`].
    pub fn stop_stratum() {
        NODE_STATE.stratum_stop_state.stop()
    }

    /// Check if [`StratumServer`] is stopping.
    pub fn is_stratum_stopping() -> bool {
        NODE_STATE.stratum_stop_state.is_stopped()
    }

    /// Check if [`Node`] is starting.
    pub fn is_starting() -> bool {
        NODE_STATE.starting.load(Ordering::Relaxed)
    }

    /// Check if [`Node`] is running.
    pub fn is_running() -> bool {
        Self::get_sync_status().is_some()
    }

    /// Check if [`Node`] is stopping.
    pub fn is_stopping() -> bool {
        NODE_STATE.stop_needed.load(Ordering::Relaxed)
    }

    /// Check if [`Node`] is restarting.
    pub fn is_restarting() -> bool {
        NODE_STATE.restart_needed.load(Ordering::Relaxed) || Self::reset_peers_needed()
    }

    /// Check if reset of [`Server`] peers is needed.
    fn reset_peers_needed() -> bool {
        NODE_STATE.reset_peers.load(Ordering::Relaxed)
    }

    /// Get node [`Server`] statistics.
    pub fn get_stats() -> Option<ServerStats> {
        NODE_STATE.stats.read().clone()
    }

    /// Check if [`Server`] is not syncing (disabled or just running after synchronization).
    pub fn not_syncing() -> bool {
        return match Node::get_sync_status() {
            None => true,
            Some(ss) => ss == SyncStatus::NoSync
        };
    }

    /// Get synchronization status, empty when [`Server`] is not running.
    pub fn get_sync_status() -> Option<SyncStatus> {
        // Return Shutdown status when node is stopping.
        if Self::is_stopping() {
            return Some(SyncStatus::Shutdown);
        }

        // Return Initial status when node is starting or restarting or peers are deleting.
        if Self::is_starting() || Self::is_restarting() {
            return Some(SyncStatus::Initial);
        }

        let stats = Self::get_stats();
        // Return sync status when server is running (stats are not empty).
        if stats.is_some() {
            return Some(stats.as_ref().unwrap().sync_status);
        }
        None
    }

    /// Get [`Server`] error.
    pub fn get_error() -> Option<NodeError> {
        let r_err = NODE_STATE.error.read();
        if r_err.is_some() {
            let e = r_err.as_ref().unwrap();
            // Setup a flag to show an error to clean up data.
            let store_err = match e {
                Error::Store(_) => true,
                Error::Chain(_) => true,
                _ => false
            };
            if store_err {
                return Some(NodeError::Storage);
            }

            // Setup a flag to show P2P or API server error.
            let p2p_api_err = match e {
                Error::P2P(_) => Some(NodeError::P2P),
                Error::API(_) => Some(NodeError::API),
                _ => None
            };
            if p2p_api_err.is_some() {
                return p2p_api_err;
            }

            // Setup a flag to show configuration error.
            let config_err = match e {
                Error::Configuration(_) => true,
                _ => false
            };
            return if config_err {
                Some(NodeError::Configuration)
            } else {
                Some(NodeError::Unknown)
            }
        }
        None
    }

    /// Start the [`Server`] at separate thread to update state with stats and handle statuses.
    fn start_server_thread() {
        thread::spawn(move || {
            NODE_STATE.starting.store(true, Ordering::Relaxed);
            // Start the server.
            match start_node_server() {
                Ok(mut server) => {
                    let mut first_start = true;
                    loop {
                        // Restart server if request or peers clean up is needed
                        if Self::is_restarting() {
                            server.stop();
                            // Wait server after stop.
                            thread::sleep(Duration::from_millis(5000));
                            // Reset peers data if requested.
                            if Self::reset_peers_needed() {
                                Node::reset_peers(true);
                            }
                            // Reset stratum stats.
                            {
                                let mut w_stratum_stats = NODE_STATE.stratum_stats.write();
                                *w_stratum_stats = StratumStats::default();
                            }
                            // Create new server.
                            match start_node_server() {
                                Ok(s) => {
                                    server = s;
                                    NODE_STATE.restart_needed.store(false, Ordering::Relaxed);
                                }
                                Err(e) => {
                                    // Setup an error.
                                    {
                                        let mut w_err = NODE_STATE.error.write();
                                        *w_err = Some(e);
                                    }
                                    // Reset server state.
                                    Self::reset_server_state(true);
                                    break;
                                }
                            }
                        } else if Self::is_stopping() {
                            // Stop the server.
                            server.stop();
                            // Clean stats and statuses.
                            Self::reset_server_state(false);
                            break;
                        }

                        // Start stratum mining server if requested.
                        let stratum_start_requested = Self::is_stratum_starting();
                        if stratum_start_requested {
                            let (s_ip, s_port) = NodeConfig::get_stratum_address();
                            if NodeConfig::is_stratum_port_available(&s_ip, &s_port) {
                                let stratum_config = server
                                    .config
                                    .stratum_mining_config
                                    .clone()
                                    .unwrap();
                                start_stratum_mining_server(&server, stratum_config);
                            }
                        }

                        // Update server stats.
                        if let Ok(stats) = server.get_server_stats() {
                            {
                                let mut w_stats = NODE_STATE.stats.write();
                                *w_stats = Some(stats.clone());
                            }

                            if first_start {
                                NODE_STATE.starting.store(false, Ordering::Relaxed);
                                first_start = false;
                            }
                        }

                        // Reset stratum server start flag.
                        if stratum_start_requested && NODE_STATE.stratum_stats.read().is_running {
                            NODE_STATE.start_stratum_needed.store(false, Ordering::Relaxed);
                        }

                        thread::sleep(Self::STATS_UPDATE_DELAY);
                    }
                }
                Err(e) => {
                    // Setup an error.
                    {
                        let mut w_err = NODE_STATE.error.write();
                        *w_err = Some(e);
                    }
                    // Reset server state.
                    Self::reset_server_state(true);
                }
            }
        });
    }

    /// Clean up [`Server`] stats and statuses.
    fn reset_server_state(has_error: bool) {
        NODE_STATE.starting.store(false, Ordering::Relaxed);
        NODE_STATE.restart_needed.store(false, Ordering::Relaxed);
        NODE_STATE.start_stratum_needed.store(false, Ordering::Relaxed);
        NODE_STATE.stop_needed.store(false, Ordering::Relaxed);

        // Reset stratum stats.
        {
            let mut w_stratum_stats = NODE_STATE.stratum_stats.write();
            *w_stratum_stats = StratumStats::default();
        }
        // Reset server stats.
        {
            let mut w_stats = NODE_STATE.stats.write();
            *w_stats = None;
        }
        // Reset an error if needed.
        if !has_error {
            let mut w_err = NODE_STATE.error.write();
            *w_err = None;
        }
    }

    /// Clean-up [`Server`] data if server is not running.
    pub fn clean_up_data() {
        if Self::is_running() {
            return;
        }
        let config = NodeConfig::node_server_config();
        let server_config = config.server.clone();
        let dirs_to_remove: Vec<&str> = vec!["header", "lmdb", "txhashset"];
        for dir in dirs_to_remove {
            let mut path = PathBuf::from(&server_config.db_root);
            path.push(dir);
            if path.exists() {
                fs::remove_dir_all(path).unwrap();
            }
        }
    }

    /// Reset [`Server`] peers data.
    pub fn reset_peers(force: bool) {
        if force || !Node::is_running() {
            // Get saved server config.
            let config = NodeConfig::node_server_config();
            let server_config = config.server.clone();
            // Remove peers folder.
            let mut peers_dir = PathBuf::from(&server_config.db_root);
            peers_dir.push("peer");
            if peers_dir.exists() {
                match fs::remove_dir_all(peers_dir) {
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
            NODE_STATE.reset_peers.store(false, Ordering::Relaxed);
        } else {
            NODE_STATE.reset_peers.store(true, Ordering::Relaxed);
        }
    }

    /// Get synchronization status i18n text.
    pub fn get_sync_status_text() -> String {
        if Node::is_stopping() {
            return t!("sync_status.shutdown");
        };

        if Node::is_starting() {
            return t!("sync_status.initial");
        };

        if Node::is_restarting() {
            return t!("sync_status.node_restarting");
        }

        let sync_status = Self::get_sync_status();

        if sync_status.is_none() {
            return t!("sync_status.node_down");
        }

        match sync_status.unwrap() {
            SyncStatus::Initial => t!("sync_status.initial"),
            SyncStatus::NoSync => t!("sync_status.no_sync"),
            SyncStatus::AwaitingPeers(_) => t!("sync_status.awaiting_peers"),
            SyncStatus::HeaderSync {
                sync_head,
                highest_height,
                ..
            } => {
                if highest_height == 0 {
                    t!("sync_status.header_sync")
                } else {
                    let percent = sync_head.height * 100 / highest_height;
                    t!("sync_status.header_sync_percent", "percent" => percent)
                }
            }
            SyncStatus::TxHashsetPibd {
                aborted: _,
                errored: _,
                completed_leaves,
                leaves_required,
                completed_to_height: _,
                required_height: _,
            } => {
                if completed_leaves == 0 {
                    t!("sync_status.tx_hashset_pibd")
                } else {
                    let percent = completed_leaves * 100 / leaves_required;
                    t!("sync_status.tx_hashset_pibd_percent",  "percent" => percent)
                }
            }
            SyncStatus::TxHashsetDownload(stat) => {
                if stat.total_size > 0 {
                    let percent = stat.downloaded_size * 100 / stat.total_size;
                    t!("sync_status.tx_hashset_download_percent", "percent" => percent)
                } else {
                    t!("sync_status.tx_hashset_download")
                }
            }
            SyncStatus::TxHashsetSetup {
                headers,
                headers_total,
                kernel_pos,
                kernel_pos_total,
            } => {
                if headers.is_some() && headers_total.is_some() {
                    let h = headers.unwrap();
                    let ht = headers_total.unwrap();
                    let percent = h * 100 / ht;
                    t!("sync_status.tx_hashset_setup_history", "percent" => percent)
                } else if kernel_pos.is_some() && kernel_pos_total.is_some() {
                    let k = kernel_pos.unwrap();
                    let kt = kernel_pos_total.unwrap();
                    let percent = k * 100 / kt;
                    t!("sync_status.tx_hashset_setup_position", "percent" => percent)
                } else {
                    t!("sync_status.tx_hashset_setup")
                }
            }
            SyncStatus::TxHashsetRangeProofsValidation {
                rproofs,
                rproofs_total,
            } => {
                let r_percent = if rproofs_total > 0 {
                    (rproofs * 100) / rproofs_total
                } else {
                    0
                };
                t!("sync_status.tx_hashset_range_proofs_validation", "percent" => r_percent)
            }
            SyncStatus::TxHashsetKernelsValidation {
                kernels,
                kernels_total,
            } => {
                let k_percent = if kernels_total > 0 {
                    (kernels * 100) / kernels_total
                } else {
                    0
                };
                t!("sync_status.tx_hashset_kernels_validation", "percent" => k_percent)
            }
            SyncStatus::TxHashsetSave | SyncStatus::TxHashsetDone => {
                t!("sync_status.tx_hashset_save")
            }
            SyncStatus::BodySync {
                current_height,
                highest_height,
            } => {
                if highest_height == 0 {
                    t!("sync_status.body_sync")
                } else {
                    let percent = current_height * 100 / highest_height;
                    t!("sync_status.body_sync_percent", "percent" => percent)
                }
            }
            SyncStatus::Shutdown => t!("sync_status.shutdown"),
        }
    }
}

/// Start the node [`Server`].
fn start_node_server() -> Result<Server, Error>  {
    // Setup server config.
    let mut config = NodeConfig::node_server_config();
    PeersConfig::load_to_server_config(&mut config);
    let mut server_config = config.server.clone();

    // Setup Mainnet DNSSeed
    if server_config.chain_type == ChainTypes::Mainnet && NodeConfig::is_default_seeding_type() {
        server_config.p2p_config.seeding_type = Seeding::List;
        server_config.p2p_config.seeds = Some(PeerAddrs::default());
        for seed in Node::MAINNET_DNS_SEEDS {
            let addr = format!("{}:3414", seed);
            if let Some(p) = PeersConfig::peer_to_addr(addr) {
                let mut seeds = server_config
                    .p2p_config
                    .seeds
                    .clone()
                    .unwrap_or(PeerAddrs::default());
                seeds.peers.insert(seeds.peers.len(), p);
                server_config.p2p_config.seeds = Some(seeds);
            }
        }
    }

    // Fix to avoid too many opened files.
    server_config.p2p_config.peer_min_preferred_outbound_count =
        server_config.p2p_config.peer_max_outbound_count;

    // Remove temporary file dir.
    {
        let mut tmp_dir = PathBuf::from(&server_config.db_root);
        tmp_dir = tmp_dir.parent().unwrap().to_path_buf();
        tmp_dir.push("tmp");
        if tmp_dir.exists() {
            match fs::remove_dir_all(tmp_dir) {
                Ok(_) => {}
                Err(_) => {}
            }
        }
    }

    // Initialize our global chain_type, feature flags (NRD kernel support currently),
    // accept_fee_base, and future_time_limit.
    // These are read via global and not read from config beyond this point.
    if !global::GLOBAL_CHAIN_TYPE.is_init() {
        global::init_global_chain_type(config.server.chain_type);
    } else {
        global::set_global_chain_type(config.server.chain_type);
        global::set_local_chain_type(config.server.chain_type);
    }

    if !global::GLOBAL_NRD_FEATURE_ENABLED.is_init() {
        match global::get_chain_type() {
            ChainTypes::Mainnet => {
                global::init_global_nrd_enabled(false);
            }
            _ => {
                global::init_global_nrd_enabled(true);
            }
        }
    } else {
        match global::get_chain_type() {
            ChainTypes::Mainnet => {
                global::set_global_nrd_enabled(false);
            }
            _ => {
                global::set_global_nrd_enabled(true);
            }
        }
    }

    let afb = config.server.pool_config.accept_fee_base;
    if !global::GLOBAL_ACCEPT_FEE_BASE.is_init() {
        global::init_global_accept_fee_base(afb);
    } else {
        global::set_global_accept_fee_base(afb);
    }

    let future_time_limit = config.server.future_time_limit;
    if !global::GLOBAL_FUTURE_TIME_LIMIT.is_init() {
        global::init_global_future_time_limit(future_time_limit);
    } else {
        global::set_global_future_time_limit(future_time_limit);
    }

    // Put flag to start stratum server if autorun is available.
    if NodeConfig::is_stratum_autorun_enabled() {
        NODE_STATE.start_stratum_needed.store(true, Ordering::Relaxed);
    }

    // Reset an error.
    {
        let mut w_err = NODE_STATE.error.write();
        *w_err = None;
    }

    // Start integrated node server.
    let api_chan: &'static mut (oneshot::Sender<()>, oneshot::Receiver<()>) =
        Box::leak(Box::new(oneshot::channel::<()>()));
    let server_result = Server::new(server_config, None, api_chan);
    server_result
}

/// Start stratum mining server on a separate thread.
pub fn start_stratum_mining_server(server: &Server, config: StratumServerConfig) {
    let proof_size = global::proofsize();
    let sync_state = server.sync_state.clone();

    let mut stratum_server = StratumServer::new(
        config,
        server.chain.clone(),
        server.tx_pool.clone(),
        NODE_STATE.stratum_stats.clone(),
    );
    let stop_state = NODE_STATE.stratum_stop_state.clone();
    stop_state.reset();
    let server_state = stop_state.clone();
    thread::spawn(move || {
            stratum_server.run_loop(proof_size, sync_state, stop_state);
            server_state.reset();
            // Reset stratum stats.
            {
                let mut w_stratum_stats = NODE_STATE.stratum_stats.write();
                *w_stratum_stats = StratumStats::default();
            }
        });
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Get sync status text for Android notification from [`NODE_STATE`] in Java string format.
pub extern "C" fn Java_mw_gri_android_BackgroundService_getSyncStatusText(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    _activity: jni::objects::JObject,
) -> jni::sys::jstring {
    let status_text = Node::get_sync_status_text();
    let j_text = _env.new_string(status_text);
    return j_text.unwrap().into_raw();
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Get sync title for Android notification in Java string format.
pub extern "C" fn Java_mw_gri_android_BackgroundService_getSyncTitle(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    _activity: jni::objects::JObject,
) -> jni::sys::jstring {
    let j_text = _env.new_string(t!("network.node"));
    return j_text.unwrap().into_raw();
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Get start text for Android notification in Java string format.
pub extern "C" fn Java_mw_gri_android_BackgroundService_getStartText(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    _activity: jni::objects::JObject,
) -> jni::sys::jstring {
    let j_text = _env.new_string(t!("network_settings.enable"));
    return j_text.unwrap().into_raw();
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Get stop text for Android notification in Java string format.
pub extern "C" fn Java_mw_gri_android_BackgroundService_getStopText(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    _activity: jni::objects::JObject,
) -> jni::sys::jstring {
    let j_text = _env.new_string(t!("network_settings.disable"));
    return j_text.unwrap().into_raw();
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Get exit text for Android notification in Java string format.
pub extern "C" fn Java_mw_gri_android_BackgroundService_getExitText(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    _activity: jni::objects::JObject,
) -> jni::sys::jstring {
    let j_text = _env.new_string(t!("modal_exit.exit"));
    return j_text.unwrap().into_raw();
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Check if node launch is possible.
pub extern "C" fn Java_mw_gri_android_BackgroundService_canStartNode(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    _activity: jni::objects::JObject,
) -> jni::sys::jboolean {
    let loading = Node::is_stopping() || Node::is_restarting() || Node::is_starting();
    return (!loading && !Node::is_running()) as jni::sys::jboolean;
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Check if node stop is possible.
pub extern "C" fn Java_mw_gri_android_BackgroundService_canStopNode(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    _activity: jni::objects::JObject,
) -> jni::sys::jboolean {
    let loading = Node::is_stopping() || Node::is_restarting() || Node::is_starting();
    return (!loading && Node::is_running()) as jni::sys::jboolean;
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Check if node stop is possible.
pub extern "C" fn Java_mw_gri_android_NotificationActionsReceiver_isNodeRunning(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    _activity: jni::objects::JObject,
) -> jni::sys::jboolean {
    return Node::is_running() as jni::sys::jboolean;
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Start node from Android Java code.
pub extern "C" fn Java_mw_gri_android_NotificationActionsReceiver_startNode(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    _activity: jni::objects::JObject,
) {
    Node::start();
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Stop node from Android Java code.
pub extern "C" fn Java_mw_gri_android_NotificationActionsReceiver_stopNode(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    _activity: jni::objects::JObject,
) {
    Node::stop(false);
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Stop node from Android Java code.
pub extern "C" fn Java_mw_gri_android_NotificationActionsReceiver_stopNodeToExit(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    _activity: jni::objects::JObject,
) {
    Node::stop(true);
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Check if app exit is needed after node stop to finish Android app at background.
pub extern "C" fn Java_mw_gri_android_BackgroundService_exitAppAfterNodeStop(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    _activity: jni::objects::JObject,
) -> jni::sys::jboolean {
    let exit_needed = !Node::is_running() && NODE_STATE.exit_after_stop.load(Ordering::Relaxed);
    return exit_needed as jni::sys::jboolean;
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Handle unexpected application termination on Android (removal from recent apps).
pub extern "C" fn Java_mw_gri_android_MainActivity_onTermination(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    _activity: jni::objects::JObject,
) {
    Node::stop(false);
}