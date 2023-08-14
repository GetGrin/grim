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
use crate::gui::views::types::ModalPosition;
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
    current_pass_edit: String,
    /// Flag to show/hide old password at [`egui::TextEdit`] field.
    hide_current_pass: bool,
    /// New wallet password [`Modal`] value.
    new_pass_edit: String,
    /// Flag to show/hide new password at [`egui::TextEdit`] field.
    hide_new_pass: bool,

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
            current_pass_edit: "".to_string(),
            hide_current_pass: true,
            new_pass_edit: "".to_string(),
            hide_new_pass: true,
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
            // Show wallet name.
            ui.add_space(2.0);
            ui.label(RichText::new(t!("wallets.name")).size(16.0).color(Colors::GRAY));
            ui.add_space(2.0);
            ui.label(RichText::new(wallet.config.name.clone()).size(16.0).color(Colors::BLACK));
            ui.add_space(8.0);

            // Show wallet name setup.
            let name_text = format!("{} {}", PENCIL, t!("change"));
            View::button(ui, name_text, Colors::BUTTON, || {
                self.name_edit = wallet.config.name.clone();
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
                self.current_pass_edit = "".to_string();
                self.new_pass_edit = "".to_string();
                self.hide_current_pass = true;
                self.hide_new_pass = true;
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
            let min_conf_text = format!("{} {}", CLOCK_COUNTDOWN, wallet.config.min_confirmations);
            View::button(ui, min_conf_text, Colors::BUTTON, || {
                self.min_confirmations_edit = wallet.config.min_confirmations.to_string();
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

            // Draw wallet name edit.
            let text_edit_resp = egui::TextEdit::singleline(&mut self.name_edit)
                .id(Id::from(modal.id).with(wallet.config.id))
                .font(TextStyle::Heading)
                .desired_width(ui.available_width())
                .cursor_at_end(true)
                .ui(ui);
            text_edit_resp.request_focus();
            if text_edit_resp.clicked() {
                cb.show_keyboard();
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
                        if !self.name_edit.is_empty() {
                            wallet.config.name = self.name_edit.clone();
                            wallet.config.save();
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
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("wallets.current_pass"))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(6.0);

            let mut rect = ui.available_rect_before_wrap();
            rect.set_height(34.0);
            ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
                // Draw button to show/hide current password.
                let eye_icon = if self.hide_current_pass { EYE } else { EYE_SLASH };
                View::button(ui, eye_icon.to_string(), Colors::WHITE, || {
                    self.hide_current_pass = !self.hide_current_pass;
                });

                let layout_size = ui.available_size();
                ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                    // Draw current wallet password text edit.
                    let old_pass_resp = egui::TextEdit::singleline(&mut self.current_pass_edit)
                        .id(Id::from(modal.id).with(wallet.config.id).with("old_pass"))
                        .font(TextStyle::Heading)
                        .desired_width(ui.available_width())
                        .cursor_at_end(true)
                        .password(self.hide_current_pass)
                        .ui(ui);
                    if old_pass_resp.clicked() {
                        cb.show_keyboard();
                    }

                    // Setup focus on input field on first modal opening.
                    if self.first_edit_pass_opening {
                        self.first_edit_pass_opening = false;
                        old_pass_resp.request_focus();
                    }
                });
            });
            ui.add_space(6.0);

            ui.label(RichText::new(t!("wallets.new_pass"))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(6.0);

            let mut new_rect = ui.available_rect_before_wrap();
            new_rect.set_height(34.0);
            ui.allocate_ui_with_layout(new_rect.size(), Layout::right_to_left(Align::Center), |ui| {
                // Draw button to show/hide new password.
                let eye_icon = if self.hide_new_pass { EYE } else { EYE_SLASH };
                View::button(ui, eye_icon.to_string(), Colors::WHITE, || {
                    self.hide_new_pass = !self.hide_new_pass;
                });

                let layout_size = ui.available_size();
                ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                    // Draw new wallet password text edit.
                    let new_pass_resp = egui::TextEdit::singleline(&mut self.new_pass_edit)
                        .id(Id::from(modal.id).with(wallet.config.id).with("new_pass"))
                        .font(TextStyle::Heading)
                        .desired_width(ui.available_width())
                        .cursor_at_end(true)
                        .password(self.hide_new_pass)
                        .ui(ui);
                    if new_pass_resp.clicked() {
                        cb.show_keyboard();
                    }
                });
            });

            // Show information when password is empty.
            if self.current_pass_edit.is_empty() || self.new_pass_edit.is_empty() {
                ui.add_space(8.0);
                ui.label(RichText::new(t!("wallets.pass_empty"))
                    .size(17.0)
                    .color(Colors::INACTIVE_TEXT));
            } else if self.wrong_pass {
                ui.add_space(8.0);
                ui.label(RichText::new(t!("wallets.wrong_pass"))
                    .size(17.0)
                    .color(Colors::RED));
            }
            ui.add_space(10.0);
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
                        let old_pass = self.current_pass_edit.clone();
                        let new_pass = self.new_pass_edit.clone();
                        match wallet.change_password(old_pass, new_pass) {
                            Ok(_) => {
                                // Clear values.
                                self.first_edit_pass_opening = true;
                                self.current_pass_edit = "".to_string();
                                self.new_pass_edit = "".to_string();
                                self.hide_current_pass = true;
                                self.hide_new_pass = true;
                                self.wrong_pass = false;
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
            let text_edit_resp = egui::TextEdit::singleline(&mut self.min_confirmations_edit)
                .id(Id::from(modal.id))
                .font(TextStyle::Heading)
                .desired_width(48.0)
                .cursor_at_end(true)
                .ui(ui);
            text_edit_resp.request_focus();
            if text_edit_resp.clicked() {
                cb.show_keyboard();
            }

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
                            wallet.config.min_confirmations = min_conf;
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