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

use crate::gui::app::is_dual_panel_mode;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::screens::{Account, Accounts, Navigator, Screen, ScreenId};
use crate::gui::views::Network;

pub struct Root {
    screens: Vec<Box<dyn Screen>>,
    network: Network
}

impl Default for Root {
    fn default() -> Self {
        Navigator::init_from(ScreenId::Accounts);

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
        let is_network_panel_open = Navigator::is_side_panel_open() || is_dual_panel_mode(frame);

        egui::SidePanel::left("network_panel")
            .resizable(false)
            .exact_width(if is_dual_panel_mode(frame) {
                min(frame.info().window_info.size.x as i64, 400) as f32
            } else {
                frame.info().window_info.size.x
            })
            .frame(egui::Frame {
                .. Default::default()
            })
            .show_animated_inside(ui, is_network_panel_open, |ui| {
                self.network.ui(ui, frame, cb);
            });

        egui::CentralPanel::default().frame(egui::Frame {
            ..Default::default()
        }).show_inside(ui, |ui| {
            self.show_current_screen(ui, frame, cb);
        });

    }

    pub fn show_current_screen(&mut self,
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