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

use egui::style::Margin;
use egui_extras::{Size, StripBuilder};

use crate::gui::Colors;
use crate::gui::views::View;

pub struct TitleAction {
    pub(crate) icon: Box<&'static str>,
    pub(crate) on_click: Box<dyn Fn()>,
}

impl TitleAction {
    pub fn new(icon: &'static str, on_click: fn()) -> Option<Self> {
        Option::from(Self { icon: Box::new(icon), on_click: Box::new(on_click) })
    }
}

pub struct TitlePanel;

impl TitlePanel {
    pub const DEFAULT_HEIGHT: f32 = 52.0;

    pub fn ui(title: String, l: Option<TitleAction>, r: Option<TitleAction>, ui: &mut egui::Ui) {
        egui::TopBottomPanel::top("title_panel")
            .resizable(false)
            .frame(egui::Frame {
                fill: Colors::YELLOW,
                inner_margin: Margin::same(0.0),
                outer_margin: Margin::same(0.0),
                stroke: egui::Stroke::NONE,
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                StripBuilder::new(ui)
                    .size(Size::exact(Self::DEFAULT_HEIGHT))
                    .vertical(|mut strip| {
                        strip.strip(|builder| {
                            builder
                                .size(Size::exact(Self::DEFAULT_HEIGHT))
                                .size(Size::remainder())
                                .size(Size::exact(Self::DEFAULT_HEIGHT))
                                .horizontal(|mut strip| {
                                    strip.cell(|ui| {
                                        Self::draw_action(ui, l);
                                    });
                                    strip.cell(|ui| {
                                        Self::draw_title(ui, title);
                                    });
                                    strip.cell(|ui| {
                                        Self::draw_action(ui, r);
                                    });
                                });
                        });
                    });
            });
    }

    fn draw_action(ui: &mut egui::Ui, action: Option<TitleAction>) {
        if action.is_some() {
            let action = action.unwrap();
            ui.centered_and_justified(|ui| {
                View::title_button(ui, &action.icon, || {
                    (action.on_click)();
                });
            });
        }
    }

    fn draw_title(ui: &mut egui::Ui, title: String) {
        ui.add_space(2.0);
        ui.centered_and_justified(|ui| {
            View::ellipsize_text(ui, title.to_uppercase(), 20.0, Colors::TITLE);
        });
    }
}
