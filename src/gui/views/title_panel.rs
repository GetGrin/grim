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


use eframe::epaint::text::{LayoutJob, TextFormat, TextWrapping};
use egui::{Color32, FontId, RichText, Sense, Stroke, Widget};
use egui::style::Margin;
use egui_extras::{Size, StripBuilder};
use crate::gui::{COLOR_DARK, COLOR_YELLOW};
use crate::gui::screens::Navigator;
use crate::gui::views::View;

pub struct PanelAction {
    pub(crate) icon: Box<str>,
    pub(crate) on_click: Box<dyn Fn(&mut Option<&mut Navigator>)>,
}

#[derive(Default)]
pub struct PanelActions {
    left: Option<PanelAction>,
    right: Option<PanelAction>
}

#[derive(Default)]
pub struct TitlePanel<'screen> {
    title: Option<&'screen String>,
    actions: PanelActions,
    navigator: Option<&'screen mut Navigator>
}

impl<'screen> TitlePanel<'screen> {
    pub fn title(mut self, title: &'screen String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn left_action(mut self, action: PanelAction) -> Self {
        self.actions.left = Some(action);
        self
    }

    pub fn right_action(mut self, action: PanelAction) -> Self {
        self.actions.right = Some(action);
        self
    }

    pub fn with_navigator(mut self, nav: &'screen mut Navigator) -> Self {
        self.navigator = Some(nav);
        self
    }
}

impl View for TitlePanel<'_> {
    fn ui(&mut self, ui: &mut egui::Ui) {
        // Disable stroke around panel
        ui.style_mut().visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;

        // Disable stroke around buttons on hover
        ui.style_mut().visuals.widgets.active.bg_stroke = Stroke::NONE;

        let Self { actions, title, navigator } = self;

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
                                        if actions.left.is_some() {
                                            let action = actions.left.as_ref().unwrap();
                                            ui.centered_and_justified(|ui| {
                                                let b = egui::widgets::Button::new(
                                                    RichText::new(&action.icon.to_string())
                                                        .size(24.0)
                                                        .color(COLOR_DARK)
                                                ).fill(Color32::TRANSPARENT)
                                                    .ui(ui)
                                                    .interact(Sense::click_and_drag());
                                                if b.drag_released() || b.clicked() {
                                                    (action.on_click)(navigator);
                                                };
                                            });
                                        }
                                    });
                                    strip.strip(|builder| {
                                        builder
                                            .size(Size::remainder())
                                            .vertical(|mut strip| {
                                                strip.cell(|ui| {
                                                    if title.is_some() {
                                                        ui.centered_and_justified(|ui| {
                                                            show_title(title.as_ref().unwrap(), ui);
                                                        });
                                                    }
                                                });
                                            });
                                    });
                                    strip.cell(|ui| {
                                        if actions.right.is_some() {
                                            let action = actions.right.as_ref().unwrap();
                                            ui.centered_and_justified(|ui| {
                                                let b = egui::widgets::Button::new(
                                                    RichText::new(action.icon.to_string())
                                                        .size(24.0)
                                                        .color(COLOR_DARK)
                                                ).fill(Color32::TRANSPARENT)
                                                    .ui(ui).interact(Sense::click_and_drag());
                                                if b.drag_released() || b.clicked() {
                                                    (action.on_click)(navigator);
                                                };
                                            });
                                        }
                                    });
                                });
                        });
                    });
            });
    }
}

fn show_title(title: &String, ui: &mut egui::Ui) {
    let mut job = LayoutJob::single_section(title.to_uppercase(), TextFormat {
        font_id: FontId::proportional(20.0),
        color: COLOR_DARK,
        .. Default::default()
    });
    job.wrap = TextWrapping {
        max_rows: 1,
        break_anywhere: false,
        overflow_character: Option::from('…'),
        ..Default::default()
    };
    ui.label(job);
}