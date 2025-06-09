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

use egui::os::OperatingSystem;
use egui::RichText;

use crate::gui::Colors;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::settings::TorSettingsContent;
use crate::gui::views::types::ContentContainer;
use crate::gui::views::View;
use crate::tor::Tor;
use crate::wallet::Wallet;

/// Wallet transport settings content.
pub struct WalletTransportSettingsContent {
    /// Flag to check if settings were changed to restart Tor service.
    settings_changed: bool,

    /// Tor transport content settings.
    tor_settings_content: TorSettingsContent,
}

impl Default for WalletTransportSettingsContent {
    fn default() -> Self {
        Self {
            settings_changed: false,
            tor_settings_content: TorSettingsContent::default(),
        }
    }
}

impl WalletTransportSettingsContent {
    /// Draw transport settings content.
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              wallet: &Wallet,
              cb: &dyn PlatformCallbacks,
              on_close: impl FnOnce()) {
        ui.add_space(8.0);
        ui.vertical_centered(|ui| {
            // Do not show bridges settings on Android.
            let os = OperatingSystem::from_target_os();
            if os != OperatingSystem::Android {
                // Show Tor settings.
                self.tor_settings_content.ui(ui, cb);
                if !self.tor_settings_content.settings_changed {
                    self.settings_changed = true;
                }
                ui.add_space(4.0);
                View::horizontal_line(ui, Colors::item_stroke());
                ui.add_space(8.0);
            }
            ui.label(RichText::new(t!("transport.tor_autorun_desc"))
                .size(17.0)
                .color(Colors::inactive_text()));
            // Show Tor service autorun checkbox.
            let autorun = wallet.auto_start_tor_listener();
            View::checkbox(ui, autorun, t!("network.autorun"), || {
                wallet.update_auto_start_tor_listener(!autorun);
                self.settings_changed = true;
            });
        });
        ui.add_space(8.0);
        ui.vertical_centered_justified(|ui| {
            View::button(ui, t!("close"), Colors::white_or_black(false), || {
                if self.settings_changed {
                    self.settings_changed = false;
                    // Restart running service or rebuild client.
                    let service_id = &wallet.identifier();
                    if Tor::is_service_running(service_id) {
                        if let Ok(key) = wallet.secret_key() {
                            let api_port = wallet.foreign_api_port().unwrap();
                            Tor::restart_service(api_port, key, service_id);
                        }
                    } else {
                        Tor::rebuild_client();
                    }
                }
                on_close();
            });
        });
        ui.add_space(6.0);
    }
}