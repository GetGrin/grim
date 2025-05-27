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

use egui::{Align, Id, Layout, RichText, ScrollArea, StrokeKind};
use egui::scroll_area::ScrollBarVisibility;
use grin_core::core::amount_to_hr_string;

use crate::gui::Colors;
use crate::gui::icons::{CHECK, CHECK_FAT, FOLDER_USER, PATH};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, TextEdit, View};
use crate::gui::views::wallets::wallet::types::GRIN;
use crate::wallet::types::WalletAccount;
use crate::wallet::{Wallet, WalletConfig};

/// Wallet accounts [`Modal`] content.
pub struct WalletAccountsModal {
    /// List of wallet accounts.
    accounts: Vec<WalletAccount>,
    /// Flag to check if account is creating.
    account_creating: bool,
    /// Account label value.
    account_label_edit: String,
    /// Flag to check if error occurred during account creation.
    account_creation_error: bool,
}

impl Default for WalletAccountsModal {
    fn default() -> Self {
        Self {
            accounts: vec![],
            account_creating: false,
            account_label_edit: "".to_string(),
            account_creation_error: false,
        }
    }
}

impl WalletAccountsModal {
    /// Create new instance from wallet accounts.
    pub fn new(accounts: Vec<WalletAccount>) -> Self {
        Self {
            accounts,
            account_creating: false,
            account_label_edit: "".to_string(),
            account_creation_error: false,
        }
    }

    /// Draw [`Modal`] content.
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              wallet: &Wallet,
              modal: &Modal,
              cb: &dyn PlatformCallbacks) {
        if self.account_creating {
            let on_create = |m: &mut WalletAccountsModal| {
                if m.account_label_edit.is_empty() {
                    return;
                }
                let label = &m.account_label_edit;
                match wallet.create_account(label) {
                    Ok(_) => {
                        let _ = wallet.set_active_account(label);
                        Modal::close();
                    },
                    Err(_) => m.account_creation_error = true
                };
            };

            ui.add_space(6.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(t!("wallets.new_account_desc"))
                    .size(17.0)
                    .color(Colors::gray()));
                ui.add_space(8.0);

                // Draw account name edit.
                let mut name_edit = TextEdit::new(Id::from(modal.id).with(wallet.get_config().id));
                name_edit.ui(ui, &mut self.account_label_edit, cb);
                if name_edit.enter_pressed {
                    on_create(self);
                }

                // Show error occurred during account creation.
                if self.account_creation_error {
                    ui.add_space(12.0);
                    ui.label(RichText::new(t!("error"))
                        .size(17.0)
                        .color(Colors::red()));
                }
                ui.add_space(12.0);
            });

            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            // Show modal buttons.
            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                        // Close modal.
                        Modal::close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("create"), Colors::white_or_black(false), || {
                        on_create(self);
                    });
                });
            });
            ui.add_space(6.0);
        } else {
            ui.add_space(3.0);

            // Show list of accounts.
            let size = self.accounts.len();
            ScrollArea::vertical()
                .id_salt("account_list_modal_scroll")
                .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                .max_height(266.0)
                .auto_shrink([true; 2])
                .show_rows(ui, ACCOUNT_ITEM_HEIGHT, size, |ui, row_range| {
                    for index in row_range {
                        // Add space before the first item.
                        if index == 0 {
                            ui.add_space(4.0);
                        }
                        let acc = self.accounts.get(index).unwrap();
                        account_item_ui(ui, wallet, acc, index, size);
                        if index == size - 1 {
                            ui.add_space(4.0);
                        }
                    }
                });

            ui.add_space(2.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(6.0);

            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            // Show modal buttons.
            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                        Modal::close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("create"), Colors::white_or_black(false), || {
                        self.account_creating = true;
                    });
                });
            });
            ui.add_space(6.0);
        }
    }

}

const ACCOUNT_ITEM_HEIGHT: f32 = 75.0;

/// Draw account item.
fn account_item_ui(ui: &mut egui::Ui,
                   wallet: &Wallet,
                   acc: &WalletAccount,
                   index: usize,
                   size: usize) {
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
            let is_current_account = wallet.get_config().account == acc.label;
            if !is_current_account {
                let button_rounding = View::item_rounding(index, size, true);
                View::item_button(ui, button_rounding, CHECK, None, || {
                    let _ = wallet.set_active_account(&acc.label);
                    Modal::close();
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
                    ui.label(RichText::new(amount_text).size(18.0).color(Colors::white_or_black(true)));
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