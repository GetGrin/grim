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

use eframe::epaint::Stroke;
use egui::{Context, Frame};
use egui::style::Margin;

use crate::gui::colors::COLOR_LIGHT;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::screens::Root;

pub struct PlatformApp<Platform> {
    pub(crate) app: App,
    pub(crate) platform: Platform,
}

pub struct App {
    root: Root,
}

impl Default for App {
    fn default() -> Self {
        Self {
            root: Root::default(),
        }
    }
}

impl App {
    pub fn ui(&mut self, ctx: &Context, frame: &mut eframe::Frame, cb: &dyn PlatformCallbacks) {
        let Self { root } = self;
        egui::CentralPanel::default()
            .frame(Frame {
                inner_margin: Margin::same(0.0),
                outer_margin: Margin::same(0.0),
                stroke: Stroke::NONE,
                fill: COLOR_LIGHT,
                .. Default::default()
            })
            .show(ctx, |ui| {
                root.ui(ui, frame, cb)
            });
    }
}

pub fn is_dual_panel_mode(frame: &mut eframe::Frame) -> bool {
    is_landscape(frame) && frame.info().window_info.size.x > 400.0
}

pub fn is_landscape(frame: &mut eframe::Frame) -> bool {
    return frame.info().window_info.size.x > frame.info().window_info.size.y
}

