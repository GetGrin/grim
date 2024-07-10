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

use egui::{RichText, ScrollArea};
use egui::scroll_area::ScrollBarVisibility;

use crate::gui::Colors;
use crate::gui::icons::{CHECK, CHECK_FAT, PLUS_CIRCLE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{ConnectionsContent, Modal, View};
use crate::gui::views::modals::ExternalConnectionModal;
use crate::wallet::{ConnectionsConfig, ExternalConnection};

/// Wallet connection content.
pub struct WalletConnectionModal {
    /// Current external connection.
    pub ext_conn: Option<ExternalConnection>,

    /// Flag to show connection creation.
    show_conn_creation: bool,

    /// External connection creation content.
    add_ext_conn_content: ExternalConnectionModal
}

impl WalletConnectionModal {
    /// Identifier for [`Modal`].
    pub const ID: &'static str = "select_connection_modal";

    /// Create from provided wallet connection.
    pub fn new(ext_conn: Option<ExternalConnection>) -> Self {
        ExternalConnection::check_ext_conn_availability(None);
        Self {
            ext_conn,
            show_conn_creation: false,
            add_ext_conn_content: ExternalConnectionModal::new(None),
        }
    }

    /// Draw [`Modal`] content.
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              modal: &Modal,
              cb: &dyn PlatformCallbacks,
              on_select: impl Fn(Option<i64>)) {
        ui.add_space(4.0);

        // Draw external connection creation content.
        if self.show_conn_creation {
            self.add_ext_conn_content.ui(ui, cb, modal, |conn| {
                on_select(Some(conn.id));
            });
            return;
        }

        let ext_conn_list = ConnectionsConfig::ext_conn_list();
        ScrollArea::vertical()
            .max_height(if ext_conn_list.len() < 4 {
                330.0
            } else {
                323.0
            })
            .id_source("integrated_node")
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .auto_shrink([true; 2])
            .show(ui, |ui| {
                ui.add_space(2.0);

                // Show integrated node selection.
                ConnectionsContent::integrated_node_item_ui(ui, |ui| {
                    let is_current_method = self.ext_conn.is_none();
                    if !is_current_method {
                        View::item_button(ui, View::item_rounding(0, 1, true), CHECK, None, || {
                            self.ext_conn = None;
                            on_select(None);
                            modal.close();
                        });
                    } else {
                        ui.add_space(14.0);
                        ui.label(RichText::new(CHECK_FAT).size(20.0).color(Colors::green()));
                        ui.add_space(14.0);
                    }
                });

                // Show button to add new external node connection.
                ui.add_space(8.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(t!("wallets.ext_conn"))
                        .size(16.0)
                        .color(Colors::gray()));
                    ui.add_space(6.0);
                    let add_node_text = format!("{} {}", PLUS_CIRCLE, t!("wallets.add_node"));
                    View::button(ui, add_node_text, Colors::button(), || {
                        self.show_conn_creation = true;
                    });
                });
                ui.add_space(4.0);

                if !ext_conn_list.is_empty() {
                    ui.add_space(8.0);
                    for (index, conn) in ext_conn_list.iter().enumerate() {
                        ui.horizontal_wrapped(|ui| {
                            // Draw external connection item.
                            let len = ext_conn_list.len();
                            ConnectionsContent::ext_conn_item_ui(ui, conn, index, len, |ui| {
                                // Draw button to select connection.
                                let is_current_method = if let Some(c) = self.ext_conn.as_ref() {
                                    c.id == conn.id
                                } else {
                                    false
                                };
                                if !is_current_method {
                                    let button_rounding = View::item_rounding(index, len, true);
                                    View::item_button(ui, button_rounding, CHECK, None, || {
                                        self.ext_conn = Some(conn.clone());
                                        on_select(Some(conn.id));
                                        modal.close();
                                    });
                                } else {
                                    ui.add_space(12.0);
                                    ui.label(RichText::new(CHECK_FAT)
                                        .size(20.0)
                                        .color(Colors::green()));
                                }
                            });
                        });
                    }
                }
                ui.add_space(4.0);
            });

        ui.add_space(2.0);
        View::horizontal_line(ui, Colors::stroke());
        ui.add_space(6.0);

        // Show button to close modal.
        ui.vertical_centered_justified(|ui| {
            View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                modal.close();
            });
        });
        ui.add_space(6.0);
    }
}