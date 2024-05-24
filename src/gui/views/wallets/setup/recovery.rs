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

use egui::{Id, RichText};
use grin_chain::SyncStatus;
use grin_util::ZeroingString;

use crate::gui::Colors;
use crate::gui::icons::{EYE, LIFEBUOY, STETHOSCOPE, TRASH, WRENCH};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, View};
use crate::gui::views::types::{ModalPosition, TextEditOptions};
use crate::node::Node;
use crate::wallet::Wallet;

/// Wallet recovery setup content.
pub struct RecoverySetup {
    /// Wallet password [`Modal`] value.
    pass_edit: String,
    /// Flag to check if wrong password was entered.
    wrong_pass: bool,

    /// Recovery phrase value.
    recovery_phrase: Option<ZeroingString>,
}

/// Identifier for recovery phrase [`Modal`].
const RECOVERY_PHRASE_MODAL: &'static str = "recovery_phrase_modal";
/// Identifier to confirm wallet deletion [`Modal`].
const DELETE_CONFIRMATION_MODAL: &'static str = "delete_wallet_confirmation_modal";

impl Default for RecoverySetup {
    fn default() -> Self {
        Self {
            wrong_pass: false,
            pass_edit: "".to_string(),
            recovery_phrase: None,
        }
    }
}

impl RecoverySetup {
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              _: &mut eframe::Frame,
              wallet: &mut Wallet,
              cb: &dyn PlatformCallbacks) {
        // Show modal content for this ui container.
        self.modal_content_ui(ui, wallet, cb);

        ui.add_space(10.0);
        View::horizontal_line(ui, Colors::STROKE);
        ui.add_space(6.0);
        View::sub_title(ui, format!("{} {}", WRENCH, t!("wallets.recovery")));
        View::horizontal_line(ui, Colors::STROKE);
        ui.add_space(4.0);

        ui.vertical_centered(|ui| {
            let integrated_node = wallet.get_current_ext_conn().is_none();
            let integrated_node_ready = Node::get_sync_status() == Some(SyncStatus::NoSync);
            if wallet.sync_error() || (integrated_node && !integrated_node_ready) {
                ui.add_space(6.0);
                ui.label(RichText::new(t!("wallets.repair_unavailable"))
                    .size(16.0)
                    .color(Colors::RED));
            } else if !wallet.is_repairing() {
                ui.add_space(6.0);

                // Draw button to repair the wallet.
                let repair_text = format!("{} {}", STETHOSCOPE, t!("wallets.repair_wallet"));
                View::button(ui, repair_text, Colors::GOLD, || {
                    wallet.repair();
                });

                ui.add_space(6.0);
                ui.label(RichText::new(t!("wallets.repair_desc"))
                    .size(16.0)
                    .color(Colors::INACTIVE_TEXT));
            }

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);

            // Draw button to restore the wallet.
            let recover_text = format!("{} {}", LIFEBUOY, t!("wallets.recover"));
            ui.add_space(4.0);
            View::colored_text_button(ui, recover_text, Colors::GREEN, Colors::BUTTON, || {
                wallet.delete_db(true);
            });
            ui.add_space(6.0);
            ui.label(RichText::new(t!("wallets.restore_wallet_desc"))
                .size(16.0)
                .color(Colors::INACTIVE_TEXT));

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);

            let recovery_text = format!("{}:", t!("wallets.recovery_phrase"));
            ui.label(RichText::new(recovery_text).size(16.0).color(Colors::GRAY));
            ui.add_space(6.0);

            // Draw button to show recovery phrase.
            let show_text = format!("{} {}", EYE, t!("show"));
            View::button(ui, show_text, Colors::BUTTON, || {
                self.show_recovery_phrase_modal(cb);
            });

            ui.add_space(12.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);
            ui.label(RichText::new(t!("wallets.delete_desc")).size(16.0).color(Colors::TEXT));
            ui.add_space(6.0);

            // Draw button to delete the wallet.
            let delete_text = format!("{} {}", TRASH, t!("wallets.delete"));
            View::colored_text_button(ui, delete_text, Colors::RED, Colors::BUTTON, || {
                Modal::new(DELETE_CONFIRMATION_MODAL)
                    .position(ModalPosition::Center)
                    .title(t!("modal.confirmation"))
                    .show();
            });
            ui.add_space(8.0);
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
                    RECOVERY_PHRASE_MODAL => {
                        Modal::ui(ui.ctx(), |ui, modal| {
                            self.recovery_phrase_modal_ui(ui, wallet, modal, cb);
                        });
                    }
                    DELETE_CONFIRMATION_MODAL => {
                        Modal::ui(ui.ctx(), |ui, modal| {
                            self.deletion_modal_ui(ui, wallet, modal);
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    /// Show recovery phrase [`Modal`].
    fn show_recovery_phrase_modal(&mut self, cb: &dyn PlatformCallbacks) {
        // Setup modal values.
        self.pass_edit = "".to_string();
        self.wrong_pass = false;
        self.recovery_phrase = None;
        // Show recovery phrase modal.
        Modal::new(RECOVERY_PHRASE_MODAL)
            .position(ModalPosition::CenterTop)
            .title(t!("wallets.recovery_phrase"))
            .show();
        cb.show_keyboard();
    }

    /// Draw recovery phrase [`Modal`] content.
    fn recovery_phrase_modal_ui(&mut self,
                                ui: &mut egui::Ui,
                                wallet: &mut Wallet,
                                modal: &Modal,
                                cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        if self.recovery_phrase.is_some() {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(self.recovery_phrase.clone().unwrap().to_string())
                    .size(17.0)
                    .color(Colors::BLACK));
            });
            ui.add_space(10.0);
            ui.vertical_centered_justified(|ui| {
                View::button(ui, t!("close"), Colors::WHITE, || {
                    self.recovery_phrase = None;
                    modal.close();
                });
            });
        } else {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(t!("wallets.pass"))
                    .size(17.0)
                    .color(Colors::GRAY));
                ui.add_space(8.0);

                // Draw current wallet password text edit.
                let pass_edit_id = Id::from(modal.id).with(wallet.get_config().id);
                let mut pass_edit_opts = TextEditOptions::new(pass_edit_id).password();
                View::text_edit(ui, cb, &mut self.pass_edit, &mut pass_edit_opts);

                // Show information when password is empty or wrong.
                if self.pass_edit.is_empty() {
                    ui.add_space(12.0);
                    ui.label(RichText::new(t!("wallets.pass_empty"))
                        .size(17.0)
                        .color(Colors::INACTIVE_TEXT));
                } else if self.wrong_pass {
                    ui.add_space(12.0);
                    ui.label(RichText::new(t!("wallets.wrong_pass"))
                        .size(17.0)
                        .color(Colors::RED));
                }
            });
            ui.add_space(12.0);

            // Show modal buttons.
            ui.scope(|ui| {
                // Setup spacing between buttons.
                ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 0.0);

                ui.columns(2, |columns| {
                    columns[0].vertical_centered_justified(|ui| {
                        View::button(ui, t!("modal.cancel"), Colors::WHITE, || {
                            self.recovery_phrase = None;
                            modal.close();
                        });
                    });
                    columns[1].vertical_centered_justified(|ui| {
                        View::button(ui, "OK".to_owned(), Colors::WHITE, || {
                            match wallet.get_recovery(self.pass_edit.clone()) {
                                Ok(phrase) => {
                                    self.wrong_pass = false;
                                    self.recovery_phrase = Some(phrase);
                                    cb.hide_keyboard();
                                }
                                Err(_) => {
                                    self.wrong_pass = true;
                                }
                            }
                        });
                    });
                });
            });
        }
        ui.add_space(6.0);
    }

    /// Draw wallet deletion [`Modal`] content.
    fn deletion_modal_ui(&mut self,
                         ui: &mut egui::Ui,
                         wallet: &mut Wallet,
                         modal: &Modal) {
        ui.add_space(8.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("wallets.delete_conf"))
                .size(17.0)
                .color(Colors::TEXT));
        });
        ui.add_space(12.0);

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::WHITE, || {
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("delete"), Colors::WHITE, || {
                        wallet.delete_wallet();
                        modal.close();
                    });
                });
            });
            ui.add_space(6.0);
        });
    }
}