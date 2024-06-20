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
use egui::scroll_area::ScrollBarVisibility;

use crate::gui::Colors;
use crate::gui::icons::ARROW_COUNTER_CLOCKWISE;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, Root, View};
use crate::gui::views::network::setup::{DandelionSetup, NodeSetup, P2PSetup, PoolSetup, StratumSetup};
use crate::gui::views::network::types::{NetworkTab, NetworkTabType};
use crate::gui::views::types::{ModalContainer, ModalPosition};
use crate::node::{Node, NodeConfig};

/// Integrated node settings tab content.
pub struct NetworkSettings {
    /// Integrated node general setup content.
    node: NodeSetup,
    /// P2P server setup content.
    p2p: P2PSetup,
    /// Stratum server setup content.
    stratum: StratumSetup,
    /// Pool setup content.
    pool: PoolSetup,
    /// Dandelion server setup content.
    dandelion: DandelionSetup,

    /// [`Modal`] identifiers allowed at this ui container.
    modal_ids: Vec<&'static str>
}

/// Identifier for settings reset confirmation [`Modal`].
pub const RESET_SETTINGS_MODAL: &'static str = "reset_settings";

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            node: NodeSetup::default(),
            p2p: P2PSetup::default(),
            stratum: StratumSetup::default(),
            pool: PoolSetup::default(),
            dandelion: DandelionSetup::default(),
            modal_ids: vec![
                RESET_SETTINGS_MODAL
            ]
        }
    }
}

impl ModalContainer for NetworkSettings {
    fn modal_ids(&self) -> &Vec<&'static str> {
        &self.modal_ids
    }

    fn modal_ui(&mut self,
                ui: &mut egui::Ui,
                modal: &Modal,
                _: &dyn PlatformCallbacks) {
        match modal.id {
            RESET_SETTINGS_MODAL  => reset_settings_confirmation_modal(ui, modal),
            _ => {}
        }
    }
}

impl NetworkTab for NetworkSettings {
    fn get_type(&self) -> NetworkTabType {
        NetworkTabType::Settings
    }

    fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        // Draw modal content for current ui container.
        self.current_modal_ui(ui, cb);

        ScrollArea::vertical()
            .id_source("network_settings")
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(1.0);
                ui.vertical_centered(|ui| {
                    View::max_width_ui(ui, Root::SIDE_PANEL_WIDTH * 1.3, |ui| {
                        // Draw node setup section.
                        self.node.ui(ui, cb);

                        ui.add_space(6.0);
                        View::horizontal_line(ui, Colors::stroke());
                        ui.add_space(4.0);

                        // Draw P2P server setup section.
                        self.p2p.ui(ui, cb);

                        ui.add_space(6.0);
                        View::horizontal_line(ui, Colors::stroke());
                        ui.add_space(4.0);

                        // Draw Stratum server setup section.
                        self.stratum.ui(ui, cb);

                        ui.add_space(6.0);
                        View::horizontal_line(ui, Colors::stroke());
                        ui.add_space(4.0);

                        // Draw pool setup section.
                        self.pool.ui(ui, cb);

                        ui.add_space(6.0);
                        View::horizontal_line(ui, Colors::stroke());
                        ui.add_space(4.0);

                        // Draw Dandelion server setup section.
                        self.dandelion.ui(ui, cb);

                        ui.add_space(6.0);
                        View::horizontal_line(ui, Colors::stroke());
                        ui.add_space(6.0);

                        // Draw reset settings content.
                        reset_settings_ui(ui);
                    });
                });
            });
    }
}

impl NetworkSettings {
    /// Reminder to restart enabled node to show on edit setting at [`Modal`].
    pub fn node_restart_required_ui(ui: &mut egui::Ui) {
        if Node::is_running() {
            ui.add_space(12.0);
            ui.label(RichText::new(t!("network_settings.restart_node_required"))
                .size(16.0)
                .color(Colors::green())
            );
        }
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
                .color(Colors::inactive_text())
            );
            ui.add_space(6.0);
        });
    }
}

/// Draw button to reset integrated node settings to default values.
fn reset_settings_ui(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.label(RichText::new(t!("network_settings.reset_settings_desc"))
            .size(16.0)
            .color(Colors::text(false)));
        ui.add_space(8.0);
        let button_text = format!("{} {}",
                                  ARROW_COUNTER_CLOCKWISE,
                                  t!("network_settings.reset_settings"));
        View::action_button(ui, button_text, || {
            // Show modal to confirm settings reset.
            Modal::new(RESET_SETTINGS_MODAL)
                .position(ModalPosition::Center)
                .title(t!("modal.confirmation"))
                .show();
        });

        // Show reminder to restart enabled node.
        if Node::is_running() {
            ui.add_space(12.0);
            ui.label(RichText::new(t!("network_settings.restart_node_required"))
                .size(16.0)
                .color(Colors::gray())
            );
        }
        ui.add_space(12.0);
    });

}

/// Confirmation to reset settings to default values.
fn reset_settings_confirmation_modal(ui: &mut egui::Ui, modal: &Modal) {
    ui.add_space(6.0);
    ui.vertical_centered(|ui| {
        let reset_text = format!("{}?", t!("network_settings.reset_settings_desc"));
        ui.label(RichText::new(reset_text)
            .size(17.0)
            .color(Colors::text(false)));
        ui.add_space(8.0);
    });

    // Show modal buttons.
    ui.scope(|ui| {
        // Setup spacing between buttons.
        ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

        ui.columns(2, |columns| {
            columns[0].vertical_centered_justified(|ui| {
                View::button(ui, t!("network_settings.reset"), Colors::white_or_black(false), || {
                    NodeConfig::reset_to_default();
                    modal.close();
                });
            });
            columns[1].vertical_centered_justified(|ui| {
                View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                    modal.close();
                });
            });
        });
        ui.add_space(6.0);
    });
}