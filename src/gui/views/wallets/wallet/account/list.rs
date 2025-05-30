// Copyright 2025 The Grim Developers
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

use crate::gui::icons::{CHECK, CHECK_FAT, FOLDER_USER, PATH};
use crate::gui::views::wallets::wallet::types::GRIN;
use crate::gui::views::View;
use crate::gui::Colors;
use crate::wallet::types::WalletAccount;
use crate::wallet::WalletConfig;

use egui::scroll_area::ScrollBarVisibility;
use egui::{Align, Layout, RichText, ScrollArea, StrokeKind};
use grin_core::core::amount_to_hr_string;

/// Wallet account list content.
pub struct WalletAccountsContent {
    /// List of wallet accounts.
    accounts: Vec<WalletAccount>,
    /// Current wallet account label.
    current_label: String,
}

const ACCOUNT_ITEM_HEIGHT: f32 = 75.0;

impl WalletAccountsContent {
    /// Create new accounts content.
    pub fn new(accounts: Vec<WalletAccount>, current: String) -> Self {
        Self { accounts, current_label: current }
    }

    /// Draw account list content.
    pub fn ui(&mut self, ui: &mut egui::Ui, mut on_select: impl FnMut(WalletAccount)) {
        let size = self.accounts.len();
        ScrollArea::vertical()
            .id_salt("account_list_scroll")
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .max_height(266.0)
            .auto_shrink([true; 2])
            .show_rows(ui, ACCOUNT_ITEM_HEIGHT, size, |ui, row_range| {
                for index in row_range {
                    // Add space before the first item.
                    if index == 0 {
                        ui.add_space(4.0);
                    }
                    let acc = self.accounts.get(index).unwrap().clone();
                    self.account_item_ui(ui, &acc, index, size, || {
                        on_select(acc.clone());
                    });
                    if index == size - 1 {
                        ui.add_space(4.0);
                    }
                }
            });
    }

    /// Draw account item.
    fn account_item_ui(&mut self,
                       ui: &mut egui::Ui,
                       acc: &WalletAccount,
                       index: usize,
                       size: usize,
                       mut on_select: impl FnMut()) {
        // Setup layout size.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(ACCOUNT_ITEM_HEIGHT);

        // Draw round background.
        let bg_rect = rect.clone();
        let item_rounding = View::item_rounding(index, size, false);
        ui.painter().rect(bg_rect,
                          item_rounding,
                          Colors::fill(),
                          View::item_stroke(),
                          StrokeKind::Middle);

        ui.vertical(|ui| {
            ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
                // Draw button to select account.
                let is_current_account = self.current_label == acc.label;
                if !is_current_account {
                    let button_rounding = View::item_rounding(index, size, true);
                    View::item_button(ui, button_rounding, CHECK, None, || {
                        on_select();
                    });
                } else {
                    ui.add_space(12.0);
                    ui.label(RichText::new(CHECK_FAT).size(20.0).color(Colors::green()));
                }

                let layout_size = ui.available_size();
                ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                    ui.add_space(6.0);
                    ui.vertical(|ui| {
                        ui.add_space(4.0);
                        // Show spendable amount.
                        let amount = amount_to_hr_string(acc.spendable_amount, true);
                        let amount_text = format!("{} {}", amount, GRIN);
                        ui.label(RichText::new(amount_text)
                            .size(18.0)
                            .color(Colors::white_or_black(true)));
                        ui.add_space(-2.0);

                        // Show account name.
                        let default_acc_label = WalletConfig::DEFAULT_ACCOUNT_LABEL.to_string();
                        let acc_label = if acc.label == default_acc_label {
                            t!("wallets.default_account")
                        } else {
                            acc.label.to_owned()
                        };
                        let acc_name = format!("{} {}", FOLDER_USER, acc_label);
                        View::ellipsize_text(ui, acc_name, 15.0, Colors::text(false));

                        // Show account BIP32 derivation path.
                        let acc_path = format!("{} {}", PATH, acc.path);
                        ui.label(RichText::new(acc_path).size(15.0).color(Colors::gray()));
                        ui.add_space(3.0);
                    });
                });
            });
        });
    }
}