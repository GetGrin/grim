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
use crate::gui::views::{Modal, View};
use crate::gui::views::network::ConnectionsContent;
use crate::gui::views::network::modals::ExternalConnectionModal;
use crate::wallet::ConnectionsConfig;
use crate::wallet::types::ConnectionMethod;

/// Wallet connection selection [`Modal`] content.
pub struct WalletConnectionModal {
    /// Current connection method.
    pub conn: ConnectionMethod,

    /// External connection content.
    ext_conn_content: Option<ExternalConnectionModal>
}

impl WalletConnectionModal {
    /// Create from provided wallet connection.
    pub fn new(conn: ConnectionMethod) -> Self {
        Self {
            conn,
            ext_conn_content: None,
        }
    }

    /// Draw [`Modal`] content.
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              modal: &Modal,
              cb: &dyn PlatformCallbacks,
              on_select: impl Fn(ConnectionMethod)) {
        // Draw external connection content.
        if let Some(ext_content) = self.ext_conn_content.as_mut() {
            ext_content.ui(ui, cb, modal, |conn| {
                on_select(ConnectionMethod::External(conn.id, conn.url));
            });
            return;
        }

        ui.add_space(4.0);

        let ext_conn_list = ConnectionsConfig::ext_conn_list();
        ScrollArea::vertical()
            .max_height(if ext_conn_list.len() < 4 {
                330.0
            } else {
                350.0
            })
            .id_source("integrated_node")
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .auto_shrink([true; 2])
            .show(ui, |ui| {
                ui.add_space(2.0);

                // Show integrated node selection.
                ConnectionsContent::integrated_node_item_ui(ui, |ui| {
                    match self.conn {
                        ConnectionMethod::Integrated => {
                            ui.add_space(14.0);
                            ui.label(RichText::new(CHECK_FAT).size(20.0).color(Colors::green()));
                            ui.add_space(14.0);
                        }
                        _ => {
                            View::item_button(ui, View::item_rounding(0, 1, true), CHECK, None, || {
                                on_select(ConnectionMethod::Integrated);
                                modal.close();
                            });
                        }
                    }
                });

                ui.add_space(8.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(t!("wallets.ext_conn"))
                        .size(16.0)
                        .color(Colors::gray()));
                    ui.add_space(6.0);
                    // Show button to add new external node connection.
                    let add_node_text = format!("{} {}", PLUS_CIRCLE, t!("wallets.add_node"));
                    View::button(ui, add_node_text, Colors::button(), || {
                        self.ext_conn_content = Some(ExternalConnectionModal::new(None));
                    });
                });
                ui.add_space(4.0);

                if !ext_conn_list.is_empty() {
                    ui.add_space(8.0);
                    for (index, conn) in ext_conn_list.iter().filter(|c| !c.deleted).enumerate() {
                        if conn.deleted {
                            continue;
                        }
                        ui.horizontal_wrapped(|ui| {
                            let len = ext_conn_list.len();
                            ConnectionsContent::ext_conn_item_ui(ui, conn, index, len, |ui| {
                                let current_ext_conn = match self.conn {
                                    ConnectionMethod::Integrated => false,
                                    ConnectionMethod::External(id, _) => id == conn.id
                                };
                                if !current_ext_conn {
                                    let button_rounding = View::item_rounding(index, len, true);
                                    View::item_button(ui, button_rounding, CHECK, None, || {
                                        on_select(
                                            ConnectionMethod::External(conn.id, conn.url.clone())
                                        );
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