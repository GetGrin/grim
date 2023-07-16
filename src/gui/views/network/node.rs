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

use eframe::epaint::Stroke;
use egui::{RichText, Rounding, ScrollArea};
use grin_servers::PeerStats;

use crate::gui::Colors;
use crate::gui::icons::{AT, CUBE, DEVICES, FLOW_ARROW, HANDSHAKE, PACKAGE, PLUGS_CONNECTED, SHARE_NETWORK};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, View};
use crate::gui::views::network::{Network, NetworkTab, NetworkTabType};
use crate::node::Node;

/// Integrated node tab content.
#[derive(Default)]
pub struct NetworkNode;

impl NetworkTab for NetworkNode {
    fn get_type(&self) -> NetworkTabType {
        NetworkTabType::Node
    }

    fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        let server_stats = Node::get_stats();
        // Show message to enable node when it's not running.
        if !Node::is_running() {
            Network::disabled_node_ui(ui);
            return;
        }

        // Show loading spinner when stats are not available.
        if server_stats.is_none() || Node::is_restarting() || Node::is_stopping() {
            ui.centered_and_justified(|ui| {
                View::big_loading_spinner(ui);
            });
            return;
        }

        let stats = server_stats.as_ref().unwrap();

        ScrollArea::vertical()
            .id_source("integrated_node")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                // Show header info.
                View::sub_title(ui, format!("{} {}", FLOW_ARROW, t!("network_node.header")));
                ui.columns(2, |columns| {
                    columns[0].vertical_centered(|ui| {
                        View::rounded_box(ui,
                                          stats.header_stats.last_block_h.to_string(),
                                          t!("network_node.hash"),
                                          [true, false, false, false]);
                    });
                    columns[1].vertical_centered(|ui| {
                        View::rounded_box(ui,
                                          stats.header_stats.height.to_string(),
                                          t!("network_node.height"),
                                          [false, true, false, false]);
                    });
                });
                ui.columns(2, |columns| {
                    columns[0].vertical_centered(|ui| {
                        View::rounded_box(ui,
                                          stats.header_stats.total_difficulty.to_string(),
                                          t!("network_node.difficulty"),
                                          [false, false, true, false]);
                    });
                    columns[1].vertical_centered(|ui| {
                        let h_ts = stats.header_stats.latest_timestamp;
                        View::rounded_box(ui,
                                          h_ts.format("%d/%m/%Y %H:%M").to_string(),
                                          t!("network_node.time_utc"),
                                          [false, false, false, true]);
                    });
                });
                ui.add_space(4.0);

                // Show block info.
                View::sub_title(ui, format!("{} {}", CUBE, t!("network_node.block")));
                ui.columns(2, |columns| {
                    columns[0].vertical_centered(|ui| {
                        View::rounded_box(ui,
                                          stats.chain_stats.last_block_h.to_string(),
                                          t!("network_node.hash"),
                                          [true, false, false, false]);
                    });
                    columns[1].vertical_centered(|ui| {
                        View::rounded_box(ui,
                                          stats.chain_stats.height.to_string(),
                                          t!("network_node.height"),
                                          [false, true, false, false]);
                    });
                });
                ui.columns(2, |columns| {
                    columns[0].vertical_centered(|ui| {
                        View::rounded_box(ui,
                                          stats.chain_stats.total_difficulty.to_string(),
                                          t!("network_node.difficulty"),
                                          [false, false, true, false]);
                    });
                    columns[1].vertical_centered(|ui| {
                        let b_ts = stats.chain_stats.latest_timestamp;
                        View::rounded_box(ui,
                                          format!("{}", b_ts.format("%d/%m/%Y %H:%M")),
                                          t!("network_node.time_utc"),
                                          [false, false, false, true]);
                    });
                });
                ui.add_space(4.0);

                // Show data info.
                View::sub_title(ui, format!("{} {}", SHARE_NETWORK, t!("network_node.data")));
                ui.columns(2, |columns| {
                    columns[0].vertical_centered(|ui| {
                        let tx_stat = match &stats.tx_stats {
                            None => "0 (0)".to_string(),
                            Some(tx) => format!("{} ({})", tx.tx_pool_size, tx.tx_pool_kernels)
                        };
                        View::rounded_box(ui,
                                          tx_stat,
                                          t!("network_node.main_pool"),
                                          [true, false, false, false]);
                    });
                    columns[1].vertical_centered(|ui| {
                        let stem_tx_stat = match &stats.tx_stats {
                            None => "0 (0)".to_string(),
                            Some(stx) => format!("{} ({})",
                                                 stx.stem_pool_size,
                                                 stx.stem_pool_kernels)
                        };
                        View::rounded_box(ui,
                                          stem_tx_stat,
                                          t!("network_node.stem_pool"),
                                          [false, true, false, false]);
                    });
                });
                ui.columns(2, |columns| {
                    columns[0].vertical_centered(|ui| {
                        View::rounded_box(ui,
                                          stats.disk_usage_gb.to_string(),
                                          t!("network_node.size"),
                                          [false, false, true, false]);
                    });
                    columns[1].vertical_centered(|ui| {
                        View::rounded_box(ui,
                                          stats.peer_count.to_string(),
                                          t!("network_node.peers"),
                                          [false, false, false, true]);
                    });
                });
                ui.add_space(4.0);

                // Show peer stats when available.
                if stats.peer_count > 0 {
                    View::sub_title(ui, format!("{} {}", HANDSHAKE, t!("network_node.peers")));
                    for (index, ps) in stats.peer_stats.iter().enumerate() {
                        let rounding = if stats.peer_count == 1 {
                            [true, true]
                        } else if index == 0 {
                            [true, false]
                        } else if index == &stats.peer_stats.len() - 1 {
                            [false, true]
                        } else {
                            [false, false]
                        };
                        ui.vertical_centered(|ui| {
                            peer_item_ui(ui, ps, rounding);
                        });
                    }
                }
            });
    }

    fn on_modal_ui(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {}
}

/// Draw connected peer info item.
fn peer_item_ui(ui: &mut egui::Ui, peer: &PeerStats, rounding: [bool; 2]) {
    ui.vertical(|ui| {
        // Draw round background.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(74.9);
        ui.painter().rect(
            rect,
            Rounding {
                nw: if rounding[0] { 8.0 } else { 0.0 },
                ne: if rounding[0] { 8.0 } else { 0.0 },
                sw: if rounding[1] { 8.0 } else { 0.0 },
                se: if rounding[1] { 8.0 } else { 0.0 },
            },
            Colors::WHITE,
            Stroke { width: 1.0, color: Colors::ITEM_STROKE }
        );

        ui.add_space(2.0);

        // Draw peer address
        ui.horizontal(|ui| {
            ui.add_space(5.0);
            let addr_text = format!("{} {}", PLUGS_CONNECTED, &peer.addr);
            ui.label(RichText::new(addr_text).color(Colors::BLACK).size(17.0));
        });
        // Draw peer difficulty and height
        ui.horizontal(|ui| {
            ui.add_space(6.0);
            let diff_text = format!("{} {}", PACKAGE, peer.total_difficulty);
            ui.label(RichText::new(diff_text).color(Colors::TITLE).size(16.0));
            ui.add_space(2.0);
            ui.label(RichText::new(AT).color(Colors::TITLE).size(16.0));
            ui.add_space(2.0);
            ui.label(RichText::new(peer.height.to_string()).color(Colors::TITLE).size(16.0));
        });
        // Draw peer user-agent
        ui.horizontal(|ui| {
            ui.add_space(6.0);
            let agent_text = format!("{} {}", DEVICES, &peer.user_agent);
            ui.label(RichText::new(agent_text).color(Colors::GRAY).size(16.0));
        });
        ui.add_space(2.0);
    });
}