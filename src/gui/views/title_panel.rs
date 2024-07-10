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

use egui::{Margin, Id, Layout, Align};

use crate::gui::Colors;
use crate::gui::views::{Content, View};
use crate::gui::views::types::{TitleContentType, TitleType};

/// Title panel with left/right action buttons and text in the middle.
pub struct TitlePanel {
    /// Widget identifier.
    id: Id
}

impl TitlePanel {
    /// Default [`TitlePanel`] content height.
    pub const DEFAULT_HEIGHT: f32 = 54.0;

    /// Create new title panel with provided identifier.
    pub fn new(id: Id) -> Self {
        Self {
            id,
        }
    }

    pub fn ui(&self,
              title: TitleType,
              mut left_content: impl FnMut(&mut egui::Ui),
              mut right_content: impl FnMut(&mut egui::Ui),
              ui: &mut egui::Ui) {
        // Draw title panel.
        egui::TopBottomPanel::top(self.id)
            .resizable(false)
            .exact_height(Self::DEFAULT_HEIGHT + View::get_top_inset())
            .frame(egui::Frame {
                inner_margin:  Margin {
                    left: View::far_left_inset_margin(ui),
                    right: View::far_right_inset_margin(ui),
                    top: View::get_top_inset(),
                    bottom: 0.0,
                },
                fill: Colors::yellow(),
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                let rect = ui.available_rect_before_wrap();
                ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Max), |ui| {
                    ui.horizontal_centered(|ui| {
                        (right_content)(ui);
                    });
                    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                        ui.horizontal_centered(|ui| {
                            (left_content)(ui);
                        });
                    });
                    match title {
                        TitleType::Single(content) => {
                            let content_rect = {
                                let mut r = rect;
                                r.min.x += Self::DEFAULT_HEIGHT;
                                r.max.x -= Self::DEFAULT_HEIGHT;
                                r
                            };
                            ui.allocate_ui_at_rect(content_rect, |ui| {
                                Self::title_text_content(ui, content);
                            });
                        }
                        TitleType::Dual(first, second) => {
                            let first_rect = {
                                let mut r = rect;
                                r.max.x = r.min.x + Content::SIDE_PANEL_WIDTH - Self::DEFAULT_HEIGHT;
                                r.min.x += Self::DEFAULT_HEIGHT;
                                r
                            };
                            // Draw first title content.
                            ui.allocate_ui_at_rect(first_rect, |ui| {
                                Self::title_text_content(ui, first);
                            });

                            let second_rect = {
                                let mut r = rect;
                                r.min.x = first_rect.max.x + 2.0 * Self::DEFAULT_HEIGHT;
                                r.max.x -= Self::DEFAULT_HEIGHT;
                                r
                            };
                            // Draw second title content.
                            ui.allocate_ui_at_rect(second_rect, |ui| {
                                Self::title_text_content(ui, second);
                            });
                        }
                    }
                });
            });
    }

    /// Setup title text content.
    fn title_text_content(ui: &mut egui::Ui, content: TitleContentType) {
        ui.vertical_centered(|ui| {
            match content {
                TitleContentType::Title(text) => {
                    ui.add_space(13.0 + if !View::is_desktop() {
                        1.0
                    } else {
                        0.0
                    });
                    View::ellipsize_text(ui, text, 19.0, Colors::title(true));
                }
                TitleContentType::WithSubTitle(text, subtitle, animate) => {
                    ui.add_space(4.0);
                    View::ellipsize_text(ui, text, 18.0, Colors::title(true));
                    ui.add_space(-2.0);
                    View::animate_text(ui, subtitle, 15.0, Colors::text(true), animate)
                }
            }
        });
    }
}
