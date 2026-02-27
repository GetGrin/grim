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

use eframe::emath::Align;
use eframe::epaint::StrokeKind;
use egui::{Id, Layout, RichText};
use grin_core::global::ChainTypes;

use crate::gui::icons::{CLOCK_CLOCKWISE, COMPUTER_TOWER, FOLDERS, PENCIL, PLUG, POWER, SHIELD, SHIELD_SLASH};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::network::settings::NetworkSettings;
use crate::gui::views::network::NetworkContent;
use crate::gui::views::types::{ContentContainer, ModalPosition};
use crate::gui::views::{FilePickContent, FilePickContentType, Modal, TextEdit, View};
use crate::gui::Colors;
use crate::node::{Node, NodeConfig};
use crate::AppConfig;

/// Integrated node general setup section content.
pub struct NodeSetup {
    /// Data path value value for [`Modal`].
    data_path_edit: String,
    /// Button to pick directory for chain data.
    pick_data_dir: FilePickContent,

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

/// Identifier for chain data path value [`Modal`].
const DATA_PATH_MODAL: &'static str = "node_data_path";
/// Identifier for API port value [`Modal`].
const API_PORT_MODAL: &'static str = "node_api_port";
/// Identifier for API secret value [`Modal`].
const API_SECRET_MODAL: &'static str = "node_api_secret";
/// Identifier for Foreign API secret value [`Modal`].
const FOREIGN_API_SECRET_MODAL: &'static str = "node_foreign_api_secret";
/// Identifier for FTL value [`Modal`].
const FTL_MODAL: &'static str = "node_ftl";

impl Default for NodeSetup {
    fn default() -> Self {
        let (api_ip, api_port) = NodeConfig::get_api_ip_port();
        let is_api_port_available = NodeConfig::is_api_port_available(&api_ip, &api_port);
        Self {
            data_path_edit: NodeConfig::get_chain_data_path(),
            pick_data_dir: FilePickContent::new(
                FilePickContentType::ItemButton(View::item_rounding(0, 1, true))
            ).no_parse().pick_folder(),
            available_ips: NodeConfig::get_ip_addrs(),
            api_port_edit: api_port,
            api_port_available_edit: is_api_port_available,
            is_api_port_available,
            secret_edit: "".to_string(),
            ftl_edit: NodeConfig::get_ftl(),
        }
    }
}

impl ContentContainer for NodeSetup {
    fn modal_ids(&self) -> Vec<&'static str> {
        vec![
            DATA_PATH_MODAL,
            API_PORT_MODAL,
            API_SECRET_MODAL,
            FOREIGN_API_SECRET_MODAL,
            FTL_MODAL
        ]
    }

    fn modal_ui(&mut self,
                ui: &mut egui::Ui,
                modal: &Modal,
                cb: &dyn PlatformCallbacks) {
        match modal.id {
            DATA_PATH_MODAL => self.data_path_edit_modal_ui(ui, cb),
            API_PORT_MODAL => self.api_port_modal(ui, modal, cb),
            API_SECRET_MODAL => self.secret_modal(ui, modal, cb),
            FOREIGN_API_SECRET_MODAL => self.secret_modal(ui, modal, cb),
            FTL_MODAL => self.ftl_modal(ui, modal, cb),
            _ => {}
        }
    }

    fn container_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        View::sub_title(ui, format!("{} {}", COMPUTER_TOWER, t!("network_settings.server")));
        View::horizontal_line(ui, Colors::stroke());
        ui.add_space(6.0);

        // Show chain type setup.
        Self::chain_type_ui(ui);
        ui.add_space(2.0);

        // Show loading indicator or controls to stop/start/restart node.
        if Node::is_stopping() || Node::is_restarting() || Node::is_starting()
            || Node::data_dir_changing() {
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

        // Show data location selection for Desktop when it already started or turned off.
        if !Node::is_restarting() && !Node::is_stopping() && !Node::is_starting() &&
            View::is_desktop() {
            self.pick_data_dir_ui(ui, cb);
            ui.add_space(6.0);
        }

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
                self.api_port_setup_ui(ui);
                // Show API secret setup.
                self.secret_ui(API_SECRET_MODAL, ui);
                ui.add_space(12.0);
                // Show Foreign API secret setup.
                self.secret_ui(FOREIGN_API_SECRET_MODAL, ui);
                ui.add_space(6.0);
            });
        }

        ui.add_space(6.0);
        View::horizontal_line(ui, Colors::item_stroke());
        ui.add_space(6.0);

        ui.vertical_centered(|ui| {
            // Show FTL setup.
            self.ftl_ui(ui);

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
}

impl NodeSetup {
    /// Draw content to change chain data directory.
    fn pick_data_dir_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        // Setup layout size.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(56.0);

        // Draw round background.
        let bg_rect = rect.clone();
        let item_rounding = View::item_rounding(0, 1, false);
        ui.painter().rect(bg_rect,
                          item_rounding,
                          Colors::fill(),
                          View::item_stroke(),
                          StrokeKind::Outside);

        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            self.pick_data_dir.ui(ui, cb, |path| {
                Node::change_data_dir(path);
            });
            View::item_button(ui, View::item_rounding(1, 3, true), PENCIL, None, || {
                self.data_path_edit = NodeConfig::get_chain_data_path();
                // Show chain data path edit modal.
                Modal::new(DATA_PATH_MODAL)
                    .position(ModalPosition::CenterTop)
                    .title(t!("network.node"))
                    .show();
            });
            let layout_size = ui.available_size();
            ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    let path = NodeConfig::get_chain_data_path();
                    View::ellipsize_text(ui, path, 18.0, Colors::title(false));
                    ui.add_space(1.0);
                    let desc = format!("{} {}", FOLDERS, t!("files_location"));
                    ui.label(RichText::new(desc).size(15.0).color(Colors::gray()));
                    ui.add_space(8.0);
                });
            });
        });
    }

    /// Draw data path input [`Modal`] content.
    fn data_path_edit_modal_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            let on_save = |path: &String| {
                Node::change_data_dir(path.clone());
                Modal::close();
            };
            ui.label(RichText::new(format!("{}:", t!("files_location")))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Draw chain data path text edit.
            let mut edit = TextEdit::new(Id::from(DATA_PATH_MODAL)).paste();
            edit.ui(ui, &mut self.data_path_edit, cb);
            if edit.enter_pressed {
                on_save(&self.data_path_edit);
            }
            ui.add_space(12.0);

            // Show modal buttons.
            ui.scope(|ui| {
                // Setup spacing between buttons.
                ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

                ui.columns(2, |columns| {
                    columns[0].vertical_centered_justified(|ui| {
                        View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                            Modal::close();
                        });
                    });
                    columns[1].vertical_centered_justified(|ui| {
                        View::button(ui, t!("modal.save"), Colors::white_or_black(false), || {
                            on_save(&self.data_path_edit);
                        });
                    });
                });
                ui.add_space(6.0);
            });
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
    fn api_port_setup_ui(&mut self, ui: &mut egui::Ui) {
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
        let on_save = |c: &mut NodeSetup| {
            // Check if port is available.
            let (api_ip, _) = NodeConfig::get_api_ip_port();
            let available = NodeConfig::is_api_port_available(&api_ip, &c.api_port_edit);
            c.api_port_available_edit = available;
            if available {
                // Save port at config if it's available.
                NodeConfig::save_api_address(&api_ip, &c.api_port_edit);
                if Node::is_running() {
                    Node::restart();
                }
                c.is_api_port_available = true;
                Modal::close();
            }
        };

        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.api_port"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(6.0);

            // Draw API port text edit.
            let mut api_port_edit = TextEdit::new(Id::from(modal.id)).h_center().numeric();
            api_port_edit.ui(ui, &mut self.api_port_edit, cb);
            if api_port_edit.enter_pressed {
                on_save(self);
            }

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

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                        Modal::close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.save"), Colors::white_or_black(false), || {
                        on_save(self);
                    });
                });
            });
            ui.add_space(6.0);
        });
    }

    /// Draw API secret token setup content.
    fn secret_ui(&mut self, modal_id: &'static str, ui: &mut egui::Ui) {
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
        });
    }

    /// Draw API secret token [`Modal`] content.
    fn secret_modal(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        let on_save = |c: &mut NodeSetup| {
            let secret = c.secret_edit.clone();
            match modal.id {
                API_SECRET_MODAL => {
                    NodeConfig::save_api_secret(&secret);
                }
                _ => {
                    NodeConfig::save_foreign_api_secret(&secret);
                }
            };
            Modal::close();
        };

        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            let description = match modal.id {
                API_SECRET_MODAL => t!("network_settings.api_secret"),
                _ => t!("network_settings.foreign_api_secret")
            };
            ui.label(RichText::new(description).size(17.0).color(Colors::gray()));
            ui.add_space(8.0);

            // Draw API secret token value text edit.
            let mut secret_edit = TextEdit::new(Id::from(modal.id))
                .copy()
                .paste();
            secret_edit.ui(ui, &mut self.secret_edit, cb);
            if secret_edit.enter_pressed {
                on_save(self);
            }

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

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                        Modal::close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.save"), Colors::white_or_black(false), || {
                        on_save(self);
                    });
                });
            });
            ui.add_space(6.0);
        });
    }

    /// Draw FTL setup content.
    fn ftl_ui(&mut self, ui: &mut egui::Ui) {
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
        });
        ui.add_space(6.0);
        ui.label(RichText::new(t!("network_settings.ftl_description"))
            .size(16.0)
            .color(Colors::inactive_text())
        );
    }

    /// Draw FTL [`Modal`] content.
    fn ftl_modal(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        // Save button callback.
        let on_save = |c: &mut NodeSetup| {
            if let Ok(ftl) = c.ftl_edit.parse::<u64>() {
                NodeConfig::save_ftl(ftl);
                Modal::close();
            }
        };

        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.ftl"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Draw ftl value text edit.
            let mut ftl_edit = TextEdit::new(Id::from(modal.id)).h_center().numeric();
            ftl_edit.ui(ui, &mut self.ftl_edit, cb);
            if ftl_edit.enter_pressed {
                on_save(self);
            }

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

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                        // Close modal.
                        Modal::close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.save"), Colors::white_or_black(false), || {
                        on_save(self);
                    });
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