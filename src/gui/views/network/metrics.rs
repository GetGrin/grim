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

use egui::{RichText, Rounding, ScrollArea, vec2};
use grin_servers::DiffBlock;

use crate::gui::Colors;
use crate::gui::icons::{AT, COINS, CUBE_TRANSPARENT, HASH, HOURGLASS_LOW, HOURGLASS_MEDIUM, TIMER};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{NetworkContent, View};
use crate::gui::views::network::types::{NetworkTab, NetworkTabType};
use crate::node::Node;

/// Chain metrics tab content.
#[derive(Default)]
pub struct NetworkMetrics;

const BLOCK_REWARD: f64 = 60.0;
// 1 year is calculated as 365 days and 6 hours (31557600).
const YEARLY_SUPPLY: f64 = ((60 * 60 * 24 * 365) + 6 * 60 * 60) as f64;

impl NetworkTab for NetworkMetrics {
    fn get_type(&self) -> NetworkTabType {
        NetworkTabType::Metrics
    }

    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame, _: &dyn PlatformCallbacks) {
        let server_stats = Node::get_stats();
        // Show message to enable node when it's not running.
        if !Node::is_running() {
            NetworkContent::disabled_node_ui(ui);
            return;
        }

        // Show loading spinner when node is stopping.
        if Node::is_stopping() {
            NetworkContent::loading_ui(ui, None);
            return;
        }

        // Show message when metrics are not available.
        if server_stats.is_none() || Node::is_restarting()
            || server_stats.as_ref().unwrap().diff_stats.height == 0 {
            NetworkContent::loading_ui(ui, Some(t!("network_metrics.loading")));
            return;
        }

        let stats = server_stats.as_ref().unwrap();

        ui.add_space(1.0);

        // Show emission info.
        View::sub_title(ui, format!("{} {}", COINS, t!("network_metrics.emission")));
        ui.columns(3, |columns| {
            let supply = stats.header_stats.height as f64 * BLOCK_REWARD;
            let rate = (YEARLY_SUPPLY * 100.0) / supply;

            columns[0].vertical_centered(|ui| {
                View::rounded_box(ui,
                                  format!("{}ツ", BLOCK_REWARD),
                                  t!("network_metrics.reward"),
                                  [true, false, true, false]);
            });
            columns[1].vertical_centered(|ui| {
                View::rounded_box(ui,
                                  format!("{:.2}%", rate),
                                  t!("network_metrics.inflation"),
                                  [false, false, false, false]);
            });
            columns[2].vertical_centered(|ui| {
                View::rounded_box(ui,
                                  supply.to_string(),
                                  t!("network_metrics.supply"),
                                  [false, true, false, true]);
            });
        });
        ui.add_space(4.0);

        // Show difficulty adjustment window info.
        let difficulty_title = t!(
                "network_metrics.difficulty_window",
                "size" => stats.diff_stats.window_size
            );
        View::sub_title(ui, format!("{} {}", HOURGLASS_MEDIUM, difficulty_title));
        ui.columns(3, |columns| {
            columns[0].vertical_centered(|ui| {
                View::rounded_box(ui,
                                  stats.diff_stats.height.to_string(),
                                  t!("network_node.height"),
                                  [true, false, true, false]);
            });
            columns[1].vertical_centered(|ui| {
                View::rounded_box(ui,
                                  format!("{}s", stats.diff_stats.average_block_time),
                                  t!("network_metrics.block_time"),
                                  [false, false, false, false]);
            });
            columns[2].vertical_centered(|ui| {
                View::rounded_box(ui,
                                  stats.diff_stats.average_difficulty.to_string(),
                                  t!("network_node.difficulty"),
                                  [false, true, false, true]);
            });
        });
        ui.add_space(4.0);

        // Show difficulty adjustment window blocks.
        let blocks_size = stats.diff_stats.last_blocks.len();
        ScrollArea::vertical()
            .id_source("difficulty_scroll")
            .auto_shrink([false; 2])
            .stick_to_bottom(true)
            .show_rows(
                ui,
                BLOCK_ITEM_HEIGHT - 1.0,
                blocks_size,
                |ui, row_range| {
                    for index in row_range {
                        // Add space before the first item.
                        if index == 0 {
                            ui.add_space(4.0);
                        }
                        let db = stats.diff_stats.last_blocks.get(index).unwrap();
                        block_item_ui(ui, db, View::item_rounding(index, blocks_size, false));
                    }
                },
            );
    }
}

const BLOCK_ITEM_HEIGHT: f32 = 77.0;

/// Draw block difficulty item.
fn block_item_ui(ui: &mut egui::Ui, db: &DiffBlock, rounding: Rounding) {
    let mut rect = ui.available_rect_before_wrap();
    rect.set_height(BLOCK_ITEM_HEIGHT);
    ui.allocate_ui_at_rect(rect, |ui| {
        ui.horizontal(|ui| {
            ui.add_space(6.0);
            ui.vertical(|ui| {
                // Draw round background.
                rect.min += vec2(8.0, 0.0);
                ui.painter().rect(rect, rounding, Colors::WHITE, View::ITEM_STROKE);

                ui.add_space(2.0);

                // Draw block hash.
                ui.horizontal(|ui| {
                    ui.add_space(5.0);
                    ui.label(RichText::new(format!("{} {}", HASH, db.block_hash))
                        .color(Colors::BLACK)
                        .size(17.0));
                });
                // Draw block difficulty and height.
                ui.horizontal(|ui| {
                    ui.add_space(6.0);
                    let diff_text = format!("{} {} {} {}",
                                            CUBE_TRANSPARENT,
                                            db.difficulty,
                                            AT,
                                            db.block_height);
                    ui.label(RichText::new(diff_text).color(Colors::TITLE).size(16.0));
                });
                // Draw block date.
                ui.horizontal(|ui| {
                    ui.add_space(6.0);
                    let block_time = View::format_time(db.time as i64);
                    ui.label(RichText::new(format!("{} {}s {} {}",
                                                   TIMER,
                                                   db.duration,
                                                   HOURGLASS_LOW,
                                                   block_time))
                        .color(Colors::GRAY)
                        .size(16.0));
                });

                ui.add_space(2.0);
            });
        });
    });
}