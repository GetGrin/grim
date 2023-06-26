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
use egui::RichText;
use crate::gui::Colors;
use crate::gui::icons::COMPUTER_TOWER;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Network, View};
use crate::node::{Node, NodeConfig};

/// Integrated node server setup ui section.
pub struct NodeSetup {
    /// API IP address to be used inside edit modal.
    api_address_edit: String,
    /// API port to be used inside edit modal.
    api_port_edit: String,
    /// Flag to check if API port is available inside edit modal.
    api_port_available_edit: bool,

    /// Flag to check if API port is available from saved config value.
    pub(crate) api_port_available: bool,

    /// API secret to be used inside edit modal.
    api_secret_edit: String,

    /// Foreign API secret to be used inside edit modal.
    foreign_api_secret_edit: String,

    /// Future Time Limit to be used inside edit modal.
    ftl_edit: String,
}

impl Default for NodeSetup {
    fn default() -> Self {
        let (api_address, api_port) = NodeConfig::get_api_address_port();
        let is_api_port_available = Network::is_port_available(api_address.as_str(), api_port);
        Self {
            api_address_edit: api_address,
            api_port_edit: api_port.to_string(),
            api_port_available_edit: is_api_port_available,
            api_port_available: is_api_port_available,
            api_secret_edit: "".to_string(),
            foreign_api_secret_edit: "".to_string(),
            ftl_edit: "".to_string(),
        }
    }
}

const SECRET_SYMBOLS: &'static str = "••••••••••••";

impl NodeSetup {
    pub const API_PORT_MODAL: &'static str = "stratum_port";

    pub fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        View::sub_title(ui, format!("{} {}", COMPUTER_TOWER, t!("network_settings.server.title")));
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
                    ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);
                    ui.add_space(6.0);

                    ui.columns(2, |columns| {
                        columns[0].vertical_centered_justified(|ui| {
                            View::button(ui, t!("network_settings.server.disable"), Colors::GOLD, || {
                                Node::stop(false);
                            });
                        });
                        columns[1].vertical_centered_justified(|ui| {
                            View::button(ui, t!("network_settings.server.restart"), Colors::GOLD, || {
                                Node::restart();
                            });
                        });
                    });
                });
            } else {
                ui.add_space(6.0);
                ui.vertical_centered(|ui| {
                    View::button(ui, t!("network_settings.server.enable"), Colors::GOLD, || {
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
        // Show error message when IP addresses are not available on the system.
        if addrs.is_empty() {
            Network::no_ip_address_ui(ui);
            ui.add_space(4.0);
        } else {
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(4.0);

            ui.vertical_centered(|ui| {
                ui.label(RichText::new(t!("network_settings.server.api_ip_address"))
                    .size(16.0)
                    .color(Colors::GRAY)
                );
                ui.add_space(6.0);
            });

            // Show API IP address setup.
            self.api_ip_address_setup_ui(ui, addrs)
        }
    }

    /// Show API IP address setup.
    fn api_ip_address_setup_ui(&mut self, ui: &mut egui::Ui, addrs: Vec<IpAddr>) {
        let (addr, port) = NodeConfig::get_api_address_port();
        let saved_ip_addr = &IpAddr::from_str(addr.as_str()).unwrap();
        let mut selected_addr = saved_ip_addr;

        // Set first IP address as current if saved is not present at system.
        if !addrs.contains(selected_addr) {
            selected_addr = addrs.get(0).unwrap();
        }

        // Show available IP addresses on the system.
        let _ = addrs.chunks(2).map(|x| {
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

        // Save stratum server address at config if it was changed and check port availability.
        if saved_ip_addr != selected_addr {
            NodeConfig::save_api_server_address_port(selected_addr.to_string(), port.to_string());
            let available = Network::is_port_available(selected_addr.to_string().as_str(), port);
            self.api_port_available = available;
        }
    }
}