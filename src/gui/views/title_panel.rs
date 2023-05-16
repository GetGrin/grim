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
use crate::gui::views::common::title_button;

pub struct TitlePanelAction {
    pub(crate) icon: Box<str>,
    pub(crate) on_click: Box<dyn Fn(&mut Navigator)>,
}

#[derive(Default)]
pub struct TitlePanelActions {
    left: Option<TitlePanelAction>,
    right: Option<TitlePanelAction>
}

pub struct TitlePanel<'nav> {
    title: Option<String>,
    actions: TitlePanelActions,
    nav: &'nav mut Navigator
}

impl<'nav> TitlePanel<'nav> {
    pub fn new(nav: &'nav mut Navigator) -> TitlePanel<'nav> {
        Self {
            title: None,
            actions: Default::default(),
            nav,
        }
    }

    pub fn title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn left_action(mut self, action: TitlePanelAction) -> Self {
        self.actions.left = Some(action);
        self
    }

    pub fn right_action(mut self, action: TitlePanelAction) -> Self {
        self.actions.right = Some(action);
        self
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // Disable stroke around panel buttons on hover
        ui.style_mut().visuals.widgets.active.bg_stroke = Stroke::NONE;

        let Self { actions, title, nav } = self;

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
                                        Self::show_action(ui, actions.left.as_ref(), nav);
                                    });
                                    strip.strip(|builder| {
                                        builder
                                            .size(Size::remainder())
                                            .vertical(|mut strip| {
                                                strip.cell(|ui| {
                                                    Self::show_title(&*title, ui);
                                                });
                                            });
                                    });
                                    strip.cell(|ui| {
                                        Self::show_action(ui, actions.right.as_ref(), nav);
                                    });
                                });
                        });
                    });
            });
    }

    fn show_action(ui: &mut egui::Ui,
                   action: Option<&TitlePanelAction>,
                   navigator: &mut Navigator) {
        if action.is_some() {
            let action = action.unwrap();
            ui.centered_and_justified(|ui| {
                title_button(ui, &action.icon, || {
                    (action.on_click)(navigator);
                });
            });
        }
    }

    fn show_title(title: &Option<String>, ui: &mut egui::Ui) {
        if title.is_some() {
            ui.centered_and_justified(|ui| {
                let title_text = title.as_ref().unwrap().to_uppercase();
                let mut job = LayoutJob::single_section(title_text, TextFormat {
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
    }
}

