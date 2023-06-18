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

use egui::RichText;
use grin_chain::SyncStatus;

use crate::gui::Colors;
use crate::gui::icons::{COMPUTER_TOWER, CPU, FADERS, POLYGON};
use crate::gui::views::{Network, NetworkTab, NetworkTabType, View};
use crate::node::Node;
use crate::Settings;

#[derive(Default)]
pub struct NetworkMining;

impl NetworkTab for NetworkMining {
    fn get_type(&self) -> NetworkTabType {
        NetworkTabType::Mining
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let server_stats = Node::get_stats();
        // Show message when node is not running or loading spinner when mining are not available.
        if !server_stats.is_some() || Node::get_sync_status().unwrap() != SyncStatus::NoSync {
            if !Node::is_running() {
                Network::disabled_server_content(ui);
            } else {
                View::center_content(ui, 162.0, |ui| {
                    View::big_loading_spinner(ui);
                    ui.add_space(18.0);
                    ui.label(RichText::new(t!("network_mining.loading"))
                        .size(16.0)
                        .color(Colors::INACTIVE_TEXT)
                    );
                });
            }
            return;
        }

        // Stratum mining server address.
        let stratum_address = Settings::node_config_to_read()
            .members.clone()
            .server.stratum_mining_config.unwrap()
            .stratum_server_addr.unwrap();

        let stratum_stats = &server_stats.as_ref().unwrap().stratum_stats;
        if !stratum_stats.is_running && !Node::is_stratum_server_starting() {
            // Show Stratum setup when mining server is not enabled.
            View::center_content(ui, 162.0, |ui| {
                let text = t!(
                    "network_mining.disabled_server",
                    "address" => stratum_address,
                    "settings" => FADERS
                );
                ui.label(RichText::new(text)
                    .size(16.0)
                    .color(Colors::INACTIVE_TEXT)
                );

                ui.add_space(10.0);

                View::button(ui, t!("network_mining.enable_server"), Colors::GOLD, || {
                    Node::start_stratum_server();
                });

                ui.add_space(2.0);

                // Check if stratum server is enabled at config
                let stratum_enabled = Settings::node_config_to_read()
                    .members.clone()
                    .server.stratum_mining_config.unwrap()
                    .enable_stratum_server.unwrap();

                View::checkbox(ui, stratum_enabled, t!("network.autorun"), || {
                    let mut w_node_config = Settings::node_config_to_update();
                    w_node_config.members
                        .server.stratum_mining_config.as_mut().unwrap()
                        .enable_stratum_server = Some(!stratum_enabled);
                    w_node_config.save();
                });
            });
            return;
        } else if Node::is_stratum_server_starting() {
            // Show loading spinner when mining server is starting.
            View::center_content(ui, 162.0, |ui| {
                View::big_loading_spinner(ui);
                ui.add_space(18.0);
                ui.label(RichText::new(t!("network_mining.starting"))
                    .size(16.0)
                    .color(Colors::INACTIVE_TEXT)
                );
            });
            return;
        }

        // Show stratum mining server info.
        ui.vertical_centered_justified(|ui| {
            View::sub_header(ui, format!("{} {}", COMPUTER_TOWER, t!("network_mining.server")));
        });
        ui.add_space(4.0);

        ui.columns(2, |columns| {
            columns[0].vertical_centered(|ui| {
                View::rounded_box(ui,
                                  stratum_address,
                                  t!("network_mining.address"),
                                  [true, false, true, false]);
            });
            columns[1].vertical_centered(|ui| {
                // Stratum mining wallet address.
                let wallet_address = Settings::node_config_to_read()
                    .members.clone()
                    .server.stratum_mining_config.unwrap()
                    .wallet_listener_url;
                View::rounded_box(ui,
                                  wallet_address,
                                  t!("network_mining.wallet"),
                                  [false, true, false, true]);
            });
        });
        ui.add_space(4.0);

        // Show network info.
        ui.vertical_centered_justified(|ui| {
            View::sub_header(ui, format!("{} {}", POLYGON, t!("network.self")));
        });
        ui.add_space(4.0);

        ui.columns(3, |columns| {
            columns[0].vertical_centered(|ui| {
                let difficulty = if stratum_stats.network_difficulty > 0 {
                    stratum_stats.network_difficulty.to_string()
                } else {
                    "-".into()
                };
                View::rounded_box(ui,
                                  difficulty,
                                  t!("network_node.difficulty"),
                                  [true, false, true, false]);
            });
            columns[1].vertical_centered(|ui| {
                let block_height = if stratum_stats.block_height > 0 {
                    stratum_stats.block_height.to_string()
                } else {
                    "-".into()
                };
                View::rounded_box(ui,
                                  block_height,
                                  t!("network_node.header"),
                                  [false, false, false, false]);
            });
            columns[2].vertical_centered(|ui| {
                let hashrate = if stratum_stats.network_hashrate > 0.0 {
                    stratum_stats.network_hashrate.to_string()
                } else {
                    "-".into()
                };
                View::rounded_box(ui,
                                  hashrate,
                                  t!("network_mining.hashrate"),
                                  [false, true, false, true]);
            });
        });
        ui.add_space(4.0);

        // Show mining info.
        ui.vertical_centered_justified(|ui| {
            View::sub_header(ui, format!("{} {}", CPU, t!("network_mining.miners")));
        });
        ui.add_space(4.0);

        ui.columns(2, |columns| {
            columns[0].vertical_centered(|ui| {
                View::rounded_box(ui,
                                  stratum_stats.num_workers.to_string(),
                                  t!("network_mining.devices"),
                                  [true, false, true, false]);
            });

            columns[1].vertical_centered(|ui| {
                View::rounded_box(ui,
                                  stratum_stats.blocks_found.to_string(),
                                  t!("network_mining.blocks_found"),
                                  [false, true, false, true]);
            });
        });
        ui.add_space(4.0);

        // Show miners info.
        if !stratum_stats.worker_stats.is_empty() {
            //TODO: miners workers
        } else if ui.available_height() > 142.0 {
            View::center_content(ui, 142.0, |ui| {
                ui.label(RichText::new(t!("network_mining.info", "settings" => FADERS))
                    .size(16.0)
                    .color(Colors::INACTIVE_TEXT)
                );
            });
        }
    }
}