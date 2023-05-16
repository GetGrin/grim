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

use std::time::SystemTime;

use chrono::{DateTime, Local, Offset, TimeZone, Utc};
use chrono::format::DelayedFormat;
use eframe::emath::Vec2;
use eframe::epaint::{FontId, Stroke};
use eframe::epaint::text::{LayoutJob, TextFormat, TextWrapping};
use egui::{Color32, RichText, Rounding, ScrollArea, Spinner, Widget};
use egui_extras::{Size, StripBuilder};
use grin_servers::common::stats::TxStats;
use grin_servers::PeerStats;

use crate::gui::colors::{COLOR_DARK, COLOR_GRAY, COLOR_LIGHT, COLOR_YELLOW};
use crate::gui::icons::{AT, CUBE, DEVICES, DOWNLOAD_SIMPLE, FLOW_ARROW, HANDSHAKE, PACKAGE, PLUGS_CONNECTED, SHARE_NETWORK};
use crate::gui::views::{DEFAULT_STROKE, NetworkTab};
use crate::gui::views::common::sub_title;
use crate::node::Node;

pub struct NetworkNode {
    title: String
}

impl Default for NetworkNode {
    fn default() -> Self {
        Self {
            title: t!("integrated_node"),
        }
    }
}

impl NetworkTab for NetworkNode {
    fn ui(&mut self, ui: &mut egui::Ui, node: &mut Node) {
        let server_stats = node.state.get_stats();
        if !server_stats.is_some() {
            ui.centered_and_justified(|ui| {
                Spinner::new().size(42.0).color(COLOR_GRAY).ui(ui);
            });
            return;
        }

        let stats = server_stats.as_ref().unwrap();

        // Make scroll bar thinner
        ui.style_mut().spacing.scroll_bar_width = 4.0;

        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                // Disable item spacing
                ui.style_mut().spacing.item_spacing = Vec2::new(0.0, 0.0);

                // Show header stats
                ui.vertical_centered_justified(|ui| {
                    sub_title(ui, format!("{} {}", FLOW_ARROW, t!("header")), COLOR_DARK);
                });
                ui.add_space(4.0);

                ui.columns(2, |columns| {
                    columns[0].vertical_centered(|ui| {
                        draw_stat_box(ui,
                                      stats.header_stats.last_block_h.to_string(),
                                      t!("hash"),
                                      StatBoxRounding::TopLeft);
                    });
                    columns[1].vertical_centered(|ui| {
                        draw_stat_box(ui,
                                      stats.header_stats.height.to_string(),
                                      t!("height"),
                                      StatBoxRounding::TopRight);
                    });
                });

                ui.columns(2, |columns| {
                    columns[0].vertical_centered(|ui| {
                        draw_stat_box(ui,
                                      stats.header_stats.total_difficulty.to_string(),
                                      t!("difficulty"),
                                      StatBoxRounding::BottomLeft);
                    });
                    columns[1].vertical_centered(|ui| {
                        let ts = stats.header_stats.latest_timestamp;
                        draw_stat_box(ui,
                                      format!("{}", ts.format("%d/%m/%Y %H:%M")),
                                      t!("time_utc"),
                                      StatBoxRounding::BottomRight);
                    });
                });

                // Show block stats
                ui.add_space(5.0);
                ui.vertical_centered_justified(|ui| {
                    sub_title(ui, format!("{} {}", CUBE, t!("block")), COLOR_DARK);
                });
                ui.add_space(4.0);

                ui.columns(2, |columns| {
                    columns[0].vertical_centered(|ui| {
                        draw_stat_box(ui,
                                      stats.chain_stats.last_block_h.to_string(),
                                      t!("hash"),
                                      StatBoxRounding::TopLeft);
                    });
                    columns[1].vertical_centered(|ui| {
                        draw_stat_box(ui,
                                      stats.chain_stats.height.to_string(),
                                      t!("height"),
                                      StatBoxRounding::TopRight);
                    });
                });

                ui.columns(2, |columns| {
                    columns[0].vertical_centered(|ui| {
                        draw_stat_box(ui,
                                      stats.chain_stats.total_difficulty.to_string(),
                                      t!("difficulty"),
                                      StatBoxRounding::BottomLeft);
                    });
                    columns[1].vertical_centered(|ui| {
                        let ts = stats.chain_stats.latest_timestamp;
                        draw_stat_box(ui,
                                      format!("{}", ts.format("%d/%m/%Y %H:%M")),
                                      t!("time_utc"),
                                      StatBoxRounding::BottomRight);
                    });
                });

                // Show data stats
                ui.add_space(5.0);
                ui.vertical_centered_justified(|ui| {
                    sub_title(ui, format!("{} {}", SHARE_NETWORK, t!("data")), COLOR_DARK);
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
                        draw_stat_box(ui, tx_stat, t!("main_pool"), StatBoxRounding::TopLeft);
                    });
                    columns[1].vertical_centered(|ui| {
                        let stem_tx_stat = match &stats.tx_stats {
                            None => { "0 (0)".to_string() }
                            Some(stx) => {
                                format!("{} ({})", stx.stem_pool_size, stx.stem_pool_kernels)
                            }
                        };
                        draw_stat_box(ui, stem_tx_stat, t!("stem_pool"), StatBoxRounding::TopRight);
                    });
                });

                ui.columns(2, |columns| {
                    columns[0].vertical_centered(|ui| {
                        draw_stat_box(ui,
                                      stats.disk_usage_gb.to_string(),
                                      t!("size"),
                                      StatBoxRounding::BottomLeft);
                    });
                    columns[1].vertical_centered(|ui| {
                        let ts = stats.chain_stats.latest_timestamp;
                        draw_stat_box(ui,
                                      stats.peer_count.to_string(),
                                      t!("peers"),
                                      StatBoxRounding::BottomRight);
                    });
                });

                // Show peers stats when available
                if stats.peer_count > 0 {
                    ui.add_space(5.0);
                    ui.vertical_centered_justified(|ui| {
                        sub_title(ui, format!("{} {}", HANDSHAKE, t!("peers")), COLOR_DARK);
                    });
                    ui.add_space(4.0);

                    for (index, ps) in stats.peer_stats.iter().enumerate() {
                        let rounding = if index == 0 {
                            if stats.peer_count == 1 {
                                [true, true];
                            }
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

    fn name(&self) -> &String {
        &self.title
    }
}

fn draw_peer_stats(ui: &mut egui::Ui, peer: &PeerStats, rounding: [bool; 2]) {
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
        Stroke { width: 1.0, color: Color32::from_gray(230) }
    );

    ui.add_space(2.0);

    ui.horizontal_top(|ui| {
        ui.add_space(6.0);

        ui.heading(RichText::new(PLUGS_CONNECTED)
            .color(Color32::BLACK)
            .size(18.0));

        ui.add_space(6.0);

        // Draw peer address
        ui.heading(RichText::new(&peer.addr)
            .color(Color32::BLACK)
            .size(18.0));
    });

    ui.horizontal_top(|ui| {
        ui.add_space(6.0);

        ui.heading(RichText::new(PACKAGE)
            .color(COLOR_DARK)
            .size(16.0));

        ui.add_space(6.0);

        // Draw peer difficulty and height
        ui.heading(RichText::new(peer.total_difficulty.to_string())
            .color(COLOR_DARK)
            .size(16.0));
        ui.add_space(2.0);
        ui.heading(RichText::new(AT).color(COLOR_DARK).size(16.0));
        ui.add_space(2.0);
        ui.heading(RichText::new(peer.height.to_string())
            .color(COLOR_DARK)
            .size(16.0));
    });

    ui.horizontal_top(|ui| {
        ui.add_space(6.0);

        ui.heading(RichText::new(DEVICES)
            .color(COLOR_GRAY)
            .size(16.0));

        ui.add_space(6.0);

        // Draw peer user-agent
        ui.heading(RichText::new(&peer.user_agent)
            .color(COLOR_GRAY)
            .size(16.0));
    });

    ui.add_space(2.0);
}

#[derive(PartialEq)]
enum StatBoxRounding {
    TopLeft, TopRight, BottomRight, BottomLeft
}

fn draw_stat_box(ui: &mut egui::Ui, value: String, label: String, rounding: StatBoxRounding) {
    let mut rect = ui.available_rect_before_wrap();
    rect.set_height(46.0);

    // Draw box background
    ui.painter().rect(
        rect,
        Rounding {
            nw: if rounding == StatBoxRounding::TopLeft { 8.0 } else { 0.0 },
            ne: if rounding == StatBoxRounding::TopRight { 8.0 } else { 0.0 },
            sw: if rounding == StatBoxRounding::BottomLeft { 8.0 } else { 0.0 },
            se: if rounding == StatBoxRounding::BottomRight { 8.0 } else { 0.0 },
        },
        Color32::WHITE,
        Stroke { width: 1.0, color: Color32::from_gray(230) },
    );

    ui.vertical_centered_justified(|ui| {
        // Correct vertical spacing between items
        ui.style_mut().spacing.item_spacing.y = -4.0;

        // Draw box value
        let mut job = LayoutJob::single_section(value, TextFormat {
            font_id: FontId::proportional(18.0),
            color: Color32::BLACK,
            .. Default::default()
        });
        job.wrap = TextWrapping {
            max_rows: 1,
            break_anywhere: false,
            overflow_character: Option::from('﹍'),
            ..Default::default()
        };
        ui.label(job);

        // Draw box label
        ui.label(RichText::new(label).color(COLOR_GRAY).size(15.0));
    });
}