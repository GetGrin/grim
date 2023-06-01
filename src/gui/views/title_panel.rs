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

use crate::gui::colors::{COLOR_DARK, COLOR_YELLOW};
use crate::gui::views::View;

pub struct TitlePanelAction<'action> {
    pub(crate) icon: Box<&'action str>,
    pub(crate) on_click: Box<dyn Fn()>,
}

impl<'action> TitlePanelAction<'action> {
    pub fn new(icon: &'action str, on_click: fn()) -> Option<Self> {
        Option::from(Self { icon: Box::new(icon), on_click: Box::new(on_click) })
    }
}

pub struct TitlePanel {
    title: String,
}

impl TitlePanel {
    const PANEL_SIZE: f32 = 52.0;

    pub fn new(title: &String) -> Self {
        Self { title: title.to_uppercase() }
    }

    pub fn ui(&self, l: Option<TitlePanelAction>, r: Option<TitlePanelAction>, ui: &mut egui::Ui) {
        let Self { title } = self;

        egui::TopBottomPanel::top("title_panel")
            .resizable(false)
            .frame(egui::Frame {
                fill: COLOR_YELLOW,
                inner_margin: Margin::same(0.0),
                outer_margin: Margin::same(0.0),
                stroke: egui::Stroke::NONE,
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                StripBuilder::new(ui)
                    .size(Size::exact(Self::PANEL_SIZE))
                    .vertical(|mut strip| {
                        strip.strip(|builder| {
                            builder
                                .size(Size::exact(Self::PANEL_SIZE))
                                .size(Size::remainder())
                                .size(Size::exact(Self::PANEL_SIZE))
                                .horizontal(|mut strip| {
                                    strip.cell(|ui| {
                                        show_action(ui, l.as_ref());
                                    });
                                    strip.cell(|ui| {
                                        ui.centered_and_justified(|ui| {
                                            View::ellipsize_text(ui, title.into(), 20.0, COLOR_DARK);
                                        });
                                    });
                                    strip.cell(|ui| {
                                        show_action(ui, r.as_ref());
                                    });
                                });
                        });
                    });
            });
    }
}

fn show_action(ui: &mut egui::Ui, action: Option<&TitlePanelAction>) {
    if action.is_some() {
        let action = action.unwrap();
        ui.centered_and_justified(|ui| {
            View::title_button(ui, &action.icon, || {
                (action.on_click)();
            });
        });
    }
}

