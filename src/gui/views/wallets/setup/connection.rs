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

use egui::{Id, RichText, ScrollArea, TextStyle, Widget};
use url::Url;

use crate::AppConfig;
use crate::gui::Colors;
use crate::gui::icons::{GLOBE, GLOBE_SIMPLE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, View};
use crate::gui::views::wallets::setup::ConnectionMethod;

/// Wallet node connection method setup content.
pub struct ConnectionSetup {
    /// Selected connection method.
    method: ConnectionMethod,

    /// External node connection URL value for [`Modal`].
    ext_node_url_edit: String,
    /// Flag to show URL format error.
    ext_node_url_error: bool,
}

impl Default for ConnectionSetup {
    fn default() -> Self {
        Self {
            method: ConnectionMethod::Integrated,
            ext_node_url_edit: "".to_string(),
            ext_node_url_error: false
        }
    }
}

impl ConnectionSetup {
    /// External node connection [`Modal`] identifier.
    pub const ADD_CONNECTION_URL_MODAL: &'static str = "add_connection_url_modal";

    //TODO: Setup for provided wallet
    // pub fn new() -> Self {
    //     Self { method: ConnectionMethod::Integrated }
    // }

    pub fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        ScrollArea::vertical()
            .id_source("wallet_connection_setup")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                View::sub_title(ui, format!("{} {}", GLOBE, t!("wallets.conn_method")));
                View::horizontal_line(ui, Colors::STROKE);
                ui.add_space(4.0);

                ui.vertical_centered(|ui| {
                    // Show integrated node selection.
                    ui.add_space(6.0);
                    View::radio_value(ui,
                                      &mut self.method,
                                      ConnectionMethod::Integrated,
                                      t!("network.node"));

                    ui.add_space(10.0);
                    ui.label(RichText::new(t!("wallets.ext_conn")).size(16.0).color(Colors::GRAY));
                    ui.add_space(6.0);

                    // Show button to add new external node connection.
                    let add_node_text = format!("{} {}", GLOBE_SIMPLE, t!("wallets.add_node_url"));
                    View::button(ui, add_node_text, Colors::GOLD, || {
                        // Setup values for Modal.
                        self.ext_node_url_edit = "".to_string();
                        self.ext_node_url_error = false;
                        // Show modal.
                        Modal::new(Self::ADD_CONNECTION_URL_MODAL)
                            .title(t!("wallets.ext_conn"))
                            .show();
                        cb.show_keyboard();
                    });
                    ui.add_space(12.0);

                    // Show external nodes URLs selection.
                    for conn in AppConfig::external_nodes_urls() {
                        View::radio_value(ui,
                                          &mut self.method,
                                          ConnectionMethod::External(conn.clone()),
                                          conn);
                        ui.add_space(12.0);
                    }
                });
            });
    }

    /// Draw external connections setup.
    fn external_conn_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {

    }

    /// Draw modal content.
    pub fn modal_ui(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            // Draw external node URL text edit.
            let text_edit_resp = egui::TextEdit::singleline(&mut self.ext_node_url_edit)
                .id(Id::from(modal.id))
                .font(TextStyle::Heading)
                .desired_width(ui.available_width())
                .cursor_at_end(true)
                .ui(ui);
            text_edit_resp.request_focus();
            if text_edit_resp.clicked() {
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

            // Add button callback.
            let on_add = || {
                let error = Url::parse(self.ext_node_url_edit.as_str()).is_err();
                self.ext_node_url_error = error;
                if !error {
                    AppConfig::add_external_node_url(self.ext_node_url_edit.clone());
                    // Close modal.
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
                    View::button(ui, t!("modal.add"), Colors::WHITE, on_add);
                });
            });
            ui.add_space(6.0);
        });
    }
}