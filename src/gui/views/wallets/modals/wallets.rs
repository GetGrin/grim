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
use egui::{Align, Layout, RichText, ScrollArea};

use crate::gui::Colors;
use crate::gui::icons::{CHECK, CHECK_FAT, COMPUTER_TOWER, GLOBE_SIMPLE, PLUGS_CONNECTED};
use crate::gui::views::{Modal, View};
use crate::wallet::{Wallet, WalletList};

/// Wallet list [`Modal`] content
pub struct WalletsModal {
    /// Selected wallet id.
    selected: Option<i64>
}

impl WalletsModal {
    pub fn new(selected: Option<i64>) -> Self {
        Self {
            selected,
        }
    }

    /// Draw [`Modal`] content.
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              modal: &Modal,
              wallets: &WalletList,
              mut on_select: impl FnMut(i64)) {
        ui.add_space(4.0);
        ScrollArea::vertical()
            .max_height(373.0)
            .id_source("select_wallet_list")
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .auto_shrink([true; 2])
            .show(ui, |ui| {
                ui.add_space(2.0);
                ui.vertical_centered(|ui| {
                    for wallet in wallets.list() {
                        // Draw wallet list item.
                        self.wallet_item_ui(ui, wallet, modal, |id| {
                            on_select(id);
                        });
                        ui.add_space(5.0);
                    }
                });
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

    /// Draw wallet list item.
    fn wallet_item_ui(&mut self,
                      ui: &mut egui::Ui,
                      wallet: &Wallet,
                      modal: &Modal,
                      mut on_select: impl FnMut(i64)) {
        let config = wallet.get_config();
        let id = config.id;

        // Draw round background.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(78.0);
        let rounding = View::item_rounding(0, 1, false);
        ui.painter().rect(rect, rounding, Colors::fill(), View::hover_stroke());

        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            // Draw button to select wallet.
            let current = self.selected.unwrap_or(0) == id;
            if current {
                ui.add_space(12.0);
                ui.label(RichText::new(CHECK_FAT).size(20.0).color(Colors::green()));
            } else {
                View::item_button(ui, View::item_rounding(0, 1, true), CHECK, None, || {
                    on_select(id);
                    modal.close();
                });
            }

            let layout_size = ui.available_size();
            ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                ui.add_space(6.0);
                ui.vertical(|ui| {
                    ui.add_space(3.0);
                    // Setup wallet name text.
                    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                        ui.add_space(1.0);
                        View::ellipsize_text(ui, config.name, 18.0, Colors::title(false));
                    });

                    // Setup wallet API text.
                    let address = if let Some(port) = config.api_port {
                        format!("127.0.0.1:{}", port)
                    } else {
                        "-".to_string()
                    };
                    let api_text = format!("{} {}", PLUGS_CONNECTED, address);
                    ui.label(RichText::new(api_text).size(15.0).color(Colors::text(false)));
                    ui.add_space(1.0);

                    // Setup wallet connection text.
                    let conn = if let Some(conn) = wallet.get_current_ext_conn() {
                        format!("{} {}", GLOBE_SIMPLE, conn.url)
                    } else {
                        format!("{} {}", COMPUTER_TOWER, t!("network.node"))
                    };
                    View::ellipsize_text(ui, conn, 15.0, Colors::gray());
                    ui.add_space(3.0);
                });
            });
        });
    }
}