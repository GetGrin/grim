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

use egui::{RichText, CornerRadius, ScrollArea, StrokeKind};
use egui::scroll_area::ScrollBarVisibility;
use grin_chain::SyncStatus;
use grin_servers::WorkerStats;

use crate::gui::Colors;
use crate::gui::icons::{BARBELL, CLOCK_AFTERNOON, CPU, CUBE, FADERS, FOLDER_DASHED, FOLDER_SIMPLE_MINUS, FOLDER_SIMPLE_PLUS, HARD_DRIVES, PLUGS, PLUGS_CONNECTED, POLYGON};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Content, View};
use crate::gui::views::network::NetworkContent;
use crate::gui::views::network::setup::StratumSetup;
use crate::gui::views::network::types::{NodeTab, NodeTabType};
use crate::gui::views::types::ContentContainer;
use crate::node::{Node, NodeConfig};

/// Mining tab content.
pub struct NetworkMining {
    /// Stratum server setup content.
    stratum_server_setup: StratumSetup,
}

impl Default for NetworkMining {
    fn default() -> Self {
        Self {
            stratum_server_setup: StratumSetup::default(),
        }
    }
}

impl NodeTab for NetworkMining {
    fn get_type(&self) -> NodeTabType {
        NodeTabType::Mining
    }

    fn tab_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        if Node::is_stratum_starting() || Node::get_sync_status().unwrap() != SyncStatus::NoSync {
            NetworkContent::loading_ui(ui, Some(t!("network_mining.loading")));
            return;
        }

        // Show stratum server setup when mining server is not running.
        let stratum_stats = Node::get_stratum_stats();
        if !stratum_stats.is_running {
            ScrollArea::vertical()
                .id_salt("stratum_setup_scroll")
                .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.add_space(1.0);
                    View::max_width_ui(ui, Content::SIDE_PANEL_WIDTH * 1.3, |ui| {
                        self.stratum_server_setup.ui(ui, cb);
                    });
                });
            return;
        }

        ui.add_space(1.0);

        // Show stratum mining server info.
        View::sub_title(ui, format!("{} {}", HARD_DRIVES, t!("network_mining.server")));
        ui.columns(2, |columns| {
            columns[0].vertical_centered(|ui| {
                let (stratum_addr, stratum_port) = NodeConfig::get_stratum_address();
                View::label_box(ui,
                                format!("{}:{}", stratum_addr, stratum_port),
                                t!("network_mining.address"),
                                [true, false, true, false]);
            });
            columns[1].vertical_centered(|ui| {
                View::label_box(ui,
                                self.stratum_server_setup
                                    .wallet_name
                                    .clone()
                                    .unwrap_or("-".to_string()),
                                t!("network_mining.rewards_wallet"),
                                [false, true, false, true]);
            });
        });
        ui.add_space(4.0);

        // Show network info.
        View::sub_title(ui, format!("{} {}", POLYGON, t!("network.self")));
        ui.columns(3, |columns| {
            columns[0].vertical_centered(|ui| {
                let difficulty = if stratum_stats.network_difficulty > 0 {
                    stratum_stats.network_difficulty.to_string()
                } else {
                    "-".into()
                };
                View::label_box(ui,
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
                View::label_box(ui,
                                block_height,
                                t!("network_node.header"),
                                [false, false, false, false]);
            });
            columns[2].vertical_centered(|ui| {
                let hashrate = if stratum_stats.network_hashrate > 0.0 {
                    format!("{:.*}", 2, stratum_stats.network_hashrate)
                } else {
                    "-".into()
                };
                View::label_box(ui,
                                hashrate,
                                t!("network_mining.hashrate", "bits" => stratum_stats.edge_bits),
                                [false, true, false, true]);
            });
        });
        ui.add_space(4.0);

        // Show mining info.
        View::sub_title(ui, format!("{} {}", CPU, t!("network_mining.miners")));
        ui.columns(2, |columns| {
            columns[0].vertical_centered(|ui| {
                View::label_box(ui,
                                stratum_stats.num_workers.to_string(),
                                t!("network_mining.devices"),
                                [true, false, true, false]);
            });

            columns[1].vertical_centered(|ui| {
                View::label_box(ui,
                                stratum_stats.blocks_found.to_string(),
                                t!("network_mining.blocks_found"),
                                [false, true, false, true]);
            });
        });
        ui.add_space(4.0);

        // Show workers stats or info text when possible.
        let workers_size = stratum_stats.worker_stats.len();
        if workers_size != 0 && stratum_stats.num_workers > 0 {
            ui.add_space(4.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(4.0);
            ScrollArea::vertical()
                .id_salt("stratum_workers_scroll")
                .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                .auto_shrink([false; 2])
                .show_rows(
                    ui,
                    WORKER_ITEM_HEIGHT,
                    workers_size,
                    |ui, row_range| {
                        for index in row_range {
                            // Add space before the first item.
                            if index == 0 {
                                ui.add_space(4.0);
                            }
                            let worker = stratum_stats.worker_stats.get(index).unwrap();
                            let item_rounding = View::item_rounding(index, workers_size, false);
                            worker_item_ui(ui, worker, item_rounding);
                        }
                    },
                );
        } else if ui.available_height() > 142.0 {
            View::center_content(ui, 142.0, |ui| {
                ui.label(RichText::new(t!("network_mining.info", "settings" => FADERS))
                    .size(16.0)
                    .color(Colors::inactive_text())
                );
            });
        }
    }
}

/// Height of Stratum server worker list item.
const WORKER_ITEM_HEIGHT: f32 = 76.0;

/// Draw worker statistics item.
fn worker_item_ui(ui: &mut egui::Ui, ws: &WorkerStats, rounding: CornerRadius) {
    ui.horizontal_wrapped(|ui| {
        ui.vertical_centered_justified(|ui| {
            // Draw round background.
            let mut rect = ui.available_rect_before_wrap();
            rect.set_height(WORKER_ITEM_HEIGHT);
            ui.painter().rect(rect,
                              rounding,
                              Colors::white_or_black(false),
                              View::item_stroke(),
                              StrokeKind::Outside);

            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(5.0);

                // Draw worker connection status.
                let (status_text, status_icon, status_color) = match ws.is_connected {
                    true => (
                        t!("network_mining.connected"),
                        PLUGS_CONNECTED,
                        Colors::white_or_black(true)
                    ),
                    false => (t!("network_mining.disconnected"), PLUGS, Colors::inactive_text())
                };
                let status_line_text = format!("{} {} {}", status_icon, ws.id, status_text);
                ui.heading(RichText::new(status_line_text)
                    .color(status_color)
                    .size(17.0));
                ui.add_space(2.0);
            });
            ui.horizontal(|ui| {
                ui.add_space(6.0);

                // Draw difficulty.
                let diff_text = format!("{} {}", BARBELL, ws.pow_difficulty);
                ui.heading(RichText::new(diff_text)
                    .color(Colors::title(false))
                    .size(16.0));
                ui.add_space(6.0);

                // Draw accepted shares.
                let accepted_text = format!("{} {}", FOLDER_SIMPLE_PLUS, ws.num_accepted);
                ui.heading(RichText::new(accepted_text)
                    .color(Colors::green())
                    .size(16.0));
                ui.add_space(6.0);

                // Draw rejected shares.
                let rejected_text = format!("{} {}", FOLDER_SIMPLE_MINUS, ws.num_rejected);
                ui.heading(RichText::new(rejected_text)
                    .color(Colors::red())
                    .size(16.0));
                ui.add_space(6.0);

                // Draw stale shares.
                let stale_text = format!("{} {}", FOLDER_DASHED, ws.num_stale);
                ui.heading(RichText::new(stale_text)
                    .color(Colors::gray())
                    .size(16.0));
                ui.add_space(6.0);

                // Draw blocks found.
                let blocks_found_text = format!("{} {}", CUBE, ws.num_blocks_found);
                ui.heading(RichText::new(blocks_found_text)
                    .color(Colors::title(false))
                    .size(16.0));
            });
            ui.horizontal(|ui| {
                ui.add_space(6.0);

                // Draw block time
                let seen_ts = ws.last_seen.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                let seen_time = View::format_time(seen_ts as i64);
                let seen_text = format!("{} {}", CLOCK_AFTERNOON, seen_time);
                ui.heading(RichText::new(seen_text)
                    .color(Colors::gray())
                    .size(16.0));
            });
        });
    });
}