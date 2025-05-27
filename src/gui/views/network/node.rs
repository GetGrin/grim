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
use grin_servers::PeerStats;

use crate::gui::Colors;
use crate::gui::icons::{AT, CUBE, DEVICES, FLOW_ARROW, HANDSHAKE, PACKAGE, SHARE_NETWORK};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Content, View};
use crate::gui::views::network::types::{NodeTab, NodeTabType};
use crate::node::{Node, NodeConfig};

/// Integrated node tab content.
#[derive(Default)]
pub struct NetworkNode;

impl NodeTab for NetworkNode {
    fn get_type(&self) -> NodeTabType {
        NodeTabType::Info
    }

    fn ui(&mut self, ui: &mut egui::Ui, _: &dyn PlatformCallbacks) {
        ScrollArea::vertical()
            .id_salt("integrated_node_info_scroll")
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(2.0);
                View::max_width_ui(ui, Content::SIDE_PANEL_WIDTH * 1.3, |ui| {
                    // Show node stats content.
                    node_stats_ui(ui);
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
            View::label_box(ui,
                            stats.header_stats.last_block_h.to_string(),
                            t!("network_node.hash"),
                            [true, false, false, false]);
        });
        columns[1].vertical_centered(|ui| {
            View::label_box(ui,
                            stats.header_stats.height.to_string(),
                            t!("network_node.height"),
                            [false, true, false, false]);
        });
    });
    ui.columns(2, |columns| {
        columns[0].vertical_centered(|ui| {
            View::label_box(ui,
                            stats.header_stats.total_difficulty.to_string(),
                            t!("network_node.difficulty"),
                            [false, false, true, false]);
        });
        columns[1].vertical_centered(|ui| {
            let h_ts = stats.header_stats.latest_timestamp.timestamp();
            let h_time = View::format_time(h_ts);
            View::label_box(ui,
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
            View::label_box(ui,
                            stats.chain_stats.last_block_h.to_string(),
                            t!("network_node.hash"),
                            [true, false, false, false]);
        });
        columns[1].vertical_centered(|ui| {
            View::label_box(ui,
                            stats.chain_stats.height.to_string(),
                            t!("network_node.height"),
                            [false, true, false, false]);
        });
    });
    ui.columns(2, |columns| {
        columns[0].vertical_centered(|ui| {
            View::label_box(ui,
                            stats.chain_stats.total_difficulty.to_string(),
                            t!("network_node.difficulty"),
                            [false, false, true, false]);
        });
        columns[1].vertical_centered(|ui| {
            let b_ts = stats.chain_stats.latest_timestamp.timestamp();
            let b_time = View::format_time(b_ts);
            View::label_box(ui,
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
            View::label_box(ui,
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
            View::label_box(ui,
                            stem_tx_stat,
                            t!("network_node.stem_pool"),
                            [false, true, false, false]);
        });
    });
    ui.columns(2, |columns| {
        columns[0].vertical_centered(|ui| {
            View::label_box(ui,
                            stats.disk_usage_gb.to_string(),
                            t!("network_node.size"),
                            [false, false, true, false]);
        });
        columns[1].vertical_centered(|ui| {
            let peers_txt = format!("{} ({})",
                                    stats.peer_count,
                                    NodeConfig::get_max_outbound_peers());
            View::label_box(ui, peers_txt, t!("network_node.peers"), [false, false, false, true]);
        });
    });
    ui.add_space(5.0);

    // Show peer stats when available.
    if stats.peer_count > 0 {
        View::sub_title(ui, format!("{} {}", HANDSHAKE, t!("network_node.peers")));
        let peers = &stats.peer_stats;
        for (index, ps) in peers.iter().enumerate() {
            peer_item_ui(ui, ps, View::item_rounding(index, peers.len(), false));
        }
        ui.add_space(5.0);
    }
}

const PEER_ITEM_HEIGHT: f32 = 77.0;

/// Draw connected peer info item.
fn peer_item_ui(ui: &mut egui::Ui, peer: &PeerStats, rounding: CornerRadius) {
    let mut rect = ui.available_rect_before_wrap();
    rect.set_height(PEER_ITEM_HEIGHT);
    ui.allocate_ui(rect.size(), |ui| {
        ui.vertical(|ui| {
            ui.add_space(4.0);

            // Draw round background.
            ui.painter().rect(rect, rounding, Colors::fill_lite(), View::item_stroke(), StrokeKind::Middle);

            // Draw IP address.
            ui.horizontal(|ui| {
                ui.add_space(7.0);
                ui.label(RichText::new(&peer.addr)
                    .color(Colors::white_or_black(true))
                    .size(17.0));
            });
            // Draw difficulty and height.
            ui.horizontal(|ui| {
                ui.add_space(6.0);
                let diff_text = format!("{} {} {} {}",
                                        PACKAGE,
                                        peer.total_difficulty,
                                        AT,
                                        peer.height);
                ui.label(RichText::new(diff_text)
                    .color(Colors::title(false))
                    .size(15.0));
            });
            // Draw user-agent.
            ui.horizontal(|ui| {
                ui.add_space(6.0);
                let agent_text = format!("{} {}", DEVICES, &peer.user_agent);
                ui.label(RichText::new(agent_text)
                    .color(Colors::gray())
                    .size(15.0));
            });

            ui.add_space(3.0);
        });
    });
}