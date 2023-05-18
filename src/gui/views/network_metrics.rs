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

use chrono::{DateTime, NaiveDateTime, Utc};
use eframe::epaint::{Color32, Rounding, Stroke};
use egui::{RichText, ScrollArea, Spinner, Widget};
use grin_servers::DiffBlock;

use crate::gui::colors::{COLOR_DARK, COLOR_GRAY, COLOR_GRAY_LIGHT};
use crate::gui::icons::{AT, CALENDAR_PLUS, COINS, CUBE, CUBE_TRANSPARENT, HASH, HOURGLASS_LOW, HOURGLASS_MEDIUM, TIMER};
use crate::gui::views::{NetworkTab, View};
use crate::node::Node;

pub struct NetworkMetrics {
    title: String
}

impl Default for NetworkMetrics {
    fn default() -> Self {
        Self {
            title: t!("metrics"),
        }
    }
}

const BLOCK_REWARD: f64 = 60.0;
// 1 year is calculated as 365 days and 6 hours (31557600).
const YEARLY_SUPPLY: f64 = ((60 * 60 * 24 * 365) + 6 * 60 * 60) as f64;

impl NetworkTab for NetworkMetrics {
    fn name(&self) -> &String {
        &self.title
    }

    fn ui(&mut self, ui: &mut egui::Ui, node: &mut Node) {
        let server_stats = node.state.get_stats();
        // Show loading widget if server is not working or difficulty height is zero.
        if !server_stats.is_some() || server_stats.as_ref().unwrap().diff_stats.height == 0 {
            ui.centered_and_justified(|ui| {
                Spinner::new().size(42.0).color(COLOR_GRAY).ui(ui);
            });
            return;
        }

        let stats = server_stats.as_ref().unwrap();

        // Show emission info
        ui.vertical_centered_justified(|ui| {
            View::sub_title(ui, format!("{} {}", COINS, t!("emission")));
        });
        ui.add_space(4.0);

        let supply = stats.header_stats.height as f64 * BLOCK_REWARD;
        let rate = (YEARLY_SUPPLY * 100.0) / supply;

        ui.columns(3, |columns| {
            columns[0].vertical_centered(|ui| {
                View::rounded_box(ui,
                                  format!("{}ツ", BLOCK_REWARD),
                                  t!("reward"),
                                  [true, false, true, false]);
            });
            columns[1].vertical_centered(|ui| {
                View::rounded_box(ui,
                                  format!("{:.2}%", rate),
                                  t!("inflation"),
                                  [false, false, false, false]);
            });
            columns[2].vertical_centered(|ui| {
                View::rounded_box(ui,
                                  supply.to_string(),
                                  t!("supply"),
                                  [false, true, false, true]);
            });
        });
        ui.add_space(6.0);

        // Show difficulty window info
        ui.vertical_centered_justified(|ui| {
            let title = t!("difficulty_at_window", "size" => stats.diff_stats.window_size);
            View::sub_title(ui, format!("{} {}", HOURGLASS_MEDIUM, title));
        });
        ui.add_space(4.0);
        ui.columns(3, |columns| {
            columns[0].vertical_centered(|ui| {
                View::rounded_box(ui,
                                  stats.diff_stats.height.to_string(),
                                  t!("height"),
                                  [true, false, true, false]);
            });
            columns[1].vertical_centered(|ui| {
                View::rounded_box(ui,
                                  format!("{}s", stats.diff_stats.average_block_time),
                                  t!("block_time"),
                                  [false, false, false, false]);
            });
            columns[2].vertical_centered(|ui| {
                View::rounded_box(ui,
                                  stats.diff_stats.average_difficulty.to_string(),
                                  t!("difficulty"),
                                  [false, true, false, true]);
            });
        });
        ui.add_space(6.0);

        // Draw difficulty window blocks
        let blocks_size = stats.diff_stats.last_blocks.len();
        ScrollArea::vertical().auto_shrink([false; 2]).stick_to_bottom(true).show_rows(
            ui,
            DIFF_BLOCK_UI_HEIGHT,
            blocks_size,
            |ui, row_range| {
                for index in row_range {
                    let db = stats.diff_stats.last_blocks.get(index).unwrap();
                    let rounding = if blocks_size == 1 {
                        [true, true]
                    } else if index == 0 {
                        [true, false]
                    } else if index == blocks_size - 1 {
                        [false, true]
                    } else {
                        [false, false]
                    };
                    draw_diff_block(ui, db, rounding)
                }
            },
        );
    }
}

const DIFF_BLOCK_UI_HEIGHT: f32 = 77.0;

fn draw_diff_block(ui: &mut egui::Ui, db: &DiffBlock, rounding: [bool; 2]) {
    // Add space before first item
    if rounding[0] {
        ui.add_space(4.0);
    }

    ui.horizontal(|ui| {
        ui.add_space(6.0);
        ui.vertical(|ui| {
            let mut rect = ui.available_rect_before_wrap();
            rect.set_height(DIFF_BLOCK_UI_HEIGHT);
            ui.painter().rect(
                rect,
                Rounding {
                    nw: if rounding[0] { 8.0 } else { 0.0 },
                    ne: if rounding[0] { 8.0 } else { 0.0 },
                    sw: if rounding[1] { 8.0 } else { 0.0 },
                    se: if rounding[1] { 8.0 } else { 0.0 },
                },
                Color32::WHITE,
                Stroke { width: 1.0, color: COLOR_GRAY_LIGHT }
            );

            ui.add_space(2.0);
            ui.horizontal_top(|ui| {
                ui.add_space(5.0);
                ui.heading(RichText::new(HASH)
                    .color(Color32::BLACK)
                    .size(18.0));
                ui.add_space(2.0);

                // Draw block hash
                ui.heading(RichText::new(db.block_hash.to_string())
                    .color(Color32::BLACK)
                    .size(18.0));
            });
            ui.horizontal_top(|ui| {
                ui.add_space(6.0);
                ui.heading(RichText::new(CUBE_TRANSPARENT)
                    .color(COLOR_DARK)
                    .size(16.0));
                ui.add_space(4.0);

                // Draw block difficulty and height
                ui.heading(RichText::new(db.difficulty.to_string())
                    .color(COLOR_DARK)
                    .size(16.0));
                ui.add_space(2.0);
                ui.heading(RichText::new(AT).color(COLOR_DARK).size(16.0));
                ui.add_space(2.0);
                ui.heading(RichText::new(db.block_height.to_string())
                    .color(COLOR_DARK)
                    .size(16.0));
            });
            ui.horizontal_top(|ui| {
                ui.add_space(6.0);
                ui.heading(RichText::new(TIMER)
                    .color(COLOR_GRAY)
                    .size(16.0));
                ui.add_space(4.0);

                // Draw block time
                ui.heading(RichText::new(format!("{}s", db.duration))
                    .color(COLOR_GRAY)
                    .size(16.0));
                ui.add_space(2.0);
                ui.heading(RichText::new(HOURGLASS_LOW).color(COLOR_GRAY).size(16.0));
                ui.add_space(2.0);

                let naive_datetime = NaiveDateTime::from_timestamp_opt(db.time as i64, 0);
                if naive_datetime.is_some() {
                    let datetime: DateTime<Utc> = DateTime::from_utc(naive_datetime.unwrap(), Utc);
                    ui.heading(RichText::new(datetime.to_string())
                        .color(COLOR_GRAY)
                        .size(16.0));
                }
            });
            ui.add_space(2.0);
        });
    });
}