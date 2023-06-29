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
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use futures::channel::oneshot;
use grin_chain::SyncStatus;
use grin_core::global;
use grin_core::global::ChainTypes;
use grin_servers::{Server, ServerStats};
use grin_servers::common::types::Error;
use jni::sys::{jboolean, jstring};
use lazy_static::lazy_static;
use crate::node::NodeConfig;

use crate::Settings;

lazy_static! {
    /// Static thread-aware state of [`Node`] to be updated from another thread.
    static ref NODE_STATE: Arc<Node> = Arc::new(Node::default());
}

/// Provides [`Server`] control, holds current status and statistics.
pub struct Node {
    /// Statistics data for UI.
    stats: Arc<RwLock<Option<ServerStats>>>,
    /// Running API server address.
    api_addr: Arc<RwLock<Option<String>>>,
    /// Running P2P server port.
    p2p_port: Arc<RwLock<Option<u16>>>,
    /// Indicator if server is starting.
    starting: AtomicBool,
    /// Thread flag to stop the server and start it again.
    restart_needed: AtomicBool,
    /// Thread flag to stop the server.
    stop_needed: AtomicBool,
    /// Flag to check if app exit is needed after server stop.
    exit_after_stop: AtomicBool,
    /// Thread flag to start stratum server at separate.
    start_stratum_server: AtomicBool,
    /// Error on [`Server`] start.
    init_error: Option<Error>
}

impl Default for Node {
    fn default() -> Self {
        Self {
            stats: Arc::new(RwLock::new(None)),
            api_addr: Arc::new(RwLock::new(None)),
            p2p_port: Arc::new(RwLock::new(None)),
            starting: AtomicBool::new(false),
            restart_needed: AtomicBool::new(false),
            stop_needed: AtomicBool::new(false),
            exit_after_stop: AtomicBool::new(false),
            start_stratum_server: AtomicBool::new(false),
            init_error: None
        }
    }
}

impl Node {
    /// Stop the [`Server`] and setup exit flag after if needed.
    pub fn stop(exit_after_stop: bool) {
        NODE_STATE.stop_needed.store(true, Ordering::Relaxed);
        NODE_STATE.exit_after_stop.store(exit_after_stop, Ordering::Relaxed);
    }

    /// Start the node.
    pub fn start() {
        if !Self::is_running() {
            Self::start_server_thread();
        }
    }

    /// Restart the node.
    pub fn restart() {
        if Self::is_running() {
            NODE_STATE.restart_needed.store(true, Ordering::Relaxed);
        } else {
            Node::start();
        }
    }

    /// Get API server address if node is running.
    pub fn get_api_addr() -> Option<String> {
        let r_api_addr = NODE_STATE.api_addr.read().unwrap();
        if r_api_addr.is_some() {
            Some(r_api_addr.as_ref().unwrap().clone())
        } else {
            None
        }
    }

    /// Get P2P server port if node is running.
    pub fn get_p2p_port() -> Option<u16> {
        let r_p2p_port = NODE_STATE.p2p_port.read().unwrap();
        if r_p2p_port.is_some() {
            Some(r_p2p_port.unwrap())
        } else {
            None
        }
    }

    /// Start stratum server.
    pub fn start_stratum_server() {
        NODE_STATE.start_stratum_server.store(true, Ordering::Relaxed);
    }

    /// Check if stratum server is starting.
    pub fn is_stratum_server_starting() -> bool {
        NODE_STATE.start_stratum_server.load(Ordering::Relaxed)
    }

    /// Check if node is starting.
    pub fn is_starting() -> bool {
        NODE_STATE.starting.load(Ordering::Relaxed)
    }

    /// Check if node is running.
    pub fn is_running() -> bool {
        Self::get_sync_status().is_some()
    }

    /// Check if node is stopping.
    pub fn is_stopping() -> bool {
        NODE_STATE.stop_needed.load(Ordering::Relaxed)
    }

    /// Check if node is restarting.
    pub fn is_restarting() -> bool {
        NODE_STATE.restart_needed.load(Ordering::Relaxed)
    }

    /// Get node [`Server`] statistics.
    pub fn get_stats() -> RwLockReadGuard<'static, Option<ServerStats>> {
        NODE_STATE.stats.read().unwrap()
    }

    /// Get synchronization status, empty when [`Server`] is not running.
    pub fn get_sync_status() -> Option<SyncStatus> {
        // Return Shutdown status when node is stopping.
        if Self::is_stopping() {
            return Some(SyncStatus::Shutdown);
        }

        // Return Initial status when node is starting or restarting.
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

    /// Start node [`Server`] at separate thread to update [`NODE_STATE`] with [`ServerStats`].
    fn start_server_thread() {
        thread::spawn(move || {
            NODE_STATE.starting.store(true, Ordering::Relaxed);

            // Start the server.
            match start_server() {
                Ok(mut server) => {
                    let mut first_start = true;
                    loop {
                        if Self::is_restarting() {
                            // Stop the server.
                            server.stop();

                            // Create new server.
                            match start_server() {
                                Ok(s) => {
                                    server = s;
                                    NODE_STATE.restart_needed.store(false, Ordering::Relaxed);
                                }
                                Err(e) => {
                                    NODE_STATE.restart_needed.store(false, Ordering::Relaxed);
                                    Self::on_start_error(&e);
                                    break;
                                }
                            }
                        } else if Self::is_stopping() {
                            // Clean server stats.
                            {
                                let mut w_stats = NODE_STATE.stats.write().unwrap();
                                *w_stats = None;
                            }

                            // Stop the server.
                            server.stop();

                            NODE_STATE.starting.store(false, Ordering::Relaxed);
                            NODE_STATE.stop_needed.store(false, Ordering::Relaxed);
                            NODE_STATE.start_stratum_server.store(false, Ordering::Relaxed);

                            // Clean launched API server address.
                            {
                                let mut w_api_addr = NODE_STATE.api_addr.write().unwrap();
                                *w_api_addr = None;
                            }
                            // Clean launched P2P server port.
                            {
                                let mut w_p2p_port = NODE_STATE.p2p_port.write().unwrap();
                                *w_p2p_port = None;
                            }
                            break;
                        } else {
                            // Start stratum mining server.
                            if Self::is_stratum_server_starting() {
                                let stratum_config = server
                                    .config
                                    .stratum_mining_config
                                    .clone()
                                    .unwrap();
                                server.start_stratum_server(stratum_config);

                                // Wait for mining server to start and update status.
                                thread::sleep(Duration::from_millis(100));
                                NODE_STATE.start_stratum_server.store(false, Ordering::Relaxed);
                            }

                            // Update server stats.
                            if let Ok(stats) = server.get_server_stats() {
                                {
                                    let mut w_stats = NODE_STATE.stats.write().unwrap();
                                    *w_stats = Some(stats);
                                }

                                if first_start {
                                    NODE_STATE.starting.store(false, Ordering::Relaxed);
                                    first_start = false;
                                }
                            }
                        }
                        thread::sleep(Duration::from_millis(250));
                    }
                }
                Err(e) => {
                    NODE_STATE.starting.store(false, Ordering::Relaxed);
                    Self::on_start_error(&e);
                }
            }
        });
    }

    /// Handle node [`Server`] error on start.
    fn on_start_error(e: &Error) {
        // Clean launched API server address.
        {
            let mut w_api_addr = NODE_STATE.api_addr.write().unwrap();
            *w_api_addr = None;
        }
        // Clean launched P2P server port.
        {
            let mut w_p2p_port = NODE_STATE.p2p_port.write().unwrap();
            *w_p2p_port = None;
        }
        //TODO: Create error
        // NODE_STATE.init_error = Some(e);

        // // Clean-up server data on data init error.
        // // TODO: Ask user to clean-up data
        // let clean_server_and_recreate = || -> Server {
        //     let mut db_path = PathBuf::from(&server_config.db_root);
        //     db_path.push("grin.lock");
        //     fs::remove_file(db_path).unwrap();
        //
        //     // Remove chain data on server start error
        //     let dirs_to_remove: Vec<&str> = vec!["header", "lmdb", "txhashset"];
        //     for dir in dirs_to_remove {
        //         let mut path = PathBuf::from(&server_config.db_root);
        //         path.push(dir);
        //         fs::remove_dir_all(path).unwrap();
        //     }
        //
        //     // Recreate server
        //     let api_chan: &'static mut (oneshot::Sender<()>, oneshot::Receiver<()>) =
        //         Box::leak(Box::new(oneshot::channel::<()>()));
        //     server_result = Server::new(server_config.clone(), None, api_chan);
        //     server_result.unwrap()
        // };

        // Show err on server init error.
        // TODO: Ask user to clean-up data
        let show_error = |err: String| {
            println!("Node server creation error:\n{}", err);
        };

        //TODO: Better error handling
        match e {
            Error::Store(_) => {
                //TODO: Set err to ask user to clean data
                //(clean_server_and_recreate)()
            }
            Error::Chain(_) => {
                //TODO: Set err to ask user to clean data
                //(clean_server_and_recreate)()
            }
            //TODO: Handle P2P error (Show config error msg)
            Error::P2P(ref e) => {
                (show_error)("P2P error".to_string());
            }
            //TODO: Handle API error (Show config error msg)
            Error::API(ref e) => {
                (show_error)(e.to_string());
            }
            //TODO: Seems like another node instance running?
            Error::IOError(ref e) => {
                (show_error)(e.to_string());
            }
            //TODO: Show config error msg
            Error::Configuration(ref e) => {
                (show_error)(e.to_string());
            }
            //TODO: Unknown error
            _ => {
                (show_error)("Unknown error".to_string());
            }
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

/// Start the [`Server`] for node.
fn start_server() -> Result<Server, Error>  {
    // Get current global config
    let config = NodeConfig::get_members();
    let server_config = config.server.clone();

    // Remove temporary file dir
    {
        let mut tmp_dir = PathBuf::from(&server_config.db_root);
        tmp_dir = tmp_dir.parent().unwrap().to_path_buf();
        tmp_dir.push("tmp");
        if tmp_dir.exists() {
            match fs::remove_dir_all(tmp_dir) {
                Ok(_) => {}
                Err(_) => { println!("Cannot remove tmp dir") }
            }
        }
    }

    // Initialize our global chain_type, feature flags (NRD kernel support currently),
    // accept_fee_base, and future_time_limit.
    // These are read via global and not read from config beyond this point.
    if !global::GLOBAL_CHAIN_TYPE.is_init() {
        global::init_global_chain_type(config.server.chain_type);
    }

    if !global::GLOBAL_NRD_FEATURE_ENABLED.is_init() {
        match global::get_chain_type() {
            ChainTypes::Mainnet => {
                // Set various mainnet specific feature flags.
                global::init_global_nrd_enabled(false);
            }
            _ => {
                // Set various non-mainnet feature flags.
                global::init_global_nrd_enabled(true);
            }
        }
    }
    if !global::GLOBAL_ACCEPT_FEE_BASE.is_init() {
        let afb = config.server.pool_config.accept_fee_base;
        global::init_global_accept_fee_base(afb);
    }
    if !global::GLOBAL_FUTURE_TIME_LIMIT.is_init() {
        let future_time_limit = config.server.future_time_limit;
        global::init_global_future_time_limit(future_time_limit);
    }

    let api_chan: &'static mut (oneshot::Sender<()>, oneshot::Receiver<()>) =
        Box::leak(Box::new(oneshot::channel::<()>()));

    // Write launching API server address.
    {
        let mut w_api_addr = NODE_STATE.api_addr.write().unwrap();
        *w_api_addr = Some(config.server.api_http_addr);
    }

    // Write launching P2P server port.
    {
        let mut w_p2p_port = NODE_STATE.p2p_port.write().unwrap();
        *w_p2p_port = Some(config.server.p2p_config.port);
    }

    let server_result = Server::new(server_config.clone(), None, api_chan);
    server_result
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
) -> jstring {
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
) -> jstring {
    let j_text = _env.new_string(t!("network.node"));
    return j_text.unwrap().into_raw();
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
) -> jboolean {
    let exit_needed = !Node::is_running() && NODE_STATE.exit_after_stop.load(Ordering::Relaxed);
    return exit_needed as jboolean;
}