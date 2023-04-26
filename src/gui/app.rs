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

use eframe::Frame;
use egui::{Color32, Context, Stroke};
use egui::epaint::Shadow;
use egui::style::Margin;
use wgpu::Color;

use crate::gui::COLOR_YELLOW;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::screens::{Root, Screen};

pub struct PlatformApp<Platform> {
    pub(crate) app: App,
    pub(crate) platform: Platform,
}

pub struct App {
    root: Root,
    network_panel_open: bool
}

impl Default for App {
    fn default() -> Self {
        Self {
            root: Root::default(),
            network_panel_open: false
        }
    }
}

impl App {
    pub fn ui(&mut self, ctx: &Context, frame: &mut Frame, cb: &dyn PlatformCallbacks) {
        let network_panel_open = self.network_panel_open || dual_panel_available(frame);

        egui::CentralPanel::default()
            .frame(egui::Frame {
                inner_margin: Margin::same(0.0),
                outer_margin: Margin::same(0.0),
                fill: COLOR_YELLOW,
                .. Default::default()
            })
            .show(ctx, |ui| {

                egui::SidePanel::left("network_panel")
                    .resizable(false)
                    .exact_width(if dual_panel_available(frame) {
                        min(frame.info().window_info.size.x as i64, 500) as f32
                    } else {
                        frame.info().window_info.size.x
                    })
                    .frame(egui::Frame {
                        inner_margin: Margin::same(0.0),
                        outer_margin: Margin::same(0.0),
                        rounding: Default::default(),
                        shadow: Shadow::NONE,
                        fill: COLOR_YELLOW,
                        stroke: Stroke::NONE,
                    })
                    .show_animated_inside(ui, network_panel_open, |ui| {
                        //TODO: Network content
                        ui.vertical_centered(|ui| {
                            ui.heading("🖧 Node");
                        });

                        ui.separator();
                    });

                egui::CentralPanel::default().frame(egui::containers::Frame {
                    inner_margin: Margin::same(3.0),
                    stroke: Stroke::new(1.0, Color32::from_gray(5)),
                    ..Default::default()
                }).show_inside(ui, |ui| {
                    self.root.ui(ui, cb);
                });
            });
    }
}

pub fn dual_panel_available(frame: &mut Frame) -> bool {
    is_landscape(frame) && frame.info().window_info.size.x > 500.0
}

pub fn is_landscape(frame: &mut Frame) -> bool {
    return frame.info().window_info.size.x > frame.info().window_info.size.y
}

