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

use std::any::Any;
use std::collections::BTreeSet;
use eframe::emath::Align;
use eframe::Frame;
use egui::{Color32, Context, Id, Layout, RichText, Sense, Separator, Stroke, Ui, Widget};
use egui::epaint::Shadow;
use egui::panel::PanelState;
use egui::style::Margin;
use wgpu::Color;
use crate::gui::PlatformCallbacks;
use crate::gui::screens::{Screen};

pub struct PlatformApp<Platform> {
    pub(crate) screens: Screens,
    pub(crate) platform: Platform,
}

pub struct Screens {
    screens: Vec<Box<dyn Screen>>,
    navigation: BTreeSet<String>,
    menu_open: bool,
}

impl Default for Screens {
    fn default() -> Self {
        Self::from_screens(vec![
            Box::new(super::screens::Wallets::default())
        ])
    }
}

impl Screens {
    fn from_screens(screens: Vec<Box<dyn Screen>>) -> Self {
        let current = screens[0].name().to_owned();
        let mut navigation = BTreeSet::new();
        navigation.insert(current);
        Self { screens, navigation, menu_open: false }
    }

    fn show_screens(&mut self, ui: &mut Ui, frame: &mut Frame, cb: &dyn PlatformCallbacks) {
        let Self { screens, navigation, .. } = self;
        for screen in screens {
            let id = screen.name();
            let show = navigation.contains(id.as_str());
            if show {
                screen.show(ui, frame, cb);
            }
            Self::set_show_screen(navigation, id, show);
        }
    }

    fn set_show_screen(navigation: &mut BTreeSet<String>, key: String, show: bool) {
        if show {
            if !navigation.contains(key.as_str()) {
                navigation.insert(key.to_owned());
            }
        } else {
            navigation.remove(key.as_str());
        }
    }

    fn current_screen_title(&self) -> &String {
        self.navigation.last().unwrap()
    }

    fn menu_is_open(&mut self, frame: &mut Frame) -> bool {
        return self.menu_open || is_landscape(frame);
    }

    pub fn ui(&mut self, ui: &mut Ui, frame: &mut Frame, cb: &dyn PlatformCallbacks) {
        let menu_open = self.menu_is_open(frame);

        egui::TopBottomPanel::top("title_panel")
            .resizable(false)
            // .default_height(120.0)
            .frame(egui::Frame {
                inner_margin: Margin::same(0.0),
                outer_margin: Margin::same(0.0),
                rounding: Default::default(),
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    let b = egui::widgets::Button::new(
                        RichText::new(" + ").size(38.0)
                    ).fill(Color32::TRANSPARENT).ui(ui).interact(Sense::click_and_drag());
                    if b.drag_released() || b.clicked() {
                        //TODO: Add wallet
                        //self.menu_open = !menu_open
                    };
                });
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    ui.heading(self.current_screen_title())
                });
                ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                    let b = egui::widgets::Button::new(
                        RichText::new(" = ").size(38.0)
                    ).fill(Color32::TRANSPARENT).ui(ui).interact(Sense::click_and_drag());
                    if b.drag_released() || b.clicked() {
                        self.menu_open = !menu_open
                    };
                });
            });

        egui::CentralPanel::default().frame(egui::containers::Frame{
            outer_margin: Margin {
                left: 0.0,
                right: 0.0,
                // top: 120.0,
                top: 0.0,
                bottom: 0.0,
            },
            inner_margin: Margin::same(3.0),
            fill: ui.ctx().style().visuals.panel_fill,
            ..Default::default()
        })
            .show_inside(ui, |ui| {
            self.show_screens(ui, frame, cb);
        });

        ui.style_mut().visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;

        // egui::SidePanel::right("screens_content")
        //     .resizable(false)
        //     .min_width(frame.info().window_info.size.x)
        //     .frame(egui::Frame {
        //         inner_margin: Margin::same(3.0),
        //         outer_margin: Margin::same(0.0),
        //         rounding: Default::default(),
        //         shadow: Shadow::NONE,
        //         fill: Color32::KHAKI,
        //         stroke: Stroke::NONE,
        //     })
        //     .show_inside(ui, |ui| {
        //         self.show_screens(ui, frame, cb);
        //     });

        egui::SidePanel::left("menu_panel")
            .resizable(false)
            .frame(egui::Frame {
                inner_margin: Margin::same(0.0),
                outer_margin: Margin::same(0.0),
                rounding: Default::default(),
                shadow: Shadow::NONE,
                fill: Color32::YELLOW,
                stroke: Stroke::NONE,
            })
            .show_animated_inside(ui, menu_open, |ui| {
                //TODO: Menu content
                ui.vertical_centered(|ui| {
                    ui.heading("💻 Backend");
                });

                ui.separator();
            });
    }
}

pub fn is_landscape(frame: &mut Frame) -> bool {
    return frame.info().window_info.size.x > frame.info().window_info.size.y
}

