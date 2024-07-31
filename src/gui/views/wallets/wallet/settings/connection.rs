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

use egui::{Align, Layout, RichText};

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

    /// Current wallet external connection.
    curr_ext_conn: Option<ExternalConnection>,

    /// External connection [`Modal`] content.
    ext_conn_modal: ExternalConnectionModal,

    /// [`Modal`] identifiers allowed at this ui container.
    modal_ids: Vec<&'static str>
}

impl Default for ConnectionSettings {
    fn default() -> Self {
        ExternalConnection::check_ext_conn_availability(None);
        Self {
            method: ConnectionMethod::Integrated,
            curr_ext_conn: None,
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
        self.ui(ui, None, cb);
    }

    /// Draw existing wallet connection setup content.
    pub fn wallet_ui(&mut self, ui: &mut egui::Ui, w: &mut Wallet, cb: &dyn PlatformCallbacks) {
        // Setup connection value from provided wallet.
        match w.get_config().ext_conn_id {
            None => self.method = ConnectionMethod::Integrated,
            Some(id) => self.method = ConnectionMethod::External(id)
        }

        // Draw setup content.
        self.ui(ui, Some(w), cb);

        // Setup wallet connection value after change.
        let changed = match self.method {
            ConnectionMethod::Integrated => {
                let changed = w.get_current_ext_conn().is_some();
                if changed {
                    w.update_ext_conn_id(None);
                }
                changed
            }
            ConnectionMethod::External(id) => {
                let changed = w.get_config().ext_conn_id != Some(id);
                if changed {
                    w.update_ext_conn_id(Some(id));
                }
                changed
            }
        };

        // Reopen wallet if connection changed.
        if changed {
            if !w.reopen_needed() {
                w.set_reopen(true);
                w.close();
            }
        }
    }

    /// Draw connection setup content.
    fn ui(&mut self,
          ui: &mut egui::Ui,
          wallet: Option<&Wallet>,
          cb: &dyn PlatformCallbacks) {
        // Draw modal content for current ui container.
        self.current_modal_ui(ui, cb);

        ui.add_space(2.0);
        View::sub_title(ui, format!("{} {}", GLOBE, t!("wallets.conn_method")));
        View::horizontal_line(ui, Colors::stroke());
        ui.add_space(6.0);

        ui.vertical_centered(|ui| {
            // Show integrated node selection.
            ui.add_space(6.0);
            ConnectionsContent::integrated_node_item_ui(ui, |ui| {
                // Draw button to select integrated node if it was not selected.
                let is_current_method = self.method == ConnectionMethod::Integrated;
                if !is_current_method {
                    View::item_button(ui, View::item_rounding(0, 1, true), CHECK, None, || {
                        self.method = ConnectionMethod::Integrated;
                    });
                } else {
                    ui.add_space(14.0);
                    ui.label(RichText::new(CHECK_FAT).size(20.0).color(Colors::green()));
                    ui.add_space(14.0);
                }
            });

            // Show external connections.
            ui.add_space(8.0);
            ui.label(RichText::new(t!("wallets.ext_conn")).size(16.0).color(Colors::gray()));
            ui.add_space(6.0);

            // Show button to add new external node connection.
            let add_node_text = format!("{} {}", PLUS_CIRCLE, t!("wallets.add_node"));
            View::button(ui, add_node_text, Colors::button(), || {
                self.show_add_ext_conn_modal(cb);
            });
            ui.add_space(4.0);

            let mut ext_conn_list = ConnectionsConfig::ext_conn_list();

            // Check if current external connection was deleted to show at 1st place.
            if let Some(wallet) = wallet {
                if let Some(conn) = wallet.get_current_ext_conn() {
                    if ext_conn_list.iter()
                        .filter(|c| c.id == conn.id)
                        .collect::<Vec<&ExternalConnection>>().is_empty() {
                        if self.curr_ext_conn.is_none() {
                            self.curr_ext_conn = Some(conn);
                        }
                        ext_conn_list.insert(0, self.curr_ext_conn.as_ref().unwrap().clone());
                    }
                }
            }

            if !ext_conn_list.is_empty() {
                ui.add_space(8.0);
                for (index, conn) in ext_conn_list.iter().enumerate() {
                    ui.horizontal_wrapped(|ui| {
                        // Draw external connection item.
                        self.ext_conn_item_ui(ui, wallet, conn, index, ext_conn_list.len());
                    });
                }
            }
        });
    }

    /// Draw external connection item content.
    fn ext_conn_item_ui(&mut self,
                        ui: &mut egui::Ui,
                        wallet: Option<&Wallet>,
                        conn: &ExternalConnection,
                        index: usize,
                        len: usize) {
        // Setup layout size.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(52.0);

        // Draw round background.
        let bg_rect = rect.clone();
        let item_rounding = View::item_rounding(index, len, false);
        ui.painter().rect(bg_rect, item_rounding, Colors::fill(), View::item_stroke());

        ui.vertical(|ui| {
            ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
                // Draw button to select connection.
                let is_current_method = if let Some(wallet) = wallet {
                    if let Some(cur) = wallet.get_config().ext_conn_id {
                        cur == conn.id
                    } else {
                        false
                    }
                } else {
                    self.method == ConnectionMethod::External(conn.id)
                };
                if !is_current_method {
                    let button_rounding = View::item_rounding(index, len, true);
                    View::item_button(ui, button_rounding, CHECK, None, || {
                        self.method = ConnectionMethod::External(conn.id);
                    });
                } else {
                    ui.add_space(12.0);
                    ui.label(RichText::new(CHECK_FAT).size(20.0).color(Colors::green()));
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

    /// Show external connection adding [`Modal`].
    fn show_add_ext_conn_modal(&mut self, cb: &dyn PlatformCallbacks) {
        self.ext_conn_modal = ExternalConnectionModal::new(None);
        // Show modal.
        Modal::new(ExternalConnectionModal::WALLET_ID)
            .position(ModalPosition::CenterTop)
            .title(t!("wallets.add_node"))
            .show();
        cb.show_keyboard();
    }
}