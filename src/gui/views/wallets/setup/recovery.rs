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

use egui::RichText;
use grin_chain::SyncStatus;
use crate::gui::Colors;
use crate::gui::icons::{EYE, STETHOSCOPE, TRASH, WRENCH};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, View};
use crate::gui::views::types::ModalPosition;
use crate::node::Node;
use crate::wallet::Wallet;

/// Wallet recovery setup content.
pub struct RecoverySetup {
    /// Wallet password [`Modal`] value.
    pass_edit: String,
    /// Flag to check if wrong password was entered.
    wrong_pass: bool,
    /// Flag to show recovery phrase when password check was passed.
    show_recovery_phrase: bool,
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
            show_recovery_phrase: false,
        }
    }
}

impl RecoverySetup {
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              _: &mut eframe::Frame,
              wallet: &mut Wallet,
              cb: &dyn PlatformCallbacks) {
        // Draw modal content for this ui container.
        self.modal_content_ui(ui, wallet, cb);

        ui.add_space(10.0);
        View::horizontal_line(ui, Colors::ITEM_STROKE);
        ui.add_space(6.0);
        View::sub_title(ui, format!("{} {}", WRENCH, t!("wallets.recovery")));
        View::horizontal_line(ui, Colors::ITEM_STROKE);
        ui.add_space(4.0);

        ui.vertical_centered(|ui| {
            let integrated_node = wallet.get_current_ext_conn_id().is_none();
            let integrated_node_ready = Node::get_sync_status() == Some(SyncStatus::NoSync);

            if wallet.sync_error() || (integrated_node && !integrated_node_ready) {
                ui.add_space(8.0);
                ui.label(RichText::new(t!("wallets.repair_unavailable"))
                    .size(16.0)
                    .color(Colors::RED));
            } else if wallet.is_repairing() {
                ui.add_space(8.0);
                View::small_loading_spinner(ui);
                ui.add_space(1.0);
            } else {
                ui.add_space(6.0);
                // Draw button to repair the wallet.
                let repair_text = format!("{} {}", STETHOSCOPE, t!("wallets.repair_wallet"));
                View::button(ui, repair_text, Colors::GOLD, || {
                    wallet.repair();
                });
            }

            ui.add_space(6.0);
            ui.label(RichText::new(t!("wallets.repair_desc"))
                .size(16.0)
                .color(Colors::INACTIVE_TEXT));

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);

            let recovery_phrase_text = format!("{}:", t!("wallets.recovery_phrase"));
            ui.label(RichText::new(recovery_phrase_text).size(16.0).color(Colors::GRAY));
            ui.add_space(6.0);

            // Draw button to show recovery phrase.
            let repair_text = format!("{} {}", EYE, t!("show"));
            View::button(ui, repair_text, Colors::BUTTON, || {
                // Setup modal values.
                self.pass_edit = "".to_string();
                self.wrong_pass = false;
                self.show_recovery_phrase = false;
                // Show recovery phrase modal.
                Modal::new(RECOVERY_PHRASE_MODAL)
                    .position(ModalPosition::CenterTop)
                    .title(t!("wallets.recovery_phrase"))
                    .show();
                cb.show_keyboard();
            });

            ui.add_space(12.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);
            ui.label(RichText::new(t!("wallets.delete_desc")).size(16.0).color(Colors::GRAY));
            ui.add_space(6.0);

            // Draw button to delete the wallet.
            let delete_text = format!("{} {}", TRASH, t!("wallets.delete"));
            View::button(ui, delete_text, Colors::GOLD, || {
                // Setup modal values.
                self.pass_edit = "".to_string();
                self.wrong_pass = false;
                // Show wallet deletion confirmation modal.
                Modal::new(DELETE_CONFIRMATION_MODAL)
                    .position(ModalPosition::CenterTop)
                    .title(t!("modal.confirmation"))
                    .show();
                cb.show_keyboard();
            });
            ui.add_space(8.0);
        });
    }

    /// Draw modal content for current ui container.
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
                            self.delete_confirmation_modal_ui(ui, wallet, modal, cb);
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    /// Draw recovery phrase [`Modal`] content.
    fn recovery_phrase_modal_ui(&mut self,
                                ui: &mut egui::Ui,
                                wallet: &mut Wallet,
                                modal: &Modal,
                                cb: &dyn PlatformCallbacks) {

    }

    /// Draw recovery phrase [`Modal`] content.
    fn delete_confirmation_modal_ui(&mut self,
                                ui: &mut egui::Ui,
                                wallet: &mut Wallet,
                                modal: &Modal,
                                cb: &dyn PlatformCallbacks) {

    }
}