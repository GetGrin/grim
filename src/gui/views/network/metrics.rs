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

use egui::{RichText, CornerRadius, ScrollArea, vec2, StrokeKind};
use egui::scroll_area::ScrollBarVisibility;
use grin_core::consensus::{DAY_HEIGHT, GRIN_BASE, HOUR_SEC, REWARD};
use grin_servers::{DiffBlock, ServerStats};

use crate::gui::Colors;
use crate::gui::icons::{AT, COINS, CUBE_TRANSPARENT, HOURGLASS_LOW, HOURGLASS_MEDIUM, TIMER};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Content, View};
use crate::gui::views::network::NetworkContent;
use crate::gui::views::network::types::{NodeTab, NodeTabType};
use crate::node::Node;

/// Chain metrics tab content.
#[derive(Default)]
pub struct NetworkMetrics;

const BLOCK_REWARD: u64 = REWARD / GRIN_BASE;
// 1 year as 365 days and 6 hours (31557600).
const YEARLY_SUPPLY: u64 = (BLOCK_REWARD * DAY_HEIGHT * 365) + 6 * HOUR_SEC;

impl NodeTab for NetworkMetrics {
    fn get_type(&self) -> NodeTabType {
        NodeTabType::Metrics
    }

    fn ui(&mut self, ui: &mut egui::Ui, _: &dyn PlatformCallbacks) {
        let server_stats = Node::get_stats();
        let stats = server_stats.as_ref().unwrap();
        if stats.diff_stats.height == 0 {
            NetworkContent::loading_ui(ui, Some(t!("network_metrics.loading")));
            return;
        }
        ui.add_space(1.0);
        View::max_width_ui(ui, Content::SIDE_PANEL_WIDTH * 1.3, |ui| {
            // Show emission and difficulty info.
            info_ui(ui, stats);
            // Show difficulty adjustment window blocks.
            blocks_ui(ui, stats);
        });
    }
}

/// Draw emission and difficulty info.
fn info_ui(ui: &mut egui::Ui, stats: &ServerStats) {
    // Show emission info.
    View::sub_title(ui, format!("{} {}", COINS, t!("network_metrics.emission")));
    ui.columns(3, |columns| {
        let supply = stats.header_stats.height * BLOCK_REWARD;
        let rate = (YEARLY_SUPPLY * 100) / supply;

        columns[0].vertical_centered(|ui| {
            View::label_box(ui,
                            format!("{}ツ", BLOCK_REWARD),
                            t!("network_metrics.reward"),
                            [true, false, true, false]);
        });
        columns[1].vertical_centered(|ui| {
            View::label_box(ui,
                            format!("{:.2}%", rate),
                            t!("network_metrics.inflation"),
                            [false, false, false, false]);
        });
        columns[2].vertical_centered(|ui| {
            View::label_box(ui,
                            supply.to_string(),
                            t!("network_metrics.supply"),
                            [false, true, false, true]);
        });
    });
    ui.add_space(5.0);

    // Show difficulty adjustment window info.
    let difficulty_title = t!(
                "network_metrics.difficulty_window",
                "size" => stats.diff_stats.window_size
            );
    View::sub_title(ui, format!("{} {}", HOURGLASS_MEDIUM, difficulty_title));
    ui.columns(3, |columns| {
        columns[0].vertical_centered(|ui| {
            View::label_box(ui,
                            stats.diff_stats.height.to_string(),
                            t!("network_node.height"),
                            [true, false, true, false]);
        });
        columns[1].vertical_centered(|ui| {
            View::label_box(ui,
                            format!("{}s", stats.diff_stats.average_block_time),
                            t!("network_metrics.block_time"),
                            [false, false, false, false]);
        });
        columns[2].vertical_centered(|ui| {
            View::label_box(ui,
                            stats.diff_stats.average_difficulty.to_string(),
                            t!("network_node.difficulty"),
                            [false, true, false, true]);
        });
    });
}

const BLOCK_ITEM_HEIGHT: f32 = 77.0;

/// Draw difficulty adjustment window blocks content.
fn blocks_ui(ui: &mut egui::Ui, stats: &ServerStats) {
    let blocks_size = stats.diff_stats.last_blocks.len();
    ui.add_space(4.0);
    ScrollArea::vertical()
        .id_salt("mining_difficulty_scroll")
        .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
        .auto_shrink([false; 2])
        .stick_to_bottom(true)
        .show_rows(
            ui,
            BLOCK_ITEM_HEIGHT,
            blocks_size,
            |ui, row_range| {
                ui.add_space(4.0);
                for index in row_range {
                    let db = stats.diff_stats.last_blocks.get(index).unwrap();
                    block_item_ui(ui, db, View::item_rounding(index, blocks_size, false));
                }
            },
        );
}

/// Draw block difficulty item.
fn block_item_ui(ui: &mut egui::Ui, db: &DiffBlock, rounding: CornerRadius) {
    let mut rect = ui.available_rect_before_wrap();
    rect.set_height(BLOCK_ITEM_HEIGHT);
    ui.allocate_ui(rect.size(), |ui| {
        ui.horizontal(|ui| {
            ui.add_space(6.0);
            ui.vertical(|ui| {
                ui.add_space(4.0);

                // Draw round background.
                rect.min += vec2(8.0, 0.0);
                rect.max -= vec2(8.0, 0.0);
                ui.painter().rect(rect,
                                  rounding,
                                  Colors::white_or_black(false),
                                  View::item_stroke(),
                                  StrokeKind::Middle);

                // Draw block hash.
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.label(RichText::new(db.block_hash.to_string())
                        .color(Colors::white_or_black(true))
                        .size(17.0));
                });
                // Draw block difficulty and height.
                ui.horizontal(|ui| {
                    ui.add_space(7.0);
                    let diff_text = format!("{} {} {} {}",
                                            CUBE_TRANSPARENT,
                                            db.difficulty,
                                            AT,
                                            db.block_height);
                    ui.label(RichText::new(diff_text)
                        .color(Colors::title(false))
                        .size(15.0));
                });
                // Draw block date.
                ui.horizontal(|ui| {
                    ui.add_space(7.0);
                    let block_time = View::format_time(db.time as i64);
                    ui.label(RichText::new(format!("{} {}s {} {}",
                                                   TIMER,
                                                   db.duration,
                                                   HOURGLASS_LOW,
                                                   block_time))
                        .color(Colors::gray())
                        .size(15.0));
                });
                ui.add_space(3.0);
            });
            ui.add_space(6.0);
        });
    });
}