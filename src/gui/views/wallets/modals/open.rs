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

use egui::{Id, RichText};
use grin_util::ZeroingString;

use crate::gui::Colors;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, View};
use crate::gui::views::types::TextEditOptions;
use crate::wallet::Wallet;

/// Wallet opening [`Modal`] content.
pub struct OpenWalletModal {
    /// Wallet to open.
    wallet: Wallet,

    /// Password to open wallet.
    pass_edit: String,
    /// Flag to check if wrong password was entered.
    wrong_pass: bool,

    /// Optional data to pass after wallet opening.
    data: Option<String>,
}

impl OpenWalletModal {
    /// Create new content instance.
    pub fn new(wallet: Wallet, data: Option<String>) -> Self {
        Self {
            wallet,
            pass_edit: "".to_string(),
            wrong_pass: false,
            data,
        }
    }
    /// Draw [`Modal`] content.
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              modal: &Modal,
              cb: &dyn PlatformCallbacks,
              mut on_continue: impl FnMut(Wallet, Option<String>)) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("wallets.pass"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Show password input.
            let mut pass_edit_opts = TextEditOptions::new(Id::from(modal.id)).password();
            View::text_edit(ui, cb, &mut self.pass_edit, &mut pass_edit_opts);

            // Show information when password is empty.
            if self.pass_edit.is_empty() {
                self.wrong_pass = false;
                ui.add_space(10.0);
                ui.label(RichText::new(t!("wallets.pass_empty"))
                    .size(17.0)
                    .color(Colors::inactive_text()));
            } else if self.wrong_pass {
                ui.add_space(10.0);
                ui.label(RichText::new(t!("wallets.wrong_pass"))
                    .size(17.0)
                    .color(Colors::red()));
            }
            ui.add_space(12.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                        // Close modal.
                        cb.hide_keyboard();
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    // Callback for button to continue.
                    let mut on_continue = || {
                        let pass = self.pass_edit.clone();
                        if pass.is_empty() {
                            return;
                        }
                        match self.wallet.open(ZeroingString::from(pass)) {
                            Ok(_) => {
                                self.pass_edit = "".to_string();
                                cb.hide_keyboard();
                                modal.close();
                                on_continue(self.wallet.clone(), self.data.clone());
                            }
                            Err(_) => self.wrong_pass = true
                        }
                    };

                    // Continue on Enter key press.
                    View::on_enter_key(ui, || {
                        (on_continue)();
                    });

                    View::button(ui, t!("continue"), Colors::white_or_black(false), on_continue);
                });
            });
            ui.add_space(6.0);
        });
    }
}