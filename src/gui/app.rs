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

use egui::Context;

use crate::gui::Colors;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::Root;

/// Implements ui entry point and contains platform-specific callbacks.
pub struct PlatformApp<Platform> {
    /// Platform specific callbacks handler.
    pub(crate) platform: Platform,
    /// Main ui content.
    root: Root
}

impl<Platform> PlatformApp<Platform> {
    pub fn new(platform: Platform) -> Self {
        Self { platform, root: Root::default() }
    }
}

impl<Platform: PlatformCallbacks> eframe::App for PlatformApp<Platform> {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        // Handle Esc keyboard key event.
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            Root::on_back();
        }

        // Show main content.
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: Colors::YELLOW,
                ..Default::default()
            })
            .show(ctx, |ui| {
                self.root.ui(ui, frame, &self.platform);
            });
    }

    fn on_close_event(&mut self) -> bool {
        Root::show_exit_modal();
        self.root.exit_allowed
    }
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Handle Back key code event from Android.
pub extern "C" fn Java_mw_gri_android_MainActivity_onBack(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    _activity: jni::objects::JObject,
) {
    Root::on_back();
}



