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
use grin_chain::SyncStatus;

use crate::gui::Colors;
use crate::gui::icons::{BARBELL, HARD_DRIVES, PLUG, POWER, TIMER};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, View};
use crate::gui::views::network::settings::NetworkSettings;
use crate::gui::views::types::{ModalContainer, ModalPosition, TextEditOptions};
use crate::gui::views::wallets::modals::WalletsModal;
use crate::node::{Node, NodeConfig};
use crate::wallet::{WalletConfig, WalletList};

/// Stratum server setup section content.
pub struct StratumSetup {
    /// Wallet list to select for mining rewards.
    wallets: WalletList,
    /// Wallets [`Modal`] content.
    wallets_modal: WalletsModal,

    /// IP Addresses available at system.
    available_ips: Vec<String>,

    /// Stratum port value.
    stratum_port_edit: String,
    /// Flag to check if stratum port is available.
    stratum_port_available_edit: bool,

    /// Flag to check if stratum port from saved config value is available.
    is_port_available: bool,

    /// Wallet name to receive rewards.
    wallet_name: Option<String>,

    /// Attempt time value in seconds to mine on a particular header.
    attempt_time_edit: String,

    /// Minimum share difficulty value to request from miners.
    min_share_diff_edit: String,

    /// [`Modal`] identifiers allowed at this ui container.
    modal_ids: Vec<&'static str>
}

/// Identifier for wallet selection [`Modal`].
const WALLET_SELECTION_MODAL: &'static str = "stratum_wallet_selection_modal";
/// Identifier for stratum port [`Modal`].
const STRATUM_PORT_MODAL: &'static str = "stratum_port";
/// Identifier for attempt time [`Modal`].
const ATTEMPT_TIME_MODAL: &'static str = "stratum_attempt_time";
/// Identifier for minimum share difficulty [`Modal`].
const MIN_SHARE_DIFF_MODAL: &'static str = "stratum_min_share_diff";

impl Default for StratumSetup {
    fn default() -> Self {
        let (ip, port) = NodeConfig::get_stratum_address();
        let is_port_available = NodeConfig::is_stratum_port_available(&ip, &port);

        // Setup mining rewards wallet name and identifier.
        let mut wallet_id = NodeConfig::get_stratum_wallet_id();
        let wallet_name = if let Some(id) = wallet_id {
            WalletConfig::name_by_id(id)
        } else {
            None
        };
        if wallet_name.is_none() {
            wallet_id = None;
        }

        Self {
            wallets: WalletList::default(),
            wallets_modal: WalletsModal::new(wallet_id, None, false),
            available_ips: NodeConfig::get_ip_addrs(),
            stratum_port_edit: port,
            stratum_port_available_edit: is_port_available,
            is_port_available,
            wallet_name,
            attempt_time_edit: NodeConfig::get_stratum_attempt_time(),
            min_share_diff_edit: NodeConfig::get_stratum_min_share_diff(),
            modal_ids: vec![
                WALLET_SELECTION_MODAL,
                STRATUM_PORT_MODAL,
                ATTEMPT_TIME_MODAL,
                MIN_SHARE_DIFF_MODAL
            ]
        }
    }
}

impl ModalContainer for StratumSetup {
    fn modal_ids(&self) -> &Vec<&'static str> {
        &self.modal_ids
    }

    fn modal_ui(&mut self,
                ui: &mut egui::Ui,
                modal: &Modal,
                cb: &dyn PlatformCallbacks) {
        match modal.id {
            WALLET_SELECTION_MODAL => {
                self.wallets_modal.ui(ui, modal, &mut self.wallets, cb, |wallet, _| {
                    let id = wallet.get_config().id;
                    NodeConfig::save_stratum_wallet_id(id);
                    self.wallet_name = WalletConfig::name_by_id(id);
                })
            },
            STRATUM_PORT_MODAL => self.port_modal(ui, modal, cb),
            ATTEMPT_TIME_MODAL => self.attempt_modal(ui, modal, cb),
            MIN_SHARE_DIFF_MODAL => self.min_diff_modal(ui, modal, cb),
            _ => {}
        }
    }
}

impl StratumSetup {
    pub fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        // Draw modal content for current ui container.
        self.current_modal_ui(ui, cb);

        View::sub_title(ui, format!("{} {}", HARD_DRIVES, t!("network_mining.server")));
        View::horizontal_line(ui, Colors::stroke());
        ui.add_space(6.0);

        ui.vertical_centered(|ui| {
            // Show loading indicator or controls to start/stop stratum server.
            if Node::get_sync_status().unwrap_or(SyncStatus::Initial) == SyncStatus::NoSync &&
                self.is_port_available && self.wallet_name.is_some() {
                if Node::is_stratum_starting() || Node::is_stratum_stopping() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(8.0);
                        View::small_loading_spinner(ui);
                        ui.add_space(8.0);
                    });
                } else if Node::get_stratum_stats().is_running {
                    ui.add_space(6.0);
                    let disable_text = format!("{} {}", POWER, t!("network_settings.disable"));
                    View::action_button(ui, disable_text, || {
                        Node::stop_stratum();
                        let (ip, port) = NodeConfig::get_stratum_address();
                        self.is_port_available = NodeConfig::is_stratum_port_available(&ip, &port);
                    });
                    ui.add_space(6.0);
                } else {
                    ui.add_space(6.0);
                    let enable_text = format!("{} {}", POWER, t!("network_settings.enable"));
                    View::action_button(ui, enable_text, || {
                        Node::start_stratum();
                    });
                    ui.add_space(6.0);
                }
            }

            // Show stratum server autorun checkbox.
            let stratum_enabled = NodeConfig::is_stratum_autorun_enabled();
            View::checkbox(ui, stratum_enabled, t!("network.autorun"), || {
                NodeConfig::toggle_stratum_autorun();
            });

            // Show reminder to restart running server.
            if Node::get_stratum_stats().is_running {
                ui.add_space(2.0);
                ui.label(RichText::new(t!("network_mining.restart_server_required"))
                    .size(16.0)
                    .color(Colors::inactive_text())
                );
            }
            ui.add_space(8.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(8.0);

            // Show wallet name.
            ui.label(RichText::new(self.wallet_name.as_ref().unwrap_or(&"-".to_string()))
                .size(16.0)
                .color(Colors::white_or_black(true)));
            ui.add_space(8.0);

            // Show button to select wallet.
            View::button(ui,
                         t!("network_settings.choose_wallet"),
                         Colors::white_or_black(false), || {
                self.show_wallets_modal();
            });
            ui.add_space(12.0);

            if self.wallet_name.is_some() {
                ui.label(RichText::new(t!("network_settings.stratum_wallet_warning"))
                    .size(16.0)
                    .color(Colors::inactive_text())
                );
                ui.add_space(12.0);
            }
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(6.0);
        });

        // Show message when IP addresses are not available on the system.
        if self.available_ips.is_empty() {
            NetworkSettings::no_ip_address_ui(ui);
            return;
        }

        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.stratum_ip"))
                .size(16.0)
                .color(Colors::gray())
            );
            ui.add_space(6.0);
            // Show stratum IP addresses to select.
            let (ip, port) = NodeConfig::get_stratum_address();
            NetworkSettings::ip_addrs_ui(ui, &ip, &self.available_ips, |selected_ip| {
                NodeConfig::save_stratum_address(selected_ip, &port);
                self.is_port_available = NodeConfig::is_stratum_port_available(selected_ip, &port);

            });
            // Show stratum port setup.
            self.port_setup_ui(ui, cb);

            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(6.0);

            // Show attempt time setup.
            self.attempt_time_ui(ui, cb);

            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(6.0);

            // Show minimum acceptable share difficulty setup.
            self.min_diff_ui(ui, cb);
        });
    }

    /// Show wallet selection [`Modal`].
    fn show_wallets_modal(&mut self) {
        self.wallets_modal = WalletsModal::new(NodeConfig::get_stratum_wallet_id(), None, false);
        // Show modal.
        Modal::new(WALLET_SELECTION_MODAL)
            .position(ModalPosition::Center)
            .title(t!("network_settings.choose_wallet"))
            .show();
    }

    /// Draw stratum port value setup content.
    fn port_setup_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("network_settings.stratum_port"))
            .size(16.0)
            .color(Colors::gray())
        );
        ui.add_space(6.0);

        let (_, port) = NodeConfig::get_stratum_address();
        View::button(ui, format!("{} {}", PLUG, &port), Colors::white_or_black(false), || {
            // Setup values for modal.
            self.stratum_port_edit = port;
            self.stratum_port_available_edit = self.is_port_available;
            // Show stratum port modal.
            Modal::new(STRATUM_PORT_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"))
                .show();
            cb.show_keyboard();
        });
        ui.add_space(12.0);

        // Show error when stratum server port is unavailable.
        if !self.is_port_available {
            ui.add_space(6.0);
            ui.label(RichText::new(t!("network_settings.port_unavailable"))
                .size(16.0)
                .color(Colors::red()));
            ui.add_space(12.0);
        }
    }

    /// Draw stratum port [`Modal`] content.
    fn port_modal(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.stratum_port"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Draw stratum port text edit.
            let mut text_edit_opts = TextEditOptions::new(Id::from(modal.id)).h_center();
            View::text_edit(ui, cb, &mut self.stratum_port_edit, &mut text_edit_opts);

            // Show error when specified port is unavailable.
            if !self.stratum_port_available_edit {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.port_unavailable"))
                    .size(17.0)
                    .color(Colors::red()));
            } else {
                server_restart_required_ui(ui);
            }

            ui.add_space(12.0);

            // Show modal buttons.
            ui.scope(|ui| {
                // Setup spacing between buttons.
                ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

                // Save button callback.
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
        });
    }

    /// Draw attempt time value setup content.
    fn attempt_time_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("network_settings.attempt_time"))
            .size(16.0)
            .color(Colors::gray())
        );
        ui.add_space(6.0);

        let time = NodeConfig::get_stratum_attempt_time();
        View::button(ui, format!("{} {}", TIMER, &time), Colors::white_or_black(false), || {
            // Setup values for modal.
            self.attempt_time_edit = time;

            // Show attempt time modal.
            Modal::new(ATTEMPT_TIME_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"))
                .show();
            cb.show_keyboard();
        });
        ui.add_space(6.0);
        ui.label(RichText::new(t!("network_settings.attempt_time_desc"))
            .size(16.0)
            .color(Colors::inactive_text())
        );
        ui.add_space(6.0);
    }

    /// Draw attempt time [`Modal`] content.
    fn attempt_modal(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.attempt_time"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Draw attempt time text edit.
            let mut text_edit_opts = TextEditOptions::new(Id::from(modal.id)).h_center();
            View::text_edit(ui, cb, &mut self.attempt_time_edit, &mut text_edit_opts);

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.attempt_time_edit.parse::<u32>().is_err() {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.not_valid_value"))
                    .size(17.0)
                    .color(Colors::red()));
            } else {
                server_restart_required_ui(ui);
            }
            ui.add_space(12.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            // Save button callback.
            let on_save = || {
                if let Ok(time) = self.attempt_time_edit.parse::<u32>() {
                    NodeConfig::save_stratum_attempt_time(time);
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

    /// Draw minimum share difficulty value setup content.
    fn min_diff_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("network_settings.min_share_diff"))
            .size(16.0)
            .color(Colors::gray())
        );
        ui.add_space(6.0);

        let diff = NodeConfig::get_stratum_min_share_diff();
        View::button(ui, format!("{} {}", BARBELL, &diff), Colors::white_or_black(false), || {
            // Setup values for modal.
            self.min_share_diff_edit = diff;

            // Show share difficulty setup modal.
            Modal::new(MIN_SHARE_DIFF_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"))
                .show();
            cb.show_keyboard();
        });
        ui.add_space(6.0);
    }

    /// Draw minimum acceptable share difficulty [`Modal`] content.
    fn min_diff_modal(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.min_share_diff"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Draw share difficulty text edit.
            let mut text_edit_opts = TextEditOptions::new(Id::from(modal.id)).h_center();
            View::text_edit(ui, cb, &mut self.min_share_diff_edit, &mut text_edit_opts);

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.min_share_diff_edit.parse::<u64>().is_err() {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.not_valid_value"))
                    .size(17.0)
                    .color(Colors::red()));
            } else {
                server_restart_required_ui(ui);
            }
            ui.add_space(12.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            // Save button callback.
            let on_save = || {
                if let Ok(diff) = self.min_share_diff_edit.parse::<u64>() {
                    NodeConfig::save_stratum_min_share_diff(diff);
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
}

/// Reminder to restart enabled node to show on edit setting at [`Modal`].
pub fn server_restart_required_ui(ui: &mut egui::Ui) {
    if Node::get_stratum_stats().is_running {
        ui.add_space(12.0);
        ui.label(RichText::new(t!("network_mining.restart_server_required"))
            .size(16.0)
            .color(Colors::green())
        );
    }
}