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
use std::sync::atomic::{AtomicBool, Ordering};

use lazy_static::lazy_static;
use crate::gui::App;

use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{AccountsContent, Modal, NetworkContent};

lazy_static! {
    /// To check if side panel is open from any part of ui.
    static ref NETWORK_PANEL_OPEN: AtomicBool = AtomicBool::new(false);
}

/// Main ui content, handles network panel state modal state.
#[derive(Default)]
pub struct Root {
    network: NetworkContent,
    accounts: AccountsContent,
}

impl Root {
    /// Default width of side panel at application UI.
    pub const SIDE_PANEL_MIN_WIDTH: i64 = 400;

    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, cb: &dyn PlatformCallbacks) {
        let (is_panel_open, panel_width) = Self::side_panel_state_width(frame);
        egui::SidePanel::left("network_panel")
            .resizable(false)
            .exact_width(panel_width)
            .frame(egui::Frame::default())
            .show_animated_inside(ui, is_panel_open, |ui| {
                self.network.ui(ui, frame, cb);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::default())
            .show_inside(ui, |ui| {
                self.accounts.ui(ui, frame, cb);
            });
    }

    /// Get side panel state and width.
    fn side_panel_state_width(frame: &mut eframe::Frame) -> (bool, f32) {
        let dual_panel_mode = Self::is_dual_panel_mode(frame);
        let is_panel_open = dual_panel_mode || Self::is_network_panel_open();
        let panel_width = if dual_panel_mode {
            min(frame.info().window_info.size.x as i64, Self::SIDE_PANEL_MIN_WIDTH) as f32
        } else {
            frame.info().window_info.size.x
        };
        (is_panel_open, panel_width)
    }

    /// Check if ui can show [`NetworkContent`] and [`AccountsContent`] at same time.
    pub fn is_dual_panel_mode(frame: &mut eframe::Frame) -> bool {
        let w = frame.info().window_info.size.x;
        let h = frame.info().window_info.size.y;
        // Screen is wide if width is greater than height or just 20% smaller.
        let is_wide_screen = w > h || w + (w * 0.2) >= h;
        // Dual panel mode is available when window is wide and its width is at least 2 times
        // greater than minimal width of the side panel.
        is_wide_screen && w >= Self::SIDE_PANEL_MIN_WIDTH as f32 * 2.0
    }

    /// Toggle [`Network`] panel state.
    pub fn toggle_network_panel() {
        let is_open = NETWORK_PANEL_OPEN.load(Ordering::Relaxed);
        NETWORK_PANEL_OPEN.store(!is_open, Ordering::Relaxed);
    }

    /// Check if side panel is open.
    pub fn is_network_panel_open() -> bool {
        NETWORK_PANEL_OPEN.load(Ordering::Relaxed)
    }

    /// Handle back button press event.
    fn on_back() {
        if Modal::on_back() {
            App::show_exit_modal()
        }
    }
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Handle back button press event from Android.
pub extern "C" fn Java_mw_gri_android_MainActivity_onBackButtonPress(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    _activity: jni::objects::JObject,
) {
    Root::on_back();
}