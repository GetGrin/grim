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

use std::sync::mpsc;
use grin_config::{config, GlobalConfig};
use grin_core::global;
use grin_core::global::ChainTypes;
use grin_util::logger::LogEntry;
use log::info;
use futures::channel::oneshot;

pub fn start(chain_type: &ChainTypes) {
    let node_config = Some(
        config::initial_setup_server(&ChainTypes::Mainnet).unwrap_or_else(|e| {
            //TODO: Error handling
            panic!("Error loading server configuration: {}", e);
        }),
    );

    let config = node_config.clone().unwrap();
    let mut server_config = config.members.as_ref().unwrap().server.clone();

    // Initialize our global chain_type, feature flags (NRD kernel support currently), accept_fee_base, and future_time_limit.
    // These are read via global and not read from config beyond this point.
    global::init_global_chain_type(config.members.as_ref().unwrap().server.chain_type);
    info!("Chain: {:?}", global::get_chain_type());
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
    let afb = config
        .members
        .as_ref()
        .unwrap()
        .server
        .pool_config
        .accept_fee_base;
    global::init_global_accept_fee_base(afb);
    info!("Accept Fee Base: {:?}", global::get_accept_fee_base());
    global::init_global_future_time_limit(config.members.unwrap().server.future_time_limit);
    info!("Future Time Limit: {:?}", global::get_future_time_limit());

    let api_chan: &'static mut (oneshot::Sender<()>, oneshot::Receiver<()>) =
        Box::leak(Box::new(oneshot::channel::<()>()));
    grin_servers::Server::start(
        server_config,
        None,
        |serv: grin_servers::Server, info: Option<mpsc::Receiver<LogEntry>>| {
            serv.get_server_stats();
            info!("Info callback")
            //serv.stop();
        },
    None,
        api_chan
    )
    .unwrap();
}