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
use std::thread::JoinHandle;
use std::time::Duration;

use futures::channel::oneshot;
use grin_chain::SyncStatus;
use grin_config::config;
use grin_core::global;
use grin_core::global::ChainTypes;
use grin_servers::{Server, ServerStats};
use jni::objects::JString;
use jni::sys::jstring;
use lazy_static::lazy_static;
use log::info;

lazy_static! {
    static ref NODE_STATE: Arc<Node> = Arc::new(Node::default());
}

pub struct Node {
    /// Data for UI
    stats: Arc<RwLock<Option<ServerStats>>>,
    /// Chain type of launched server
    chain_type: Arc<RwLock<ChainTypes>>,
    /// Indicator if server is starting
    starting: AtomicBool,
    /// Thread flag to stop the server and start it again
    restart_needed: AtomicBool,
    /// Thread flag to stop the server
    stop_needed: AtomicBool,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            stats: Arc::new(RwLock::new(None)),
            chain_type: Arc::new(RwLock::new(ChainTypes::Mainnet)),
            starting: AtomicBool::new(false),
            restart_needed: AtomicBool::new(false),
            stop_needed: AtomicBool::new(false),
        }
    }
}

impl Node {
    /// Stop server
    pub fn stop() {
        NODE_STATE.stop_needed.store(true, Ordering::Relaxed);
    }

    /// Start server with provided chain type
    pub fn start(chain_type: ChainTypes) {
        if !Self::is_running() {
            let mut w_chain_type = NODE_STATE.chain_type.write().unwrap();
            *w_chain_type = chain_type;
            Self::start_server_thread();
        }
    }

    /// Restart server with provided chain type
    pub fn restart(chain_type: ChainTypes) {
        if Self::is_running() {
            let mut w_chain_type = NODE_STATE.chain_type.write().unwrap();
            *w_chain_type = chain_type;
            NODE_STATE.restart_needed.store(true, Ordering::Relaxed);
        } else {
            Node::start(chain_type);
        }
    }

    /// Check if server is starting
    pub fn is_starting() -> bool {
        NODE_STATE.starting.load(Ordering::Relaxed)
    }

    /// Check if server is running
    pub fn is_running() -> bool {
        Self::get_stats().is_some() || Self::is_starting()
    }

    /// Check if server is stopping
    pub fn is_stopping() -> bool {
        NODE_STATE.stop_needed.load(Ordering::Relaxed)
    }

    /// Check if server is restarting
    pub fn is_restarting() -> bool {
        NODE_STATE.restart_needed.load(Ordering::Relaxed)
    }

    /// Get server stats
    pub fn get_stats() -> RwLockReadGuard<'static, Option<ServerStats>> {
        NODE_STATE.stats.read().unwrap()
    }

    /// Get server sync status, empty when server is not running
    pub fn get_sync_status() -> Option<SyncStatus> {
        // return Shutdown status when node is stopping
        if Self::is_stopping() {
            return Some(SyncStatus::Shutdown)
        }

        // return Initial status when node is starting
        if Self::is_starting() {
            return Some(SyncStatus::Initial)
        }

        let stats = Self::get_stats();
        // return sync status when server is running (stats are not empty)
        if stats.is_some() {
            return Some(stats.as_ref().unwrap().sync_status)
        }
        None
    }

    /// Start a thread to launch server and update state with server stats
    fn start_server_thread() -> JoinHandle<()> {
        thread::spawn(move || {
            NODE_STATE.starting.store(true, Ordering::Relaxed);

            let mut server = start_server(&NODE_STATE.chain_type.read().unwrap());
            let mut first_start = true;

            loop {
                if Self::is_restarting() {
                    server.stop();

                    // Create new server with current chain type
                    server = start_server(&NODE_STATE.chain_type.read().unwrap());

                    NODE_STATE.restart_needed.store(false, Ordering::Relaxed);
                } else if Self::is_stopping() {
                    server.stop();

                    let mut w_stats = NODE_STATE.stats.write().unwrap();
                    *w_stats = None;

                    NODE_STATE.stop_needed.store(false, Ordering::Relaxed);
                    break;
                } else {
                    let stats = server.get_server_stats();
                    if stats.is_ok() {
                        let mut w_stats = NODE_STATE.stats.write().unwrap();
                        *w_stats = Some(stats.as_ref().ok().unwrap().clone());

                        if first_start {
                            NODE_STATE.starting.store(false, Ordering::Relaxed);
                            first_start = false;
                        }
                    }
                }
                thread::sleep(Duration::from_millis(300));
            }
        })
    }

    pub fn get_sync_status_text(sync_status: Option<SyncStatus>) -> String {
        if Node::is_restarting() {
            return t!("server_restarting")
        }

        if sync_status.is_none() {
            return t!("server_down")
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
            SyncStatus::TxHashsetDownload(stat) => {
                if stat.total_size > 0 {
                    let percent = stat.downloaded_size * 100 / stat.total_size;
                    t!("sync_status.tx_hashset_download_percent", "percent" => percent)
                } else {
                    t!("sync_status.tx_hashset_download")
                }
            }
            SyncStatus::TxHashsetSetup => {
                t!("sync_status.tx_hashset_setup")
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

/// Start server with provided chain type
fn start_server(chain_type: &ChainTypes) -> Server {
    let mut node_config_result = config::initial_setup_server(chain_type);
    if node_config_result.is_err() {
        // Remove config file on init error
        let mut grin_path = dirs::home_dir().unwrap();
        grin_path.push(".grin");
        grin_path.push(chain_type.shortname());
        grin_path.push(config::SERVER_CONFIG_FILE_NAME);
        fs::remove_file(grin_path).unwrap();

        // Reinit config
        node_config_result = config::initial_setup_server(chain_type);
    }

    let node_config = node_config_result.ok();
    let config = node_config.clone().unwrap();
    let server_config = config.members.as_ref().unwrap().server.clone();

    // Remove lock file (in case if we have running node from another app)
    {
        let mut lock_file = PathBuf::from(&server_config.db_root);
        lock_file.push("grin.lock");
        if lock_file.exists() {
            match fs::remove_file(lock_file) {
                Ok(_) => {}
                Err(_) => { println!("Cannot remove grin.lock file") }
            };
        }
    }

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
        global::init_global_chain_type(config.members.as_ref().unwrap().server.chain_type);
    }
    info!("Chain: {:?}", global::get_chain_type());

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
        let afb = config
            .members
            .as_ref()
            .unwrap()
            .server
            .pool_config
            .accept_fee_base;
        global::init_global_accept_fee_base(afb);
        info!("Accept Fee Base: {:?}", global::get_accept_fee_base());
    }
    if !global::GLOBAL_FUTURE_TIME_LIMIT.is_init() {
        global::init_global_future_time_limit(config.members.unwrap().server.future_time_limit);
        info!("Future Time Limit: {:?}", global::get_future_time_limit());
    }

    let api_chan: &'static mut (oneshot::Sender<()>, oneshot::Receiver<()>) =
        Box::leak(Box::new(oneshot::channel::<()>()));
    let mut server_result = Server::new(server_config.clone(), None, api_chan);
    if server_result.is_err() {
        let mut db_path = PathBuf::from(&server_config.db_root);
        db_path.push("grin.lock");
        fs::remove_file(db_path).unwrap();

        // Remove chain data on server start error
        let dirs_to_remove: Vec<&str> = vec!["header", "lmdb", "txhashset"];
        for dir in dirs_to_remove {
            let mut path = PathBuf::from(&server_config.db_root);
            path.push(dir);
            fs::remove_dir_all(path).unwrap();
        }

        // Recreate server
        let config = node_config.clone().unwrap();
        let server_config = config.members.as_ref().unwrap().server.clone();
        let api_chan: &'static mut (oneshot::Sender<()>, oneshot::Receiver<()>) =
            Box::leak(Box::new(oneshot::channel::<()>()));
        server_result = Server::new(server_config.clone(), None, api_chan);
    }

    server_result.unwrap()
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn Java_mw_gri_android_BackgroundService_getSyncStatusText(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    _activity: jni::objects::JObject,
) -> jstring {
    let sync_status = Node::get_sync_status();
    let status_text = Node::get_sync_status_text(sync_status);
    let j_text = _env.new_string(status_text);
    return j_text.unwrap().into_raw();
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn Java_mw_gri_android_BackgroundService_getSyncTitle(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    _activity: jni::objects::JObject,
) -> jstring {
    let j_text = _env.new_string(t!("integrated_node"));
    return j_text.unwrap().into_raw();
}