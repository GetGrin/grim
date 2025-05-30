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

use egui::{Id, RichText};

use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, TextEdit, View};
use crate::gui::Colors;
use crate::wallet::Wallet;

/// Account creation [`Modal`] content.
pub struct CreateAccountContent {
    /// Account label value.
    account_label_edit: String,
    /// Flag to check if error occurred during account creation.
    account_creation_error: bool,
}

impl Default for CreateAccountContent {
    fn default() -> Self {
        Self {
            account_label_edit: "".to_string(),
            account_creation_error: false,
        }
    }
}

impl CreateAccountContent {
    /// Draw account creation [`Modal`] content.
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              wallet: &Wallet,
              modal: &Modal,
              cb: &dyn PlatformCallbacks) {
        let on_create = |m: &mut CreateAccountContent| {
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
    }
}