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

use egui::{Id, RichText, TextStyle, Widget};

use crate::gui::{Colors, Navigator};
use crate::gui::icons::{BARBELL, HARD_DRIVES, PLUG, TIMER};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, ModalPosition, View};
use crate::gui::views::network::node_settings::NetworkNodeSettings;
use crate::node::{Node, NodeConfig};

/// Stratum server setup ui section.
pub struct StratumServerSetup {
    /// Stratum port value to be used inside edit modal.
    stratum_port_edit: String,
    /// Flag to check if stratum port is available inside edit modal.
    stratum_port_available_edit: bool,

    /// Flag to check if stratum port is available from saved config value.
    pub(crate) is_port_available: bool,

    /// Attempt time value to be used inside edit modal.
    attempt_time_edit: String,

    /// Minimum share difficulty value to be used inside edit modal.
    min_share_diff_edit: String
}

impl Default for StratumServerSetup {
    fn default() -> Self {
        let (ip, port) = NodeConfig::get_stratum_address();
        let is_port_available = NodeConfig::is_stratum_port_available(&ip, &port);
        let attempt_time = NodeConfig::get_stratum_attempt_time();
        let min_share_diff = NodeConfig::get_stratum_min_share_diff();
        Self {
            stratum_port_edit: port,
            stratum_port_available_edit: is_port_available,
            is_port_available,
            attempt_time_edit: attempt_time,
            min_share_diff_edit: min_share_diff

        }
    }
}

impl StratumServerSetup {
    /// Identifier for stratum port [`Modal`].
    pub const STRATUM_PORT_MODAL: &'static str = "stratum_port";
    /// Identifier for attempt time [`Modal`].
    pub const STRATUM_ATTEMPT_TIME_MODAL: &'static str = "stratum_attempt_time";
    /// Identifier for minimum share difficulty [`Modal`].
    pub const STRATUM_MIN_SHARE_MODAL: &'static str = "stratum_min_share";

    pub fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        View::sub_title(ui, format!("{} {}", HARD_DRIVES, t!("network_mining.server_setup")));
        View::horizontal_line(ui, Colors::ITEM_STROKE);
        ui.add_space(6.0);

        ui.vertical_centered(|ui| {
            // Show button to enable stratum server if port is available and server is not running.
            if self.is_port_available && !Node::is_stratum_server_starting() && Node::is_running()
                && !Node::get_stratum_stats().is_running {
                ui.add_space(6.0);
                View::button(ui, t!("network_mining.enable_server"), Colors::GOLD, || {
                    Node::start_stratum_server();
                });
                ui.add_space(6.0);
            }

            // Show stratum server autorun checkbox.
            let stratum_enabled = NodeConfig::is_stratum_autorun_enabled();
            View::checkbox(ui, stratum_enabled, t!("network.autorun"), || {
                NodeConfig::toggle_stratum_autorun();
            });
            ui.add_space(4.0);

            // Show message to restart node after changing of stratum settings
            ui.label(RichText::new(t!("network_mining.info_settings"))
                .size(16.0)
                .color(Colors::INACTIVE_TEXT)
            );
            ui.add_space(8.0);
        });

        View::horizontal_line(ui, Colors::ITEM_STROKE);
        ui.add_space(6.0);

        // Show message when IP addresses are not available on the system.
        let all_ips = NodeConfig::get_ip_addrs();
        if all_ips.is_empty() {
            NetworkNodeSettings::no_ip_address_ui(ui);
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
            NetworkNodeSettings::ip_addrs_ui(ui, &ip, &all_ips, |selected_ip| {
                NodeConfig::save_stratum_address(selected_ip, &port);
                self.is_port_available = NodeConfig::is_stratum_port_available(selected_ip, &port);

            });
            // Show stratum port setup.
            self.port_setup_ui(ui, cb);

            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);

            // Show attempt time setup.
            self.attempt_time_ui(ui, cb);

            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);

            // Show minimum acceptable share difficulty setup.
            self.min_diff_ui(ui, cb);
        });
    }

    /// Draw stratum port value setup ui.
    fn port_setup_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("network_settings.port"))
            .size(16.0)
            .color(Colors::GRAY)
        );
        ui.add_space(6.0);

        let (_, port) = NodeConfig::get_stratum_address();
        View::button(ui, format!("{} {}", PLUG, port.clone()), Colors::BUTTON, || {
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
        ui.add_space(12.0);

        // Show error when stratum server port is unavailable.
        if !self.is_port_available {
            ui.add_space(6.0);
            ui.label(RichText::new(t!("network_settings.port_unavailable"))
                .size(16.0)
                .color(Colors::RED));
            ui.add_space(12.0);
        }
    }

    /// Draw stratum port [`Modal`] content ui.
    pub fn port_modal(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.stratum_port"))
                .size(18.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw stratum port text edit.
            let text_edit_resp = egui::TextEdit::singleline(&mut self.stratum_port_edit)
                .id(Id::from(modal.id))
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

    /// Draw attempt time value setup ui.
    fn attempt_time_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("network_settings.attempt_time_desc"))
            .size(16.0)
            .color(Colors::GRAY)
        );
        ui.add_space(6.0);

        let time = NodeConfig::get_stratum_attempt_time();
        View::button(ui, format!("{} {}", TIMER, time.clone()), Colors::BUTTON, || {
            // Setup values for modal.
            self.attempt_time_edit = time;

            // Show attempt time modal.
            let time_modal = Modal::new(Self::STRATUM_ATTEMPT_TIME_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"));
            Navigator::show_modal(time_modal);
            cb.show_keyboard();
        });
        ui.add_space(12.0);
    }

    /// Draw attempt time [`Modal`] content ui.
    pub fn attempt_modal(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.attempt_time"))
                .size(18.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw stratum port text edit.
            let text_edit_resp = egui::TextEdit::singleline(&mut self.attempt_time_edit)
                .id(Id::from(modal.id))
                .font(TextStyle::Heading)
                .desired_width(34.0)
                .cursor_at_end(true)
                .ui(ui);
            text_edit_resp.request_focus();
            if text_edit_resp.clicked() {
                cb.show_keyboard();
            }

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.attempt_time_edit.parse::<u32>().is_err() {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.not_valid_value"))
                    .size(18.0)
                    .color(Colors::RED));
            } else {
                NetworkNodeSettings::node_restart_required_ui(ui);
            }
            ui.add_space(12.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            // Save button callback
            let on_save = || {
                if let Ok(time) = self.attempt_time_edit.parse::<u32>() {
                    NodeConfig::save_stratum_attempt_time(time);
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
    }

    /// Draw minimum share difficulty value setup ui.
    fn min_diff_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("network_settings.min_share_diff"))
            .size(16.0)
            .color(Colors::GRAY)
        );
        ui.add_space(6.0);

        let diff = NodeConfig::get_stratum_min_share_diff();
        View::button(ui, format!("{} {}", BARBELL, diff.clone()), Colors::BUTTON, || {
            // Setup values for modal.
            self.min_share_diff_edit = diff;

            // Show attempt time modal.
            let diff_modal = Modal::new(Self::STRATUM_MIN_SHARE_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"));
            Navigator::show_modal(diff_modal);
            cb.show_keyboard();
        });
        ui.add_space(12.0);
    }

    /// Draw minimum acceptable share difficulty [`Modal`] content ui.
    pub fn min_diff_modal(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.min_share_diff"))
                .size(18.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw stratum port text edit.
            let text_edit_resp = egui::TextEdit::singleline(&mut self.min_share_diff_edit)
                .id(Id::from(modal.id))
                .font(TextStyle::Heading)
                .desired_width(34.0)
                .cursor_at_end(true)
                .ui(ui);
            text_edit_resp.request_focus();
            if text_edit_resp.clicked() {
                cb.show_keyboard();
            }

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.min_share_diff_edit.parse::<u64>().is_err() {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.not_valid_value"))
                    .size(18.0)
                    .color(Colors::RED));
            } else {
                NetworkNodeSettings::node_restart_required_ui(ui);
            }
            ui.add_space(12.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            // Save button callback
            let on_save = || {
                if let Ok(diff) = self.min_share_diff_edit.parse::<u64>() {
                    NodeConfig::save_stratum_min_share_diff(diff);
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
    }
}