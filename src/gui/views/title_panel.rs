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

use egui::{Margin, Color32, Id, lerp, Rgba};
use egui_extras::{Size, Strip, StripBuilder};

use crate::gui::Colors;
use crate::gui::views::{Root, View};
use crate::gui::views::types::{TitleContentType, TitleType};

/// Title panel with left/right action buttons and text in the middle.
pub struct TitlePanel;

impl TitlePanel {
    /// Default [`TitlePanel`] content height.
    pub const DEFAULT_HEIGHT: f32 = 54.0;

    pub fn ui(title: TitleType,
              mut left_content: impl FnMut(&mut egui::Ui, &mut eframe::Frame),
              mut right_content: impl FnMut(&mut egui::Ui, &mut eframe::Frame),
              ui: &mut egui::Ui,
              frame: &mut eframe::Frame) {
        // Setup identifier and title type.
        let (id, dual_title) = match &title {
            TitleType::Single(content) => {
                let text = match content {
                    TitleContentType::Title(text) => text,
                    TitleContentType::WithSubTitle(text, _, _) => text
                };
                (Id::from(text.clone()), false)
            },
            TitleType::Dual(first, _) => {
                let first_text = match first {
                    TitleContentType::Title(text) => text,
                    TitleContentType::WithSubTitle(text, _, _) => text
                };
                let second_text = match first {
                    TitleContentType::Title(text) => text,
                    TitleContentType::WithSubTitle(text, _, _) => text
                };
                let id = Id::from(first_text.to_owned()).with(second_text);
                (id, true)
            },
        };
        // Draw title panel.
        egui::TopBottomPanel::top(id)
            .resizable(false)
            .exact_height(Self::DEFAULT_HEIGHT)
            .frame(egui::Frame {
                inner_margin: Self::inner_margin(ui),
                fill: Colors::YELLOW,
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                StripBuilder::new(ui)
                    .size(Size::exact(Self::DEFAULT_HEIGHT))
                    .size(if dual_title {
                        Size::exact(Root::SIDE_PANEL_WIDTH - 2.0 * Self::DEFAULT_HEIGHT)
                    } else {
                        Size::remainder()
                    })
                    .size(if dual_title {
                        Size::exact(Self::DEFAULT_HEIGHT * 2.0)
                    } else {
                        Size::exact(0.0)
                    })
                    .size(if dual_title {
                        Size::remainder()
                    } else {
                        Size::exact(0.0)
                    })
                    .size(Size::exact(Self::DEFAULT_HEIGHT))
                    .horizontal(|mut strip| {
                        strip.cell(|ui| {
                            // Draw left panel action content.
                            ui.centered_and_justified(|ui| {
                                (left_content)(ui, frame);
                            });
                        });
                        // Draw title text content.
                        match title {
                            TitleType::Single(content) => {
                                Self::title_text_content(&mut strip, content);
                                strip.empty();
                                strip.empty();
                            }
                            TitleType::Dual(first, second) => {
                                Self::title_text_content(&mut strip, first);
                                strip.empty();
                                Self::title_text_content(&mut strip, second);
                            }
                        }
                        strip.cell(|ui| {
                            // Draw right panel action content.
                            ui.centered_and_justified(|ui| {
                                (right_content)(ui, frame);
                            });
                        });
                    });
            });
    }

    /// Setup title text content.
    fn title_text_content(strip: &mut Strip, content: TitleContentType) {
        match content {
            TitleContentType::Title(text) => {
                strip.cell(|ui| {
                    ui.add_space(2.0);
                    ui.centered_and_justified(|ui| {
                        View::ellipsize_text(ui, text, 19.0, Colors::TITLE);
                    });
                });
            }
            TitleContentType::WithSubTitle(text, subtitle, animate) => {
                strip.strip(|builder| {
                    Self::with_sub_title(builder, text, subtitle, animate);
                });
            }
        }
    }

    /// Calculate inner margin based on display insets (cutouts).
    fn inner_margin(ui: &mut egui::Ui) -> Margin {
        Margin {
            left: View::far_left_inset_margin(ui),
            right: View::far_right_inset_margin(ui),
            top: View::get_top_inset(),
            bottom: 0.0,
        }
    }

    /// Draw content for [`TitleType::WithSubTitle`] type.
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
