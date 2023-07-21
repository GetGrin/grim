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

use egui::{Color32, Id, lerp, Rgba};
use egui::style::Margin;
use egui_extras::{Size, StripBuilder};

use crate::gui::Colors;
use crate::gui::views::View;

/// Represents title content, can be single title or with animated sub-title.
pub enum TitleType {
    Single(String),
    WithSubTitle(String, String, bool)
}

/// Title panel with left/right action buttons and text in the middle.
pub struct TitlePanel;

impl TitlePanel {
    pub const DEFAULT_HEIGHT: f32 = 54.0;

    pub fn ui(title: TitleType,
              mut left_content: impl FnMut(&mut egui::Ui, &mut eframe::Frame),
              mut right_content: impl FnMut(&mut egui::Ui, &mut eframe::Frame),
              ui: &mut egui::Ui,
              frame: &mut eframe::Frame) {
        // Setup identifier.
        let id = match &title {
            TitleType::Single(text) => Id::from(text.clone()),
            TitleType::WithSubTitle(text, _, _) => Id::from(text.clone())
        };
        egui::TopBottomPanel::top(id)
            .resizable(false)
            .exact_height(Self::DEFAULT_HEIGHT)
            .frame(egui::Frame {
                inner_margin: Self::inner_margin(ui, frame),
                fill: Colors::YELLOW,
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                StripBuilder::new(ui)
                    .size(Size::exact(Self::DEFAULT_HEIGHT))
                    .size(Size::remainder())
                    .size(Size::exact(Self::DEFAULT_HEIGHT))
                    .horizontal(|mut strip| {
                        strip.cell(|ui| {
                            ui.centered_and_justified(|ui| {
                                (left_content)(ui, frame);
                            });
                        });
                        match title {
                            TitleType::Single(text) => {
                                strip.cell(|ui| {
                                    ui.add_space(2.0);
                                    ui.centered_and_justified(|ui| {
                                        View::ellipsize_text(ui, text, 19.0, Colors::TITLE);
                                    });
                                });
                            }
                            TitleType::WithSubTitle(text, subtitle_text, animate_sub) => {
                                strip.strip(|builder| {
                                    Self::with_sub_title(builder, text, subtitle_text, animate_sub);
                                });
                            }
                        }
                        strip.cell(|ui| {
                            ui.centered_and_justified(|ui| {
                                (right_content)(ui, frame);
                            });
                        });
                    });
            });
    }

    /// Calculate inner margin based on display insets (cutouts).
    fn inner_margin(ui: &mut egui::Ui, frame: &mut eframe::Frame) -> Margin {
        Margin {
            left: View::far_left_inset_margin(ui),
            right: View::far_right_inset_margin(ui, frame),
            top: View::get_top_inset(),
            bottom: 0.0,
        }
    }

    /// Draw title text for [`TitleType::WithSubTitle`] type.
    fn with_sub_title(builder: StripBuilder, title: String, subtitle: String, animate_sub: bool) {
        builder
            .size(Size::remainder())
            .size(Size::exact(30.0))
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    ui.add_space(4.0);
                    ui.centered_and_justified(|ui| {
                        View::ellipsize_text(ui, title, 18.0, Colors::TITLE);
                    });
                });
                strip.cell(|ui| {
                    ui.centered_and_justified(|ui| {
                        // Setup text color animation if needed.
                        let (dark, bright) = (0.3, 1.0);
                        let color_factor = if animate_sub {
                            lerp(dark..=bright, ui.input(|i| i.time).cos().abs()) as f32
                        } else {
                            bright as f32
                        };

                        // Draw subtitle text.
                        let sub_color_rgba = Rgba::from(Colors::TEXT) * color_factor;
                        let sub_color = Color32::from(sub_color_rgba);
                        View::ellipsize_text(ui, subtitle, 15.0, sub_color);

                        // Repaint delay based on animation status.
                        if animate_sub {
                            ui.ctx().request_repaint();
                        }
                    });
                });
            });
    }
}
