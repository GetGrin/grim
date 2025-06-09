// Copyright 2025 The Grim Developers
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

use crate::gui::icons::GLOBE_SIMPLE;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::settings::{InterfaceSettingsContent, NetworkSettingsContent};
use crate::gui::views::types::ContentContainer;
use crate::gui::views::View;
use crate::gui::Colors;

/// Application settings content.
pub struct SettingsContent {
    /// User interface settings.
    interface_settings: InterfaceSettingsContent,
    /// Network communication settings.
    network_settings: NetworkSettingsContent,
    // tor_settings: TorSettingsContent,
}

impl Default for SettingsContent {
    fn default() -> Self {
        Self {
            interface_settings: InterfaceSettingsContent::default(),
            network_settings: NetworkSettingsContent::default(),
            //tor_settings: TorSettingsContent::default(),
        }
    }
}

impl SettingsContent {
    /// Draw application settings content.
    pub fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        // Show interface settings.
        self.interface_settings.ui(ui, cb);

        ui.add_space(8.0);
        View::horizontal_line(ui, Colors::stroke());
        ui.add_space(6.0);

        View::sub_title(ui, format!("{} {}", GLOBE_SIMPLE, t!("network.self")));
        View::horizontal_line(ui, Colors::stroke());
        ui.add_space(6.0);

        // Show network settings.
        self.network_settings.ui(ui, cb);
        ui.add_space(8.0);

        // Do not show Tor settings on Android.
        // let os = OperatingSystem::from_target_os();
        // let show_tor = os != OperatingSystem::Android;
        // if show_tor {
        //     View::horizontal_line(ui, Colors::stroke());
        //     ui.add_space(6.0);
        // 
        //     View::sub_title(ui, format!("{} {}", CIRCLE_HALF, t!("transport.tor_network")));
        //     View::horizontal_line(ui, Colors::stroke());
        //     ui.add_space(6.0);
        // 
        //     // Show Tor settings.
        //     self.tor_settings.ui(ui, cb);
        //     ui.add_space(8.0);
        // }
    }
}