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

use std::sync::atomic::{AtomicBool, Ordering};

use egui::Context;
use lazy_static::lazy_static;

use crate::gui::Colors;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::Root;

lazy_static! {
    /// State to check if platform Back button was pressed.
    static ref BACK_BUTTON_PRESSED: AtomicBool = AtomicBool::new(false);
}

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
        // Handle Esc keyboard key event and platform Back button key event.
        let back_button_pressed = BACK_BUTTON_PRESSED.load(Ordering::Relaxed);
        if ctx.input(|i| i.key_pressed(egui::Key::Escape) || back_button_pressed) {
            if back_button_pressed {
                BACK_BUTTON_PRESSED.store(false, Ordering::Relaxed);
            }
            self.root.on_back();
            // Request repaint to update previous content.
            ctx.request_repaint();
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
        let exit =  self.root.exit_allowed;
        if !exit {
            Root::show_exit_modal();
        }
        exit
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
    BACK_BUTTON_PRESSED.store(true, Ordering::Relaxed);
}