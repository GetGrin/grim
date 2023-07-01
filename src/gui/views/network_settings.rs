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

use crate::gui::Colors;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, NetworkTab, NetworkTabType, View};
use crate::gui::views::settings_node::NodeSetup;
use crate::node::Node;

#[derive(Default)]
pub struct NetworkSettings {
    node_setup: NodeSetup
}

impl NetworkTab for NetworkSettings {
    fn get_type(&self) -> NetworkTabType {
        NetworkTabType::Settings
    }

    fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        ScrollArea::vertical()
            .id_source("network_settings")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                self.node_setup.ui(ui, cb);
            });
    }

    fn on_modal_ui(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        match modal.id {
            Self::NODE_RESTART_REQUIRED_MODAL => {
                self.node_restart_required_modal(ui, modal);
            }
            NodeSetup::API_PORT_MODAL => {
                self.node_setup.api_port_modal(ui, modal, cb);
            },
            NodeSetup::API_SECRET_MODAL => {
                self.node_setup.secret_modal(ui, modal, cb);
            },
            NodeSetup::FOREIGN_API_SECRET_MODAL => {
                self.node_setup.secret_modal(ui, modal, cb);
            },
            NodeSetup::FTL_MODAL => {
                self.node_setup.ftl_modal(ui, modal, cb);
            }
            _ => {}
        }
    }
}

impl NetworkSettings {
    pub const NODE_RESTART_REQUIRED_MODAL: &'static str = "node_restart_required";

    /// Node restart reminder modal content.
    pub fn node_restart_required_modal(&self, ui: &mut egui::Ui, modal: &Modal) {
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

    /// List of available IP addresses.
    pub fn get_ip_addrs() -> Vec<IpAddr> {
        let mut ip_addrs = Vec::new();
        for net_if in pnet::datalink::interfaces() {
            for ip in net_if.ips {
                if ip.is_ipv4() {
                    ip_addrs.push(ip.ip());
                }
            }
        }
        ip_addrs
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