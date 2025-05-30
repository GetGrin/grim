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

use std::sync::Arc;
use std::thread;
use parking_lot::RwLock;
use egui::{Id, RichText};
use grin_core::core::{amount_from_hr_string, amount_to_hr_string};
use grin_wallet_libwallet::Error;

use crate::gui::Colors;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, TextEdit, View};
use crate::gui::views::wallets::wallet::WalletTransactionModal;
use crate::wallet::types::WalletTransaction;
use crate::wallet::Wallet;

/// Invoice or sending request creation [`Modal`] content.
pub struct MessageRequestModal {
    /// Flag to check if invoice or sending request was opened.
    invoice: bool,

    /// Amount to send or receive.
    amount_edit: String,

    /// Flag to check if request is loading.
    request_loading: bool,
    /// Request result if there is no error.
    request_result: Arc<RwLock<Option<Result<WalletTransaction, Error>>>>,
    /// Flag to check if there is an error happened on request creation.
    request_error: Option<String>,

    /// Request result transaction content.
    result_tx_content: Option<WalletTransactionModal>,
}

impl MessageRequestModal {
    /// Create new content instance.
    pub fn new(invoice: bool) -> Self {
        Self {
            invoice,
            amount_edit: "".to_string(),
            request_loading: false,
            request_result: Arc::new(RwLock::new(None)),
            request_error: None,
            result_tx_content: None,
        }
    }

    /// Draw [`Modal`] content.
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              wallet: &Wallet,
              modal: &Modal,
              cb: &dyn PlatformCallbacks) {
        // Draw transaction information on request result.
        if let Some(tx) = self.result_tx_content.as_mut() {
            tx.ui(ui, wallet, modal, cb);
            return;
        }

        // Setup callback on continue.
        let on_continue = |m: &mut MessageRequestModal| {
            if m.amount_edit.is_empty() {
                return;
            }
            if let Ok(a) = amount_from_hr_string(m.amount_edit.as_str()) {
                modal.disable_closing();
                // Setup data for request.
                let wallet = wallet.clone();
                let invoice = m.invoice.clone();
                let result = m.request_result.clone();
                // Send request at another thread.
                m.request_loading = true;
                thread::spawn(move || {
                    let res = if invoice {
                        wallet.issue_invoice(a)
                    } else {
                        wallet.send(a, None)
                    };
                    let mut w_result = result.write();
                    *w_result = Some(res);
                });
            } else {
                let err = if m.invoice {
                    t!("wallets.invoice_slatepack_err")
                } else {
                    t!("wallets.send_slatepack_err")
                };
                m.request_error = Some(err);
            }
        };

        ui.add_space(6.0);

        // Draw content on request loading.
        if self.request_loading {
            self.loading_request_ui(ui, modal);
            return;
        }

        // Draw amount input content.
        ui.vertical_centered(|ui| {
            let enter_text = if self.invoice {
                t!("wallets.enter_amount_receive")
            } else {
                let data = wallet.get_data().unwrap();
                let amount = amount_to_hr_string(data.info.amount_currently_spendable, true);
                t!("wallets.enter_amount_send","amount" => amount)
            };
            ui.label(RichText::new(enter_text)
                .size(17.0)
                .color(Colors::gray()));
        });
        ui.add_space(8.0);

        // Draw request amount text input.
        let amount_edit_before = self.amount_edit.clone();
        let mut amount_edit = TextEdit::new(Id::from(modal.id).with(wallet.get_config().id))
            .h_center()
            .numeric();
        amount_edit.ui(ui, &mut self.amount_edit, cb);
        if amount_edit.enter_pressed {
            on_continue(self);
        }

        // Check value if input was changed.
        if amount_edit_before != self.amount_edit {
            self.request_error = None;
            if !self.amount_edit.is_empty() {
                self.amount_edit = self.amount_edit.trim().replace(",", ".");
                match amount_from_hr_string(self.amount_edit.as_str()) {
                    Ok(a) => {
                        if !self.amount_edit.contains(".") {
                            // To avoid input of several "0".
                            if a == 0 {
                                self.amount_edit = "0".to_string();
                                return;
                            }
                        } else {
                            // Check input after ".".
                            let parts = self.amount_edit
                                .split(".")
                                .collect::<Vec<&str>>();
                            if parts.len() == 2 && parts[1].len() > 9 {
                                self.amount_edit = amount_edit_before;
                                return;
                            }
                        }

                        // Do not input amount more than balance in sending.
                        if !self.invoice {
                            let b = wallet.get_data().unwrap().info.amount_currently_spendable;
                            if b < a {
                                self.amount_edit = amount_edit_before;
                            }
                        }
                    }
                    Err(_) => {
                        self.amount_edit = amount_edit_before;
                    }
                }
            }
        }

        // Show request creation error.
        if let Some(err) = &self.request_error {
            ui.add_space(12.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(err)
                    .size(17.0)
                    .color(Colors::red()));
            });
        }

        ui.add_space(12.0);

        // Setup spacing between buttons.
        ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

        ui.columns(2, |columns| {
            columns[0].vertical_centered_justified(|ui| {
                View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                    self.amount_edit = "".to_string();
                    self.request_error = None;
                    Modal::close();
                });
            });
            columns[1].vertical_centered_justified(|ui| {
                // Button to create Slatepack message request.
                View::button(ui, t!("continue"), Colors::white_or_black(false), || {
                    on_continue(self);
                });
            });
        });
        ui.add_space(6.0);
    }

    /// Draw loading request content.
    fn loading_request_ui(&mut self, ui: &mut egui::Ui, modal: &Modal) {
        ui.add_space(34.0);
        ui.vertical_centered(|ui| {
            View::big_loading_spinner(ui);
        });
        ui.add_space(50.0);

        // Check if there is request result error.
        if self.request_error.is_some() {
            modal.enable_closing();
            self.request_loading = false;
            return;
        }

        // Update data on request result.
        let r_request = self.request_result.read();
        if r_request.is_some() {
            modal.enable_closing();
            let result = r_request.as_ref().unwrap();
            match result {
                Ok(tx) => {
                    self.result_tx_content =
                        Some(WalletTransactionModal::new(Some(tx.data.id), false));
                }
                Err(err) => {
                    match err {
                        Error::NotEnoughFunds { .. } => {
                            let m = t!(
                                    "wallets.pay_balance_error",
                                    "amount" => self.amount_edit
                                );
                            self.request_error = Some(m);
                        }
                        _ => {
                            let m = if self.invoice {
                                t!("wallets.invoice_slatepack_err")
                            } else {
                                t!("wallets.send_slatepack_err")
                            };
                            self.request_error = Some(m);
                        }
                    }
                    self.request_loading = false;
                }
            }
        }
    }
}