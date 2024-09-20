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

use egui::{Align, Layout, RichText, Rounding};

use crate::AppConfig;
use crate::gui::Colors;
use crate::gui::icons::{CARET_RIGHT, CHECK_CIRCLE, COMPUTER_TOWER, DOTS_THREE_CIRCLE, GLOBE_SIMPLE, PENCIL, PLUS_CIRCLE, POWER, TRASH, WARNING_CIRCLE, X_CIRCLE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, View};
use crate::gui::views::network::modals::ExternalConnectionModal;
use crate::gui::views::network::NodeSetup;
use crate::gui::views::types::{ModalContainer, ModalPosition};
use crate::node::{Node, NodeConfig};
use crate::wallet::{ConnectionsConfig, ExternalConnection};

/// Network connections content.
pub struct ConnectionsContent {
    /// External connection [`Modal`] content.
    ext_conn_modal: ExternalConnectionModal,

    /// [`Modal`] identifiers allowed at this ui container.
    modal_ids: Vec<&'static str>
}

impl Default for ConnectionsContent {
    fn default() -> Self {
        Self {
            ext_conn_modal: ExternalConnectionModal::new(None),
            modal_ids: vec![
                ExternalConnectionModal::NETWORK_ID
            ],
        }
    }
}

impl ModalContainer for ConnectionsContent {
    fn modal_ids(&self) -> &Vec<&'static str> {
        &self.modal_ids
    }

    fn modal_ui(&mut self,
                ui: &mut egui::Ui,
                modal: &Modal,
                cb: &dyn PlatformCallbacks) {
        match modal.id {
            ExternalConnectionModal::NETWORK_ID => {
                self.ext_conn_modal.ui(ui, cb, modal, |_| {});
            },
            _ => {}
        }
    }
}

impl ConnectionsContent {
    /// Draw connections content.
    pub fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        self.current_modal_ui(ui, cb);

        ui.add_space(2.0);

        // Show network type selection.
        let saved_chain_type = AppConfig::chain_type();
        NodeSetup::chain_type_ui(ui);
        ui.add_space(6.0);

        // Check connections availability.
        if saved_chain_type != AppConfig::chain_type() {
            ExternalConnection::check(None, ui.ctx());
        }

        // Show integrated node info content.
        Self::integrated_node_item_ui(ui, |ui| {
            // Draw button to show integrated node info.
            View::item_button(ui, View::item_rounding(0, 1, true), CARET_RIGHT, None, || {
                AppConfig::toggle_show_connections_network_panel();
            });
        });

        // Show external connections.
        ui.add_space(8.0);
        ui.label(RichText::new(t!("wallets.ext_conn")).size(16.0).color(Colors::gray()));
        ui.add_space(6.0);

        // Show button to add new external node connection.
        let add_node_text = format!("{} {}", PLUS_CIRCLE, t!("wallets.add_node"));
        View::button(ui, add_node_text, Colors::white_or_black(false), || {
            self.show_add_ext_conn_modal(None, cb);
        });

        ui.add_space(4.0);

        let ext_conn_list = ConnectionsConfig::ext_conn_list();
        let ext_conn_size = ext_conn_list.len();
        if ext_conn_size != 0 {
            ui.add_space(8.0);
            for (index, conn) in ext_conn_list.iter().filter(|c| !c.deleted).enumerate() {
                ui.horizontal_wrapped(|ui| {
                    // Draw connection list item.
                    Self::ext_conn_item_ui(ui, conn, index, ext_conn_size, |ui| {
                        let button_rounding = View::item_rounding(index, ext_conn_size, true);
                        View::item_button(ui, button_rounding, TRASH, None, || {
                            ConnectionsConfig::remove_ext_conn(conn.id);
                        });
                        View::item_button(ui, Rounding::default(), PENCIL, None, || {
                            self.show_add_ext_conn_modal(Some(conn.clone()), cb);
                        });
                    });
                });
            }
        }
    }

    /// Draw integrated node connection item content.
    pub fn integrated_node_item_ui(ui: &mut egui::Ui, custom_button: impl FnOnce(&mut egui::Ui)) {
        // Draw round background.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(78.0);
        let rounding = View::item_rounding(0, 1, false);
        ui.painter().rect(rect, rounding, Colors::fill(), View::item_stroke());

        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            // Draw custom button.
            custom_button(ui);

            // Draw buttons to start/stop node.
            if Node::get_error().is_none() {
                if !Node::is_running() {
                    View::item_button(ui, Rounding::default(), POWER, Some(Colors::green()), || {
                        Node::start();
                    });
                } else if !Node::is_starting() && !Node::is_stopping() && !Node::is_restarting() {
                    View::item_button(ui, Rounding::default(), POWER, Some(Colors::red()), || {
                        Node::stop(false);
                    });
                }
            }

            let layout_size = ui.available_size();
            ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                ui.add_space(6.0);
                ui.vertical(|ui| {
                    ui.add_space(3.0);
                    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                        ui.add_space(1.0);
                        ui.label(RichText::new(t!("network.node"))
                            .size(18.0)
                            .color(Colors::title(false)));
                    });

                    // Setup node status text.
                    let has_error = Node::get_error().is_some();
                    let status_icon = if has_error {
                        WARNING_CIRCLE
                    } else if !Node::is_running() {
                        X_CIRCLE
                    } else if Node::not_syncing() {
                        CHECK_CIRCLE
                    } else {
                        DOTS_THREE_CIRCLE
                    };
                    let status_text = format!("{} {}", status_icon, if has_error {
                        t!("error")
                    } else {
                        Node::get_sync_status_text()
                    });
                    View::ellipsize_text(ui, status_text, 15.0, Colors::text(false));
                    ui.add_space(1.0);

                    // Setup node API address text.
                    let api_address = NodeConfig::get_api_address();
                    let address_text = format!("{} http://{}", COMPUTER_TOWER, api_address);
                    ui.label(RichText::new(address_text).size(15.0).color(Colors::gray()));
                })
            });
        });
    }

    /// Draw external connection item content.
    pub fn ext_conn_item_ui(ui: &mut egui::Ui,
                            conn: &ExternalConnection,
                            index: usize,
                            len: usize,
                            buttons_ui: impl FnOnce(&mut egui::Ui)) {
        // Setup layout size.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(52.0);

        // Draw round background.
        let bg_rect = rect.clone();
        let item_rounding = View::item_rounding(index, len, false);
        ui.painter().rect(bg_rect, item_rounding, Colors::fill(), View::item_stroke());

        ui.vertical(|ui| {
            ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
                // Draw provided buttons.
                buttons_ui(ui);

                let layout_size = ui.available_size();
                ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                    ui.add_space(6.0);
                    ui.vertical(|ui| {
                        // Draw connections URL.
                        ui.add_space(4.0);
                        let conn_text = format!("{} {}", GLOBE_SIMPLE, conn.url);
                        View::ellipsize_text(ui, conn_text, 15.0, Colors::title(false));
                        ui.add_space(1.0);

                        // Setup connection status text.
                        let status_text = if let Some(available) = conn.available {
                            if available {
                                format!("{} {}", CHECK_CIRCLE, t!("network.available"))
                            } else {
                                format!("{} {}", X_CIRCLE, t!("network.not_available"))
                            }
                        } else {
                            format!("{} {}", DOTS_THREE_CIRCLE, t!("network.availability_check"))
                        };
                        ui.label(RichText::new(status_text).size(15.0).color(Colors::gray()));
                        ui.add_space(3.0);
                    });
                });
            });
        });
    }

    /// Show [`Modal`] to add external connection.
    pub fn show_add_ext_conn_modal(&mut self,
                                   conn: Option<ExternalConnection>,
                                   cb: &dyn PlatformCallbacks) {
        self.ext_conn_modal = ExternalConnectionModal::new(conn);
        // Show modal.
        Modal::new(ExternalConnectionModal::NETWORK_ID)
            .position(ModalPosition::CenterTop)
            .title(t!("wallets.add_node"))
            .show();
        cb.show_keyboard();
    }
}