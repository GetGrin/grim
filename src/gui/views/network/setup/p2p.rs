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

use egui::{Align, Id, Layout, RichText, StrokeKind};
use grin_core::global::ChainTypes;

use crate::AppConfig;
use crate::gui::Colors;
use crate::gui::icons::{ARROW_FAT_LINES_DOWN, ARROW_FAT_LINES_UP, GLOBE_SIMPLE, HANDSHAKE, PLUG, PLUS_CIRCLE, PROHIBIT_INSET, TRASH};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, TextEdit, View};
use crate::gui::views::network::settings::NetworkSettings;
use crate::gui::views::types::{ContentContainer, ModalPosition};
use crate::node::{Node, NodeConfig, PeersConfig};

/// Type of peer.
#[derive(Eq, PartialEq)]
enum PeerType {
    DefaultSeed,
    CustomSeed,
    Allowed,
    Denied,
    Preferred
}

/// P2P server setup section content.
pub struct P2PSetup {
    /// P2P port value.
    port_edit: String,
    /// Flag to check if p2p port is available.
    port_available_edit: bool,

    /// Flag to check if p2p port from saved config value is available.
    is_port_available: bool,

    /// Flag to check if entered peer address is correct and/or available.
    is_correct_address_edit: bool,
    /// Peer edit value for modal.
    peer_edit: String,

    /// Default main network seeds.
    default_main_seeds: Vec<String>,
    /// Default test network seeds.
    default_test_seeds: Vec<String>,

    /// How long banned peer should stay banned.
    ban_window_edit: String,

    /// Maximum number of inbound peer connections.
    max_inbound_count: String,

    /// Maximum number of outbound peer connections.
    max_outbound_count: String,

    /// Flag to check if reset of peers was called.
    peers_reset: bool,
}

/// Identifier for port value [`Modal`].
pub const PORT_MODAL: &'static str = "p2p_port";
/// Identifier for custom seed [`Modal`].
pub const CUSTOM_SEED_MODAL: &'static str = "p2p_custom_seed";
/// Identifier for allowed peer [`Modal`].
pub const ALLOW_PEER_MODAL: &'static str = "p2p_allow_peer";
/// Identifier for denied peer [`Modal`].
pub const DENY_PEER_MODAL: &'static str = "p2p_deny_peer";
/// Identifier for preferred peer [`Modal`].
pub const PREFER_PEER_MODAL: &'static str = "p2p_prefer_peer";
/// Identifier for ban window [`Modal`].
pub const BAN_WINDOW_MODAL: &'static str = "p2p_ban_window";
/// Identifier for maximum number of inbound peers [`Modal`].
pub const MAX_INBOUND_MODAL: &'static str = "p2p_max_inbound";
/// Identifier for maximum number of outbound peers [`Modal`].
pub const MAX_OUTBOUND_MODAL: &'static str = "p2p_max_outbound";

impl Default for P2PSetup {
    fn default() -> Self {
        let port = NodeConfig::get_p2p_port();
        let is_port_available = NodeConfig::is_p2p_port_available(&port);
        let default_main_seeds = Node::MAINNET_DNS_SEEDS
            .iter()
            .map(|s| s.to_string())
            .collect();
        let default_test_seeds = grin_servers::TESTNET_DNS_SEEDS
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        Self {
            port_edit: port,
            port_available_edit: is_port_available,
            is_correct_address_edit: true,
            is_port_available,
            peer_edit: "".to_string(),
            default_main_seeds,
            default_test_seeds,
            ban_window_edit: NodeConfig::get_p2p_ban_window(),
            max_inbound_count: NodeConfig::get_max_inbound_peers(),
            max_outbound_count: NodeConfig::get_max_outbound_peers(),
            peers_reset: false,
        }
    }
}

impl ContentContainer for P2PSetup {
    fn modal_ids(&self) -> Vec<&'static str> {
        vec![
            PORT_MODAL,
            CUSTOM_SEED_MODAL,
            ALLOW_PEER_MODAL,
            DENY_PEER_MODAL,
            PREFER_PEER_MODAL,
            BAN_WINDOW_MODAL,
            MAX_INBOUND_MODAL,
            MAX_OUTBOUND_MODAL
        ]
    }

    fn modal_ui(&mut self,
                ui: &mut egui::Ui,
                modal: &Modal,
                cb: &dyn PlatformCallbacks) {
        match modal.id {
            PORT_MODAL => self.port_modal(ui, modal, cb),
            CUSTOM_SEED_MODAL => self.peer_modal(ui, modal, cb),
            ALLOW_PEER_MODAL => self.peer_modal(ui, modal, cb),
            DENY_PEER_MODAL => self.peer_modal(ui, modal, cb),
            PREFER_PEER_MODAL => self.peer_modal(ui, modal, cb),
            BAN_WINDOW_MODAL => self.ban_window_modal(ui, modal, cb),
            MAX_INBOUND_MODAL => self.max_inbound_modal(ui, modal, cb),
            MAX_OUTBOUND_MODAL => self.max_outbound_modal(ui, modal, cb),
            _ => {}
        }
    }

    fn container_ui(&mut self, ui: &mut egui::Ui, _: &dyn PlatformCallbacks) {
        View::sub_title(ui, format!("{} {}", HANDSHAKE, t!("network_settings.p2p_server")));
        View::horizontal_line(ui, Colors::stroke());
        ui.add_space(6.0);

        ui.vertical_centered(|ui| {
            // Show p2p port setup.
            self.port_ui(ui);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(6.0);

            // Show seeding type setup.
            self.seeding_type_ui(ui);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(6.0);

            ui.label(RichText::new(t!("network_settings.allow_list"))
                .size(16.0)
                .color(Colors::gray()));
            ui.add_space(6.0);
            // Show allowed peers setup.
            self.peer_list_ui(ui, &PeerType::Allowed);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(6.0);

            ui.label(RichText::new(t!("network_settings.deny_list"))
                .size(16.0)
                .color(Colors::gray()));
            ui.add_space(6.0);
            // Show denied peers setup.
            self.peer_list_ui(ui, &PeerType::Denied);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(6.0);

            ui.label(RichText::new(t!("network_settings.favourites"))
                .size(16.0)
                .color(Colors::gray()));
            ui.add_space(6.0);
            // Show preferred peers setup.
            self.peer_list_ui(ui, &PeerType::Preferred);


            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(6.0);

            // Show ban window setup.
            self.ban_window_ui(ui);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(6.0);

            // Show maximum inbound peers value setup.
            self.max_inbound_ui(ui);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(6.0);

            // Show maximum outbound peers value setup.
            self.max_outbound_ui(ui);

            if !Node::is_restarting() && !self.peers_reset {
                ui.add_space(6.0);
                View::horizontal_line(ui, Colors::item_stroke());
                ui.add_space(6.0);

                // Show peers data reset content.
                self.reset_peers_ui(ui);
            }
        });
    }
}

/// Title for custom DNS Seeds setup section.
const DNS_SEEDS_TITLE: &'static str = "DNS Seeds";

impl P2PSetup {
    /// Draw p2p port setup content.
    fn port_ui(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(t!("network_settings.p2p_port"))
            .size(16.0)
            .color(Colors::gray())
        );
        ui.add_space(6.0);

        let port = NodeConfig::get_p2p_port();
        View::button(ui,
                     format!("{} {}", PLUG, &port),
                     Colors::white_or_black(false), || {
                // Setup values for modal.
                self.port_edit = port;
                self.port_available_edit = self.is_port_available;
                // Show p2p port modal.
                Modal::new(PORT_MODAL)
                    .position(ModalPosition::CenterTop)
                    .title(t!("network_settings.change_value"))
                    .show();
            });
        ui.add_space(6.0);

        // Show error when p2p port is unavailable.
        if !self.is_port_available {
            ui.add_space(6.0);
            ui.label(RichText::new(t!("network_settings.port_unavailable"))
                .size(16.0)
                .color(Colors::red()));
            ui.add_space(12.0);
        }
    }

    /// Draw p2p port [`Modal`] content.
    fn port_modal(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        let on_save = |c: &mut P2PSetup| {
            // Check if port is available.
            let available = NodeConfig::is_p2p_port_available(&c.port_edit);
            c.port_available_edit = available;

            // Save port at config if it's available.
            if available {
                NodeConfig::save_p2p_port(c.port_edit.parse::<u16>().unwrap());
                if Node::is_running() {
                    Node::restart();
                }
                c.is_port_available = true;
                Modal::close();
            }
        };

        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.p2p_port"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Draw p2p port text edit.
            let mut edit = TextEdit::new(Id::from(modal.id)).h_center().numeric();
            edit.ui(ui, &mut self.port_edit, cb);
            if edit.enter_pressed {
                on_save(self);
            }

            // Show error when specified port is unavailable.
            if !self.port_available_edit {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.port_unavailable"))
                    .size(17.0)
                    .color(Colors::red()));
            }

            ui.add_space(12.0);

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
        });
    }

    /// Draw peer list content based on provided [`PeerType`].
    fn peer_list_ui(&mut self, ui: &mut egui::Ui, peer_type: &PeerType) {
        let peers = match peer_type {
            PeerType::DefaultSeed => {
                if AppConfig::chain_type() == ChainTypes::Testnet {
                    self.default_test_seeds.clone()
                } else {
                    self.default_main_seeds.clone()
                }
            }
            PeerType::CustomSeed => NodeConfig::get_custom_seeds(),
            PeerType::Allowed => NodeConfig::get_allowed_peers(),
            PeerType::Denied => NodeConfig::get_denied_peers(),
            PeerType::Preferred => NodeConfig::get_preferred_peers()
        };
        for (index, peer) in peers.iter().enumerate() {
            ui.horizontal_wrapped(|ui| {
                // Draw peer list item.
                peer_item_ui(ui, peer, peer_type, index, peers.len());
            });
        }

        if peer_type != &PeerType::DefaultSeed {
            // Draw description.
            if peer_type != &PeerType::CustomSeed {
                if !peers.is_empty() {
                    ui.add_space(12.0);
                }
                let desc = match peer_type {
                    PeerType::Allowed => t!("network_settings.allow_list_desc"),
                    PeerType::Denied => t!("network_settings.deny_list_desc"),
                    &_ => t!("network_settings.favourites_desc"),
                };
                ui.label(RichText::new(desc)
                    .size(16.0)
                    .color(Colors::inactive_text()));
                ui.add_space(12.0);
            } else if !peers.is_empty() {
                ui.add_space(12.0);
            }

            let add_text = if peer_type == &PeerType::CustomSeed {
                format!("{} {}", PLUS_CIRCLE, t!("network_settings.add_seed"))
            } else {
                format!("{} {}", PLUS_CIRCLE, t!("network_settings.add_peer"))

            };
            View::button(ui, add_text, Colors::white_or_black(false), || {
                // Setup values for modal.
                self.is_correct_address_edit = true;
                self.peer_edit = "".to_string();
                // Select modal id.
                let modal_id = match peer_type {
                    PeerType::Allowed => ALLOW_PEER_MODAL,
                    PeerType::Denied => DENY_PEER_MODAL,
                    PeerType::Preferred => PREFER_PEER_MODAL,
                    _ => CUSTOM_SEED_MODAL
                };
                // Select modal title.
                let modal_title = match peer_type {
                    PeerType::Allowed => t!("network_settings.allow_list"),
                    PeerType::Denied => t!("network_settings.deny_list"),
                    PeerType::Preferred => t!("network_settings.favourites"),
                    _ => DNS_SEEDS_TITLE.to_string()
                };
                // Show modal to add peer.
                Modal::new(modal_id)
                    .position(ModalPosition::CenterTop)
                    .title(modal_title)
                    .show();
            });
        }
        ui.add_space(6.0);
    }

    /// Draw peer creation [`Modal`] content.
    fn peer_modal(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        let on_save = |c: &mut P2PSetup| {
            // Check if peer is correct and/or available.
            let peer = c.peer_edit.clone();
            let is_correct_address = PeersConfig::peer_to_addr(peer.clone()).is_some();
            c.is_correct_address_edit = is_correct_address;

            // Save peer at config.
            if is_correct_address {
                match modal.id {
                    CUSTOM_SEED_MODAL => NodeConfig::save_custom_seed(peer),
                    ALLOW_PEER_MODAL => NodeConfig::allow_peer(peer),
                    DENY_PEER_MODAL => NodeConfig::deny_peer(peer),
                    PREFER_PEER_MODAL => NodeConfig::prefer_peer(peer),
                    &_ => {}
                }

                c.is_port_available = true;
                Modal::close();
            }
        };

        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            let label_text = match modal.id {
                CUSTOM_SEED_MODAL => t!("network_settings.seed_address"),
                &_ => t!("network_settings.peer_address")
            };
            ui.label(RichText::new(label_text).size(17.0).color(Colors::gray()));
            ui.add_space(8.0);

            // Draw peer address text edit.
            let mut edit = TextEdit::new(Id::from(modal.id)).paste();
            edit.ui(ui, &mut self.peer_edit, cb);
            if edit.enter_pressed {
                on_save(self);
            }

            // Show error when specified address is incorrect.
            if !self.is_correct_address_edit {
                ui.add_space(10.0);
                ui.label(RichText::new(t!("network_settings.peer_address_error"))
                    .size(16.0)
                    .color(Colors::red()));
            }
            ui.add_space(12.0);

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
        });
    }

    /// Draw seeding type setup content.
    fn seeding_type_ui(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(DNS_SEEDS_TITLE).size(16.0).color(Colors::gray()));
        ui.add_space(2.0);

        let default_seeding = NodeConfig::is_default_seeding_type();
        View::checkbox(ui, default_seeding, t!("network_settings.default"), || {
            NodeConfig::toggle_seeding_type();
        });
        ui.add_space(8.0);

        let peers_type = if default_seeding {
            PeerType::DefaultSeed
        } else {
            PeerType::CustomSeed
        };
        self.peer_list_ui(ui, &peers_type);
    }

    /// Draw ban window setup content.
    fn ban_window_ui(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(t!("network_settings.ban_window"))
            .size(16.0)
            .color(Colors::gray())
        );
        ui.add_space(6.0);

        let ban_window = NodeConfig::get_p2p_ban_window();
        View::button(ui,
                     format!("{} {}", PROHIBIT_INSET, &ban_window),
                     Colors::white_or_black(false), || {
                // Setup values for modal.
                self.ban_window_edit = ban_window;
                // Show ban window period setup modal.
                Modal::new(BAN_WINDOW_MODAL)
                    .position(ModalPosition::CenterTop)
                    .title(t!("network_settings.change_value"))
                    .show();
            });
        ui.add_space(6.0);
        ui.label(RichText::new(t!("network_settings.ban_window_desc"))
            .size(16.0)
            .color(Colors::inactive_text())
        );
        ui.add_space(2.0);
    }

    /// Draw ban window [`Modal`] content.
    fn ban_window_modal(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        let on_save = |c: &mut P2PSetup| {
            if let Ok(ban_window) = c.ban_window_edit.parse::<i64>() {
                NodeConfig::save_p2p_ban_window(ban_window);
                Modal::close();
            }
        };

        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.ban_window"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Draw ban window text edit.
            let mut edit = TextEdit::new(Id::from(modal.id)).h_center().numeric();
            edit.ui(ui, &mut self.ban_window_edit, cb);
            if edit.enter_pressed {
                on_save(self);
            }

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.ban_window_edit.parse::<i64>().is_err() {
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

    /// Draw maximum number of inbound peers setup content.
    fn max_inbound_ui(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(t!("network_settings.max_inbound_count"))
            .size(16.0)
            .color(Colors::gray())
        );
        ui.add_space(6.0);

        let max_inbound = NodeConfig::get_max_inbound_peers();
        View::button(ui,
                     format!("{} {}", ARROW_FAT_LINES_DOWN, &max_inbound),
                     Colors::white_or_black(false), || {
                // Setup values for modal.
                self.max_inbound_count = max_inbound;
                // Show maximum number of inbound peers setup modal.
                Modal::new(MAX_INBOUND_MODAL)
                    .position(ModalPosition::CenterTop)
                    .title(t!("network_settings.change_value"))
                    .show();
            });
        ui.add_space(6.0);
    }

    /// Draw maximum number of inbound peers [`Modal`] content.
    fn max_inbound_modal(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        let on_save = |c: &mut P2PSetup| {
            if let Ok(max_inbound) = c.max_inbound_count.parse::<u32>() {
                NodeConfig::save_max_inbound_peers(max_inbound);
                Modal::close();
            }
        };

        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.max_inbound_count"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Draw maximum number of inbound peers text edit.
            let mut edit = TextEdit::new(Id::from(modal.id)).h_center().numeric();
            edit.ui(ui, &mut self.max_inbound_count, cb);
            if edit.enter_pressed {
                on_save(self);
            }

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.max_inbound_count.parse::<u32>().is_err() {
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

    /// Draw maximum number of outbound peers setup content.
    fn max_outbound_ui(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(t!("network_settings.max_outbound_count"))
            .size(16.0)
            .color(Colors::gray())
        );
        ui.add_space(6.0);
        let max_outbound = NodeConfig::get_max_outbound_peers();
        View::button(ui,
                     format!("{} {}", ARROW_FAT_LINES_UP, &max_outbound),
                     Colors::white_or_black(false), || {
                // Setup values for modal.
                self.max_outbound_count = max_outbound;
                // Show maximum number of outbound peers setup modal.
                Modal::new(MAX_OUTBOUND_MODAL)
                    .position(ModalPosition::CenterTop)
                    .title(t!("network_settings.change_value"))
                    .show();
            });
        ui.add_space(6.0);
    }

    /// Draw maximum number of outbound peers [`Modal`] content.
    fn max_outbound_modal(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        let on_save = |c: &mut P2PSetup| {
            if let Ok(max_outbound) = c.max_outbound_count.parse::<u32>() {
                NodeConfig::save_max_outbound_peers(max_outbound);
                Modal::close();
            }
        };

        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.max_outbound_count"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Draw maximum number of outbound peers text edit.
            let mut edit = TextEdit::new(Id::from(modal.id)).h_center().numeric();
            edit.ui(ui, &mut self.max_outbound_count, cb);
            if edit.enter_pressed {
                on_save(self);
            }

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.max_outbound_count.parse::<u32>().is_err() {
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

    /// Draw content to reset peers data.
    fn reset_peers_ui(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        View::colored_text_button(ui,
                                  format!("{} {}", TRASH, t!("network_settings.reset_peers")),
                                  Colors::red(),
                                  Colors::white_or_black(false), || {
                Node::reset_peers(false);
                self.peers_reset = true;
            });
        ui.add_space(6.0);
        ui.label(RichText::new(t!("network_settings.reset_peers_desc"))
            .size(16.0)
            .color(Colors::inactive_text())
        );
    }
}

/// Draw peer list item.
fn peer_item_ui(ui: &mut egui::Ui,
                peer_addr: &String,
                peer_type: &PeerType,
                index: usize,
                len: usize,) {
    // Setup layout size.
    let mut rect = ui.available_rect_before_wrap();
    rect.set_height(42.0);

    // Draw round background.
    let mut bg_rect = rect.clone();
    bg_rect.min += egui::emath::vec2(6.0, 0.0);
    let item_rounding = View::item_rounding(index, len, false);
    ui.painter().rect(bg_rect,
                      item_rounding,
                      Colors::white_or_black(false),
                      View::item_stroke(),
                      StrokeKind::Middle);

    ui.vertical(|ui| {
        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            // Draw delete button for non-default seed peers.
            if peer_type != &PeerType::DefaultSeed {
                View::item_button(ui, View::item_rounding(index, len, true), TRASH, None, || {
                    match peer_type {
                        PeerType::CustomSeed => {
                            NodeConfig::remove_custom_seed(peer_addr);
                        }
                        PeerType::Allowed => {
                            NodeConfig::remove_allowed_peer(peer_addr);
                        }
                        PeerType::Denied => {
                            NodeConfig::remove_denied_peer(peer_addr);
                        }
                        PeerType::Preferred => {
                            NodeConfig::remove_preferred_peer(peer_addr);
                        }
                        PeerType::DefaultSeed => {}
                    }
                });
            }

            let layout_size = ui.available_size();
            ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                ui.add_space(12.0);
                // Draw peer address.
                let peer_text = format!("{} {}", GLOBE_SIMPLE, &peer_addr);
                ui.label(RichText::new(peer_text)
                    .color(Colors::text_button())
                    .size(16.0));
            });
        });
    });
}