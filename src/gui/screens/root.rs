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

use crate::gui::Navigator;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::screens::{Account, Accounts, Screen, ScreenId};
use crate::gui::views::{Network, View};

pub struct Root {
    screens: Vec<Box<dyn Screen>>,
    network: Network
}

impl Default for Root {
    fn default() -> Self {
        Navigator::init(ScreenId::Accounts);

        Self {
            screens: (vec![
                Box::new(Accounts::default()),
                Box::new(Account::default())
            ]),
            network: Network::default()
        }
    }
}

impl Root {
    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, cb: &dyn PlatformCallbacks) {
        let (is_panel_open, panel_width) = dual_panel_state_width(frame);
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
                self.show_current_screen(ui, frame, cb);
            });
    }

    fn show_current_screen(&mut self,
                               ui: &mut egui::Ui,
                               frame: &mut eframe::Frame,
                               cb: &dyn PlatformCallbacks) {
        let Self { screens, .. } = self;
        for screen in screens.iter_mut() {
            if Navigator::is_current(&screen.id()) {
                screen.ui(ui, frame, cb);
                break;
            }
        }
    }
}

/// Get dual panel state and width
fn dual_panel_state_width(frame: &mut eframe::Frame) -> (bool, f32) {
    let dual_panel_mode = View::is_dual_panel_mode(frame);
    let is_panel_open = dual_panel_mode || Navigator::is_side_panel_open();
    let panel_width = if dual_panel_mode {
        min(frame.info().window_info.size.x as i64, View::SIDE_PANEL_MIN_WIDTH) as f32
    } else {
        frame.info().window_info.size.x
    };
    (is_panel_open, panel_width)
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn Java_mw_gri_android_MainActivity_onBackButtonPress(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    _activity: jni::objects::JObject,
) {
    Navigator::back();
}