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

use egui::{RichText, TextStyle, Widget};

use crate::gui::{Colors, Navigator};
use crate::gui::icons::WRENCH;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, ModalPosition, View};
use crate::gui::views::network_settings::NetworkSettings;
use crate::node::NodeConfig;

/// Stratum server setup ui section.
pub struct StratumServerSetup {
    /// Stratum port  to be used inside edit modal.
    stratum_port_edit: String,
    /// Flag to check if stratum port is available inside edit modal.
    stratum_port_available_edit: bool,

    /// Flag to check if stratum port is available from saved config value.
    pub(crate) is_port_available: bool
}

impl Default for StratumServerSetup {
    fn default() -> Self {
        let (ip, port) = NodeConfig::get_stratum_address();
        let is_port_available = NodeConfig::is_stratum_port_available(&ip, &port);
        Self {
            stratum_port_edit: port,
            stratum_port_available_edit: is_port_available,
            is_port_available
        }
    }
}

impl StratumServerSetup {
    /// Identifier for stratum port [`Modal`].
    pub const STRATUM_PORT_MODAL: &'static str = "stratum_port";

    pub fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        View::sub_title(ui, format!("{} {}", WRENCH, t!("network_mining.server_setup")));
        View::horizontal_line(ui, Colors::ITEM_STROKE);
        ui.add_space(4.0);

        // Show message when IP addresses are not available on the system.
        let all_ips = NetworkSettings::get_ip_addrs();
        if all_ips.is_empty() {
            NetworkSettings::no_ip_address_ui(ui);
            return;
        }

        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.ip"))
                .size(16.0)
                .color(Colors::GRAY)
            );
            ui.add_space(6.0);
            // Show stratum IP addresses to select.
            let (ip, port) = NodeConfig::get_stratum_address();
            NetworkSettings::ip_addrs_ui(ui, &ip, &all_ips, |selected_ip| {
                self.is_port_available = NodeConfig::is_stratum_port_available(selected_ip, &port);
                NodeConfig::save_stratum_address(selected_ip, &port);
            });

            ui.label(RichText::new(t!("network_settings.port"))
                .size(16.0)
                .color(Colors::GRAY)
            );
            ui.add_space(6.0);
            // Show stratum port setup.
            self.port_setup_ui(ui, cb);

            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);
        });
    }

    /// Draw stratum port setup ui.
    fn port_setup_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        let (_, port) = NodeConfig::get_stratum_address();
        // Show button to enter stratum server port.
        View::button(ui, port.clone(), Colors::BUTTON, || {
            // Setup values for modal.
            self.stratum_port_edit = port;
            self.stratum_port_available_edit = self.is_port_available;
            // Show stratum port modal.
            let port_modal = Modal::new(Self::STRATUM_PORT_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"));
            Navigator::show_modal(port_modal);
            cb.show_keyboard();
        });
        ui.add_space(14.0);

        // Show error when stratum server port is unavailable.
        if !self.is_port_available {
            ui.label(RichText::new(t!("network_settings.port_unavailable"))
                .size(16.0)
                .color(Colors::RED));
            ui.add_space(12.0);
        }
    }

    /// Draw stratum port [`Modal`] content ui.
    pub fn stratum_port_modal_ui(&mut self,
                                 ui: &mut egui::Ui,
                                 modal: &Modal,
                                 cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.stratum_port"))
                .size(18.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw stratum port text edit.
            let text_edit_resp = egui::TextEdit::singleline(&mut self.stratum_port_edit)
                .font(TextStyle::Heading)
                .desired_width(58.0)
                .cursor_at_end(true)
                .ui(ui);
            text_edit_resp.request_focus();
            if text_edit_resp.clicked() {
                cb.show_keyboard();
            }

            // Show error when specified port is unavailable.
            if !self.stratum_port_available_edit {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.port_unavailable"))
                    .size(18.0)
                    .color(Colors::RED));
            }

            ui.add_space(12.0);

            // Show modal buttons.
            ui.scope(|ui| {
                // Setup spacing between buttons.
                ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

                // Save button callback
                let on_save = || {
                    // Check if port is available.
                    let (stratum_ip, _) = NodeConfig::get_stratum_address();
                    let available = NodeConfig::is_stratum_port_available(
                        &stratum_ip,
                        &self.stratum_port_edit
                    );
                    self.stratum_port_available_edit = available;

                    // Save port at config if it's available.
                    if available {
                        NodeConfig::save_stratum_address(&stratum_ip, &self.stratum_port_edit);

                        self.is_port_available = true;
                        cb.hide_keyboard();
                        modal.close();
                    }
                };

                ui.columns(2, |columns| {
                    columns[0].vertical_centered_justified(|ui| {
                        View::button(ui, t!("modal.cancel"), Colors::WHITE, || {
                            // Close modal.
                            cb.hide_keyboard();
                            modal.close();
                        });
                    });
                    columns[1].vertical_centered_justified(|ui| {
                        View::button(ui, t!("modal.save"), Colors::WHITE, on_save);
                    });
                });
                ui.add_space(6.0);
            });
        });
    }
}