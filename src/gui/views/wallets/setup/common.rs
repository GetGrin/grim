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

use egui::{Align, Id, Layout, RichText, TextStyle, Widget};

use crate::gui::Colors;
use crate::gui::icons::{CLOCK_COUNTDOWN, EYE, EYE_SLASH, PASSWORD, PENCIL};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, View};
use crate::gui::views::types::{ModalPosition, TextEditOptions};
use crate::wallet::Wallet;

/// Common wallet setup content.
pub struct CommonSetup {
    /// Wallet name [`Modal`] value.
    name_edit: String,

    /// Flag to check if password change [`Modal`] was opened at first time to focus input field.
    first_edit_pass_opening: bool,
    /// Flag to check if wrong password was entered.
    wrong_pass: bool,
    /// Current wallet password [`Modal`] value.
    old_pass_edit: String,
    /// New wallet password [`Modal`] value.
    new_pass_edit: String,

    /// Minimum confirmations number value.
    min_confirmations_edit: String
}

/// Identifier for wallet name [`Modal`].
const NAME_EDIT_MODAL: &'static str = "wallet_name_edit_modal";
/// Identifier for wallet password [`Modal`].
const PASS_EDIT_MODAL: &'static str = "wallet_pass_edit_modal";
/// Identifier for minimum confirmations [`Modal`].
const MIN_CONFIRMATIONS_EDIT_MODAL: &'static str = "wallet_min_conf_edit_modal";

impl Default for CommonSetup {
    fn default() -> Self {
        Self {
            name_edit: "".to_string(),
            first_edit_pass_opening: true,
            wrong_pass: false,
            old_pass_edit: "".to_string(),
            new_pass_edit: "".to_string(),
            min_confirmations_edit: "".to_string()
        }
    }
}

impl CommonSetup {
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              _: &mut eframe::Frame,
              wallet: &mut Wallet,
              cb: &dyn PlatformCallbacks) {
        // Show modal content for this ui container.
        self.modal_content_ui(ui, wallet, cb);

        ui.vertical_centered(|ui| {
            let wallet_name = wallet.get_config().name;
            // Show wallet name.
            ui.add_space(2.0);
            ui.label(RichText::new(t!("wallets.name")).size(16.0).color(Colors::GRAY));
            ui.add_space(2.0);
            ui.label(RichText::new(wallet_name.clone()).size(16.0).color(Colors::BLACK));
            ui.add_space(8.0);

            // Show wallet name setup.
            let name_text = format!("{} {}", PENCIL, t!("change"));
            View::button(ui, name_text, Colors::BUTTON, || {
                self.name_edit = wallet_name;
                // Show wallet name modal.
                Modal::new(NAME_EDIT_MODAL)
                    .position(ModalPosition::CenterTop)
                    .title(t!("wallets.wallet"))
                    .show();
                cb.show_keyboard();
            });

            ui.add_space(12.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);
            ui.label(RichText::new(t!("wallets.pass")).size(16.0).color(Colors::GRAY));
            ui.add_space(6.0);

            // Show wallet password setup.
            let pass_text = format!("{} {}", PASSWORD, t!("change"));
            View::button(ui, pass_text, Colors::BUTTON, || {
                // Setup modal values.
                self.first_edit_pass_opening = true;
                self.old_pass_edit = "".to_string();
                self.new_pass_edit = "".to_string();
                self.wrong_pass = false;
                // Show wallet password modal.
                Modal::new(PASS_EDIT_MODAL)
                    .position(ModalPosition::CenterTop)
                    .title(t!("wallets.wallet"))
                    .show();
                cb.show_keyboard();
            });

            ui.add_space(12.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);
            ui.label(RichText::new(t!("wallets.min_tx_conf_count")).size(16.0).color(Colors::GRAY));
            ui.add_space(6.0);

            // Show minimum amount of confirmations value setup.
            let min_confirmations = wallet.get_config().min_confirmations;
            let min_conf_text = format!("{} {}", CLOCK_COUNTDOWN, min_confirmations);
            View::button(ui, min_conf_text, Colors::BUTTON, || {
                self.min_confirmations_edit = min_confirmations.to_string();
                // Show minimum amount of confirmations value modal.
                Modal::new(MIN_CONFIRMATIONS_EDIT_MODAL)
                    .position(ModalPosition::CenterTop)
                    .title(t!("network_settings.change_value"))
                    .show();
                cb.show_keyboard();
            });

            ui.add_space(12.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(4.0);
        });
    }

    /// Draw [`Modal`] content for this ui container.
    fn modal_content_ui(&mut self,
                        ui: &mut egui::Ui,
                        wallet: &mut Wallet,
                        cb: &dyn PlatformCallbacks) {
        match Modal::opened() {
            None => {}
            Some(id) => {
                match id {
                    NAME_EDIT_MODAL => {
                        Modal::ui(ui.ctx(), |ui, modal| {
                            self.name_modal_ui(ui, wallet, modal, cb);
                        });
                    }
                    PASS_EDIT_MODAL => {
                        Modal::ui(ui.ctx(), |ui, modal| {
                            self.pass_modal_ui(ui, wallet, modal, cb);
                        });
                    }
                    MIN_CONFIRMATIONS_EDIT_MODAL => {
                        Modal::ui(ui.ctx(), |ui, modal| {
                            self.min_conf_modal_ui(ui, wallet, modal, cb);
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    /// Draw wallet name [`Modal`] content.
    fn name_modal_ui(&mut self,
                     ui: &mut egui::Ui,
                     wallet: &mut Wallet,
                     modal: &Modal,
                     cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("wallets.name"))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Show wallet name text edit.
            let name_edit_id = Id::from(modal.id).with(wallet.get_config().id);
            let name_edit_opts = TextEditOptions::new(name_edit_id);
            View::text_edit(ui, cb, &mut self.name_edit, name_edit_opts);
            ui.add_space(12.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::WHITE, || {
                        // Close modal.
                        cb.hide_keyboard();
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    // Save button callback.
                    let mut on_save = || {
                        if !self.name_edit.is_empty() {
                            wallet.change_name(self.name_edit.clone());
                            cb.hide_keyboard();
                            modal.close();
                        }
                    };

                    View::on_enter_key(ui, || {
                        (on_save)();
                    });

                    View::button(ui, t!("modal.save"), Colors::WHITE, on_save);
                });
            });
            ui.add_space(6.0);
        });
    }

    /// Draw wallet pass [`Modal`] content.
    fn pass_modal_ui(&mut self,
                     ui: &mut egui::Ui,
                     wallet: &mut Wallet,
                     modal: &Modal,
                     cb: &dyn PlatformCallbacks) {
        let wallet_id = wallet.get_config().id;

        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("wallets.current_pass"))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw old password text edit.
            let pass_edit_id = Id::from(modal.id).with(wallet_id).with("old_pass");
            let mut pass_edit_opts = TextEditOptions::new(pass_edit_id).password().no_focus();
            if self.first_edit_pass_opening {
                self.first_edit_pass_opening = false;
                pass_edit_opts.focus = true;
            }
            View::text_edit(ui, cb, &mut self.old_pass_edit, pass_edit_opts);
            ui.add_space(8.0);

            ui.label(RichText::new(t!("wallets.new_pass"))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw new password text edit.
            let new_pass_edit_id = Id::from(modal.id).with(wallet_id).with("new_pass");
            let new_pass_edit_opts = TextEditOptions::new(new_pass_edit_id).password().no_focus();
            View::text_edit(ui, cb, &mut self.new_pass_edit, new_pass_edit_opts);

            // Show information when password is empty.
            if self.old_pass_edit.is_empty() || self.new_pass_edit.is_empty() {
                ui.add_space(10.0);
                ui.label(RichText::new(t!("wallets.pass_empty"))
                    .size(17.0)
                    .color(Colors::INACTIVE_TEXT));
            } else if self.wrong_pass {
                ui.add_space(10.0);
                ui.label(RichText::new(t!("wallets.wrong_pass"))
                    .size(17.0)
                    .color(Colors::RED));
            }
            ui.add_space(12.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::WHITE, || {
                        // Close modal.
                        cb.hide_keyboard();
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    // Callback for button to continue.
                    let mut on_continue = || {
                        if self.new_pass_edit.is_empty() {
                            return;
                        }
                        let old_pass = self.old_pass_edit.clone();
                        let new_pass = self.new_pass_edit.clone();
                        match wallet.change_password(old_pass, new_pass) {
                            Ok(_) => {
                                // Clear password values.
                                self.old_pass_edit = "".to_string();
                                self.new_pass_edit = "".to_string();
                                // Close modal.
                                cb.hide_keyboard();
                                modal.close();
                            }
                            Err(_) => self.wrong_pass = true
                        }
                    };

                    // Continue on Enter key press.
                    View::on_enter_key(ui, || {
                        (on_continue)();
                    });

                    View::button(ui, t!("change"), Colors::WHITE, on_continue);
                });
            });
            ui.add_space(6.0);
        });
    }

    /// Draw wallet name [`Modal`] content.
    fn min_conf_modal_ui(&mut self,
                         ui: &mut egui::Ui,
                         wallet: &mut Wallet,
                         modal: &Modal,
                         cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("wallets.min_tx_conf_count"))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Minimum amount of confirmations text edit.
            let text_edit_opts = TextEditOptions::new(Id::from(modal.id)).h_center();
            View::text_edit(ui, cb, &mut self.min_confirmations_edit, text_edit_opts);

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.min_confirmations_edit.parse::<u64>().is_err() {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.not_valid_value"))
                    .size(17.0)
                    .color(Colors::RED));
            }
            ui.add_space(12.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::WHITE, || {
                        // Close modal.
                        cb.hide_keyboard();
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    // Save button callback.
                    let mut on_save = || {
                        if let Ok(min_conf) = self.min_confirmations_edit.parse::<u64>() {
                            wallet.update_min_confirmations(min_conf);
                            cb.hide_keyboard();
                            modal.close();
                        }
                    };

                    View::on_enter_key(ui, || {
                        (on_save)();
                    });

                    View::button(ui, t!("modal.save"), Colors::WHITE, on_save);
                });
            });
            ui.add_space(6.0);
        });
    }
}