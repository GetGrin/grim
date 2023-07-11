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

use egui::{RichText, ScrollArea};

use crate::gui::{Colors, Navigator};
use crate::gui::icons::ARROW_COUNTER_CLOCKWISE;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, ModalPosition, View};
use crate::gui::views::network::{NetworkTab, NetworkTabType};
use crate::gui::views::network::configs::dandelion::DandelionSetup;
use crate::gui::views::network::configs::node::NodeSetup;
use crate::gui::views::network::configs::p2p::P2PSetup;
use crate::gui::views::network::configs::pool::PoolSetup;
use crate::gui::views::network::configs::stratum::StratumSetup;
use crate::node::{Node, NodeConfig};

/// Integrated node settings tab content.
#[derive(Default)]
pub struct NetworkSettings {
    node: NodeSetup,
    p2p: P2PSetup,
    stratum: StratumSetup,
    pool: PoolSetup,
    dandelion: DandelionSetup
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
                self.node.ui(ui, cb);

                ui.add_space(6.0);
                View::horizontal_line(ui, Colors::STROKE);
                ui.add_space(4.0);

                self.p2p.ui(ui, cb);

                ui.add_space(6.0);
                View::horizontal_line(ui, Colors::STROKE);
                ui.add_space(4.0);

                self.stratum.ui(ui, cb);

                ui.add_space(6.0);
                View::horizontal_line(ui, Colors::STROKE);
                ui.add_space(4.0);

                self.pool.ui(ui, cb);

                ui.add_space(6.0);
                View::horizontal_line(ui, Colors::STROKE);
                ui.add_space(4.0);

                self.dandelion.ui(ui, cb);

                ui.add_space(6.0);
                View::horizontal_line(ui, Colors::STROKE);
                ui.add_space(6.0);

                self.reset_settings_ui(ui);
            });
    }

    fn on_modal_ui(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        match modal.id {
            // Settings modals.
            Self::NODE_RESTART_REQUIRED_MODAL => self.node_restart_required_modal(ui, modal),
            Self::RESET_SETTINGS_MODAL  => self.reset_settings_confirmation_modal(ui, modal),
            // Node server setup modals.
            NodeSetup::API_PORT_MODAL => self.node.api_port_modal(ui, modal, cb),
            NodeSetup::API_SECRET_MODAL => self.node.secret_modal(ui, modal, cb),
            NodeSetup::FOREIGN_API_SECRET_MODAL => self.node.secret_modal(ui, modal, cb),
            NodeSetup::FTL_MODAL => self.node.ftl_modal(ui, modal, cb),
            // P2P setup modals.
            P2PSetup::PORT_MODAL => self.p2p.port_modal(ui, modal, cb),
            P2PSetup::CUSTOM_SEED_MODAL => self.p2p.peer_modal(ui, modal, cb),
            P2PSetup::ALLOW_PEER_MODAL => self.p2p.peer_modal(ui, modal, cb),
            P2PSetup::DENY_PEER_MODAL => self.p2p.peer_modal(ui, modal, cb),
            P2PSetup::PREFER_PEER_MODAL => self.p2p.peer_modal(ui, modal, cb),
            P2PSetup::BAN_WINDOW_MODAL => self.p2p.ban_window_modal(ui, modal, cb),
            P2PSetup::MAX_INBOUND_MODAL => self.p2p.max_inbound_modal(ui, modal, cb),
            P2PSetup::MAX_OUTBOUND_MODAL => self.p2p.max_outbound_modal(ui, modal, cb),
            P2PSetup::MIN_OUTBOUND_MODAL => self.p2p.min_outbound_modal(ui, modal, cb),
            // Stratum server setup modals.
            StratumSetup::STRATUM_PORT_MODAL => self.stratum.port_modal(ui, modal, cb),
            StratumSetup::ATTEMPT_TIME_MODAL => self.stratum.attempt_modal(ui, modal, cb),
            StratumSetup::MIN_SHARE_DIFF_MODAL => self.stratum.min_diff_modal(ui, modal, cb),
            // Pool setup modals.
            PoolSetup::FEE_BASE_MODAL => self.pool.fee_base_modal(ui, modal, cb),
            PoolSetup::REORG_PERIOD_MODAL => self.pool.reorg_period_modal(ui, modal, cb),
            PoolSetup::POOL_SIZE_MODAL => self.pool.pool_size_modal(ui, modal, cb),
            PoolSetup::STEMPOOL_SIZE_MODAL => self.pool.stempool_size_modal(ui, modal, cb),
            PoolSetup::MAX_WEIGHT_MODAL => self.pool.max_weight_modal(ui, modal, cb),
            // Dandelion setup modals.
            DandelionSetup::EPOCH_MODAL => self.dandelion.epoch_modal(ui, modal, cb),
            DandelionSetup::EMBARGO_MODAL => self.dandelion.embargo_modal(ui, modal, cb),
            DandelionSetup::AGGREGATION_MODAL => self.dandelion.aggregation_modal(ui, modal, cb),
            DandelionSetup::STEM_PROBABILITY_MODAL => self.dandelion.stem_prob_modal(ui, modal, cb),
            _ => {}
        }
    }
}

impl NetworkSettings {
    /// Identifier for node restart confirmation [`Modal`].
    pub const NODE_RESTART_REQUIRED_MODAL: &'static str = "node_restart_required";
    /// Identifier for settings reset confirmation [`Modal`].
    pub const RESET_SETTINGS_MODAL: &'static str = "reset_settings";

    /// Draw button to reset integrated node settings to default values.
    fn reset_settings_ui(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.reset_settings_desc"))
                .size(16.0)
                .color(Colors::TEXT));
            ui.add_space(8.0);
            let button_text = format!("{} {}",
                                      ARROW_COUNTER_CLOCKWISE,
                                      t!("network_settings.reset_settings"));
            View::button(ui, button_text, Colors::GOLD, || {
                // Show modal to confirm settings reset.
                let reset_modal = Modal::new(Self::RESET_SETTINGS_MODAL)
                    .position(ModalPosition::Center)
                    .title(t!("modal.confirmation"));
                Navigator::show_modal(reset_modal);
            });

            // Show reminder to restart enabled node.
            if Node::is_running() {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.restart_node_required"))
                    .size(16.0)
                    .color(Colors::GRAY)
                );
            }
            ui.add_space(12.0);
        });

    }

    /// Confirmation to reset settings to default values.
    fn reset_settings_confirmation_modal(&self, ui: &mut egui::Ui, modal: &Modal) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.reset_settings_desc"))
                .size(18.0)
                .color(Colors::TEXT));
            ui.add_space(8.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("network_settings.reset"), Colors::WHITE, || {
                        NodeConfig::reset_to_default();
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
            let port_modal = Modal::new(Self::NODE_RESTART_REQUIRED_MODAL)
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
                       ips: &Vec<String>,
                       on_change: impl FnOnce(&String)) {
        let mut selected_ip = saved_ip;

        // Set first IP address as current if saved is not present at system.
        if !ips.contains(saved_ip) {
            selected_ip = ips.get(0).unwrap();
        }

        ui.add_space(2.0);

        // Show available IP addresses on the system.
        let _ = ips.chunks(2).map(|x| {
            if x.len() == 2 {
                ui.columns(2, |columns| {
                    let ip_left = x.get(0).unwrap();
                    columns[0].vertical_centered(|ui| {
                        View::radio_value(ui, &mut selected_ip, ip_left, ip_left.to_string());
                    });
                    let ip_right = x.get(1).unwrap();
                    columns[1].vertical_centered(|ui| {
                        View::radio_value(ui, &mut selected_ip, ip_right, ip_right.to_string());
                    })
                });
            } else {
                let ip = x.get(0).unwrap();
                View::radio_value(ui, &mut selected_ip, ip, ip.to_string());
            }
            ui.add_space(12.0);
        }).collect::<Vec<_>>();

        if saved_ip != selected_ip {
            (on_change)(&selected_ip.to_string());
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