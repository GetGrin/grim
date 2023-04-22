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
use eframe::epaint::FontFamily;
use eframe::Frame;
use egui::{Color32, Context, Direction, Id, Layout, RichText, Rounding, Sense, Separator, Stroke, Widget};
use egui::epaint::Shadow;
use egui::panel::PanelState;
use egui::style::Margin;
use egui_extras::Size;
use egui_extras::StripBuilder;
use wgpu::Color;
use crate::gui::*;
use crate::gui::screens::{Screen};

pub struct PlatformApp<Platform> {
    pub(crate) screens: Screens,
    pub(crate) platform: Platform,
}

pub struct Screens {
    screens: Vec<Box<dyn Screen>>,
    navigation: BTreeSet<String>,
    network_screen_open: bool,
}

impl Default for Screens {
    fn default() -> Self {
        Self::from_screens(vec![
            Box::new(screens::Wallets::default())
        ])
    }
}

impl Screens {
    fn from_screens(screens: Vec<Box<dyn Screen>>) -> Self {
        let current = screens[0].name().to_owned();
        let mut navigation = BTreeSet::new();
        navigation.insert(current);
        Self { screens, navigation, network_screen_open: false }
    }

    fn show_screens(&mut self, ui: &mut egui::Ui, frame: &mut Frame, cb: &dyn PlatformCallbacks) {
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
        return self.network_screen_open;
    }

    pub fn ui(&mut self, ctx: &Context, frame: &mut Frame, cb: &dyn PlatformCallbacks) {
        egui::CentralPanel::default()
            .frame(egui::Frame {
                inner_margin: Margin::same(0.0),
                outer_margin: Margin::same(0.0),
                fill: COLOR_YELLOW,
                .. Default::default()
            })
            .show(ctx, |ui| {
                let menu_open = self.menu_is_open(frame);

                let bg_stroke_default = ui.style().visuals.widgets.noninteractive.bg_stroke;
                ui.style_mut().visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;
                ui.style_mut().visuals.widgets.active.bg_stroke = Stroke::NONE;

                egui::SidePanel::left("menu_panel")
                    .resizable(false)
                    .exact_width(if !is_landscape(frame) {
                        frame.info().window_info.size.x - 58.0
                    } else {
                        frame.info().window_info.size.y - 58.0
                    })
                    .frame(egui::Frame {
                        inner_margin: Margin::same(0.0),
                        outer_margin: Margin::same(0.0),
                        rounding: Default::default(),
                        shadow: Shadow::NONE,
                        fill: COLOR_YELLOW,
                        stroke: Stroke::NONE,
                    })
                    .show_animated_inside(ui, menu_open, |ui| {
                        //TODO: Menu content
                        ui.vertical_centered(|ui| {
                            ui.heading("🖧 Node");
                        });

                        ui.separator();
                    });


                egui::TopBottomPanel::top("title_panel")
                    .resizable(false)
                    // .default_height(120.0)
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
                                                ui.centered_and_justified(|ui| {
                                                    let b = egui::widgets::Button::new(
                                                        RichText::new(if !menu_open {
                                                            SYM_NETWORK
                                                        } else {
                                                            if is_landscape(frame) {
                                                                SYM_MENU
                                                            } else {
                                                                SYM_WALLET
                                                            }
                                                        }).size(24.0)
                                                    ).fill(Color32::TRANSPARENT)
                                                        .ui(ui)
                                                        .interact(Sense::click_and_drag());
                                                    if b.drag_released() || b.clicked() {
                                                        self.network_screen_open = !menu_open
                                                    };
                                                });
                                            });
                                            if !menu_open || is_landscape(frame) {
                                                strip.strip(|builder| {
                                                    builder
                                                        .size(Size::remainder())
                                                        .vertical(|mut strip| {
                                                            strip.cell(|ui| {
                                                                ui.centered_and_justified(|ui| {
                                                                    ui.label(RichText::new(self.current_screen_title()
                                                                        .to_uppercase())
                                                                        .size(20.0)
                                                                        .color(Color32::BLACK)
                                                                    );
                                                                });
                                                            });
                                                        });
                                                });
                                                strip.cell(|ui| {
                                                    ui.centered_and_justified(|ui| {
                                                        let b = egui::widgets::Button::new(
                                                            RichText::new(SYM_ADD).size(24.0)
                                                        ).fill(Color32::TRANSPARENT).ui(ui).interact(Sense::click_and_drag());
                                                        if b.drag_released() || b.clicked() {
                                                            //TODO: Add wallet
                                                            //self.menu_open = !menu_open
                                                        };
                                                    });
                                                });
                                            }
                                        });
                                });
                            });
                    });

                // ctx.style_mut().visuals.widgets.noninteractive.bg_stroke = bg_stroke_default;


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

                if !menu_open || is_landscape(frame) {
                    egui::CentralPanel::default().frame(egui::containers::Frame {
                        inner_margin: Margin::same(3.0),
                        fill: Color32::from_gray(30),
                        stroke: Stroke::new(1.0, Color32::from_gray(5)),
                        ..Default::default()
                    })
                        .show_inside(ui, |ui| {
                            self.show_screens(ui, frame, cb);
                        });
                }
            });
    }
}

pub fn is_landscape(frame: &mut Frame) -> bool {
    return frame.info().window_info.size.x > frame.info().window_info.size.y
}

