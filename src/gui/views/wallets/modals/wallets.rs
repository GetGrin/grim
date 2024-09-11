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
use crate::gui::icons::{CHECK, CHECK_FAT, COMPUTER_TOWER, FOLDER_OPEN, GLOBE_SIMPLE, PLUGS_CONNECTED};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, View};
use crate::gui::views::types::ModalPosition;
use crate::gui::views::wallets::modals::OpenWalletModal;
use crate::gui::views::wallets::wallet::types::status_text;
use crate::wallet::{Wallet, WalletList};

/// Wallet list [`Modal`] content
pub struct WalletsModal {
    /// Selected wallet id.
    selected: Option<i64>,

    /// Optional data to pass after wallet selection.
    data: Option<String>,

    /// Flag to check if wallet can be opened from the list.
    can_open: bool,
    /// Wallet opening content.
    open_wallet_content: Option<OpenWalletModal>,
}

impl WalletsModal {
    /// Create new content instance.
    pub fn new(selected: Option<i64>, data: Option<String>, can_open: bool) -> Self {
        Self { selected, data, can_open, open_wallet_content: None }
    }

    /// Draw content.
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              modal: &Modal,
              wallets: &mut WalletList,
              cb: &dyn PlatformCallbacks,
              mut on_select: impl FnMut(i64, Option<String>)) {
        // Draw wallet opening content if requested.
        if let Some(open_content) = self.open_wallet_content.as_mut() {
            open_content.ui(ui, modal, wallets, cb, |data| {
                modal.close();
                if let Some(id) = self.selected {
                    on_select(id, data);
                }
                self.data = None;
            });
            return;
        }

        ui.add_space(4.0);
        ScrollArea::vertical()
            .max_height(373.0)
            .id_source("select_wallet_list")
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .auto_shrink([true; 2])
            .show(ui, |ui| {
                ui.add_space(2.0);
                ui.vertical_centered(|ui| {
                    let data = self.data.clone();
                    for wallet in wallets.clone().list() {
                        // Draw wallet list item.
                        self.wallet_item_ui(ui, wallet, wallets, |id| {
                            modal.close();
                            on_select(id, data.clone());
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
                self.data = None;
                modal.close();
            });
        });
        ui.add_space(6.0);
    }

    /// Draw wallet list item with provided callback on select.
    fn wallet_item_ui(&mut self,
                      ui: &mut egui::Ui,
                      wallet: &Wallet,
                      wallets: &mut WalletList,
                      mut select: impl FnMut(i64)) {
        let config = wallet.get_config();
        let id = config.id;

        // Draw round background.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(78.0);
        let rounding = View::item_rounding(0, 1, false);
        ui.painter().rect(rect, rounding, Colors::fill(), View::hover_stroke());

        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            if self.can_open {
                // Show button to select or open closed wallet.
                let icon = if wallet.is_open() {
                    CHECK
                } else {
                    FOLDER_OPEN
                };
                View::item_button(ui, View::item_rounding(0, 1, true), icon, None, || {
                    wallets.select(Some(id));
                    if wallet.is_open() {
                        select(id);
                    } else {
                        self.selected = wallets.selected_id;
                        Modal::change_position(ModalPosition::CenterTop);
                        self.open_wallet_content = Some(OpenWalletModal::new(self.data.clone()));
                    }
                });
            } else {
                // Draw button to select wallet.
                let current = self.selected.unwrap_or(0) == id;
                if current {
                    ui.add_space(12.0);
                    ui.label(RichText::new(CHECK_FAT).size(20.0).color(Colors::green()));
                } else {
                    View::item_button(ui, View::item_rounding(0, 1, true), CHECK, None, || {
                        select(id);
                    });
                }
            }

            let layout_size = ui.available_size();
            ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                ui.add_space(6.0);
                ui.vertical(|ui| {
                    ui.add_space(3.0);
                    // Show wallet name text.
                    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                        ui.add_space(1.0);
                        View::ellipsize_text(ui, config.name, 18.0, Colors::title(false));
                    });

                    // Show wallet connection text.
                    let conn = if let Some(conn) = wallet.get_current_ext_conn() {
                        format!("{} {}", GLOBE_SIMPLE, conn.url)
                    } else {
                        format!("{} {}", COMPUTER_TOWER, t!("network.node"))
                    };
                    View::ellipsize_text(ui, conn, 15.0, Colors::text(false));
                    ui.add_space(1.0);

                    // Show wallet API text or open status.
                    if self.can_open {
                        ui.label(RichText::new(status_text(wallet))
                            .size(15.0)
                            .color(Colors::gray()));
                    } else {
                        let address = if let Some(port) = config.api_port {
                            format!("127.0.0.1:{}", port)
                        } else {
                            "-".to_string()
                        };
                        let api_text = format!("{} {}", PLUGS_CONNECTED, address);
                        ui.label(RichText::new(api_text).size(15.0).color(Colors::gray()));
                    }
                    ui.add_space(3.0);
                });
            });
        });
    }
}