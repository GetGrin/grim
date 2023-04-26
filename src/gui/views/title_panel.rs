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


use egui::{Color32, RichText, Sense, Stroke, Widget};
use egui::style::Margin;
use egui_extras::{Size, StripBuilder};
use crate::gui::COLOR_YELLOW;
use crate::gui::views::View;

pub struct PanelAction {
    icon: Box<str>,
    on_click: Box<dyn Fn()>
}

#[derive(Default)]
pub struct PanelActions {
    left: Option<PanelAction>,
    right: Option<PanelAction>
}

#[derive(Default)]
pub struct TitlePanel {
    title: Option<String>,
    actions: PanelActions
}

impl TitlePanel {
    pub fn title(mut self, title: String) -> Self {
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
}

impl View for TitlePanel {
    fn ui(&mut self, ui: &mut egui::Ui) {
        // Disable stroke around panels
        let panel_stroke_default = ui.style().visuals.widgets.noninteractive.bg_stroke;
        ui.style_mut().visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;

        // Disable stroke around buttons on hover
        let button_hover_stroke_default = ui.style().visuals.widgets.active.bg_stroke;
        ui.style_mut().visuals.widgets.active.bg_stroke = Stroke::NONE;

        let Self { actions, title } = self;

        egui::TopBottomPanel::top("title_panel")
            .resizable(false)
            .frame(egui::Frame {
                fill: COLOR_YELLOW,
                inner_margin: Margin::same(0.0),
                outer_margin: Margin::same(0.0),
                rounding: Default::default(),
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                StripBuilder::new(ui)
                    .size(Size::exact(58.0))
                    .vertical(|mut strip| {
                        strip.strip(|builder| {
                            builder
                                .size(Size::exact(58.0))
                                .size(Size::remainder())
                                .size(Size::exact(58.0))
                                .horizontal(|mut strip| {
                                    strip.cell(|ui| {
                                        if actions.left.is_some() {
                                            let action = actions.left.as_ref().unwrap();
                                            ui.centered_and_justified(|ui| {
                                                let b = egui::widgets::Button::new(
                                                    RichText::new(&action.icon.to_string()).size(24.0)
                                                ).fill(Color32::TRANSPARENT)
                                                    .ui(ui)
                                                    .interact(Sense::click_and_drag());
                                                if b.drag_released() || b.clicked() {
                                                    (action.on_click)();
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
                                                            ui.label(RichText::new(
                                                                title.as_ref().unwrap().to_uppercase()
                                                            ).size(20.0).color(Color32::BLACK));
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
                                                    RichText::new(action.icon.to_string()).size(24.0)
                                                ).fill(Color32::TRANSPARENT)
                                                    .ui(ui).interact(Sense::click_and_drag());
                                                if b.drag_released() || b.clicked() {
                                                    (action.on_click)();
                                                };
                                            });
                                        }
                                    });
                                });
                        });
                    });
            });

        // Enable stroke around panels
        ui.style_mut().visuals.widgets.noninteractive.bg_stroke = panel_stroke_default;

        // Enable stroke around buttons on hover
        ui.style_mut().visuals.widgets.active.bg_stroke = button_hover_stroke_default;
    }
}