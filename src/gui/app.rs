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

use egui::{Context, Modifiers};
use lazy_static::lazy_static;

use crate::AppConfig;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::Root;

lazy_static! {
    /// State to check if platform Back button was pressed.
    static ref BACK_BUTTON_PRESSED: AtomicBool = AtomicBool::new(false);
}

/// Implements ui entry point and contains platform-specific callbacks.
pub struct App<Platform> {
    /// Platform specific callbacks handler.
    pub(crate) platform: Platform,
    /// Main ui content.
    root: Root
}

impl<Platform: PlatformCallbacks> App<Platform> {
    pub fn new(platform: Platform) -> Self {
        Self { platform, root: Root::default() }
    }

    /// Draw application content.
    pub fn ui(&mut self, ctx: &Context) {
        // Handle Esc keyboard key event and platform Back button key event.
        let back_button_pressed = BACK_BUTTON_PRESSED.load(Ordering::Relaxed);
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, egui::Key::Escape)) || back_button_pressed {
            self.root.on_back();
            if back_button_pressed {
                BACK_BUTTON_PRESSED.store(false, Ordering::Relaxed);
            }
            // Request repaint to update previous content.
            ctx.request_repaint();
        }

        // Handle Close event (on desktop).
        if ctx.input(|i| i.viewport().close_requested()) {
            if !self.root.exit_allowed {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                Root::show_exit_modal();
            } else {
                ctx.input(|i| {
                    if let Some(rect) = i.viewport().inner_rect {
                        AppConfig::save_window_size(rect.width(), rect.height());
                    }
                    if let Some(rect) = i.viewport().outer_rect {
                        AppConfig::save_window_pos(rect.left(), rect.top());
                    }
                });
            }
        }

        // Show main content.
        egui::CentralPanel::default()
            .frame(egui::Frame {
                ..Default::default()
            })
            .show(ctx, |ui| {
                self.root.ui(ui, &self.platform);
            });
    }
}

/// To draw with egui`s eframe (for wgpu, glow backends and wasm target).
impl<Platform: PlatformCallbacks> eframe::App for App<Platform> {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        self.ui(ctx);
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