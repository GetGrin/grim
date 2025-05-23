// Copyright 2024 The Grim Developers
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
use url::Url;
use crate::gui::Colors;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, View};
use crate::gui::views::types::TextEditOptions;
use crate::wallet::{ConnectionsConfig, ExternalConnection};

/// Content to create or update external wallet connection.
pub struct ExternalConnectionModal {
    /// Flag to check if [`Modal`] was just opened to focus on input field.
    first_modal_launch: bool,
    /// External connection URL value for [`Modal`].
    ext_node_url_edit: String,
    /// External connection API secret value for [`Modal`].
    ext_node_secret_edit: String,
    /// Flag to show URL format error at [`Modal`].
    ext_node_url_error: bool,
    /// Editing external connection identifier for [`Modal`].
    ext_conn_id: Option<i64>,
}



impl ExternalConnectionModal {
    /// Network [`Modal`] identifier.
    pub const NETWORK_ID: &'static str = "net_ext_conn_modal";
    /// Wallet [`Modal`] identifier.
    pub const WALLET_ID: &'static str = "wallet_ext_conn_modal";

    /// Create new instance from optional provided connection to update.
    pub fn new(conn: Option<ExternalConnection>) -> Self {
        let (ext_node_url_edit, ext_node_secret_edit, ext_conn_id) = if let Some(c) = conn {
            (c.url, c.secret.unwrap_or("".to_string()), Some(c.id))
        } else {
            ("".to_string(), "".to_string(), None)
        };
        Self {
            first_modal_launch: true,
            ext_node_url_edit,
            ext_node_secret_edit,
            ext_node_url_error: false,
            ext_conn_id,
        }
    }

    /// Draw external connection [`Modal`] content.
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              cb: &dyn PlatformCallbacks,
              modal: &Modal,
              on_save: impl Fn(ExternalConnection)) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("wallets.node_url"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Draw node URL text edit.
            let url_edit_id = Id::from(modal.id).with(self.ext_conn_id);
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
            let secret_edit_id = Id::from(modal.id).with(self.ext_conn_id).with("node_secret");
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
                        self.ext_node_url_edit = "".to_string();
                        self.ext_node_secret_edit = "".to_string();
                        self.ext_node_url_error = false;
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    // Add connection button callback.
                    let mut on_add = |ui: &mut egui::Ui| {
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
                            if let Some(id) = self.ext_conn_id {
                                ext_conn.id = id;
                            }
                            ConnectionsConfig::add_ext_conn(ext_conn.clone());
                            ExternalConnection::check(Some(ext_conn.id), ui.ctx());
                            on_save(ext_conn);

                            // Close modal.
                            self.ext_node_url_edit = "".to_string();
                            self.ext_node_secret_edit = "".to_string();
                            self.ext_node_url_error = false;
                            modal.close();
                        }
                    };

                    // Handle Enter key press.
                    let mut enter = false;
                    View::on_enter_key(ui, || {
                        enter = true;
                    });
                    if enter {
                        (on_add)(ui);
                    }

                    View::button_ui(ui, if self.ext_conn_id.is_some() {
                        t!("modal.save")
                    } else {
                        t!("modal.add")
                    }, Colors::white_or_black(false), on_add);
                });
            });
            ui.add_space(6.0);
        });
    }
}