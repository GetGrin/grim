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
use crate::gui::icons::WRENCH;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, ModalLocation, ModalPosition, Network, View};
use crate::node::NodeConfig;

/// Stratum server setup ui section.
pub struct StratumServerSetup {
    /// Stratum address to be used inside edit modal.
    stratum_address_edit: String,
    /// Stratum port  to be used inside edit modal.
    stratum_port_edit: String,
    /// Flag to check if stratum port is available inside edit modal.
    port_available_edit: bool,

    /// Flag to check if stratum port is available from saved config value.
    pub(crate) stratum_port_available: bool
}

impl Default for StratumServerSetup {
    fn default() -> Self {
        let (stratum_address, stratum_port) = NodeConfig::get_stratum_address_port();
        let is_port_available = Network::is_port_available(stratum_address.as_str(), stratum_port);
        Self {
            stratum_address_edit: stratum_address,
            stratum_port_edit: stratum_port.to_string(),
            port_available_edit: is_port_available,
            stratum_port_available: is_port_available
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

        // Show error message when IP addresses are not available on the system.
        let mut addresses = Network::get_ip_list();
        if addresses.is_empty() {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(t!("network_mining.no_ip_addresses"))
                    .size(16.0)
                    .color(Colors::INACTIVE_TEXT)
                );
                ui.add_space(6.0);
            });
            return;
        }

        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.ip_address"))
                .size(16.0)
                .color(Colors::GRAY)
            );
            ui.add_space(6.0);

            // Show stratum IP address setup.
            Self::ip_address_setup_ui(ui, addresses);

            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);

            ui.label(RichText::new(t!("network_settings.port"))
                .size(16.0)
                .color(Colors::GRAY)
            );

            // Show button to choose stratum server port.
            ui.add_space(6.0);
            let (stratum_address, stratum_port) = NodeConfig::get_stratum_address_port();
            View::button(ui, stratum_port.to_string(), Colors::BUTTON, || {
                // Setup values for modal.
                self.stratum_address_edit = stratum_address.clone();
                self.stratum_port_edit = stratum_port.to_string();
                self.port_available_edit = Network::is_port_available(
                    stratum_address.as_str(),
                    stratum_port
                );

                // Show stratum port modal.
                let port_modal = Modal::new(Self::STRATUM_PORT_MODAL,
                                            ModalLocation::SidePanel)
                    .position(ModalPosition::CenterTop)
                    .title(t!("network_settings.change_port"));
                Navigator::open_modal(port_modal);
                cb.show_keyboard();
            });
            ui.add_space(12.0);

            // Show error when stratum server port is unavailable.
            if !self.stratum_port_available {
                ui.label(RichText::new(t!("network_mining.port_unavailable"))
                    .size(16.0)
                    .color(Colors::RED));
                ui.add_space(12.0);
            }

            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);
        });
    }

    /// Draw stratum port [`Modal`] content.
    pub fn stratum_port_modal_ui(&mut self,
                                 ui: &mut egui::Ui,
                                 modal: &Modal,
                                 cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.enter_value"))
                .size(16.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw stratum port text edit.
            let text_edit_resp = egui::TextEdit::singleline(&mut self.stratum_port_edit)
                .font(TextStyle::Button)
                .desired_width(48.0)
                .cursor_at_end(true)
                .ui(ui);
            text_edit_resp.request_focus();
            if text_edit_resp.clicked() {
                cb.show_keyboard();
            }

            // Show error when specified port is unavailable.
            if !self.port_available_edit {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_mining.port_unavailable"))
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
                            // Close modal.
                            cb.hide_keyboard();
                            modal.close();
                        });
                    });
                    columns[1].vertical_centered_justified(|ui| {
                        View::button(ui, t!("modal.save"), Colors::WHITE, || {
                            // Check if port is available.
                            let port_parse = self.stratum_port_edit.parse::<u16>();
                            let is_available = port_parse.is_ok() && Network::is_port_available(
                                self.stratum_address_edit.as_str(),
                                port_parse.unwrap()
                            );
                            self.port_available_edit = is_available;

                            // Save port at config if it's available.
                            if self.port_available_edit {
                                NodeConfig::save_stratum_address_port(
                                    self.stratum_address_edit.clone(),
                                    self.stratum_port_edit.clone()
                                );

                                self.stratum_port_available = true;
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

    /// Show stratum IP address setup.
    fn ip_address_setup_ui(ui: &mut egui::Ui, addresses: Vec<IpAddr>) {
        let (addr, port) = NodeConfig::get_stratum_address_port();
        let saved_ip_addr = &IpAddr::from_str(addr.as_str()).unwrap();
        let mut selected_addr = saved_ip_addr;

        // Set first IP address as current if saved is not present at system.
        if !addresses.contains(selected_addr) {
            selected_addr = addresses.get(0).unwrap();
        }

        // Show available IP addresses on the system.
        let _ = addresses.chunks(2).map(|x| {
            if x.len() == 2 {
                ui.columns(2, |columns| {
                    let addr0 = x.get(0).unwrap();
                    columns[0].vertical_centered(|ui| {
                        View::radio_value(ui,
                                          &mut selected_addr,
                                          addr0,
                                          addr0.to_string());
                    });
                    let addr1 = x.get(1).unwrap();
                    columns[1].vertical_centered(|ui| {
                        View::radio_value(ui,
                                          &mut selected_addr,
                                          addr1,
                                          addr1.to_string());
                    })
                });
            } else {
                let addr = x.get(0).unwrap();
                View::radio_value(ui,
                                  &mut selected_addr,
                                  addr,
                                  addr.to_string());
            }
            ui.add_space(10.0);
        }).collect::<Vec<_>>();

        // Save stratum server address at config if it was changed.
        if saved_ip_addr != selected_addr {
            NodeConfig::save_stratum_address_port(selected_addr.to_string(), port.to_string());
        }
    }
}