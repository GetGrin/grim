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

use std::net::IpAddr;
use std::str::FromStr;
use egui::{RichText, TextStyle, Widget};
use crate::gui::{Colors, Navigator};
use crate::gui::icons::{COMPUTER_TOWER, POWER};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, ModalPosition, Network, View};
use crate::node::{Node, NodeConfig};

/// Integrated node server setup ui section.
pub struct NodeSetup {
    /// API port to be used inside edit modal.
    api_port_edit: String,
    /// Flag to check if API port is available inside edit modal.
    api_port_available_edit: bool,

    /// Flag to check if API port is available from saved config value.
    pub(crate) is_api_port_available: bool,

    /// API secret to be used inside edit modal.
    api_secret_edit: String,

    /// Foreign API secret to be used inside edit modal.
    foreign_api_secret_edit: String,

    /// Future Time Limit to be used inside edit modal.
    ftl_edit: String,
}

impl Default for NodeSetup {
    fn default() -> Self {
        let (api_ip, api_port) = NodeConfig::get_api_address_port();
        let is_api_port_available = NodeConfig::is_api_port_available(&api_ip, &api_port);
        Self {
            api_port_edit: api_port,
            api_port_available_edit: is_api_port_available,
            is_api_port_available,
            api_secret_edit: "".to_string(),
            foreign_api_secret_edit: "".to_string(),
            ftl_edit: "".to_string(),
        }
    }
}

const SECRET_SYMBOLS: &'static str = "••••••••••••";

impl NodeSetup {
    pub const API_PORT_MODAL: &'static str = "api_port";

    pub fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        View::sub_title(ui, format!("{} {}", COMPUTER_TOWER, t!("network_settings.server")));
        View::horizontal_line(ui, Colors::ITEM_STROKE);
        ui.add_space(4.0);

        // Show loading indicator or controls to stop/start/restart node.
        if Node::is_stopping() || Node::is_restarting() || Node::is_starting() {
            ui.vertical_centered(|ui| {
                ui.add_space(6.0);
                View::small_loading_spinner(ui);
            });
        } else {
            if Node::is_running() {
                ui.scope(|ui| {
                    // Setup spacing between buttons.
                    ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 0.0);
                    ui.add_space(6.0);

                    ui.columns(2, |columns| {
                        columns[0].vertical_centered_justified(|ui| {
                            View::button(ui, t!("network_settings.disable"), Colors::GOLD, || {
                                Node::stop(false);
                            });
                        });
                        columns[1].vertical_centered_justified(|ui| {
                            View::button(ui, t!("network_settings.restart"), Colors::GOLD, || {
                                Node::restart();
                            });
                        });
                    });
                });
            } else {
                ui.add_space(6.0);
                ui.vertical_centered(|ui| {
                    let enable_text = format!("{} {}", POWER, t!("network_settings.enable"));
                    View::button(ui, enable_text, Colors::GOLD, || {
                        Node::start();
                    });
                });
            }
        }

        ui.add_space(4.0);
        ui.vertical_centered(|ui| {
            Network::autorun_node_checkbox(ui);
        });
        ui.add_space(4.0);

        let addrs = Network::get_ip_list();
        // Show message when IP addresses are not available on the system.
        if addrs.is_empty() {
            Network::no_ip_address_ui(ui);
            ui.add_space(4.0);
        } else {
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(4.0);

            ui.vertical_centered(|ui| {
                ui.label(RichText::new(t!("network_settings.api_ip"))
                    .size(16.0)
                    .color(Colors::GRAY)
                );
                ui.add_space(6.0);
                // Show API IP addresses to select.
                let (api_ip, api_port) = NodeConfig::get_api_address_port();
                Network::ip_list_ui(ui, &api_ip, &addrs, |selected_ip| {
                    println!("12345 selected_ip {}", selected_ip);
                    let api_available = NodeConfig::is_api_port_available(selected_ip, &api_port);
                    println!("12345 selected_ip is_api_port_available {}", api_available);
                    self.is_api_port_available = api_available;
                    println!("12345 before save");
                    NodeConfig::save_api_address_port(selected_ip, &api_port);
                    println!("12345 after save");
                });

                ui.label(RichText::new(t!("network_settings.api_port"))
                    .size(16.0)
                    .color(Colors::GRAY)
                );
                ui.add_space(6.0);
                // Show API port setup.
                self.api_port_setup_ui(ui, cb);

                View::horizontal_line(ui, Colors::ITEM_STROKE);
                ui.add_space(6.0);
            });
        }
    }

    /// Draw API port setup ui.
    fn api_port_setup_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        let (_, port) = NodeConfig::get_api_address_port();
        // Show button to choose API server port.
        View::button(ui, port.clone(), Colors::BUTTON, || {
            // Setup values for modal.
            self.api_port_edit = port;
            self.api_port_available_edit = self.is_api_port_available;

            // Show API port modal.
            let port_modal = Modal::new(Self::API_PORT_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_port"));
            Navigator::show_modal(port_modal);
            cb.show_keyboard();
        });
        ui.add_space(6.0);

        if !self.is_api_port_available {
            // Show error when API server port is unavailable.
            ui.label(RichText::new(t!("network_settings.port_unavailable"))
                .size(16.0)
                .color(Colors::RED));
            ui.add_space(6.0);
        } else if Node::is_running() {
            // Show reminder to restart node if settings are changed.
            ui.label(RichText::new(t!("network_settings.restart_node_required"))
                .size(16.0)
                .color(Colors::INACTIVE_TEXT)
            );
            ui.add_space(6.0);
        }
    }

    /// Draw API port [`Modal`] content ui.
    pub fn api_port_modal_ui(&mut self,
                                 ui: &mut egui::Ui,
                                 modal: &Modal,
                                 cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.enter_value"))
                .size(16.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw API port text edit.
            let text_edit_resp = egui::TextEdit::singleline(&mut self.api_port_edit)
                .font(TextStyle::Heading)
                .desired_width(58.0)
                .cursor_at_end(true)
                .ui(ui);
            text_edit_resp.request_focus();
            if text_edit_resp.clicked() {
                cb.show_keyboard();
            }

            // Show error when specified port is unavailable.
            if !self.api_port_available_edit {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.port_unavailable"))
                    .size(16.0)
                    .color(Colors::RED));
            }

            ui.add_space(12.0);

            // Show modal buttons.
            ui.scope(|ui| {
                // Setup spacing between buttons.
                ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 0.0);

                ui.columns(2, |columns| {
                    columns[0].vertical_centered_justified(|ui| {
                        View::button(ui, t!("modal.cancel"), Colors::WHITE, || {
                            cb.hide_keyboard();
                            modal.close();
                        });
                    });
                    columns[1].vertical_centered_justified(|ui| {
                        View::button(ui, t!("modal.save"), Colors::WHITE, || {
                            // Check if port is available.
                            let (ip, _) = NodeConfig::get_api_address_port();
                            let available = NodeConfig::is_api_port_available(
                                &ip,
                                &self.api_port_edit
                            );
                            self.api_port_available_edit = available;

                            if self.api_port_available_edit {
                                // Save port at config if it's available.
                                NodeConfig::save_api_address_port(
                                    &ip,
                                    &self.api_port_edit
                                );

                                self.is_api_port_available = true;
                                cb.hide_keyboard();
                                modal.close();
                            }
                        });
                    });
                });
                ui.add_space(6.0);
            });
        });
    }
}