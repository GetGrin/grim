// Copyright 2024 The Grim Developers
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
use egui::{Id, RichText};

use crate::gui::Colors;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{CameraContent, Modal, TextEdit, View};
use crate::tor::{Tor, TorBridge, TorConfig};
use crate::wallet::Wallet;

/// Transport settings [`Modal`] content.
pub struct TransportSettingsModal {
    /// Flag to check if Tor settings were changed.
    settings_changed: bool,

    /// Tor bridge binary path edit text.
    bridge_bin_path_edit: String,
    /// Tor bridge connection line edit text.
    bridge_conn_line_edit: String,
    /// Address QR code scanner [`Modal`] content.
    bridge_qr_scan_content: Option<CameraContent>,
}

impl Default for TransportSettingsModal {
    fn default() -> Self {
        // Setup Tor bridge binary path edit text.
        let bridge = TorConfig::get_bridge();
        let (bin_path, conn_line) = if let Some(b) = bridge {
            (b.binary_path(), b.connection_line())
        } else {
            ("".to_string(), "".to_string())
        };
        Self {
            settings_changed: false,
            bridge_bin_path_edit: bin_path,
            bridge_conn_line_edit: conn_line,
            bridge_qr_scan_content: None,
        }
    }
}

impl TransportSettingsModal {
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              wallet: &Wallet,
              modal: &Modal,
              cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);

        // Draw QR code scanner content if requested.
        if let Some(scanner) = self.bridge_qr_scan_content.as_mut() {
            let on_stop = || {
                cb.stop_camera();
                modal.enable_closing();
            };

            if let Some(result) = scanner.qr_scan_result() {
                self.bridge_conn_line_edit = result.text();
                on_stop();
                self.bridge_qr_scan_content = None;
            } else {
                scanner.ui(ui, cb);
                ui.add_space(12.0);

                // Setup spacing between buttons.
                ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

                // Show buttons to close modal or come back to sending input.
                ui.columns(2, |cols| {
                    cols[0].vertical_centered_justified(|ui| {
                        View::button(ui, t!("close"), Colors::white_or_black(false), || {
                            on_stop();
                            self.bridge_qr_scan_content = None;
                            Modal::close();
                        });
                    });
                    cols[1].vertical_centered_justified(|ui| {
                        View::button(ui, t!("back"), Colors::white_or_black(false), || {
                            on_stop();
                            self.bridge_qr_scan_content = None;
                        });
                    });
                });
                ui.add_space(6.0);
            }
            return;
        }

        // Do not show bridges setup on Android.
        let os = OperatingSystem::from_target_os();
        let show_bridges = os != OperatingSystem::Android;
        if show_bridges {
            let bridge = TorConfig::get_bridge();
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(t!("transport.bridges_desc"))
                    .size(17.0)
                    .color(Colors::inactive_text()));

                // Draw checkbox to enable/disable bridges.
                View::checkbox(ui, bridge.is_some(), t!("transport.bridges"), || {
                    // Save value.
                    let value = if bridge.is_some() {
                        None
                    } else {
                        let default_bridge = TorConfig::get_obfs4();
                        self.bridge_bin_path_edit = default_bridge.binary_path();
                        self.bridge_conn_line_edit = default_bridge.connection_line();
                        Some(default_bridge)
                    };
                    TorConfig::save_bridge(value);
                    self.settings_changed = true;
                });
            });

            // Draw bridges selection and path.
            if bridge.is_some() {
                let current_bridge = bridge.unwrap();
                let mut bridge = current_bridge.clone();

                ui.add_space(6.0);
                ui.columns(2, |columns| {
                    columns[0].vertical_centered(|ui| {
                        // Draw Obfs4 bridge selector.
                        let obfs4 = TorConfig::get_obfs4();
                        let name = obfs4.protocol_name().to_uppercase();
                        View::radio_value(ui, &mut bridge, obfs4, name);
                    });
                    columns[1].vertical_centered(|ui| {
                        // Draw Snowflake bridge selector.
                        let snowflake = TorConfig::get_snowflake();
                        let name = snowflake.protocol_name().to_uppercase();
                        View::radio_value(ui, &mut bridge, snowflake, name);
                    });
                });
                ui.add_space(12.0);

                // Check if bridge type was changed to save.
                if current_bridge != bridge {
                    self.settings_changed = true;
                    TorConfig::save_bridge(Some(bridge.clone()));
                    self.bridge_bin_path_edit = bridge.binary_path();
                    self.bridge_conn_line_edit = bridge.connection_line();
                }

                // Draw binary path text edit.
                let bin_edit_id = Id::from(modal.id)
                    .with(wallet.get_config().id)
                    .with("_bin_edit");
                let mut bin_edit = TextEdit::new(bin_edit_id)
                    .no_soft_keyboard()
                    .paste()
                    .focus(false);
                let bin_edit_before = self.bridge_bin_path_edit.clone();
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(t!("transport.bin_file"))
                        .size(17.0)
                        .color(Colors::inactive_text()));
                    ui.add_space(6.0);
                    bin_edit.ui(ui, &mut self.bridge_bin_path_edit, cb);
                    ui.add_space(6.0);
                });

                // Draw connection line text edit.
                let conn_edit_before = self.bridge_conn_line_edit.clone();
                let conn_edit_id = Id::from(modal.id)
                    .with(wallet.get_config().id)
                    .with("_conn_edit");
                let mut conn_edit = TextEdit::new(conn_edit_id)
                    .no_soft_keyboard()
                    .paste()
                    .focus(false)
                    .scan_qr();
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(t!("transport.conn_line"))
                        .size(17.0)
                        .color(Colors::inactive_text()));
                    ui.add_space(6.0);
                    conn_edit.ui(ui, &mut self.bridge_conn_line_edit, cb);
                    // Check if scan button was pressed.
                    if conn_edit.scan_pressed {
                        modal.disable_closing();
                        self.bridge_qr_scan_content = Some(CameraContent::default());
                    }
                });

                // Check if bin path or connection line text was changed to save bridge.
                if conn_edit_before != self.bridge_conn_line_edit ||
                    bin_edit_before != self.bridge_bin_path_edit {
                    let bin_path = self.bridge_bin_path_edit.trim().to_string();
                    let conn_line = self.bridge_conn_line_edit.trim().to_string();
                    let b = match bridge {
                        TorBridge::Snowflake(_, _) => {
                            TorBridge::Snowflake(bin_path, conn_line)
                        },
                        TorBridge::Obfs4(_, _) => {
                            TorBridge::Obfs4(bin_path, conn_line)
                        }
                    };
                    TorConfig::save_bridge(Some(b));
                    self.settings_changed = true;
                }

                ui.add_space(2.0);
            }

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(6.0);
        }

        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("transport.tor_autorun_desc"))
                .size(17.0)
                .color(Colors::inactive_text()));

            // Show Tor service autorun checkbox.
            let autorun = wallet.auto_start_tor_listener();
            View::checkbox(ui, autorun, t!("network.autorun"), || {
                wallet.update_auto_start_tor_listener(!autorun);
            });
        });
        ui.add_space(6.0);
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
                Modal::close();
            });
        });
        ui.add_space(6.0);
    }
}