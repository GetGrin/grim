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

use egui::{Align, Layout, RichText, StrokeKind};

use crate::gui::Colors;
use crate::gui::icons::{CHECK, CHECK_CIRCLE, CHECK_FAT, DOTS_THREE_CIRCLE, GLOBE, GLOBE_SIMPLE, PLUS_CIRCLE, X_CIRCLE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, View};
use crate::gui::views::network::ConnectionsContent;
use crate::gui::views::network::modals::ExternalConnectionModal;
use crate::gui::views::types::{ModalContainer, ModalPosition};
use crate::wallet::{ConnectionsConfig, ExternalConnection, Wallet};
use crate::wallet::types::ConnectionMethod;

/// Wallet connection settings content.
pub struct ConnectionSettings {
    /// Selected connection method.
    pub method: ConnectionMethod,

    /// External connection [`Modal`] content.
    ext_conn_modal: ExternalConnectionModal,

    /// [`Modal`] identifiers allowed at this ui container.
    modal_ids: Vec<&'static str>
}

impl Default for ConnectionSettings {
    fn default() -> Self {
        Self {
            method: ConnectionMethod::Integrated,
            ext_conn_modal: ExternalConnectionModal::new(None),
            modal_ids: vec![
                ExternalConnectionModal::WALLET_ID
            ]
        }
    }
}

impl ModalContainer for ConnectionSettings {
    fn modal_ids(&self) -> &Vec<&'static str> {
        &self.modal_ids
    }

    fn modal_ui(&mut self,
                ui: &mut egui::Ui,
                modal: &Modal,
                cb: &dyn PlatformCallbacks) {
        match modal.id {
            ExternalConnectionModal::WALLET_ID => {
                self.ext_conn_modal.ui(ui, cb, modal, |_| {});
            },
            _ => {}
        }
    }
}

impl ConnectionSettings {
    /// Draw wallet creation setup content.
    pub fn create_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        self.ui(ui, cb);
    }

    /// Draw existing wallet connection setup content.
    pub fn wallet_ui(&mut self, ui: &mut egui::Ui, wallet: &Wallet, cb: &dyn PlatformCallbacks) {
        self.method = wallet.get_current_connection();

        // Draw setup content.
        let changed = self.ui(ui, cb);

        if changed {
            wallet.update_connection(&self.method);
            // Reopen wallet if connection changed.
            if !wallet.reopen_needed() {
                wallet.set_reopen(true);
                wallet.close();
            }
        }
    }

    /// Draw connection setup content, returning `true` if connection was changed.
    fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) -> bool {
        let mut changed = false;
        // Draw modal content for current ui container.
        self.current_modal_ui(ui, cb);

        ui.add_space(2.0);
        View::sub_title(ui, format!("{} {}", GLOBE, t!("wallets.conn_method")));
        View::horizontal_line(ui, Colors::stroke());
        ui.add_space(6.0);

        ui.vertical_centered(|ui| {
            ui.add_space(6.0);
            // Show integrated node selection.
            ConnectionsContent::integrated_node_item_ui(ui, |ui| {
                let is_current_method = self.method == ConnectionMethod::Integrated;
                if !is_current_method {
                    View::item_button(ui, View::item_rounding(0, 1, true), CHECK, None, || {
                        self.method = ConnectionMethod::Integrated;
                        changed = true;
                    });
                } else {
                    ui.add_space(14.0);
                    ui.label(RichText::new(CHECK_FAT).size(20.0).color(Colors::green()));
                    ui.add_space(14.0);
                }
            });

            ui.add_space(8.0);
            ui.label(RichText::new(t!("wallets.ext_conn")).size(16.0).color(Colors::gray()));
            ui.add_space(6.0);

            // Show button to add new external node connection.
            let add_node_text = format!("{} {}", PLUS_CIRCLE, t!("wallets.add_node"));
            View::button(ui, add_node_text, Colors::white_or_black(false), || {
                self.ext_conn_modal = ExternalConnectionModal::new(None);
                Modal::new(ExternalConnectionModal::WALLET_ID)
                    .position(ModalPosition::CenterTop)
                    .title(t!("wallets.add_node"))
                    .show();
            });
            ui.add_space(4.0);

            // Check for removed active connection.
            let cur_method = &self.method.clone();
            let mut ext_conn_list = ConnectionsConfig::ext_conn_list();
            let has_method = !ext_conn_list.iter().filter(|c| {
                match cur_method {
                    ConnectionMethod::Integrated => true,
                    ConnectionMethod::External(id, url) => id == &c.id || url == &c.url
                }
            }).collect::<Vec<&ExternalConnection>>().is_empty();
            if !has_method {
                match cur_method {
                    ConnectionMethod::External(id, url) => {
                        ext_conn_list.push(ExternalConnection {
                            id: *id,
                            url: url.clone(),
                            secret: None,
                            available: Some(true),
                        })
                    }
                    _ => {}
                }
            }

            let ext_size = ext_conn_list.len();
            if ext_size != 0 {
                ui.add_space(8.0);
                for (i, c) in ext_conn_list.iter().enumerate() {
                    ui.horizontal_wrapped(|ui| {
                        // Draw external connection item.
                        let is_current = match cur_method {
                            ConnectionMethod::External(id, url) => id == &c.id || url == &c.url,
                            _ => false
                        };
                        Self::ext_conn_item_ui(ui, c, is_current, i, ext_size, || {
                            self.method = ConnectionMethod::External(c.id, c.url.clone());
                            changed = true;
                        });
                    });
                }
            }
        });
        changed
    }

    /// Draw external connection item content.
    fn ext_conn_item_ui(ui: &mut egui::Ui,
                        conn: &ExternalConnection,
                        is_current: bool,
                        index: usize,
                        len: usize,
                        mut on_select: impl FnMut()) {
        // Setup layout size.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(52.0);

        // Draw round background.
        let bg_rect = rect.clone();
        let item_rounding = View::item_rounding(index, len, false);
        ui.painter().rect(bg_rect,
                          item_rounding,
                          Colors::fill(),
                          View::item_stroke(),
                          StrokeKind::Middle);

        ui.vertical(|ui| {
            ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
                if is_current {
                    ui.add_space(12.0);
                    ui.label(RichText::new(CHECK_FAT).size(20.0).color(Colors::green()));
                } else {
                    // Draw button to select connection.
                    let button_rounding = View::item_rounding(index, len, true);
                    View::item_button(ui, button_rounding, CHECK, None, || {
                        on_select();
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
}