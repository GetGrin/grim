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

use crate::gui::icons::SCAN;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::network::types::ShareConnection;
use crate::gui::views::{CameraContent, Modal, TextEdit, View};
use crate::gui::Colors;
use crate::wallet::{ConnectionsConfig, ExternalConnection};

/// Content to create or update external wallet connection.
pub struct ExternalConnectionModal {
    /// Flag to check if content was just rendered.
    first_draw: bool,

    /// Editing external connection identifier.
    id: Option<i64>,

    /// External connection URL.
    url_edit: String,
    /// Flag to show URL format error.
    url_error: bool,

    // /// External connection username.
    // username_edit: String,
    /// External connection API secret.
    secret_edit: String,

    /// QR code scanner content.
    scan_qr_content: Option<CameraContent>,
}

impl ExternalConnectionModal {
    /// Network [`Modal`] identifier.
    pub const NETWORK_ID: &'static str = "net_ext_conn_modal";
    /// Wallet [`Modal`] identifier.
    pub const WALLET_ID: &'static str = "wallet_ext_conn_modal";

    /// Create new instance from optional provided connection to update.
    pub fn new(conn: Option<ExternalConnection>) -> Self {
        let (url_edit, secret_edit, id) = if let Some(c) = conn {
            // let username = c.username.unwrap_or("grin".to_string());
            let secret = c.secret.unwrap_or("".to_string());
            (c.url, secret, Some(c.id))
        } else {
            ("".to_string(), "".to_string(), None)
        };
        Self {
            first_draw: true,
            url_edit,
            url_error: false,
            secret_edit,
            id,
            scan_qr_content: None
        }
    }

    /// Draw external connection [`Modal`] content.
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              cb: &dyn PlatformCallbacks,
              modal: &Modal,
              on_save: impl Fn(ExternalConnection)) {
        // Show QR code scanner content.
        if let Some(scan_content) = self.scan_qr_content.as_mut() {
            if let Some(result) = scan_content.qr_scan_result() {
                cb.stop_camera();
                modal.enable_closing();
                self.scan_qr_content = None;
                // Parse scan result.
                if let Ok(c) = serde_json::from_str::<ShareConnection>(&result.text()) {
                    let ext_conn = ExternalConnection::new(c.url, Some(c.username), Some(c.secret));
                    ConnectionsConfig::add_ext_conn(ext_conn.clone());
                    ExternalConnection::check(Some(ext_conn.id), ui.ctx());
                    Modal::close();
                }
            } else {
                scan_content.ui(ui, cb);
            }
            ui.add_space(8.0);

            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            // Show buttons to close modal or scanner.
            ui.columns(2, |cols| {
                cols[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("close"), Colors::white_or_black(false), || {
                        cb.stop_camera();
                        self.scan_qr_content = None;
                        Modal::close();
                    });
                });
                cols[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("back"), Colors::white_or_black(false), || {
                        cb.stop_camera();
                        self.scan_qr_content = None;
                        modal.enable_closing();
                    });
                });
            });
            ui.add_space(6.0);
            return;
        }
        // Add connection button callback.
        let on_add = |ui: &mut egui::Ui, m: &mut ExternalConnectionModal| {
            let url = if !m.url_edit.starts_with("http") {
                format!("https://{}", m.url_edit)
            } else {
                m.url_edit.clone()
            };
            let error = Url::parse(url.trim()).is_err();
            m.url_error = error;
            if !error {
                let username = if m.secret_edit.is_empty() {
                    Some("grin".to_string())
                } else {
                    Some(m.secret_edit.clone())
                };
                let secret = if m.secret_edit.is_empty() {
                    None
                } else {
                    Some(m.secret_edit.clone())
                };
                // Update or create new connection.
                let mut ext_conn = ExternalConnection::new(url, username, secret);
                if let Some(id) = m.id {
                    ext_conn.id = id;
                }
                ConnectionsConfig::add_ext_conn(ext_conn.clone());
                ExternalConnection::check(Some(ext_conn.id), ui.ctx());
                on_save(ext_conn);

                // Close modal.
                m.url_edit = "".to_string();
                m.secret_edit = "".to_string();
                m.url_error = false;
                Modal::close();
            }
        };

        ui.vertical_centered(|ui| {
            ui.add_space(6.0);
            ui.label(RichText::new(t!("wallets.node_url"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Draw node URL text edit.
            let url_edit_id = Id::from(modal.id).with(self.id).with("node_url");
            let mut url_edit = TextEdit::new(url_edit_id).paste().focus(self.first_draw);
            let url_edit_before = self.url_edit.clone();
            url_edit.ui(ui, &mut self.url_edit, cb);
            if self.url_edit != url_edit_before {
                self.url_error = false;
            }

            // ui.add_space(8.0);
            // ui.label(RichText::new(t!("wallets.name"))
            //     .size(17.0)
            //     .color(Colors::gray()));
            // ui.add_space(8.0);
            //
            // // Draw node username text edit (disabled by default).
            // let username_edit_id = Id::from(modal.id).with(self.id).with("node_username");
            // let mut username_edit = TextEdit::new(username_edit_id).focus(false).disable();
            // username_edit.ui(ui, &mut self.username_edit, cb);

            ui.add_space(8.0);
            ui.label(RichText::new(t!("wallets.node_secret"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Draw node API secret text edit.
            let secret_edit_id = Id::from(modal.id).with(self.id).with("node_secret");
            let mut secret_edit = TextEdit::new(secret_edit_id)
                .h_center()
                .password()
                .paste()
                .focus(false);
            if url_edit.enter_pressed {
                secret_edit.focus_request();
            }
            secret_edit.ui(ui, &mut self.secret_edit, cb);
            if secret_edit.enter_pressed {
                on_add(ui, self);
            }

            // Show error when specified URL is not valid.
            if self.url_error {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("wallets.invalid_url"))
                    .size(17.0)
                    .color(Colors::red()));
            }
            ui.add_space(12.0);

            let scan_text = format!("{} {}", SCAN, t!("scan"));
            View::button(ui, scan_text, Colors::white_or_black(false), || {
                modal.disable_closing();
                self.scan_qr_content = Some(CameraContent::default());
                cb.start_camera();
            });
        });

        ui.add_space(8.0);
        View::horizontal_line(ui, Colors::item_stroke());
        ui.add_space(8.0);

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                        // Close modal.
                        self.url_edit = "".to_string();
                        self.secret_edit = "".to_string();
                        self.url_error = false;
                        Modal::close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button_ui(ui, if self.id.is_some() {
                        t!("modal.save")
                    } else {
                        t!("modal.add")
                    }, Colors::white_or_black(false), |ui| {
                        (on_add)(ui, self);
                    });
                });
            });
            ui.add_space(6.0);
        });

        self.first_draw = false;
    }
}