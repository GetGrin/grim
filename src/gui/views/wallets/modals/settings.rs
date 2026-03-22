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

use egui::scroll_area::ScrollBarVisibility;
use egui::{RichText, ScrollArea};

use crate::gui::icons::{PLUS_CIRCLE, TRASH};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::network::modals::ExternalConnectionModal;
use crate::gui::views::network::ConnectionsContent;
use crate::gui::views::types::ModalPosition;
use crate::gui::views::wallets::WalletsContent;
use crate::gui::views::{Modal, View};
use crate::gui::Colors;
use crate::wallet::types::ConnectionMethod;
use crate::wallet::{ConnectionsConfig, ExternalConnection};

/// Wallet connection selection [`Modal`] content.
pub struct WalletSettingsModal {
    /// Current connection method.
    pub conn: ConnectionMethod,

    /// External connection creation content.
    new_ext_conn_content: Option<ExternalConnectionModal>
}

impl WalletSettingsModal {
    /// Create from provided wallet connection.
    pub fn new(conn: ConnectionMethod) -> Self {
        Self {
            conn,
            new_ext_conn_content: None,
        }
    }

    /// Draw [`Modal`] content.
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              modal: &Modal,
              cb: &dyn PlatformCallbacks,
              on_select: impl Fn(ConnectionMethod)) {
        // Draw external connection creation content.
        if let Some(ext_content) = self.new_ext_conn_content.as_mut() {
            ext_content.ui(ui, cb, modal, |conn| {
                on_select(ConnectionMethod::External(conn.id, conn.url));
            });
            return;
        }

        // Check connections state on first draw.
        if Modal::first_draw() {
            ExternalConnection::check(None, ui.ctx());
        }

        ui.add_space(4.0);

        let ext_conn_list = ConnectionsConfig::ext_conn_list();
        ScrollArea::vertical()
            .max_height(if ext_conn_list.len() < 4 {
                330.0
            } else {
                350.0
            })
            .id_salt("connections_scroll")
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .auto_shrink([true; 2])
            .show(ui, |ui| {
                ui.add_space(2.0);

                // Show integrated node selection.
                let cur_integrated = self.conn == ConnectionMethod::Integrated;
                let bg = if cur_integrated {
                    Colors::fill()
                } else {
                    Colors::fill_lite()
                };
                ConnectionsContent::integrated_node_item_ui(ui, bg, (!cur_integrated, || {
                    on_select(ConnectionMethod::Integrated);
                    Modal::close();
                }), |ui| {
                    if cur_integrated {
                        View::selected_item_check(ui);
                    }
                    cur_integrated
                });

                ui.add_space(8.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(t!("wallets.ext_conn"))
                        .size(16.0)
                        .color(Colors::gray()));
                    ui.add_space(6.0);
                    // Show button to add new external node connection.
                    let add_node_text = format!("{} {}", PLUS_CIRCLE, t!("wallets.add_node"));
                    View::button(ui, add_node_text, Colors::white_or_black(false), || {
                        self.new_ext_conn_content = Some(ExternalConnectionModal::new(None));
                    });
                });
                ui.add_space(4.0);

                if !ext_conn_list.is_empty() {
                    ui.add_space(8.0);
                }
                for (i, c) in ext_conn_list.iter().enumerate() {
                    ui.horizontal_wrapped(|ui| {
                        let len = ext_conn_list.len();
                        let is_current = match self.conn {
                            ConnectionMethod::External(id, _) => id == c.id,
                            _ => false
                        };
                        let bg = if is_current {
                            Colors::fill()
                        } else {
                            Colors::fill_lite()
                        };
                        ConnectionsContent::ext_conn_item_ui(ui, bg, c, i, len, (!is_current, || {
                            on_select(
                                ConnectionMethod::External(c.id, c.url.clone())
                            );
                            Modal::close();
                        }), |ui| {
                            if is_current {
                                View::selected_item_check(ui);
                            }
                        });
                    });
                    }
                ui.add_space(4.0);
            });

        ui.add_space(2.0);
        View::horizontal_line(ui, Colors::item_stroke());
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            // Draw button to delete the wallet.
            View::colored_text_button(ui,
                                      format!("{} {}", TRASH, t!("wallets.delete")),
                                      Colors::red(),
                                      Colors::white_or_black(false), || {
                    Modal::new(WalletsContent::DELETE_CONFIRMATION_MODAL)
                        .position(ModalPosition::Center)
                        .title(t!("confirmation"))
                        .show();
                });
        });
        ui.add_space(6.0);
        View::horizontal_line(ui, Colors::item_stroke());
        ui.add_space(6.0);

        // Show button to close modal.
        ui.vertical_centered_justified(|ui| {
            View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                Modal::close();
            });
        });
        ui.add_space(6.0);
    }
}