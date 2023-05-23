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

use eframe::epaint::{FontId, Stroke};
use eframe::epaint::text::{LayoutJob, TextFormat, TextWrapping};
use egui::style::Margin;
use egui_extras::{Size, StripBuilder};

use crate::gui::colors::{COLOR_DARK, COLOR_YELLOW};
use crate::gui::screens::Navigator;
use crate::gui::views::View;

pub struct TitlePanelAction {
    pub(crate) icon: Box<str>,
    pub(crate) on_click: Box<dyn Fn()>,
}

impl TitlePanelAction {
    pub fn new(icon: Box<str>, on_click: fn()) -> Option<Self> {
        Option::from(Self { icon, on_click: Box::new(on_click) })
    }
}

#[derive(Default)]
pub struct TitlePanelActions {
    left: Option<TitlePanelAction>,
    right: Option<TitlePanelAction>
}

pub struct TitlePanel {
    title: String,
    actions: TitlePanelActions,
}

impl TitlePanel {
    pub fn new(title: String) -> Self {
        Self {
            title,
            actions: TitlePanelActions::default()
        }
    }

    pub fn left_action(mut self, action: Option<TitlePanelAction>) -> Self {
        self.actions.left = action;
        self
    }

    pub fn right_action(mut self, action: Option<TitlePanelAction>) -> Self {
        self.actions.right = action;
        self
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let Self { actions, title } = self;

        egui::TopBottomPanel::top("title_panel")
            .resizable(false)
            .frame(egui::Frame {
                fill: COLOR_YELLOW,
                inner_margin: Margin::same(0.0),
                outer_margin: Margin::same(0.0),
                stroke: Stroke::NONE,
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                StripBuilder::new(ui)
                    .size(Size::exact(52.0))
                    .vertical(|mut strip| {
                        strip.strip(|builder| {
                            builder
                                .size(Size::exact(52.0))
                                .size(Size::remainder())
                                .size(Size::exact(52.0))
                                .horizontal(|mut strip| {
                                    strip.cell(|ui| {
                                        show_action(ui, actions.left.as_ref());
                                    });
                                    strip.strip(|builder| {
                                        builder
                                            .size(Size::remainder())
                                            .vertical(|mut strip| {
                                                strip.cell(|ui| {
                                                    show_title(title, ui);
                                                });
                                            });
                                    });
                                    strip.cell(|ui| {
                                        show_action(ui, actions.right.as_ref());
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

fn show_title(title: &String, ui: &mut egui::Ui) {
    ui.centered_and_justified(|ui| {
        let mut job = LayoutJob::single_section(title.to_uppercase(), TextFormat {
            font_id: FontId::proportional(20.0),
            color: COLOR_DARK,
            .. Default::default()
        });
        job.wrap = TextWrapping {
            max_rows: 1,
            break_anywhere: false,
            overflow_character: Option::from('﹍'),
            ..Default::default()
        };
        ui.label(job);

    });
}

