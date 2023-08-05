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

use egui::{Align, Id, Layout, RichText, Rounding, TextStyle, Widget};
use url::Url;

use crate::AppConfig;
use crate::gui::Colors;
use crate::gui::icons::{CARET_RIGHT, CHECK_CIRCLE, COMPUTER_TOWER, DOTS_THREE_CIRCLE, GLOBE_SIMPLE, PENCIL, PLAY, STOP, TRASH, X_CIRCLE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, View};
use crate::gui::views::types::{ModalContainer, ModalPosition};
use crate::node::{Node, NodeConfig};
use crate::wallet::{ConnectionsConfig, ExternalConnection};

/// Network connections content.
pub struct ConnectionsContent {
    /// Flag to check if modal was just opened.
    first_modal_launch: bool,
    /// External connection URL value for [`Modal`].
    ext_node_url_edit: String,
    /// External connection API secret value for [`Modal`].
    ext_node_secret_edit: String,
    /// Flag to show URL format error.
    ext_node_url_error: bool,
    /// Flag to check if existing connection is editing.
    edit_ext_conn: Option<ExternalConnection>,

    /// [`Modal`] identifiers allowed at this ui container.
    modal_ids: Vec<&'static str>
}

impl Default for ConnectionsContent {
    fn default() -> Self {
        Self {
            first_modal_launch: true,
            ext_node_url_edit: "".to_string(),
            ext_node_secret_edit: "".to_string(),
            ext_node_url_error: false,
            edit_ext_conn: None,
            modal_ids: vec![
                Self::NETWORK_EXT_CONNECTION_MODAL
            ]
        }
    }
}

impl ModalContainer for ConnectionsContent {
    fn modal_ids(&self) -> &Vec<&'static str> {
        &self.modal_ids
    }

    fn modal_ui(&mut self,
                ui: &mut egui::Ui,
                _: &mut eframe::Frame,
                modal: &Modal,
                cb: &dyn PlatformCallbacks) {
        match modal.id {
            Self::NETWORK_EXT_CONNECTION_MODAL => self.ext_conn_modal_ui(ui, modal, cb),
            _ => {}
        }
    }
}

impl ConnectionsContent {
    /// External connection [`Modal`] identifier.
    pub const NETWORK_EXT_CONNECTION_MODAL: &'static str = "network_ext_connection_modal";

    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, cb: &dyn PlatformCallbacks) {
        // Draw modal content for current ui container.
        self.current_modal_ui(ui, frame, cb);

        // Show integrated node info content.
        Self::integrated_node_item_ui(ui);

        ui.add_space(8.0);
        ui.label(RichText::new(t!("wallets.ext_conn")).size(16.0).color(Colors::GRAY));
        ui.add_space(6.0);

        // Show external connections.
        let ext_conn_list = ConnectionsConfig::external_connections();
        for (index, conn) in ext_conn_list.iter().enumerate() {
            ui.horizontal_wrapped(|ui| {
                // Draw connection list item.
                self.ext_conn_item_ui(ui, conn, index, ext_conn_list.len(), cb);
            });
        }
    }

    /// Draw integrated node connection item content.
    fn integrated_node_item_ui(ui: &mut egui::Ui) {
        // Draw round background.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(78.0);
        let rounding = View::item_rounding(0, 1, false);
        let bg_color = Colors::FILL_DARK;
        ui.painter().rect(rect, rounding, bg_color, View::HOVER_STROKE);

        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            // Setup padding for item buttons.
            ui.style_mut().spacing.button_padding = egui::vec2(14.0, 0.0);
            // Setup rounding for item buttons.
            ui.style_mut().visuals.widgets.inactive.rounding = Rounding::same(8.0);
            ui.style_mut().visuals.widgets.hovered.rounding = Rounding::same(8.0);
            ui.style_mut().visuals.widgets.active.rounding = Rounding::same(8.0);

            // Draw button to show integrated node.
            View::item_button(ui, View::item_rounding(0, 1, true), CARET_RIGHT, || {
                AppConfig::toggle_show_connections_network_panel();
            });

            if !Node::is_running() {
                // Draw button to start integrated node.
                View::item_button(ui, Rounding::none(), PLAY, || {
                    Node::start();
                });
            } else if !Node::is_starting() && !Node::is_stopping() && !Node::is_restarting() {
                // Show button to open closed wallet.
                View::item_button(ui, Rounding::none(), STOP, || {
                    Node::stop(false);
                });
            }

            let layout_size = ui.available_size();
            ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                ui.add_space(7.0);
                ui.vertical(|ui| {
                    ui.add_space(3.0);
                    ui.label(RichText::new(t!("network.node"))
                                 .size(18.0)
                                 .color(Colors::TITLE));

                    // Setup node API address text.
                    let api_address = NodeConfig::get_api_address();
                    let address_text = format!("{} http://{}", COMPUTER_TOWER, api_address);
                    ui.label(RichText::new(address_text).size(15.0).color(Colors::TEXT));
                    ui.add_space(1.0);

                    // Setup node status text.
                    let status_icon = if !Node::is_running() {
                        X_CIRCLE
                    } else if Node::not_syncing() {
                        CHECK_CIRCLE
                    } else {
                        DOTS_THREE_CIRCLE
                    };
                    let status_text = format!("{} {}", status_icon, Node::get_sync_status_text());
                    ui.label(RichText::new(status_text).size(15.0).color(Colors::GRAY));
                    ui.add_space(4.0);
                })
            });
        });
    }

    /// Draw external connection item content.
    fn ext_conn_item_ui(&mut self,
                        ui: &mut egui::Ui,
                        conn: &ExternalConnection,
                        index: usize,
                        len: usize,
                        cb: &dyn PlatformCallbacks) {
        // Setup layout size.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(42.0);

        // Draw round background.
        let bg_rect = rect.clone();
        let item_rounding = View::item_rounding(index, len, false);
        ui.painter().rect(bg_rect, item_rounding, Colors::FILL, View::ITEM_STROKE);

        ui.vertical(|ui| {
            ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
                // Draw buttons for non-default connections.
                if conn.url != ExternalConnection::DEFAULT_EXTERNAL_NODE_URL {
                    let button_rounding = View::item_rounding(index, len, true);
                    View::item_button(ui, button_rounding, TRASH, || {
                        ConnectionsConfig::remove_external_connection(conn);
                    });
                    View::item_button(ui, Rounding::none(), PENCIL, || {
                        // Setup values for Modal.
                        self.first_modal_launch = true;
                        self.ext_node_url_edit = conn.url.clone();
                        self.ext_node_secret_edit = conn.secret.clone().unwrap_or("".to_string());
                        self.ext_node_url_error = false;
                        self.edit_ext_conn = Some(conn.clone());
                        // Show modal.
                        Modal::new(Self::NETWORK_EXT_CONNECTION_MODAL)
                            .position(ModalPosition::CenterTop)
                            .title(t!("wallets.add_node"))
                            .show();
                        cb.show_keyboard();
                    });
                }

                let layout_size = ui.available_size();
                ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                    ui.add_space(6.0);
                    // Draw connections URL.
                    let conn_text = format!("{} {}", GLOBE_SIMPLE, conn.url);
                    ui.label(RichText::new(conn_text)
                        .color(Colors::TEXT_BUTTON)
                        .size(16.0));
                });
            });
        });
    }

    /// Show [`Modal`] to add external connection.
    pub fn show_add_ext_conn_modal(&mut self, cb: &dyn PlatformCallbacks) {
        // Setup values for Modal.
        self.first_modal_launch = true;
        self.ext_node_url_edit = "".to_string();
        self.ext_node_secret_edit = "".to_string();
        self.ext_node_url_error = false;
        self.edit_ext_conn = None;
        // Show modal.
        Modal::new(Self::NETWORK_EXT_CONNECTION_MODAL)
            .position(ModalPosition::CenterTop)
            .title(t!("wallets.add_node"))
            .show();
        cb.show_keyboard();
    }

    /// Draw external connection [`Modal`] content.
    pub fn ext_conn_modal_ui(&mut self,
                             ui: &mut egui::Ui,
                             modal: &Modal,
                             cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("wallets.node_url"))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw node URL text edit.
            let url_edit_resp = egui::TextEdit::singleline(&mut self.ext_node_url_edit)
                .id(Id::from(modal.id).with("node_url_edit"))
                .font(TextStyle::Heading)
                .desired_width(ui.available_width())
                .cursor_at_end(true)
                .ui(ui);
            ui.add_space(8.0);
            if self.first_modal_launch {
                self.first_modal_launch = false;
                url_edit_resp.request_focus();
            }
            if url_edit_resp.clicked() {
                cb.show_keyboard();
            }

            ui.label(RichText::new(t!("wallets.node_secret"))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw node API secret text edit.
            let secret_edit_resp = egui::TextEdit::singleline(&mut self.ext_node_secret_edit)
                .id(Id::from(modal.id).with("node_secret_edit"))
                .font(TextStyle::Heading)
                .desired_width(ui.available_width())
                .cursor_at_end(true)
                .ui(ui);
            ui.add_space(8.0);
            if secret_edit_resp.clicked() {
                cb.show_keyboard();
            }

            // Show error when specified URL is not valid.
            if self.ext_node_url_error {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("wallets.invalid_url"))
                    .size(17.0)
                    .color(Colors::RED));
            }
            ui.add_space(12.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::WHITE, || {
                        // Close modal.
                        cb.hide_keyboard();
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    // Add connection button callback.
                    let mut on_add = || {
                        let error = Url::parse(self.ext_node_url_edit.as_str()).is_err();
                        self.ext_node_url_error = error;
                        if !error {
                            // Save external connection.
                            let url = self.ext_node_url_edit.to_owned();
                            let secret = if self.ext_node_secret_edit.is_empty() {
                                None
                            } else {
                                Some(self.ext_node_secret_edit.to_owned())
                            };

                            // Update or create new connections.
                            let ext_conn = ExternalConnection::new(url.clone(), secret);
                            if let Some(edit_conn) = self.edit_ext_conn.clone() {
                                ConnectionsConfig::update_external_connection(edit_conn, ext_conn);
                                self.edit_ext_conn = None;
                            } else {
                                ConnectionsConfig::add_external_connection(ext_conn);
                            }

                            // Close modal.
                            cb.hide_keyboard();
                            modal.close();
                        }
                    };

                    // Add connection on Enter button press.
                    View::on_enter_key(ui, || {
                        (on_add)();
                    });

                    View::button(ui, t!("modal.save"), Colors::WHITE, on_add);
                });
            });
            ui.add_space(6.0);
        });
    }
}