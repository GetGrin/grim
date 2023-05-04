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
use std::sync::{Arc, LockResult, mpsc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicBool, Ordering};
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
    node_state: Arc<Mutex<NodeState>>,
}

impl Node {
    /// Instantiate new node with provided chain type, start server if needed
    pub fn new(chain_type: ChainTypes, start: bool) -> Self {
        let stop_state = Arc::new(StopState::new());
        let mut state = NodeState::new(chain_type, stop_state.clone());
        let node_state = Arc::new(Mutex::new(state));
        if start {
            let server = start_server(&chain_type, stop_state);
            start_server_thread(node_state.clone(), server);
        } else {
            stop_state.stop();
        }
        Self { node_state }
    }

    /// Acquire node state to be used by a current thread
    pub fn acquire_state(&self) -> MutexGuard<'_, NodeState> {
        self.node_state.lock().unwrap()
    }

    /// Stop server
    pub fn stop(&self) {
        self.acquire_state().stop_needed = true;
    }

    /// Start server with provided chain type
    pub fn start(&self, chain_type: ChainTypes) {
        let mut state = self.node_state.lock().unwrap();
        if state.stop_state.is_stopped() {
            self.start_with_acquired_state(state, chain_type);
        }
    }

    /// Restart server with provided chain type
    pub fn restart(&mut self, chain_type: ChainTypes) {
        let mut state = self.acquire_state();
        if !state.stop_state.is_stopped() {
            state.chain_type = chain_type;
            state.restart_needed = true;
        } else {
            self.start_with_acquired_state(state, chain_type);
        }
    }

    /// Start server with provided acquired state
    fn start_with_acquired_state(&self, mut state: MutexGuard<NodeState>, chain_type: ChainTypes) {
        state.chain_type = chain_type;
        state.stop_state = Arc::new(StopState::new());

        let server = start_server(&chain_type, state.stop_state.clone());
        start_server_thread(self.node_state.clone(), server);
    }
}

pub struct NodeState {
    /// To check server state
    stop_state: Arc<StopState>,
    /// Data for UI, None means server is not started
    pub(crate) stats: Option<ServerStats>,
    /// Chain type of launched server
    chain_type: ChainTypes,
    /// Thread flag to stop the server and start it again
    restart_needed: bool,
    /// Thread flag to stop the server
    stop_needed: bool,
}

impl NodeState {
    /// Instantiate new node state with provided chain type and server state
    pub fn new(chain_type: ChainTypes, stop_state: Arc<StopState>) -> Self {
        Self {
            stop_state,
            stats: None,
            chain_type,
            restart_needed: false,
            stop_needed: false,
        }
    }

    /// Check if server is stopping at separate thread
    pub fn is_stopping(&self) -> bool {
        return self.stop_needed
    }

    /// Check if server is restarting at separate thread
    pub fn is_restarting(&self) -> bool {
        return self.restart_needed
    }
}

/// Start server with provided chain type and node state
fn start_server(chain_type: &ChainTypes, stop_state: Arc<StopState>) -> Server {
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

    let mut db_path = PathBuf::from(&server_config.db_root);
    db_path.push("grin.lock");
    fs::remove_file(db_path).unwrap();

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
    let mut server_result = Server::new(server_config.clone(), Some(stop_state.clone()), api_chan);
    if server_result.is_err() {
        let mut db_path = PathBuf::from(&server_config.db_root);
        db_path.push("grin.lock");
        fs::remove_file(db_path).unwrap();

        // Remove chain data on server start error
        let dirs_to_remove: Vec<&str> = vec!["header", "lmdb", "txhashset", "peer"];
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
        server_result = Server::new(server_config.clone(), Some(stop_state.clone()), api_chan);
    }

    server_result.unwrap()
}

/// Start a thread to launch server and update node state with server stats
fn start_server_thread(node_state: Arc<Mutex<NodeState>>, mut server: Server) -> JoinHandle<()> {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(500));
        let mut state = node_state.lock().unwrap();
        if state.restart_needed {
            server.stop();

            // Create new server with new stop state
            state.stop_state = Arc::new(StopState::new());
            server = start_server(&state.chain_type, state.stop_state.clone());

            state.restart_needed = false;
        } else if state.stop_needed {
            server.stop();
            state.stats = None;
            state.stop_needed = false;
            break;
        }
        if !state.stop_state.is_stopped() {
            let stats = server.get_server_stats();
            if stats.is_ok() {
                state.stats = Some(stats.as_ref().ok().unwrap().clone());
            }
        }
    })
}