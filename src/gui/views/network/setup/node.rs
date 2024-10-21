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

use egui::{Id, RichText};
use grin_core::global::ChainTypes;

use crate::AppConfig;
use crate::gui::Colors;
use crate::gui::icons::{CLOCK_CLOCKWISE, COMPUTER_TOWER, PLUG, POWER, SHIELD, SHIELD_SLASH};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, View};
use crate::gui::views::network::NetworkContent;
use crate::gui::views::network::settings::NetworkSettings;
use crate::gui::views::types::{ModalContainer, ModalPosition, TextEditOptions};
use crate::node::{Node, NodeConfig};

/// Integrated node general setup section content.
pub struct NodeSetup {
    /// IP Addresses available at system.
    available_ips: Vec<String>,

    /// API port value.
    api_port_edit: String,
    /// Flag to check if API port is available.
    api_port_available_edit: bool,

    /// Flag to check if API port from saved config value is available.
    is_api_port_available: bool,

    /// Secret edit value for modal.
    secret_edit: String,

    /// Future Time Limit value.
    ftl_edit: String,

    /// [`Modal`] identifiers allowed at this ui container.
    modal_ids: Vec<&'static str>
}

/// Identifier for API port value [`Modal`].
pub const API_PORT_MODAL: &'static str = "api_port";
/// Identifier for API secret value [`Modal`].
pub const API_SECRET_MODAL: &'static str = "api_secret";
/// Identifier for Foreign API secret value [`Modal`].
pub const FOREIGN_API_SECRET_MODAL: &'static str = "foreign_api_secret";
/// Identifier for FTL value [`Modal`].
pub const FTL_MODAL: &'static str = "ftl";

impl Default for NodeSetup {
    fn default() -> Self {
        let (api_ip, api_port) = NodeConfig::get_api_ip_port();
        let is_api_port_available = NodeConfig::is_api_port_available(&api_ip, &api_port);
        Self {
            available_ips: NodeConfig::get_ip_addrs(),
            api_port_edit: api_port,
            api_port_available_edit: is_api_port_available,
            is_api_port_available,
            secret_edit: "".to_string(),
            ftl_edit: NodeConfig::get_ftl(),
            modal_ids: vec![
                API_PORT_MODAL,
                API_SECRET_MODAL,
                FOREIGN_API_SECRET_MODAL,
                FTL_MODAL
            ]
        }
    }
}

impl ModalContainer for NodeSetup {
    fn modal_ids(&self) -> &Vec<&'static str> {
        &self.modal_ids
    }

    fn modal_ui(&mut self,
                ui: &mut egui::Ui,
                modal: &Modal,
                cb: &dyn PlatformCallbacks) {
        match modal.id {
            API_PORT_MODAL => self.api_port_modal(ui, modal, cb),
            API_SECRET_MODAL => self.secret_modal(ui, modal, cb),
            FOREIGN_API_SECRET_MODAL => self.secret_modal(ui, modal, cb),
            FTL_MODAL => self.ftl_modal(ui, modal, cb),
            _ => {}
        }
    }
}

impl NodeSetup {
    pub fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        // Draw modal content for current ui container.
        self.current_modal_ui(ui, cb);

        View::sub_title(ui, format!("{} {}", COMPUTER_TOWER, t!("network_settings.server")));
        View::horizontal_line(ui, Colors::stroke());
        ui.add_space(6.0);

        // Show chain type setup.
        Self::chain_type_ui(ui);
        ui.add_space(2.0);

        // Show loading indicator or controls to stop/start/restart node.
        if Node::is_stopping() || Node::is_restarting() || Node::is_starting() {
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                View::small_loading_spinner(ui);
                ui.add_space(2.0);
            });
        } else {
            if Node::is_running() {
                ui.scope(|ui| {
                    // Setup spacing between buttons.
                    ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 0.0);
                    ui.add_space(6.0);

                    ui.columns(2, |columns| {
                        columns[0].vertical_centered_justified(|ui| {
                            View::action_button(ui, t!("network_settings.disable"), || {
                                Node::stop(false);
                            });
                        });
                        columns[1].vertical_centered_justified(|ui| {
                            View::action_button(ui, t!("network_settings.restart"), || {
                                Node::restart();
                            });
                        });
                    });
                });
            } else {
                ui.add_space(6.0);
                ui.vertical_centered(|ui| {
                    let enable_text = format!("{} {}", POWER, t!("network_settings.enable"));
                    View::action_button(ui, enable_text, || {
                        Node::start();
                    });
                });
            }
        }

        // Autorun node setup.
        ui.vertical_centered(|ui| {
            ui.add_space(6.0);
            NetworkContent::autorun_node_ui(ui);
            if Node::is_running() {
                ui.add_space(2.0);
                ui.label(RichText::new(t!("network_settings.restart_node_required"))
                    .size(16.0)
                    .color(Colors::inactive_text())
                );
                ui.add_space(4.0);
            }
        });
        ui.add_space(6.0);

        if self.available_ips.is_empty() {
            // Show message when IP addresses are not available on the system.
            NetworkSettings::no_ip_address_ui(ui);
        } else {
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(6.0);

            ui.vertical_centered(|ui| {
                ui.label(RichText::new(t!("network_settings.api_ip"))
                    .size(16.0)
                    .color(Colors::gray())
                );
                ui.add_space(6.0);

                // Show API IP addresses to select.
                let (api_ip, api_port) = NodeConfig::get_api_ip_port();
                NetworkSettings::ip_addrs_ui(ui, &api_ip, &self.available_ips, |selected_ip| {
                    let api_available = NodeConfig::is_api_port_available(selected_ip, &api_port);
                    self.is_api_port_available = api_available;
                    NodeConfig::save_api_address(selected_ip, &api_port);
                });
                // Show API port setup.
                self.api_port_setup_ui(ui, cb);
                // Show API secret setup.
                self.secret_ui(API_SECRET_MODAL, ui, cb);
                ui.add_space(12.0);
                // Show Foreign API secret setup.
                self.secret_ui(FOREIGN_API_SECRET_MODAL, ui, cb);
                ui.add_space(6.0);
            });
        }

        ui.add_space(6.0);
        View::horizontal_line(ui, Colors::item_stroke());
        ui.add_space(6.0);

        ui.vertical_centered(|ui| {
            // Show FTL setup.
            self.ftl_ui(ui, cb);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(6.0);

            // Validation setup.
            self.validation_mode_ui(ui);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(6.0);

            // Archive mode setup.
            self.archive_mode_ui(ui);
        });
    }

    /// Draw [`ChainTypes`] setup content.
    pub fn chain_type_ui(ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network.type")).size(16.0).color(Colors::gray()));
        });

        let saved_chain_type = AppConfig::chain_type();
        let mut selected_chain_type = saved_chain_type;

        ui.add_space(8.0);
        ui.columns(2, |columns| {
            columns[0].vertical_centered(|ui| {
                let main_type = ChainTypes::Mainnet;
                View::radio_value(ui, &mut selected_chain_type, main_type, t!("network.mainnet"));
            });
            columns[1].vertical_centered(|ui| {
                let test_type = ChainTypes::Testnet;
                View::radio_value(ui, &mut selected_chain_type, test_type, t!("network.testnet"));
            })
        });
        ui.add_space(8.0);

        if saved_chain_type != selected_chain_type && !Node::is_restarting() {
            AppConfig::change_chain_type(&selected_chain_type);
            if Node::is_running() {
                Node::restart();
            }
        }
    }

    /// Draw API port setup content.
    fn api_port_setup_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("network_settings.api_port")).size(16.0).color(Colors::gray()));
        ui.add_space(6.0);

        let (_, port) = NodeConfig::get_api_ip_port();
        View::button(ui, format!("{} {}", PLUG, &port), Colors::white_or_black(false), || {
            // Setup values for modal.
            self.api_port_edit = port;
            self.api_port_available_edit = self.is_api_port_available;

            // Show API port modal.
            Modal::new(API_PORT_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"))
                .show();
            cb.show_keyboard();
        });
        ui.add_space(6.0);

        if !self.is_api_port_available {
            // Show error when API server port is unavailable.
            ui.label(RichText::new(t!("network_settings.port_unavailable"))
                .size(16.0)
                .color(Colors::red()));
            ui.add_space(6.0);
        }
        ui.add_space(6.0);
    }

    /// Draw API port [`Modal`] content.
    fn api_port_modal(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.api_port"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(6.0);

            // Draw API port text edit.
            let mut api_port_edit_opts = TextEditOptions::new(Id::from(modal.id)).h_center();
            View::text_edit(ui, cb, &mut self.api_port_edit, &mut api_port_edit_opts);

            // Show error when specified port is unavailable or reminder to restart enabled node.
            if !self.api_port_available_edit {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.port_unavailable"))
                    .size(16.0)
                    .color(Colors::red()));
            } else {
                NetworkSettings::node_restart_required_ui(ui);
            }
            ui.add_space(12.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            // Save button callback.
            let on_save = || {
                // Check if port is available.
                let (api_ip, _) = NodeConfig::get_api_ip_port();
                let available = NodeConfig::is_api_port_available(&api_ip, &self.api_port_edit);
                self.api_port_available_edit = available;

                if available {
                    // Save port at config if it's available.
                    NodeConfig::save_api_address(&api_ip, &self.api_port_edit);

                    if Node::is_running() {
                        Node::restart();
                    }

                    self.is_api_port_available = true;
                    cb.hide_keyboard();
                    modal.close();
                }
            };

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                        cb.hide_keyboard();
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.save"), Colors::white_or_black(false), on_save);
                });
            });
            ui.add_space(6.0);
        });
    }

    /// Draw API secret token setup content.
    fn secret_ui(&mut self, modal_id: &'static str, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        let secret_title = match modal_id {
            API_SECRET_MODAL => t!("network_settings.api_secret"),
            _ => t!("network_settings.foreign_api_secret")
        };
        ui.label(RichText::new(secret_title)
            .size(16.0)
            .color(Colors::gray())
        );
        ui.add_space(6.0);

        let secret_value = match modal_id {
            API_SECRET_MODAL => NodeConfig::get_api_secret(false),
            _ => NodeConfig::get_api_secret(true)
        };

        let secret_text = if secret_value.is_some() {
            format!("{} {}", SHIELD, t!("network_settings.enabled"))
        } else {
            format!("{} {}", SHIELD_SLASH, t!("network_settings.disabled"))
        };

        View::button(ui, secret_text, Colors::white_or_black(false), || {
            // Setup values for modal.
            self.secret_edit = secret_value.unwrap_or("".to_string());
            // Show secret edit modal.
            Modal::new(modal_id)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"))
                .show();
            cb.show_keyboard();
        });
    }

    /// Draw API secret token [`Modal`] content.
    fn secret_modal(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            let description = match modal.id {
                API_SECRET_MODAL => t!("network_settings.api_secret"),
                _ => t!("network_settings.foreign_api_secret")
            };
            ui.label(RichText::new(description).size(17.0).color(Colors::gray()));
            ui.add_space(8.0);

            // Draw API secret token value text edit.
            let mut secret_edit_opts = TextEditOptions::new(Id::from(modal.id)).copy().paste();
            View::text_edit(ui, cb, &mut self.secret_edit, &mut secret_edit_opts);
            ui.add_space(6.0);

            // Show reminder to restart enabled node.
            if Node::is_running() {
                ui.label(RichText::new(t!("network_settings.restart_node_required"))
                    .size(16.0)
                    .color(Colors::green())
                );
                ui.add_space(6.0);
            }
            ui.add_space(4.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            // Save button callback.
            let on_save = || {
                let secret = self.secret_edit.clone();
                match modal.id {
                    API_SECRET_MODAL => {
                        NodeConfig::save_api_secret(&secret);
                    }
                    _ => {
                        NodeConfig::save_foreign_api_secret(&secret);
                    }
                };
                cb.hide_keyboard();
                modal.close();
            };

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                        cb.hide_keyboard();
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.save"), Colors::white_or_black(false), on_save);
                });
            });
            ui.add_space(6.0);
        });
    }

    /// Draw FTL setup content.
    fn ftl_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("network_settings.ftl"))
            .size(16.0)
            .color(Colors::gray())
        );
        ui.add_space(6.0);

        let ftl = NodeConfig::get_ftl();
        View::button(ui,
                     format!("{} {}", CLOCK_CLOCKWISE, &ftl),
                     Colors::white_or_black(false), || {
            // Setup values for modal.
            self.ftl_edit = ftl;
            // Show ftl value setup modal.
            Modal::new(FTL_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"))
                .show();
            cb.show_keyboard();
        });
        ui.add_space(6.0);
        ui.label(RichText::new(t!("network_settings.ftl_description"))
            .size(16.0)
            .color(Colors::inactive_text())
        );
    }

    /// Draw FTL [`Modal`] content.
    fn ftl_modal(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.ftl"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Draw ftl value text edit.
            let mut ftl_edit_opts = TextEditOptions::new(Id::from(modal.id)).h_center();
            View::text_edit(ui, cb, &mut self.ftl_edit, &mut ftl_edit_opts);

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.ftl_edit.parse::<u64>().is_err() {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.not_valid_value"))
                    .size(17.0)
                    .color(Colors::red()));
            } else {
                NetworkSettings::node_restart_required_ui(ui);
            }
            ui.add_space(12.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            // Save button callback.
            let on_save = || {
                if let Ok(ftl) = self.ftl_edit.parse::<u64>() {
                    NodeConfig::save_ftl(ftl);
                    cb.hide_keyboard();
                    modal.close();
                }
            };

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                        // Close modal.
                        cb.hide_keyboard();
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.save"), Colors::white_or_black(false), on_save);
                });
            });
            ui.add_space(6.0);
        });
    }

    /// Draw chain validation mode setup content.
    pub fn validation_mode_ui(&mut self, ui: &mut egui::Ui) {
        let validate = NodeConfig::is_full_chain_validation();
        View::checkbox(ui, validate, t!("network_settings.full_validation"), || {
            NodeConfig::toggle_full_chain_validation();
        });
        ui.add_space(4.0);
        ui.label(RichText::new(t!("network_settings.full_validation_description"))
            .size(16.0)
            .color(Colors::inactive_text())
        );
    }

    /// Draw archive mode setup content.
    fn archive_mode_ui(&mut self, ui: &mut egui::Ui) {
        let archive_mode = NodeConfig::is_archive_mode();
        View::checkbox(ui, archive_mode, t!("network_settings.archive_mode"), || {
            NodeConfig::toggle_archive_mode();
        });
        ui.add_space(4.0);
        ui.label(RichText::new(t!("network_settings.archive_mode_desc"))
            .size(16.0)
            .color(Colors::inactive_text())
        );
    }
}