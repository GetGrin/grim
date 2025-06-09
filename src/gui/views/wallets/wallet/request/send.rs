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

use grin_core::core::{amount_from_hr_string, amount_to_hr_string};
use grin_wallet_libwallet::{Error, SlatepackAddress};
use parking_lot::RwLock;
use std::sync::Arc;
use std::thread;
use egui::{Id, RichText};

use crate::gui::Colors;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::wallets::wallet::WalletTransactionContent;
use crate::gui::views::{CameraContent, Modal, TextEdit, View};
use crate::gui::views::types::ModalPosition;
use crate::wallet::types::WalletTransaction;
use crate::wallet::Wallet;

/// Content to create a request to send funds.
pub struct SendRequestContent {
    /// Amount to send or receive.
    amount_edit: String,
    /// Receiver address.
    address_edit: String,
    /// Flag to check if entered address is incorrect.
    address_error: bool,

    /// Flag to check if request is loading.
    request_loading: bool,
    /// Request result if there is no error.
    request_result: Arc<RwLock<Option<Result<WalletTransaction, Error>>>>,
    /// Flag to check if there is an error happened on request creation.
    request_error: Option<String>,

    /// Address QR code scanner content.
    address_scan_content: Option<CameraContent>,

    /// Request result transaction content.
    result_tx_content: Option<WalletTransactionContent>,
}

impl SendRequestContent {
    /// Create new content instance with optional receiver address.
    pub fn new(addr: Option<String>) -> Self {
        Self {
            amount_edit: "".to_string(),
            address_edit: addr.unwrap_or("".to_string()),
            address_error: false,
            request_loading: false,
            request_result: Arc::new(RwLock::new(None)),
            request_error: None,
            address_scan_content: None,
            result_tx_content: None,
        }
    }

    /// Draw [`Modal`] content.
    pub fn modal_ui(&mut self,
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
        let on_continue = |m: &mut SendRequestContent| {
            if m.amount_edit.is_empty() {
                return;
            }
            // Check address to send over Tor if enabled.
            let addr_str = m.address_edit.as_str();
            if let Ok(addr) = SlatepackAddress::try_from(addr_str.trim()) {
                if let Ok(a) = amount_from_hr_string(m.amount_edit.as_str()) {
                    Modal::change_position(ModalPosition::Center);
                    modal.disable_closing();

                    let mut wallet = wallet.clone();
                    let res = m.request_result.clone();

                    // Send request at another thread.
                    m.request_loading = true;
                    thread::spawn(move || {
                        tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .unwrap()
                            .block_on(async {
                                let result = wallet.send_tor(a, &addr).await;
                                let mut w_res = res.write();
                                *w_res = Some(result);
                            });
                    });
                }
            } else if !addr_str.is_empty() {
                m.address_error = true;
            } else if let Ok(amount) = amount_from_hr_string(m.amount_edit.as_str()) {
                Modal::change_position(ModalPosition::Center);
                modal.disable_closing();

                let wallet = wallet.clone();
                let result = m.request_result.clone();

                // Send request at another thread.
                m.request_loading = true;
                thread::spawn(move || {
                    let res = wallet.send(amount, None);
                    let mut w_result = result.write();
                    *w_result = Some(res);
                });
            } else {
                m.request_error = Some(t!("wallets.send_slatepack_err"));
            }
        };

        // Draw content on request loading.
        if self.request_loading {
            self.loading_request_ui(ui, modal);
            return;
        }

        ui.add_space(6.0);

        // Draw QR code scanner content if requested.
        if let Some(scanner) = self.address_scan_content.as_mut() {
            let on_stop = || {
                cb.stop_camera();
                modal.enable_closing();
            };

            if let Some(result) = scanner.qr_scan_result() {
                self.address_edit = result.text();
                on_stop();
                self.address_scan_content = None;
            } else {
                ui.add_space(6.0);
                scanner.ui(ui, cb);
                ui.add_space(6.0);

                // Setup spacing between buttons.
                ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

                // Show buttons to close modal or come back to sending input.
                ui.columns(2, |cols| {
                    cols[0].vertical_centered_justified(|ui| {
                        View::button(ui, t!("close"), Colors::white_or_black(false), || {
                            self.close();
                        });
                    });
                    cols[1].vertical_centered_justified(|ui| {
                        View::button(ui, t!("back"), Colors::white_or_black(false), || {
                            on_stop();
                            self.address_scan_content = None;
                        });
                    });
                });
                ui.add_space(6.0);
            }
            return;
        }

        ui.vertical_centered(|ui| {
            let data = wallet.get_data().unwrap();
            let amount = amount_to_hr_string(data.info.amount_currently_spendable, true);
            let enter_text = t!("wallets.enter_amount_send","amount" => amount);
            ui.label(RichText::new(enter_text)
                .size(17.0)
                .color(Colors::gray()));
        });
        ui.add_space(8.0);

        // Draw amount text edit.
        let amount_edit_id = Id::from(modal.id).with("amount").with(wallet.get_config().id);
        let mut amount_edit = TextEdit::new(amount_edit_id)
            .h_center()
            .numeric()
            .focus(Modal::first_draw());
        let amount_edit_before = self.amount_edit.clone();
        amount_edit.ui(ui, &mut self.amount_edit, cb);
        ui.add_space(8.0);

        // Check value if input was changed.
        if amount_edit_before != self.amount_edit {
            if !self.amount_edit.is_empty() {
                // Trim text, replace "," by "." and parse amount.
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
                            // Check input after `.`.
                            let parts = self.amount_edit.split(".").collect::<Vec<&str>>();
                            if parts.len() == 2 && parts[1].len() > 9 {
                                self.amount_edit = amount_edit_before;
                                return;
                            }
                        }

                        // Do not input amount more than balance in sending.
                        let b = wallet.get_data().unwrap().info.amount_currently_spendable;
                        if b < a {
                            self.amount_edit = amount_edit_before;
                        }
                    }
                    Err(_) => {
                        self.amount_edit = amount_edit_before;
                    }
                }
            }
        }

        // Show address error or input description.
        ui.vertical_centered(|ui| {
            if self.address_error {
                ui.label(RichText::new(t!("transport.incorrect_addr_err"))
                    .size(17.0)
                    .color(Colors::red()));
            } else {
                ui.label(RichText::new(t!("transport.receiver_address"))
                    .size(17.0)
                    .color(Colors::gray()));
            }
        });
        ui.add_space(6.0);

        // Show address text edit.
        let addr_edit_before = self.address_edit.clone();
        let address_edit_id = Id::from(modal.id).with("_address").with(wallet.get_config().id);
        let mut address_edit = TextEdit::new(address_edit_id)
            .paste()
            .focus(false)
            .scan_qr();
        if amount_edit.enter_pressed {
            address_edit.focus_request();
        }
        address_edit.ui(ui, &mut self.address_edit, cb);
        // Check if scan button was pressed.
        if address_edit.scan_pressed {
            modal.disable_closing();
            self.address_scan_content = Some(CameraContent::default());
        }
        ui.add_space(12.0);

        // Check value if input was changed.
        if addr_edit_before != self.address_edit {
            self.address_error = false;
        }

        // Continue on Enter press.
        if address_edit.enter_pressed {
            on_continue(self);
        }

        // Show request creation error.
        if let Some(err) = &self.request_error {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(err)
                    .size(17.0)
                    .color(Colors::red()));
            });
            ui.add_space(12.0);
        }

        // Setup spacing between buttons.
        ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

        ui.columns(2, |columns| {
            columns[0].vertical_centered_justified(|ui| {
                View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                    self.close();
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

    /// Close modal and clear data.
    fn close(&mut self) {
        self.amount_edit = "".to_string();
        self.address_edit = "".to_string();

        let mut w_res = self.request_result.write();
        *w_res = None;

        self.result_tx_content = None;
        self.address_scan_content = None;

        Modal::close();
    }

    /// Draw loading request content.
    fn loading_request_ui(&mut self, ui: &mut egui::Ui, modal: &Modal) {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            View::big_loading_spinner(ui);
        });
        ui.add_space(40.0);

        if !self.address_edit.is_empty() {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(t!("transport.tor_sending", "amount" => self.amount_edit))
                    .size(17.0)
                    .color(Colors::inactive_text()));
            });
            ui.add_space(12.0);
        }

        // Update data on request result.
        let has_res = {
            let r_request = self.request_result.read();
            r_request.is_some()
        };
        if has_res {
            self.request_loading = false;
            modal.enable_closing();
            let r_request = self.request_result.read();
            let result = r_request.as_ref().unwrap();
            match result {
                Ok(tx) => {
                    self.result_tx_content =
                        Some(WalletTransactionContent::new(tx, false));
                }
                Err(err) => {
                    let m = match err {
                        Error::NotEnoughFunds { .. } => {
                            t!("wallets.pay_balance_error", "amount" => self.amount_edit)
                        }
                        _ => {
                            if !self.address_edit.is_empty() {
                                t!("transport.tor_send_error")
                            } else {
                                t!("wallets.send_slatepack_err")
                            }
                        }
                    };
                    self.request_error = Some(m);
                }
            }
        }

        // Check if there is request result error.
        if self.request_error.is_some() {
            Modal::change_position(ModalPosition::CenterTop);
            modal.enable_closing();
            let mut w_request = self.request_result.write();
            *w_request = None;
            self.request_loading = false;
        }
    }
}