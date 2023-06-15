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
use egui::{Color32, RichText, Rounding, ScrollArea, Spinner, Widget};
use grin_servers::PeerStats;

use crate::gui::Colors;
use crate::gui::icons::{AT, CUBE, DEVICES, FLOW_ARROW, HANDSHAKE, PACKAGE, PLUGS_CONNECTED, SHARE_NETWORK};
use crate::gui::views::{Network, NetworkTab, View};
use crate::node::Node;

#[derive(Default)]
pub struct NetworkNode;

impl NetworkTab for NetworkNode {
    fn name(&self) -> String {
        t!("network.node")
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let server_stats = Node::get_stats();
        if !server_stats.is_some() {
            if !Node::is_running() {
                Network::disabled_server_content(ui);
            } else {
                ui.centered_and_justified(|ui| {
                    Spinner::new().size(104.0).color(Colors::GOLD).ui(ui);
                });
            }
            return;
        }

        let stats = server_stats.as_ref().unwrap();

        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                // Show header stats
                ui.vertical_centered_justified(|ui| {
                    View::sub_header(ui, format!("{} {}", FLOW_ARROW, t!("network_node.header")));
                });
                ui.add_space(4.0);
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
                                          format!("{}", h_ts.format("%d/%m/%Y %H:%M")),
                                          t!("network_node.time_utc"),
                                          [false, false, false, true]);
                    });
                });

                // Show block stats
                ui.add_space(6.0);
                ui.vertical_centered_justified(|ui| {
                    View::sub_header(ui, format!("{} {}", CUBE, t!("network_node.block")));
                });
                ui.add_space(4.0);
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

                // Show data stats
                ui.add_space(6.0);
                ui.vertical_centered_justified(|ui| {
                    View::sub_header(ui, format!("{} {}", SHARE_NETWORK, t!("network_node.data")));
                });
                ui.add_space(4.0);
                ui.columns(2, |columns| {
                    columns[0].vertical_centered(|ui| {
                        let tx_stat = match &stats.tx_stats {
                            None => { "0 (0)".to_string() }
                            Some(tx) => {
                                format!("{} ({})", tx.tx_pool_size, tx.tx_pool_kernels)
                            }
                        };
                        View::rounded_box(ui,
                                          tx_stat,
                                          t!("network_node.main_pool"),
                                          [true, false, false, false]);
                    });
                    columns[1].vertical_centered(|ui| {
                        let stem_tx_stat = match &stats.tx_stats {
                            None => { "0 (0)".to_string() }
                            Some(stx) => {
                                format!("{} ({})", stx.stem_pool_size, stx.stem_pool_kernels)
                            }
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

                // Show peers stats when available
                if stats.peer_count > 0 {
                    ui.add_space(6.0);
                    ui.vertical_centered_justified(|ui| {
                        View::sub_header(ui, format!("{} {}", HANDSHAKE, t!("network_node.peers")));
                    });
                    ui.add_space(4.0);

                    for index in 0..stats.peer_stats.len() {
                        let ps = stats.peer_stats.get(index).unwrap();
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
                            draw_peer_stats(ui, ps, rounding);
                        });
                    }
                }
            });
    }
}

fn draw_peer_stats(ui: &mut egui::Ui, peer: &PeerStats, rounding: [bool; 2]) {
    ui.vertical(|ui| {
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(77.0);

        ui.painter().rect(
            rect,
            Rounding {
                nw: if rounding[0] { 8.0 } else { 0.0 },
                ne: if rounding[0] { 8.0 } else { 0.0 },
                sw: if rounding[1] { 8.0 } else { 0.0 },
                se: if rounding[1] { 8.0 } else { 0.0 },
            },
            Color32::WHITE,
            Stroke { width: 1.0, color: Colors::ITEM_STROKE }
        );

        ui.add_space(2.0);
        ui.horizontal_top(|ui| {
            ui.add_space(5.0);
            ui.heading(RichText::new(PLUGS_CONNECTED)
                .color(Colors::BLACK)
                .size(18.0));
            ui.add_space(3.0);

            // Draw peer address
            ui.heading(RichText::new(&peer.addr)
                .color(Colors::BLACK)
                .size(18.0));
        });
        ui.horizontal_top(|ui| {
            ui.add_space(6.0);
            ui.heading(RichText::new(PACKAGE)
                .color(Colors::TITLE)
                .size(16.0));
            ui.add_space(4.0);

            // Draw peer difficulty and height
            ui.heading(RichText::new(peer.total_difficulty.to_string())
                .color(Colors::TITLE)
                .size(16.0));
            ui.add_space(2.0);
            ui.heading(RichText::new(AT).color(Colors::TITLE).size(16.0));
            ui.add_space(2.0);
            ui.heading(RichText::new(peer.height.to_string())
                .color(Colors::TITLE)
                .size(16.0));
        });

        ui.horizontal_top(|ui| {
            ui.add_space(6.0);
            ui.heading(RichText::new(DEVICES)
                .color(Colors::GRAY)
                .size(16.0));
            ui.add_space(4.0);

            // Draw peer user-agent
            ui.heading(RichText::new(&peer.user_agent)
                .color(Colors::GRAY)
                .size(16.0));
        });
        ui.add_space(2.0);
    });

    // Add space after last item
    if rounding[1] {
        ui.add_space(3.0);
    }
}