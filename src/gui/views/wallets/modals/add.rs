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
use crate::gui::views::{Modal, TextEdit, View};

/// Initial wallet creation [`Modal`] content.
pub struct AddWalletModal {
    /// Wallet name.
    pub name_edit: String,
    /// Password to encrypt created wallet.
    pub pass_edit: String,
}

impl Default for AddWalletModal {
    fn default() -> Self {
        Self {
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
        let mut on_next = |m: &mut AddWalletModal| {
            let name = m.name_edit.clone();
            let pass = m.pass_edit.clone();
            if name.is_empty() || pass.is_empty() {
                return;
            }
            modal.close();
            on_input(name, ZeroingString::from(pass));
        };
        ui.vertical_centered(|ui| {
            ui.add_space(6.0);
            ui.label(RichText::new(t!("wallets.name"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Show wallet name text edit.
            let mut name_input = TextEdit::new(Id::from(modal.id).with("name"))
                .focus(Modal::first_draw());
            
            name_input.ui(ui, &mut self.name_edit, cb);

            ui.add_space(8.0);
            ui.label(RichText::new(t!("wallets.pass"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Show wallet password text edit.
            let mut pass_input = TextEdit::new(Id::from(modal.id).with("pass"))
                .password()
                .focus(false);
            if name_input.enter_pressed {
                pass_input.focus_request();
            }
            pass_input.ui(ui, &mut self.pass_edit, cb);
            if pass_input.enter_pressed {
                (on_next)(self);
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
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("continue"), Colors::white_or_black(false), || {
                        (on_next)(self);
                    });
                });
            });
            ui.add_space(6.0);
        });
    }
}