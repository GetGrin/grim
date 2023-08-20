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

use crate::gui::Colors;
use crate::gui::icons::{CHECK, CHECK_CIRCLE, CHECK_FAT, COMPUTER_TOWER, DOTS_THREE_CIRCLE, GLOBE, PLUS_CIRCLE, POWER, X_CIRCLE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, View};
use crate::gui::views::types::{ModalContainer, ModalPosition};
use crate::node::{Node, NodeConfig};
use crate::wallet::{ConnectionsConfig, ExternalConnection, Wallet};
use crate::wallet::types::ConnectionMethod;

/// Wallet connection setup content.
pub struct ConnectionSetup {
    /// Selected connection method.
    pub method: ConnectionMethod,

    /// Flag to check if modal was just opened.
    first_modal_launch: bool,
    /// External connection URL value for [`Modal`].
    ext_node_url_edit: String,
    /// External connection API secret value for [`Modal`].
    ext_node_secret_edit: String,
    /// Flag to show URL format error.
    ext_node_url_error: bool,

    /// [`Modal`] identifiers allowed at this ui container.
    modal_ids: Vec<&'static str>
}

/// Identifier for [`Modal`] to add external connection.
pub const ADD_EXT_CONNECTION_MODAL: &'static str = "add_ext_connection_modal";

/// Identifier for [`Modal`] to confirm wallet reopening after connection change.
pub const REOPEN_WALLET_CONFIRMATION_MODAL: &'static str = "change_conn_reopen_wallet_modal";

impl Default for ConnectionSetup {
    fn default() -> Self {
        Self {
            method: ConnectionMethod::Integrated,
            first_modal_launch: true,
            ext_node_url_edit: "".to_string(),
            ext_node_secret_edit: "".to_string(),
            ext_node_url_error: false,
            modal_ids: vec![
                ADD_EXT_CONNECTION_MODAL
            ]
        }
    }
}

impl ModalContainer for ConnectionSetup {
    fn modal_ids(&self) -> &Vec<&'static str> {
        &self.modal_ids
    }

    fn modal_ui(&mut self,
                ui: &mut egui::Ui,
                _: &mut eframe::Frame,
                modal: &Modal,
                cb: &dyn PlatformCallbacks) {
        match modal.id {
            ADD_EXT_CONNECTION_MODAL => self.add_ext_conn_modal_ui(ui, modal, cb),
            _ => {}
        }
    }
}

impl ConnectionSetup {
    /// Draw wallet creation setup content.
    pub fn create_ui(&mut self,
                     ui: &mut egui::Ui,
                     frame: &mut eframe::Frame,
                     cb: &dyn PlatformCallbacks) {
        self.ui(ui, frame, cb);
    }

    /// Draw existing wallet connection setup content.
    pub fn wallet_ui(&mut self,
              ui: &mut egui::Ui,
              frame: &mut eframe::Frame,
              wallet: &mut Wallet,
              cb: &dyn PlatformCallbacks) {
        // Show modal content to reopen the wallet.
        if Modal::opened() == Some(REOPEN_WALLET_CONFIRMATION_MODAL) {
            Modal::ui(ui.ctx(), |ui, modal| {
                self.reopen_modal_content(ui, wallet, modal, cb);
            });
        }

        // Setup connection value from provided wallet.
        match wallet.config.ext_conn_id {
            None => self.method = ConnectionMethod::Integrated,
            Some(id) => self.method = ConnectionMethod::External(id)
        }

        // Draw setup content.
        self.ui(ui, frame, cb);

        // Setup wallet connection value after change.
        let changed = match self.method {
            ConnectionMethod::Integrated => {
                let changed = wallet.config.ext_conn_id.is_some();
                if changed {
                    wallet.config.ext_conn_id = None;
                }
                changed
            }
            ConnectionMethod::External(id) => {
                let changed = wallet.config.ext_conn_id != Some(id);
                if changed {
                    wallet.config.ext_conn_id = Some(id);
                }
                changed
            }
        };

        if changed {
            wallet.config.save();
            // Show reopen confirmation modal.
            Modal::new(REOPEN_WALLET_CONFIRMATION_MODAL)
                .position(ModalPosition::Center)
                .title(t!("modal.confirmation"))
                .show();
        }
    }

    /// Draw connection setup content.
    fn ui(&mut self,
          ui: &mut egui::Ui,
          frame: &mut eframe::Frame,
          cb: &dyn PlatformCallbacks) {
        // Draw modal content for current ui container.
        self.current_modal_ui(ui, frame, cb);

        ui.add_space(2.0);
        View::sub_title(ui, format!("{} {}", GLOBE, t!("wallets.conn_method")));
        View::horizontal_line(ui, Colors::ITEM_STROKE);
        ui.add_space(4.0);

        ui.vertical_centered(|ui| {
            // Show integrated node selection.
            ui.add_space(6.0);
            self.integrated_node_item_ui(ui);

            let ext_conn_list = ConnectionsConfig::ext_conn_list();
            if !ext_conn_list.is_empty() {
                ui.add_space(6.0);
                ui.label(RichText::new(t!("wallets.ext_conn")).size(16.0).color(Colors::GRAY));
                ui.add_space(6.0);

                // Show button to add new external node connection.
                let add_node_text = format!("{} {}", PLUS_CIRCLE, t!("wallets.add_node"));
                View::button(ui, add_node_text, Colors::GOLD, || {
                    self.show_add_ext_conn_modal(cb);
                });
                ui.add_space(12.0);

                // Show external connections.
                for (index, conn) in ext_conn_list.iter().enumerate() {
                    ui.horizontal_wrapped(|ui| {
                        // Draw connection list item.
                        self.ext_conn_item_ui(ui, conn, index, ext_conn_list.len());
                    });
                }
            }
        });
    }

    /// Draw integrated node connection item content.
    fn integrated_node_item_ui(&mut self, ui: &mut egui::Ui) {
        // Draw round background.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(78.0);
        let rounding = View::item_rounding(0, 1, false);
        ui.painter().rect(rect, rounding, Colors::FILL, View::ITEM_STROKE);

        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            // Setup padding for item buttons.
            ui.style_mut().spacing.button_padding = egui::vec2(14.0, 0.0);

            // Draw button to select integrated node if it was not selected.
            let is_current_method = self.method == ConnectionMethod::Integrated;
            if !is_current_method {
                View::item_button(ui, View::item_rounding(0, 1, true), CHECK, None, || {
                    self.method = ConnectionMethod::Integrated;
                });
            } else {
                ui.add_space(14.0);
                ui.label(RichText::new(CHECK_FAT).size(20.0).color(Colors::GREEN));
                ui.add_space(14.0);
            }

            if !Node::is_running() {
                // Draw button to start integrated node.
                View::item_button(ui, Rounding::none(), POWER, Some(Colors::GREEN), || {
                    Node::start();
                });
            }

            let layout_size = ui.available_size();
            ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                ui.add_space(6.0);
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
                })
            });
        });
    }

    /// Draw external connection item content.
    fn ext_conn_item_ui(&mut self,
                        ui: &mut egui::Ui,
                        conn: &ExternalConnection,
                        index: usize,
                        len: usize) {
        // Setup layout size.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(52.0);

        // Draw round background.
        let bg_rect = rect.clone();
        let item_rounding = View::item_rounding(index, len, false);
        ui.painter().rect(bg_rect, item_rounding, Colors::FILL, View::ITEM_STROKE);

        ui.vertical(|ui| {
            ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
                // Draw button to select connection.
                let is_current_method = self.method == ConnectionMethod::External(conn.id);
                if !is_current_method {
                    let button_rounding = View::item_rounding(index, len, true);
                    View::item_button(ui, button_rounding, CHECK, None, || {
                        self.method = ConnectionMethod::External(conn.id);
                    });
                } else {
                    ui.add_space(12.0);
                    ui.label(RichText::new(CHECK_FAT).size(20.0).color(Colors::GREEN));
                }

                let layout_size = ui.available_size();
                ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                    ui.add_space(6.0);
                    ui.vertical(|ui| {
                        // Draw connections URL.
                        ui.add_space(4.0);
                        let conn_text = format!("{} {}", COMPUTER_TOWER, conn.url);
                        View::ellipsize_text(ui, conn_text, 15.0, Colors::TITLE);
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
                        ui.label(RichText::new(status_text).size(15.0).color(Colors::GRAY));
                        ui.add_space(3.0);
                    });
                });
            });
        });
    }

    /// Show external connection adding [`Modal`].
    fn show_add_ext_conn_modal(&mut self, cb: &dyn PlatformCallbacks) {
        // Setup values for Modal.
        self.first_modal_launch = true;
        self.ext_node_url_edit = "".to_string();
        self.ext_node_secret_edit = "".to_string();
        self.ext_node_url_error = false;
        // Show modal.
        Modal::new(ADD_EXT_CONNECTION_MODAL)
            .position(ModalPosition::CenterTop)
            .title(t!("wallets.add_node"))
            .show();
        cb.show_keyboard();
    }

    /// Draw external connection adding [`Modal`] content.
    pub fn add_ext_conn_modal_ui(&mut self,
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
                .id(Id::from(modal.id))
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
                ui.add_space(2.0);
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
                            // Add external connection.
                            let url = self.ext_node_url_edit.to_owned();
                            let secret = if self.ext_node_secret_edit.is_empty() {
                                None
                            } else {
                                Some(self.ext_node_secret_edit.to_owned())
                            };
                            let ext_conn = ExternalConnection::new(url.clone(), secret);
                            ext_conn.check_conn_availability();
                            ConnectionsConfig::add_ext_conn(ext_conn.clone());

                            // Set added connection as current.
                            self.method = ConnectionMethod::External(ext_conn.id);

                            // Close modal.
                            cb.hide_keyboard();
                            modal.close();
                        }
                    };

                    // Add connection on Enter button press.
                    View::on_enter_key(ui, || {
                        (on_add)();
                    });

                    View::button(ui, t!("modal.add"), Colors::WHITE, on_add);
                });
            });
            ui.add_space(6.0);
        });
    }

    /// Draw confirmation modal content to reopen the [`Wallet`].
    fn reopen_modal_content(&mut self,
                            ui: &mut egui::Ui,
                            wallet: &Wallet,
                            modal: &Modal,
                            cb: &dyn PlatformCallbacks) {
        ui.add_space(8.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("wallets.change_server_confirmation"))
                .size(17.0)
                .color(Colors::TEXT));
        });
        ui.add_space(10.0);

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::WHITE, || {
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, "OK".to_owned(), Colors::WHITE, || {
                        modal.disable_closing();
                        wallet.set_reopen(true);
                        wallet.close();
                        modal.close()
                    });
                });
            });
            ui.add_space(6.0);
        });
    }
}