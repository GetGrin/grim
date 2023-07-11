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

use crate::gui::{App, PlatformApp};
use crate::gui::platform::PlatformCallbacks;

#[derive(Default)]
pub struct Desktop;

impl PlatformCallbacks for Desktop {
    fn show_keyboard(&self) {}

    fn hide_keyboard(&self) {}

    fn copy_string_to_buffer(&self, data: String) {}

    fn get_string_from_buffer(&self) -> String {
        "".to_string()
    }

    fn exit(&self) {}
}

impl PlatformApp<Desktop> {
    pub fn new(platform: Desktop) -> Self {
        Self {
            app: App::default(),
            platform,
        }
    }
}

impl eframe::App for PlatformApp<Desktop> {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.app.ui(ctx, frame, &self.platform);
    }

    fn on_close_event(&mut self) -> bool {
        self.app.exit_requested = true;
        self.app.exit_allowed
    }
}
