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

use egui::{Align, Id, Layout, RichText, Rounding};
use url::Url;

use crate::AppConfig;
use crate::gui::Colors;
use crate::gui::icons::{CARET_RIGHT, CHECK_CIRCLE, COMPUTER_TOWER, DOTS_THREE_CIRCLE, GLOBE_SIMPLE, PENCIL, PLUS_CIRCLE, POWER, TRASH, X_CIRCLE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, NodeSetup, View};
use crate::gui::views::types::{ModalContainer, ModalPosition, TextEditOptions};
use crate::node::{Node, NodeConfig};
use crate::wallet::{ConnectionsConfig, ExternalConnection};

/// Network connections content.
pub struct ConnectionsContent {
    /// Flag to check if [`Modal`] was just opened to focus on input field.
    first_modal_launch: bool,
    /// External connection URL value for [`Modal`].
    ext_node_url_edit: String,
    /// External connection API secret value for [`Modal`].
    ext_node_secret_edit: String,
    /// Flag to show URL format error at [`Modal`].
    ext_node_url_error: bool,
    /// Editing external connection identifier for [`Modal`].
    ext_conn_id_edit: Option<i64>,

    /// [`Modal`] identifiers allowed at this ui container.
    modal_ids: Vec<&'static str>
}

impl Default for ConnectionsContent {
    fn default() -> Self {
        if AppConfig::show_connections_network_panel() {
            ExternalConnection::start_ext_conn_availability_check();
        }
        Self {
            first_modal_launch: true,
            ext_node_url_edit: "".to_string(),
            ext_node_secret_edit: "".to_string(),
            ext_node_url_error: false,
            ext_conn_id_edit: None,
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

        ui.add_space(2.0);

        // Show network type selection.
        let saved_chain_type = AppConfig::chain_type();
        NodeSetup::chain_type_ui(ui);

        // Start external connections availability check on network type change.
        if saved_chain_type != AppConfig::chain_type() {
            ExternalConnection::start_ext_conn_availability_check();
        }
        ui.add_space(6.0);

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
        if !ext_conn_list.is_empty() {
            ui.add_space(8.0);
            for (index, conn) in ext_conn_list.iter().enumerate() {
                ui.horizontal_wrapped(|ui| {
                    // Draw connection list item.
                    self.ext_conn_item_ui(ui, conn, index, ext_conn_list.len(), cb);
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

            if !Node::is_running() {
                // Draw button to start integrated node.
                View::item_button(ui, Rounding::default(), POWER, Some(Colors::green()), || {
                    Node::start();
                });
            } else if !Node::is_starting() && !Node::is_stopping() && !Node::is_restarting() {
                // Draw button to stop integrated node.
                View::item_button(ui, Rounding::default(), POWER, Some(Colors::red()), || {
                    Node::stop(false);
                });
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

                    // Setup node API address text.
                    let api_address = NodeConfig::get_api_address();
                    let address_text = format!("{} http://{}", COMPUTER_TOWER, api_address);
                    ui.label(RichText::new(address_text).size(15.0).color(Colors::text(false)));
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
                    ui.label(RichText::new(status_text).size(15.0).color(Colors::gray()));
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
        rect.set_height(52.0);

        // Draw round background.
        let bg_rect = rect.clone();
        let item_rounding = View::item_rounding(index, len, false);
        ui.painter().rect(bg_rect, item_rounding, Colors::fill(), View::item_stroke());

        ui.vertical(|ui| {
            ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
                // Draw buttons for non-default connections.
                if conn.url != ExternalConnection::DEFAULT_MAIN_URL {
                    let button_rounding = View::item_rounding(index, len, true);
                    View::item_button(ui, button_rounding, TRASH, None, || {
                        ConnectionsConfig::remove_ext_conn(conn.id);
                    });
                    View::item_button(ui, Rounding::default(), PENCIL, None, || {
                        self.show_add_ext_conn_modal(Some(conn), cb);
                    });
                }

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
                                   conn: Option<&ExternalConnection>,
                                   cb: &dyn PlatformCallbacks) {
        // Setup values.
        self.first_modal_launch = true;
        self.ext_node_url_error = false;
        if let Some(c) = conn {
            self.ext_node_url_edit = c.url.to_owned();
            self.ext_node_secret_edit = c.secret.clone().unwrap_or("".to_string());
            self.ext_conn_id_edit = Some(c.id);
        } else {
            self.ext_node_url_edit = "".to_string();
            self.ext_node_secret_edit = "".to_string();
            self.ext_conn_id_edit = None;
        }
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
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Draw node URL text edit.
            let url_edit_id = Id::from(modal.id).with(self.ext_conn_id_edit);
            let mut url_edit_opts = TextEditOptions::new(url_edit_id).paste().no_focus();
            if self.first_modal_launch {
                self.first_modal_launch = false;
                url_edit_opts.focus = true;
            }
            View::text_edit(ui, cb, &mut self.ext_node_url_edit, &mut url_edit_opts);
            ui.add_space(8.0);

            ui.label(RichText::new(t!("wallets.node_secret"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Draw node API secret text edit.
            let secret_edit_id = Id::from(modal.id).with(self.ext_conn_id_edit).with("node_secret");
            let mut secret_edit_opts = TextEditOptions::new(secret_edit_id).paste().no_focus();
            View::text_edit(ui, cb, &mut self.ext_node_secret_edit, &mut secret_edit_opts);

            // Show error when specified URL is not valid.
            if self.ext_node_url_error {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("wallets.invalid_url"))
                    .size(17.0)
                    .color(Colors::red()));
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
                        cb.hide_keyboard();
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    // Add connection button callback.
                    let mut on_add = || {
                        if !self.ext_node_url_edit.starts_with("http") {
                            self.ext_node_url_edit = format!("http://{}", self.ext_node_url_edit)
                        }
                        let error = Url::parse(self.ext_node_url_edit.as_str()).is_err();
                        self.ext_node_url_error = error;
                        if !error {
                            let url = self.ext_node_url_edit.to_owned();
                            let secret = if self.ext_node_secret_edit.is_empty() {
                                None
                            } else {
                                Some(self.ext_node_secret_edit.to_owned())
                            };

                            // Update or create new connection.
                            let mut ext_conn = ExternalConnection::new(url, secret);
                            ext_conn.check_conn_availability();
                            if let Some(id) = self.ext_conn_id_edit {
                                ext_conn.id = id;
                            }
                            self.ext_conn_id_edit = None;
                            ConnectionsConfig::add_ext_conn(ext_conn);

                            // Close modal.
                            cb.hide_keyboard();
                            modal.close();
                        }
                    };

                    // Add connection on Enter button press.
                    View::on_enter_key(ui, || {
                        (on_add)();
                    });

                    View::button(ui, t!("modal.save"), Colors::white_or_black(false), on_add);
                });
            });
            ui.add_space(6.0);
        });
    }
}