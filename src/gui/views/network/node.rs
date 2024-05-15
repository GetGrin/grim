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

use egui::{RichText, Rounding, ScrollArea};
use grin_servers::PeerStats;

use crate::gui::Colors;
use crate::gui::icons::{AT, CUBE, DEVICES, FLOW_ARROW, HANDSHAKE, PACKAGE, PLUGS_CONNECTED, SHARE_NETWORK};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{NetworkContent, Root, View};
use crate::gui::views::network::types::{NetworkTab, NetworkTabType};
use crate::node::Node;

/// Integrated node tab content.
#[derive(Default)]
pub struct NetworkNode;

impl NetworkTab for NetworkNode {
    fn get_type(&self) -> NetworkTabType {
        NetworkTabType::Node
    }

    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame, _: &dyn PlatformCallbacks) {
        // Show an error content when available.
        let node_err = Node::get_error();
        if node_err.is_some() {
            NetworkContent::node_error_ui(ui, node_err.unwrap());
            return;
        }

        // Show message to enable node when it's not running.
        if !Node::is_running() {
            NetworkContent::disabled_node_ui(ui);
            return;
        }

        // Show loading spinner when stats are not available.
        let server_stats = Node::get_stats();
        if server_stats.is_none() || Node::is_restarting() || Node::is_stopping() {
            NetworkContent::loading_ui(ui, None);
            return;
        }

        ScrollArea::vertical()
            .id_source("integrated_node")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(2.0);
                ui.vertical_centered(|ui| {
                    View::max_width_ui(ui, Root::SIDE_PANEL_WIDTH * 1.3, |ui| {
                        // Show node stats content.
                        node_stats_ui(ui);
                    });
                });
            });
    }
}

/// Draw node statistics content.
fn node_stats_ui(ui: &mut egui::Ui) {
    let server_stats = Node::get_stats();
    let stats = server_stats.as_ref().unwrap();

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
            let h_ts = stats.header_stats.latest_timestamp.timestamp();
            let h_time = View::format_time(h_ts);
            View::rounded_box(ui,
                              h_time,
                              t!("network_node.time"),
                              [false, false, false, true]);
        });
    });
    ui.add_space(5.0);

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
            let b_ts = stats.chain_stats.latest_timestamp.timestamp();
            let b_time = View::format_time(b_ts);
            View::rounded_box(ui,
                              b_time,
                              t!("network_node.time"),
                              [false, false, false, true]);
        });
    });
    ui.add_space(5.0);

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
    ui.add_space(5.0);

    // Show peer stats when available.
    if stats.peer_count > 0 {
        View::sub_title(ui, format!("{} {}", HANDSHAKE, t!("network_node.peers")));
        let peers = &stats.peer_stats;
        for (index, ps) in peers.iter().enumerate() {
            peer_item_ui(ui, ps, View::item_rounding(index, peers.len(), false));
            // Add space after the last item.
            if index == peers.len() - 1 {
                ui.add_space(5.0);
            }
        }
    }
}

/// Draw connected peer info item.
fn peer_item_ui(ui: &mut egui::Ui, peer: &PeerStats, rounding: Rounding) {
    let mut rect = ui.available_rect_before_wrap();
    rect.set_height(77.0);
    ui.allocate_ui_at_rect(rect, |ui| {
        ui.vertical(|ui| {
            // Draw round background.
            ui.painter().rect(rect, rounding, Colors::WHITE, View::ITEM_STROKE);

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
                let diff_text = format!("{} {} {} {}",
                                        PACKAGE,
                                        peer.total_difficulty,
                                        AT,
                                        peer.height);
                ui.label(RichText::new(diff_text).color(Colors::TITLE).size(16.0));
            });
            // Draw peer user-agent
            ui.horizontal(|ui| {
                ui.add_space(6.0);
                let agent_text = format!("{} {}", DEVICES, &peer.user_agent);
                ui.label(RichText::new(agent_text).color(Colors::GRAY).size(16.0));
            });

            ui.add_space(2.0);
        });
    });
}