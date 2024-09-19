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

/// Initial wallet creation [`Modal`] content.
pub struct AddWalletModal {
    /// Flag to check if it's first draw to focus on first field.
    first_draw: bool,
    /// Wallet name.
    pub name_edit: String,
    /// Password to encrypt created wallet.
    pub pass_edit: String,
}

impl Default for AddWalletModal {
    fn default() -> Self {
        Self {
            first_draw: true,
            name_edit: t!("wallets.default_wallet"),
            pass_edit: "".to_string(),
        }
    }
}

impl AddWalletModal {
    /// Draw creating wallet name/password input [`Modal`] content.
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              modal: &Modal,
              cb: &dyn PlatformCallbacks,
              mut on_input: impl FnMut(String, ZeroingString)) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("wallets.name"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Show wallet name text edit.
            let mut name_edit_opts = TextEditOptions::new(Id::from(modal.id).with("name"))
                .no_focus();
            if self.first_draw {
                self.first_draw = false;
                name_edit_opts.focus = true;
            }
            View::text_edit(ui, cb, &mut self.name_edit, &mut name_edit_opts);
            ui.add_space(8.0);

            ui.label(RichText::new(t!("wallets.pass"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Draw wallet password text edit.
            let mut pass_text_edit_opts = TextEditOptions::new(Id::from(modal.id).with("pass"))
                .password()
                .no_focus();
            View::text_edit(ui, cb, &mut self.pass_edit, &mut pass_text_edit_opts);
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
                    let mut on_next = || {
                        let name = self.name_edit.clone();
                        let pass = self.pass_edit.clone();
                        if name.is_empty() || pass.is_empty() {
                            return;
                        }
                        cb.hide_keyboard();
                        modal.close();
                        on_input(name, ZeroingString::from(pass));
                    };

                    // Go to next creation step on Enter button press.
                    View::on_enter_key(ui, || {
                        (on_next)();
                    });

                    View::button(ui, t!("continue"), Colors::white_or_black(false), on_next);
                });
            });
            ui.add_space(6.0);
        });
    }
}