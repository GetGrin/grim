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

use egui::{RichText, ScrollArea};

use crate::gui::{Colors, Navigator};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, ModalPosition, View};
use crate::gui::views::network::{NetworkTab, NetworkTabType};
use crate::gui::views::network::settings::server::ServerSetup;
use crate::gui::views::network::settings::stratum::StratumServerSetup;
use crate::node::Node;

#[derive(Default)]
pub struct NetworkNodeSettings {
    server_setup: ServerSetup,
    stratum_server_setup: StratumServerSetup
}

impl NetworkTab for NetworkNodeSettings {
    fn get_type(&self) -> NetworkTabType {
        NetworkTabType::Settings
    }

    fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        ScrollArea::vertical()
            .id_source("network_settings")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                self.server_setup.ui(ui, cb);
                self.stratum_server_setup.ui(ui, cb);
            });
    }

    fn on_modal_ui(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        match modal.id {
            Self::NODE_RESTART_REQUIRED_MODAL => {
                self.node_restart_required_modal(ui, modal);
            }

            ServerSetup::API_PORT_MODAL => {
                self.server_setup.api_port_modal(ui, modal, cb);
            },
            ServerSetup::API_SECRET_MODAL => {
                self.server_setup.secret_modal(ui, modal, cb);
            },
            ServerSetup::FOREIGN_API_SECRET_MODAL => {
                self.server_setup.secret_modal(ui, modal, cb);
            },
            ServerSetup::FTL_MODAL => {
                self.server_setup.ftl_modal(ui, modal, cb);
            },

            StratumServerSetup::STRATUM_PORT_MODAL => {
                self.stratum_server_setup.port_modal(ui, modal, cb);
            }
            StratumServerSetup::STRATUM_ATTEMPT_TIME_MODAL => {
                self.stratum_server_setup.attempt_modal(ui, modal, cb);
            }
            StratumServerSetup::STRATUM_MIN_SHARE_MODAL => {
                self.stratum_server_setup.min_diff_modal(ui, modal, cb);
            }
            _ => {}
        }
    }
}

impl NetworkNodeSettings {
    pub const NODE_RESTART_REQUIRED_MODAL: &'static str = "node_restart_required";

    /// Reminder to restart enabled node to show on edit setting at [`Modal`].
    pub fn node_restart_required_ui(ui: &mut egui::Ui) {
        if Node::is_running() {
            ui.add_space(12.0);
            ui.label(RichText::new(t!("network_settings.restart_node_required"))
                .size(16.0)
                .color(Colors::GREEN)
            );
        }
    }

    /// Show [`Modal`] to ask node restart if setting is changed on enabled node.
    pub fn show_node_restart_required_modal() {
        if Node::is_running() {
            // Show modal to apply changes by node restart.
            let port_modal = Modal::new(NetworkNodeSettings::NODE_RESTART_REQUIRED_MODAL)
                .position(ModalPosition::Center)
                .title(t!("network.settings"));
            Navigator::show_modal(port_modal);
        }
    }

    /// Node restart reminder modal content.
    fn node_restart_required_modal(&self, ui: &mut egui::Ui, modal: &Modal) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.restart_node_required"))
                .size(18.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("network_settings.restart"), Colors::WHITE, || {
                        Node::restart();
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::WHITE, || {
                        modal.close();
                    });
                });
            });
            ui.add_space(6.0);
        });
    }

    /// Draw IP addresses as radio buttons.
    pub fn ip_addrs_ui(ui: &mut egui::Ui,
                       saved_ip: &String,
                       ip_addrs: &Vec<IpAddr>,
                       on_change: impl FnOnce(&String)) {
        let saved_ip_addr = &IpAddr::from_str(saved_ip.as_str()).unwrap();
        let mut selected_ip_addr = saved_ip_addr;

        // Set first IP address as current if saved is not present at system.
        if !ip_addrs.contains(selected_ip_addr) {
            selected_ip_addr = ip_addrs.get(0).unwrap();
        }

        ui.add_space(2.0);
        // Show available IP addresses on the system.
        let _ = ip_addrs.chunks(2).map(|x| {
            if x.len() == 2 {
                ui.columns(2, |columns| {
                    let ip_addr_l = x.get(0).unwrap();
                    columns[0].vertical_centered(|ui| {
                        View::radio_value(ui,
                                          &mut selected_ip_addr,
                                          ip_addr_l,
                                          ip_addr_l.to_string());
                    });
                    let ip_addr_r = x.get(1).unwrap();
                    columns[1].vertical_centered(|ui| {
                        View::radio_value(ui,
                                          &mut selected_ip_addr,
                                          ip_addr_r,
                                          ip_addr_r.to_string());
                    })
                });
            } else {
                let ip_addr = x.get(0).unwrap();
                View::radio_value(ui,
                                  &mut selected_ip_addr,
                                  ip_addr,
                                  ip_addr.to_string());
            }
            ui.add_space(12.0);
        }).collect::<Vec<_>>();

        if saved_ip_addr != selected_ip_addr {
            (on_change)(&selected_ip_addr.to_string());
        }
    }

    /// Show message when IP addresses are not available at system.
    pub fn no_ip_address_ui(ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network.no_ips"))
                .size(16.0)
                .color(Colors::INACTIVE_TEXT)
            );
            ui.add_space(6.0);
        });
    }
}