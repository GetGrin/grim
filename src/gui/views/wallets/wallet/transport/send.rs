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
use egui::{Id, RichText};
use grin_core::core::{amount_from_hr_string, amount_to_hr_string};
use grin_wallet_libwallet::{Error, SlatepackAddress};
use parking_lot::RwLock;

use crate::gui::Colors;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{CameraContent, Modal, View};
use crate::gui::views::types::TextEditOptions;
use crate::gui::views::wallets::wallet::WalletTransactionModal;
use crate::wallet::types::WalletTransaction;
use crate::wallet::Wallet;

/// Transport sending [`Modal`] content.
pub struct TransportSendModal {
    /// Flag to focus on first input field after opening.
    first_draw: bool,

    /// Flag to check if transaction is sending to show progress.
    sending: bool,
    /// Flag to check if there is an error to repeat.
    error: bool,
    /// Transaction result.
    send_result: Arc<RwLock<Option<Result<WalletTransaction, Error>>>>,

    /// Entered amount value.
    amount_edit: String,
    /// Entered address value.
    address_edit: String,
    /// Flag to check if entered address is incorrect.
    address_error: bool,

    /// Address QR code scanner content.
    address_scan_content: Option<CameraContent>,

    /// Transaction information content.
    tx_info_content: Option<WalletTransactionModal>,
}

impl TransportSendModal {
    /// Create new instance from provided address.
    pub fn new(addr: Option<String>) -> Self {
        Self {
            first_draw: true,
            sending: false,
            error: false,
            send_result: Arc::new(RwLock::new(None)),
            amount_edit: "".to_string(),
            address_edit: addr.unwrap_or("".to_string()),
            address_error: false,
            address_scan_content: None,
            tx_info_content: None,
        }
    }

    /// Draw [`Modal`] content.
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              wallet: &Wallet,
              modal: &Modal,
              cb: &dyn PlatformCallbacks) {
        // Draw transaction information on request result.
        if let Some(tx) = self.tx_info_content.as_mut() {
            tx.ui(ui, wallet, modal, cb);
            return;
        }

        // Draw sending content, progress or an error.
        if self.sending {
            self.progress_ui(ui, wallet);
        } else if self.error {
            self.error_ui(ui, wallet, modal, cb);
        } else {
            self.content_ui(ui, wallet, modal, cb);
        }
    }

    /// Draw content to send.
    fn content_ui(&mut self, ui: &mut egui::Ui, wallet: &Wallet, modal: &Modal,
                  cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        // Draw QR code scanner content if requested.
        if let Some(scanner) = self.address_scan_content.as_mut() {
            let mut on_stop = || {
                self.first_draw = true;
                cb.stop_camera();
                modal.enable_closing();
            };

            if let Some(result) = scanner.qr_scan_result() {
                self.address_edit = result.text();
                on_stop();
                self.address_scan_content = None;
                cb.show_keyboard();
            } else {
                scanner.ui(ui, cb);
                ui.add_space(6.0);

                // Setup spacing between buttons.
                ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

                // Show buttons to close modal or come back to sending input.
                ui.columns(2, |cols| {
                    cols[0].vertical_centered_justified(|ui| {
                        View::button(ui, t!("close"), Colors::white_or_black(false), || {
                            on_stop();
                            self.address_scan_content = None;
                            modal.close();
                        });
                    });
                    cols[1].vertical_centered_justified(|ui| {
                        View::button(ui, t!("back"), Colors::white_or_black(false), || {
                            on_stop();
                            self.address_scan_content = None;
                            cb.show_keyboard();
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
        let mut amount_edit_opts = TextEditOptions::new(amount_edit_id).h_center().no_focus();
        let amount_edit_before = self.amount_edit.clone();
        if self.first_draw {
            self.first_draw = false;
            amount_edit_opts.focus = true;
        }
        View::text_edit(ui, cb, &mut self.amount_edit, &mut amount_edit_opts);
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
                            // Check input after ".".
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

        // Draw address text edit.
        let addr_edit_before = self.address_edit.clone();
        let address_edit_id = Id::from(modal.id).with("_address").with(wallet.get_config().id);
        let mut address_edit_opts = TextEditOptions::new(address_edit_id)
            .paste()
            .no_focus()
            .scan_qr();
        View::text_edit(ui, cb, &mut self.address_edit, &mut address_edit_opts);
        // Check if scan button was pressed.
        if address_edit_opts.scan_pressed {
            cb.hide_keyboard();
            modal.disable_closing();
            address_edit_opts.scan_pressed = false;
            self.address_scan_content = Some(CameraContent::default());
        }
        ui.add_space(12.0);

        // Check value if input was changed.
        if addr_edit_before != self.address_edit {
            self.address_error = false;
        }

        // Setup spacing between buttons.
        ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

        ui.columns(2, |columns| {
            columns[0].vertical_centered_justified(|ui| {
                View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                    self.close(modal, cb);
                });
            });
            columns[1].vertical_centered_justified(|ui| {
                View::button(ui, t!("continue"), Colors::white_or_black(false), || {
                    self.send(wallet, modal, cb);
                });
            });
        });
        ui.add_space(6.0);
    }

    /// Draw error content.
    fn error_ui(&mut self,
                ui: &mut egui::Ui,
                wallet: &Wallet,
                modal: &Modal,
                cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("transport.tor_send_error"))
                .size(17.0)
                .color(Colors::red()));
        });
        ui.add_space(12.0);

        // Setup spacing between buttons.
        ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

        ui.columns(2, |columns| {
            columns[0].vertical_centered_justified(|ui| {
                View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                   self.close(modal, cb);
                });
            });
            columns[1].vertical_centered_justified(|ui| {
                View::button(ui, t!("repeat"), Colors::white_or_black(false), || {
                    self.send(wallet, modal, cb);
                });
            });
        });
        ui.add_space(6.0);
    }

    /// Close modal and clear data.
    fn close(&mut self, modal: &Modal, cb: &dyn PlatformCallbacks) {
        self.amount_edit = "".to_string();
        self.address_edit = "".to_string();

        let mut w_res = self.send_result.write();
        *w_res = None;

        self.tx_info_content = None;
        self.address_scan_content = None;

        cb.hide_keyboard();
        modal.close();
    }

    /// Send entered amount to address.
    fn send(&mut self, wallet: &Wallet, modal: &Modal, cb: &dyn PlatformCallbacks) {
        if self.amount_edit.is_empty() {
            return;
        }
        let addr_str = self.address_edit.as_str();
        if let Ok(addr) = SlatepackAddress::try_from(addr_str) {
            if let Ok(a) = amount_from_hr_string(self.amount_edit.as_str()) {
                cb.hide_keyboard();
                modal.disable_closing();
                // Send amount over Tor.
                let mut wallet = wallet.clone();
                let res = self.send_result.clone();
                self.sending = true;
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
        } else {
            self.address_error = true;
        }
    }

    /// Draw sending progress content.
    fn progress_ui(&mut self, ui: &mut egui::Ui, wallet: &Wallet) {
        ui.add_space(16.0);
        ui.vertical_centered(|ui| {
            View::small_loading_spinner(ui);
            ui.add_space(12.0);
            ui.label(RichText::new(t!("transport.tor_sending", "amount" => self.amount_edit))
                .size(17.0)
                .color(Colors::gray()));
        });
        ui.add_space(10.0);

        // Check sending result.
        let has_result = {
            let r_result = self.send_result.read();
            r_result.is_some()
        };
        if has_result {
            {
                let res = self.send_result.read().clone().unwrap();
                match res {
                    Ok(tx) => {
                        self.tx_info_content =
                            Some(WalletTransactionModal::new(wallet, &tx, false));
                    }
                    Err(_) => {
                        self.error = true;
                    }
                }
            }
            let mut w_res = self.send_result.write();
            *w_res = None;
            self.sending = false;
        }
    }
}