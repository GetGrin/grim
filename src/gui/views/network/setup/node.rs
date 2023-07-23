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

use egui::{Id, RichText, TextStyle, Ui, Widget};
use egui_extras::{Size, StripBuilder};
use grin_core::global::ChainTypes;

use crate::AppConfig;
use crate::gui::Colors;
use crate::gui::icons::{CLIPBOARD_TEXT, CLOCK_CLOCKWISE, COMPUTER_TOWER, COPY, PLUG, POWER, SHIELD, SHIELD_SLASH};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, ModalPosition, Network, View};
use crate::gui::views::network::settings::NetworkSettings;
use crate::node::{Node, NodeConfig};

/// Integrated node server setup ui section.
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
}

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
        }
    }
}

impl NodeSetup {
    pub const API_PORT_MODAL: &'static str = "api_port";
    pub const API_SECRET_MODAL: &'static str = "api_secret";
    pub const FOREIGN_API_SECRET_MODAL: &'static str = "foreign_api_secret";
    pub const FTL_MODAL: &'static str = "ftl";

    pub fn ui(&mut self, ui: &mut Ui, cb: &dyn PlatformCallbacks) {
        View::sub_title(ui, format!("{} {}", COMPUTER_TOWER, t!("network_settings.server")));
        View::horizontal_line(ui, Colors::STROKE);
        ui.add_space(4.0);

        // Show chain type setup.
        self.chain_type_ui(ui);

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

        // Autorun node setup.
        ui.vertical_centered(|ui| {
            ui.add_space(6.0);
            Network::autorun_node_ui(ui);
            if Node::is_running() {
                ui.add_space(2.0);
                ui.label(RichText::new(t!("network_settings.restart_node_required"))
                    .size(16.0)
                    .color(Colors::INACTIVE_TEXT)
                );
                ui.add_space(4.0);
            }
        });
        ui.add_space(6.0);

        if self.available_ips.is_empty() {
            // Show message when IP addresses are not available on the system.
            NetworkSettings::no_ip_address_ui(ui);
        } else {
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);

            ui.vertical_centered(|ui| {
                ui.label(RichText::new(t!("network_settings.api_ip"))
                    .size(16.0)
                    .color(Colors::GRAY)
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
                self.secret_ui(Self::API_SECRET_MODAL, ui, cb);
                ui.add_space(12.0);
                // Show Foreign API secret setup.
                self.secret_ui(Self::FOREIGN_API_SECRET_MODAL, ui, cb);
                ui.add_space(6.0);
            });
        }

        ui.add_space(6.0);
        View::horizontal_line(ui, Colors::ITEM_STROKE);
        ui.add_space(6.0);

        ui.vertical_centered(|ui| {
            // Show FTL setup.
            self.ftl_ui(ui, cb);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);

            // Validation setup.
            self.validation_mode_ui(ui);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);

            // Archive mode setup.
            self.archive_mode_ui(ui);
        });
    }

    /// Draw [`ChainTypes`] setup content.
    fn chain_type_ui(&mut self, ui: &mut Ui) {
        let saved_chain_type = AppConfig::chain_type();
        let mut selected_chain_type = saved_chain_type;

        ui.add_space(8.0);
        ui.columns(2, |columns| {
            columns[0].vertical_centered(|ui| {
                let main_type = ChainTypes::Mainnet;
                View::radio_value(ui, &mut selected_chain_type, main_type, "Mainnet".to_string());
            });
            columns[1].vertical_centered(|ui| {
                let test_type = ChainTypes::Testnet;
                View::radio_value(ui, &mut selected_chain_type, test_type, "Testnet".to_string());
            })
        });
        ui.add_space(8.0);

        if saved_chain_type != selected_chain_type {
            AppConfig::change_chain_type(&selected_chain_type);
            NetworkSettings::show_node_restart_required_modal();
        }
    }

    /// Draw API port setup content.
    fn api_port_setup_ui(&mut self, ui: &mut Ui, cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("network_settings.api_port"))
            .size(16.0)
            .color(Colors::GRAY)
        );
        ui.add_space(6.0);

        let (_, port) = NodeConfig::get_api_ip_port();
        View::button(ui, format!("{} {}", PLUG, port.clone()), Colors::BUTTON, || {
            // Setup values for modal.
            self.api_port_edit = port;
            self.api_port_available_edit = self.is_api_port_available;

            // Show API port modal.
            Modal::new(Self::API_PORT_MODAL)
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
                .color(Colors::RED));
            ui.add_space(6.0);
        }
        ui.add_space(6.0);
    }

    /// Draw API port [`Modal`] content.
    pub fn api_port_modal(&mut self, ui: &mut Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.api_port"))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(6.0);

            // Draw API port text edit.
            let text_edit_resp = egui::TextEdit::singleline(&mut self.api_port_edit)
                .font(TextStyle::Heading)
                .desired_width(64.0)
                .cursor_at_end(true)
                .ui(ui);
            text_edit_resp.request_focus();
            if text_edit_resp.clicked() {
                cb.show_keyboard();
            }

            // Show error when specified port is unavailable or reminder to restart enabled node.
            if !self.api_port_available_edit {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.port_unavailable"))
                    .size(16.0)
                    .color(Colors::RED));
            } else {
                NetworkSettings::node_restart_required_ui(ui);
            }
            ui.add_space(12.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 0.0);

            // Save button callback.
            let on_save = || {
                // Check if port is available.
                let (api_ip, _) = NodeConfig::get_api_ip_port();
                let available = NodeConfig::is_api_port_available(&api_ip, &self.api_port_edit);
                self.api_port_available_edit = available;

                if available {
                    // Save port at config if it's available.
                    NodeConfig::save_api_address(&api_ip, &self.api_port_edit);

                    self.is_api_port_available = true;
                    cb.hide_keyboard();
                    modal.close();
                }
            };

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::WHITE, || {
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

    /// Draw API secret token setup content.
    fn secret_ui(&mut self, modal_id: &'static str, ui: &mut Ui, cb: &dyn PlatformCallbacks) {
        let secret_title = match modal_id {
            Self::API_SECRET_MODAL => t!("network_settings.api_secret"),
            _ => t!("network_settings.foreign_api_secret")
        };
        ui.label(RichText::new(secret_title)
            .size(16.0)
            .color(Colors::GRAY)
        );
        ui.add_space(6.0);

        let secret_value = match modal_id {
            Self::API_SECRET_MODAL => NodeConfig::get_api_secret(),
            _ => NodeConfig::get_foreign_api_secret()
        };

        let secret_text = if secret_value.is_some() {
            format!("{} {}", SHIELD, t!("network_settings.enabled"))
        } else {
            format!("{} {}", SHIELD_SLASH, t!("network_settings.disabled"))
        };

        View::button(ui, secret_text, Colors::BUTTON, || {
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
    pub fn secret_modal(&mut self, ui: &mut Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            let description = match modal.id {
                Self::API_SECRET_MODAL => t!("network_settings.api_secret"),
                _ => t!("network_settings.foreign_api_secret")
            };
            ui.label(RichText::new(description).size(17.0).color(Colors::GRAY));
            ui.add_space(8.0);
            StripBuilder::new(ui)
                .size(Size::exact(42.0))
                .vertical(|mut strip| {
                    strip.strip(|builder| {
                        builder
                            .size(Size::remainder())
                            .size(Size::exact(48.0))
                            .size(Size::exact(48.0))
                            .horizontal(|mut strip| {
                                strip.cell(|ui| {
                                    ui.add_space(2.0);
                                    // Draw API secret token value text edit.
                                    let edit = egui::TextEdit::singleline(&mut self.secret_edit)
                                        .id(Id::from(modal.id))
                                        .font(TextStyle::Button)
                                        .cursor_at_end(true)
                                        .ui(ui);
                                    edit.request_focus();
                                    if edit.clicked() {
                                        cb.show_keyboard();
                                    }
                                });
                                strip.cell(|ui| {
                                    ui.vertical_centered(|ui| {
                                        // Draw copy button.
                                        let copy_icon = COPY.to_string();
                                        View::button(ui, copy_icon, Colors::WHITE, || {
                                            cb.copy_string_to_buffer(self.secret_edit.clone());
                                        });
                                    });
                                });
                                strip.cell(|ui| {
                                    ui.vertical_centered(|ui| {
                                        // Draw paste button.
                                        let paste_icon = CLIPBOARD_TEXT.to_string();
                                        View::button(ui, paste_icon, Colors::WHITE, || {
                                            self.secret_edit = cb.get_string_from_buffer();
                                        });
                                    });
                                });
                            });
                    })
                });

            // Show reminder to restart enabled node.
            if Node::is_running() {
                ui.label(RichText::new(t!("network_settings.restart_node_required"))
                    .size(16.0)
                    .color(Colors::GREEN)
                );
                ui.add_space(8.0);
            }
            ui.add_space(4.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 0.0);

            // Save button callback.
            let on_save = || {
                let secret = self.secret_edit.clone();
                match modal.id {
                    Self::API_SECRET_MODAL => {
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
                    View::button(ui, t!("modal.cancel"), Colors::WHITE, || {
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

    /// Draw FTL setup content.
    fn ftl_ui(&mut self, ui: &mut Ui, cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("network_settings.ftl"))
            .size(16.0)
            .color(Colors::GRAY)
        );
        ui.add_space(6.0);

        let ftl = NodeConfig::get_ftl();
        View::button(ui, format!("{} {}", CLOCK_CLOCKWISE, ftl.clone()), Colors::BUTTON, || {
            // Setup values for modal.
            self.ftl_edit = ftl;
            // Show ftl value setup modal.
            Modal::new(Self::FTL_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"))
                .show();
            cb.show_keyboard();
        });
        ui.add_space(6.0);
        ui.label(RichText::new(t!("network_settings.ftl_description"))
            .size(16.0)
            .color(Colors::INACTIVE_TEXT)
        );
    }

    /// Draw FTL [`Modal`] content.
    pub fn ftl_modal(&mut self, ui: &mut Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.ftl"))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw ftl value text edit.
            let text_edit_resp = egui::TextEdit::singleline(&mut self.ftl_edit)
                .id(Id::from(modal.id))
                .font(TextStyle::Heading)
                .desired_width(52.0)
                .cursor_at_end(true)
                .ui(ui);
            text_edit_resp.request_focus();
            if text_edit_resp.clicked() {
                cb.show_keyboard();
            }

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.ftl_edit.parse::<u64>().is_err() {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.not_valid_value"))
                    .size(17.0)
                    .color(Colors::RED));
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

    /// Draw chain validation mode setup content.
    pub fn validation_mode_ui(&mut self, ui: &mut Ui) {
        let validate = NodeConfig::is_full_chain_validation();
        View::checkbox(ui, validate, t!("network_settings.full_validation"), || {
            NodeConfig::toggle_full_chain_validation();
            NetworkSettings::show_node_restart_required_modal();
        });
        ui.add_space(4.0);
        ui.label(RichText::new(t!("network_settings.full_validation_description"))
            .size(16.0)
            .color(Colors::INACTIVE_TEXT)
        );
    }

    /// Draw archive mode setup content.
    pub fn archive_mode_ui(&mut self, ui: &mut Ui) {
        let archive_mode = NodeConfig::is_archive_mode();
        View::checkbox(ui, archive_mode, t!("network_settings.archive_mode"), || {
            NodeConfig::toggle_archive_mode();
            NetworkSettings::show_node_restart_required_modal();
        });
        ui.add_space(4.0);
        ui.label(RichText::new(t!("network_settings.archive_mode_desc"))
            .size(16.0)
            .color(Colors::INACTIVE_TEXT)
        );
    }
}