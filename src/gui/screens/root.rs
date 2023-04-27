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

use std::cmp::min;
use eframe::epaint::{Shadow, Stroke};
use eframe::Frame;
use egui::style::Margin;
use egui::Ui;
use crate::gui::{App, COLOR_YELLOW};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::screens::{Account, Accounts, Navigator, Screen, ScreenId};

pub struct Root {
    navigator: Navigator,
    screens: Vec<Box<dyn Screen>>,
}

impl Default for Root {
    fn default() -> Self {
        Self {
            navigator: Navigator::default(),
            screens: (vec![
                Box::new(Accounts::default()),
                Box::new(Account::default())
            ]),
        }
    }
}

impl Root {
    fn id(&self) -> ScreenId {
        ScreenId::Root
    }

    pub fn ui(&mut self, ui: &mut Ui, frame: &mut Frame, cb: &dyn PlatformCallbacks) {
        let is_network_panel_open = self.navigator.left_panel_open || dual_panel_mode(frame);

        egui::SidePanel::left("network_panel")
            .resizable(false)
            .exact_width(if dual_panel_mode(frame) {
                min(frame.info().window_info.size.x as i64, 400) as f32
            } else {
                frame.info().window_info.size.x
            })
            .frame(egui::Frame {
                inner_margin: Margin::same(0.0),
                outer_margin: Margin::same(0.0),
                fill: COLOR_YELLOW,
                .. Default::default()
            })
            .show_animated_inside(ui, is_network_panel_open, |ui| {
                //TODO: Network content
                ui.vertical_centered(|ui| {
                    ui.heading("🖧 Node");
                });

                ui.separator();
            });

        egui::CentralPanel::default().frame(egui::containers::Frame {
            ..Default::default()
        }).show_inside(ui, |ui| {
            self.show_current_screen(ui, frame, cb);
        });

    }

    pub fn show_current_screen(&mut self, ui: &mut Ui, frame: &mut Frame, cb: &dyn PlatformCallbacks) {
        let Self { navigator, screens, .. } = self;
        let current = navigator.stack.last().unwrap();
        for screen in screens.iter_mut() {
            if screen.id() == *current {
                screen.show(ui, frame, navigator, cb);
                break;
            }
        }
    }
}

pub fn dual_panel_mode(frame: &mut Frame) -> bool {
    is_landscape(frame) && frame.info().window_info.size.x > 400.0
}

pub fn is_landscape(frame: &mut Frame) -> bool {
    return frame.info().window_info.size.x > frame.info().window_info.size.y
}