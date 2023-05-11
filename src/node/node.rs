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
use std::fmt::format;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LockResult, RwLock, RwLockReadGuard};
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use futures::channel::oneshot;
use grin_chain::SyncStatus;
use grin_config::config;
use grin_core::global;
use grin_core::global::ChainTypes;
use grin_servers::{Server, ServerStats};
use grin_servers::common::types::Error;
use grin_util::logger::LogEntry;
use grin_util::StopState;
use log::info;

pub struct Node {
    /// Node state updated from the separate thread
    pub(crate) state: Arc<NodeState>,
}

impl Node {
    /// Instantiate new node with provided chain type, start server if needed
    pub fn new(chain_type: ChainTypes, start: bool) -> Self {
        let state = Arc::new(NodeState::new(chain_type));
        if start {
            start_server_thread(state.clone(), chain_type);
        }
        Self { state }
    }

    /// Stop server
    pub fn stop(&self) {
        self.state.stop_needed.store(true, Ordering::Relaxed);
    }

    /// Start server with provided chain type
    pub fn start(&self, chain_type: ChainTypes) {
        if !self.state.is_running() {
            start_server_thread(self.state.clone(), chain_type);
        }
    }

    /// Restart server or start when not running
    pub fn restart(&mut self) {
        if self.state.is_running() {
            self.state.restart_needed.store(true, Ordering::Relaxed);
        } else {
            self.start(*self.state.chain_type);
        }
    }
}

pub struct NodeState {
    /// Data for UI, None means server is not running
    stats: Arc<RwLock<Option<ServerStats>>>,
    /// Chain type of launched server
    chain_type: Arc<ChainTypes>,
    /// Thread flag to stop the server and start it again
    restart_needed: AtomicBool,
    /// Thread flag to stop the server
    stop_needed: AtomicBool,
}

impl NodeState {
    /// Instantiate new node state with provided chain type and server state
    pub fn new(chain_type: ChainTypes) -> Self {
        Self {
            stats: Arc::new(RwLock::new(None)),
            chain_type: Arc::new(chain_type),
            restart_needed: AtomicBool::new(false),
            stop_needed: AtomicBool::new(false),
        }
    }

    /// Check if server is running when stats are not empty
    pub fn is_running(&self) -> bool {
        self.get_stats().is_some()
    }

    /// Check if server is stopping
    pub fn is_stopping(&self) -> bool {
        self.stop_needed.load(Ordering::Relaxed)
    }

    /// Check if server is restarting
    pub fn is_restarting(&self) -> bool {
        self.restart_needed.load(Ordering::Relaxed)
    }

    /// Get server stats
    pub fn get_stats(&self) -> RwLockReadGuard<'_, Option<ServerStats>> {
        self.stats.read().unwrap()
    }

    /// Get server sync status, empty when server is not running
    pub fn get_sync_status(&self) -> Option<SyncStatus> {
        // return shutdown status when node is stopping
        if self.is_stopping() {
            return Some(SyncStatus::Shutdown)
        }

        let stats = self.get_stats();
        // return sync status when server is running (stats are not empty)
        if stats.is_some() {
            return Some(stats.as_ref().unwrap().sync_status)
        }
        None
    }

    /// Check if server is syncing based on sync status
    pub fn is_syncing(&self) -> bool {
        let sync_status = self.get_sync_status();
        match sync_status {
            None => { self.is_restarting() }
            Some(s) => { s!= SyncStatus::NoSync }
        }
    }
}

/// Start a thread to launch server and update node state with server stats
fn start_server_thread(state: Arc<NodeState>, chain_type: ChainTypes) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut server = start_server(&chain_type);

        loop {
            thread::sleep(Duration::from_millis(500));

            if state.is_restarting() {
                server.stop();

                // Create new server with current chain type
                server = start_server(&state.chain_type);

                state.restart_needed.store(false, Ordering::Relaxed);
            } else if state.is_stopping() {
                server.stop();

                let mut w_stats = state.stats.write().unwrap();
                *w_stats = None;

                state.stop_needed.store(false, Ordering::Relaxed);
                break;
            } else {
                let stats = server.get_server_stats();
                if stats.is_ok() {
                    let mut w_stats = state.stats.write().unwrap();
                    *w_stats = Some(stats.as_ref().ok().unwrap().clone());
                }
            }
        }
    })
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
        let mut db_path = PathBuf::from(&server_config.db_root);
        db_path.push("grin.lock");
        fs::remove_file(db_path).unwrap();
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