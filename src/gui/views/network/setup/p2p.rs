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

use egui::{Id, RichText, Rounding, Stroke, TextStyle, Ui, Widget};
use egui_extras::{Size, StripBuilder};
use grin_core::global::ChainTypes;

use crate::AppConfig;
use crate::gui::Colors;
use crate::gui::icons::{HANDSHAKE, PLUG, TRASH, GLOBE_SIMPLE, PLUS_CIRCLE, ARROW_FAT_LINES_UP, ARROW_FAT_LINES_DOWN, ARROW_FAT_LINE_UP, PROHIBIT_INSET, CLIPBOARD_TEXT};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, ModalPosition, View};
use crate::gui::views::network::settings::NetworkSettings;
use crate::node::{NodeConfig, PeersConfig};

/// Type of peer.
#[derive(Eq, PartialEq)]
enum PeerType {
    DefaultSeed,
    CustomSeed,
    Allowed,
    Denied,
    Preferred
}

/// P2P server setup ui section.
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

    /// Preferred minimum number of outbound peers.
    min_outbound_count: String
}

impl Default for P2PSetup {
    fn default() -> Self {
        let port = NodeConfig::get_p2p_port();
        let is_port_available = NodeConfig::is_p2p_port_available(&port);
        let default_main_seeds = grin_servers::MAINNET_DNS_SEEDS
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
            min_outbound_count: NodeConfig::get_min_outbound_peers(),
        }
    }
}

impl P2PSetup {
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
    /// Identifier for minimum number of outbound peers [`Modal`].
    pub const MIN_OUTBOUND_MODAL: &'static str = "p2p_min_outbound";

    /// Title for custom DNS Seeds setup section.
    const DNS_SEEDS_TITLE: &'static str = "DNS Seeds";

    pub fn ui(&mut self, ui: &mut Ui, cb: &dyn PlatformCallbacks) {
        View::sub_title(ui, format!("{} {}", HANDSHAKE, t!("network_settings.p2p_server")));
        View::horizontal_line(ui, Colors::STROKE);
        ui.add_space(6.0);

        ui.vertical_centered(|ui| {
            // Show p2p port setup.
            self.port_ui(ui, cb);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);

            // Show seeding type setup.
            self.seeding_type_ui(ui, cb);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);

            ui.label(RichText::new(t!("network_settings.allow_list"))
                .size(16.0)
                .color(Colors::GRAY));
            ui.add_space(6.0);
            // Show allowed peers setup.
            self.peer_list_ui(ui, &PeerType::Allowed, cb);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);

            ui.label(RichText::new(t!("network_settings.deny_list"))
                .size(16.0)
                .color(Colors::GRAY));
            ui.add_space(6.0);
            // Show denied peers setup.
            self.peer_list_ui(ui, &PeerType::Denied, cb);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);

            ui.label(RichText::new(t!("network_settings.favourites"))
                .size(16.0)
                .color(Colors::GRAY));
            ui.add_space(6.0);
            // Show preferred peers setup.
            self.peer_list_ui(ui, &PeerType::Preferred, cb);


            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);

            // Show ban window setup.
            self.ban_window_ui(ui, cb);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);

            // Show maximum inbound peers value setup.
            self.max_inbound_ui(ui, cb);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);

            // Show maximum outbound peers value setup.
            self.max_outbound_ui(ui, cb);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);

            // Show minimum outbound peers value setup.
            self.min_outbound_ui(ui, cb);
        });
    }

    /// Draw p2p port setup content.
    fn port_ui(&mut self, ui: &mut Ui, cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("network_settings.p2p_port"))
            .size(16.0)
            .color(Colors::GRAY)
        );
        ui.add_space(6.0);

        let port = NodeConfig::get_p2p_port();
        View::button(ui, format!("{} {}", PLUG, port.clone()), Colors::BUTTON, || {
            // Setup values for modal.
            self.port_edit = port;
            self.port_available_edit = self.is_port_available;
            // Show p2p port modal.
            Modal::new(Self::PORT_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"))
                .show();
            cb.show_keyboard();
        });
        ui.add_space(6.0);

        // Show error when p2p port is unavailable.
        if !self.is_port_available {
            ui.add_space(6.0);
            ui.label(RichText::new(t!("network_settings.port_unavailable"))
                .size(16.0)
                .color(Colors::RED));
            ui.add_space(12.0);
        }
    }

    /// Draw p2p port [`Modal`] content.
    pub fn port_modal(&mut self, ui: &mut Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.p2p_port"))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw p2p port text edit.
            let text_edit_resp = egui::TextEdit::singleline(&mut self.port_edit)
                .id(Id::from(modal.id))
                .font(TextStyle::Heading)
                .desired_width(64.0)
                .cursor_at_end(true)
                .ui(ui);
            text_edit_resp.request_focus();
            if text_edit_resp.clicked() {
                cb.show_keyboard();
            }

            // Show error when specified port is unavailable.
            if !self.port_available_edit {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.port_unavailable"))
                    .size(17.0)
                    .color(Colors::RED));
            }

            ui.add_space(12.0);

            // Show modal buttons.
            ui.scope(|ui| {
                // Setup spacing between buttons.
                ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

                // Save button callback.
                let on_save = || {
                    // Check if port is available.
                    let available = NodeConfig::is_p2p_port_available(&self.port_edit);
                    self.port_available_edit = available;

                    // Save port at config if it's available.
                    if available {
                        NodeConfig::save_p2p_port(self.port_edit.parse::<u16>().unwrap());

                        self.is_port_available = true;
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
        });
    }

    /// Draw peer list content based on provided [`PeerType`].
    fn peer_list_ui(&mut self, ui: &mut Ui, peer_type: &PeerType, cb: &dyn PlatformCallbacks) {
        let peer_list = match peer_type {
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
        for (index, peer) in peer_list.iter().enumerate() {
            let rounding = if peer_list.len() == 1 {
                [true, true]
            } else if index == 0 {
                [true, false]
            } else if index == peer_list.len() - 1 {
                [false, true]
            } else {
                [false, false]
            };
            ui.horizontal_wrapped(|ui| {
                // Draw peer list item.
                Self::peer_item_ui(ui, peer, peer_type, rounding);
            });
        }

        if peer_type != &PeerType::DefaultSeed {
            // Draw description.
            if peer_type != &PeerType::CustomSeed {
                if !peer_list.is_empty() {
                    ui.add_space(12.0);
                }
                let desc = match peer_type {
                    PeerType::Allowed => t!("network_settings.allow_list_desc"),
                    PeerType::Denied => t!("network_settings.deny_list_desc"),
                    &_ => t!("network_settings.favourites_desc"),
                };
                ui.label(RichText::new(desc)
                    .size(16.0)
                    .color(Colors::INACTIVE_TEXT));
                ui.add_space(12.0);
            }

            let add_text = if peer_type == &PeerType::CustomSeed {
                format!("{} {}", PLUS_CIRCLE, t!("network_settings.add_seed"))
            } else {
                format!("{} {}", PLUS_CIRCLE, t!("network_settings.add_peer"))

            };
            View::button(ui, add_text, Colors::BUTTON, || {
                // Setup values for modal.
                self.peer_edit = "".to_string();
                // Select modal id.
                let modal_id = match peer_type {
                    PeerType::Allowed => Self::ALLOW_PEER_MODAL,
                    PeerType::Denied => Self::DENY_PEER_MODAL,
                    PeerType::Preferred => Self::PREFER_PEER_MODAL,
                    _ => Self::CUSTOM_SEED_MODAL
                };
                // Select modal title.
                let modal_title = match peer_type {
                    PeerType::Allowed => t!("network_settings.allow_list"),
                    PeerType::Denied => t!("network_settings.deny_list"),
                    PeerType::Preferred => t!("network_settings.favourites"),
                    _ => Self::DNS_SEEDS_TITLE.to_string()
                };
                // Show modal to add peer.
                Modal::new(modal_id)
                    .position(ModalPosition::CenterTop)
                    .title(modal_title)
                    .show();
                cb.show_keyboard();
            });
        }
        ui.add_space(6.0);
    }

    /// Draw peer creation [`Modal`] content.
    pub fn peer_modal(&mut self, ui: &mut Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            let label_text = match modal.id {
                Self::CUSTOM_SEED_MODAL => t!("network_settings.seed_address"),
                &_ => t!("network_settings.peer_address")
            };
            ui.label(RichText::new(label_text).size(17.0).color(Colors::GRAY));
            ui.add_space(8.0);
            StripBuilder::new(ui)
                .size(Size::exact(42.0))
                .vertical(|mut strip| {
                    strip.strip(|builder| {
                        builder
                            .size(Size::remainder())
                            .size(Size::exact(48.0))
                            .horizontal(|mut strip| {
                                strip.cell(|ui| {
                                    ui.add_space(2.0);
                                    // Draw peer address text edit.
                                    let text_edit = egui::TextEdit::singleline(&mut self.peer_edit)
                                        .id(Id::from(modal.id))
                                        .font(TextStyle::Button)
                                        .desired_width(ui.available_width())
                                        .cursor_at_end(true)
                                        .ui(ui);
                                    text_edit.request_focus();
                                    if text_edit.clicked() {
                                        cb.show_keyboard();
                                    }
                                });
                                strip.cell(|ui| {
                                    ui.vertical_centered(|ui| {
                                        // Draw paste button.
                                        let paste_icon = CLIPBOARD_TEXT.to_string();
                                        View::button(ui, paste_icon, Colors::WHITE, || {
                                            self.peer_edit = cb.get_string_from_buffer();
                                        });
                                    });
                                });
                            });
                    })
                });

            // Show error when specified address is incorrect.
            if !self.is_correct_address_edit {
                ui.label(RichText::new(t!("network_settings.peer_address_error"))
                    .size(16.0)
                    .color(Colors::RED));
                ui.add_space(6.0);
            }

            ui.add_space(4.0);

            // Show modal buttons.
            ui.scope(|ui| {
                // Setup spacing between buttons.
                ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

                // Save button callback.
                let on_save = || {
                    // Check if peer is correct and/or available.
                    let peer = self.peer_edit.clone();
                    let is_correct_address = PeersConfig::peer_to_addr(peer.clone()).is_some();
                    self.is_correct_address_edit = is_correct_address;

                    // Save peer at config.
                    if is_correct_address {
                        match modal.id {
                            Self::CUSTOM_SEED_MODAL => NodeConfig::save_custom_seed(peer),
                            Self::ALLOW_PEER_MODAL => NodeConfig::allow_peer(peer),
                            Self::DENY_PEER_MODAL => NodeConfig::deny_peer(peer),
                            Self::PREFER_PEER_MODAL => NodeConfig::prefer_peer(peer),
                            &_ => {}
                        }

                        self.is_port_available = true;
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
        });
    }

    /// Draw peer list item.
    fn peer_item_ui(ui: &mut Ui, peer_addr: &String, peer_type: &PeerType, rounding: [bool; 2]) {
        // Draw round background.
        let mut rect = ui.available_rect_before_wrap();
        rect.min += egui::emath::vec2(6.0, 0.0);
        rect.set_height(42.0);
        ui.painter().rect(
            rect,
            Rounding {
                nw: if rounding[0] { 6.0 } else { 0.0 },
                ne: if rounding[0] { 6.0 } else { 0.0 },
                sw: if rounding[1] { 6.0 } else { 0.0 },
                se: if rounding[1] { 6.0 } else { 0.0 },
            },
            Colors::WHITE,
            Stroke { width: 1.0, color: Colors::ITEM_STROKE }
        );

        StripBuilder::new(ui)
            .size(Size::exact(42.0))
            .vertical(|mut strip| {
                strip.strip(|builder| {
                    builder
                        .size(Size::exact(13.0))
                        .size(Size::remainder())
                        .size(Size::exact(46.0))
                        .horizontal(|mut strip| {
                            strip.empty();
                            strip.cell(|ui| {
                                ui.horizontal_centered(|ui| {
                                    // Draw peer address.
                                    let peer_text = format!("{} {}", GLOBE_SIMPLE, &peer_addr);
                                    ui.label(RichText::new(peer_text)
                                        .color(Colors::TEXT_BUTTON)
                                        .size(17.0));
                                });
                            });
                            if peer_type != &PeerType::DefaultSeed {
                                strip.cell(|ui| {
                                    // Draw delete button for non-default seed peers.
                                    View::button(ui, TRASH.to_string(), Colors::BUTTON, || {
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
                                });
                            } else {
                                strip.empty();
                            }
                        });
                });
            });
    }

    /// Draw seeding type setup content.
    fn seeding_type_ui(&mut self, ui: &mut Ui, cb: &dyn PlatformCallbacks) {
        let title = Self::DNS_SEEDS_TITLE;
        ui.label(RichText::new(title).size(16.0).color(Colors::GRAY));
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
        self.peer_list_ui(ui, &peers_type, cb);
    }

    /// Draw ban window setup content.
    fn ban_window_ui(&mut self, ui: &mut Ui, cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("network_settings.ban_window"))
            .size(16.0)
            .color(Colors::GRAY)
        );
        ui.add_space(6.0);

        let ban_window = NodeConfig::get_p2p_ban_window();
        View::button(ui, format!("{} {}", PROHIBIT_INSET, ban_window.clone()), Colors::BUTTON, || {
            // Setup values for modal.
            self.ban_window_edit = ban_window;
            // Show ban window period setup modal.
            Modal::new(Self::BAN_WINDOW_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"))
                .show();
            cb.show_keyboard();
        });
        ui.add_space(6.0);
        ui.label(RichText::new(t!("network_settings.ban_window_desc"))
            .size(16.0)
            .color(Colors::INACTIVE_TEXT)
        );
        ui.add_space(2.0);
    }

    /// Draw ban window [`Modal`] content.
    pub fn ban_window_modal(&mut self, ui: &mut Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.ban_window"))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw ban window text edit.
            let text_edit_resp = egui::TextEdit::singleline(&mut self.ban_window_edit)
                .id(Id::from(modal.id))
                .font(TextStyle::Heading)
                .desired_width(84.0)
                .cursor_at_end(true)
                .ui(ui);
            text_edit_resp.request_focus();
            if text_edit_resp.clicked() {
                cb.show_keyboard();
            }

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.ban_window_edit.parse::<i64>().is_err() {
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
                if let Ok(ban_window) = self.ban_window_edit.parse::<i64>() {
                    NodeConfig::save_p2p_ban_window(ban_window);
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

    /// Draw maximum number of inbound peers setup content.
    fn max_inbound_ui(&mut self, ui: &mut Ui, cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("network_settings.max_inbound_count"))
            .size(16.0)
            .color(Colors::GRAY)
        );
        ui.add_space(6.0);

        let max_inbound = NodeConfig::get_max_inbound_peers();
        let button_text = format!("{} {}", ARROW_FAT_LINES_DOWN, max_inbound.clone());
        View::button(ui, button_text, Colors::BUTTON, || {
            // Setup values for modal.
            self.max_inbound_count = max_inbound;
            // Show maximum number of inbound peers setup modal.
            Modal::new(Self::MAX_INBOUND_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"))
                .show();
            cb.show_keyboard();
        });
        ui.add_space(6.0);
    }

    /// Draw maximum number of inbound peers [`Modal`] content.
    pub fn max_inbound_modal(&mut self, ui: &mut Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.max_inbound_count"))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw maximum number of inbound peers text edit.
            let text_edit_resp = egui::TextEdit::singleline(&mut self.max_inbound_count)
                .id(Id::from(modal.id))
                .font(TextStyle::Heading)
                .desired_width(42.0)
                .cursor_at_end(true)
                .ui(ui);
            text_edit_resp.request_focus();
            if text_edit_resp.clicked() {
                cb.show_keyboard();
            }

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.max_inbound_count.parse::<u32>().is_err() {
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
                if let Ok(max_inbound) = self.max_inbound_count.parse::<u32>() {
                    NodeConfig::save_max_inbound_peers(max_inbound);
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

    /// Draw maximum number of outbound peers setup content.
    fn max_outbound_ui(&mut self, ui: &mut Ui, cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("network_settings.max_outbound_count"))
            .size(16.0)
            .color(Colors::GRAY)
        );
        ui.add_space(6.0);

        let max_outbound = NodeConfig::get_max_outbound_peers();
        let button_text = format!("{} {}", ARROW_FAT_LINES_UP, max_outbound.clone());
        View::button(ui, button_text, Colors::BUTTON, || {
            // Setup values for modal.
            self.max_outbound_count = max_outbound;
            // Show maximum number of outbound peers setup modal.
            Modal::new(Self::MAX_OUTBOUND_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"))
                .show();
            cb.show_keyboard();
        });
        ui.add_space(6.0);
    }

    /// Draw maximum number of outbound peers [`Modal`] content.
    pub fn max_outbound_modal(&mut self, ui: &mut Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.max_outbound_count"))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw maximum number of outbound peers text edit.
            let text_edit_resp = egui::TextEdit::singleline(&mut self.max_outbound_count)
                .id(Id::from(modal.id))
                .font(TextStyle::Heading)
                .desired_width(42.0)
                .cursor_at_end(true)
                .ui(ui);
            text_edit_resp.request_focus();
            if text_edit_resp.clicked() {
                cb.show_keyboard();
            }

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.max_outbound_count.parse::<u32>().is_err() {
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
                if let Ok(max_outbound) = self.max_outbound_count.parse::<u32>() {
                    NodeConfig::save_max_outbound_peers(max_outbound);
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

    /// Draw minimum number of outbound peers setup content.
    fn min_outbound_ui(&mut self, ui: &mut Ui, cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("network_settings.min_outbound_count"))
            .size(16.0)
            .color(Colors::GRAY)
        );
        ui.add_space(6.0);

        let min_outbound = NodeConfig::get_min_outbound_peers();
        let button_text = format!("{} {}", ARROW_FAT_LINE_UP, min_outbound.clone());
        View::button(ui, button_text, Colors::BUTTON, || {
            // Setup values for modal.
            self.min_outbound_count = min_outbound;
            // Show maximum number of outbound peers setup modal.
            Modal::new(Self::MIN_OUTBOUND_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"))
                .show();
            cb.show_keyboard();
        });
        ui.add_space(6.0);
        ui.label(RichText::new(t!("network_settings.min_outbound_desc"))
            .size(16.0)
            .color(Colors::INACTIVE_TEXT)
        );
    }

    /// Draw minimum number of outbound peers [`Modal`] content.
    pub fn min_outbound_modal(&mut self, ui: &mut Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.min_outbound_count"))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw maximum number of outbound peers text edit.
            let text_edit_resp = egui::TextEdit::singleline(&mut self.min_outbound_count)
                .id(Id::from(modal.id))
                .font(TextStyle::Heading)
                .desired_width(42.0)
                .cursor_at_end(true)
                .ui(ui);
            text_edit_resp.request_focus();
            if text_edit_resp.clicked() {
                cb.show_keyboard();
            }

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.min_outbound_count.parse::<u32>().is_err() {
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
                if let Ok(max_outbound) = self.min_outbound_count.parse::<u32>() {
                    NodeConfig::save_min_outbound_peers(max_outbound);
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
}