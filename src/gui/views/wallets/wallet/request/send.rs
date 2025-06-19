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
use grin_core::core::{amount_from_hr_string, amount_to_hr_string};
use grin_wallet_libwallet::SlatepackAddress;

use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{CameraContent, Modal, TextEdit, View};
use crate::gui::Colors;
use crate::wallet::types::WalletTask;
use crate::wallet::Wallet;

/// Content to create a request to send funds.
pub struct SendRequestContent {
    /// Amount to send or receive.
    amount_edit: String,
    /// Receiver address.
    address_edit: String,
    /// Flag to check if entered address is incorrect.
    address_error: bool,

    /// Address QR code scanner content.
    address_scan_content: Option<CameraContent>,
}

impl SendRequestContent {
    /// Create new content instance with optional receiver address.
    pub fn new(addr: Option<String>) -> Self {
        Self {
            amount_edit: "".to_string(),
            address_edit: addr.unwrap_or("".to_string()),
            address_error: false,
            address_scan_content: None,
        }
    }

    /// Draw [`Modal`] content.
    pub fn modal_ui(&mut self,
                    ui: &mut egui::Ui,
                    wallet: &Wallet,
                    modal: &Modal,
                    cb: &dyn PlatformCallbacks) {
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
                            on_stop();
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
            self.on_continue(wallet);
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
                    self.on_continue(wallet);
                });
            });
        });
        ui.add_space(6.0);
    }

    /// Callback when Continue button was pressed.
    fn on_continue(&mut self, wallet: &Wallet) {
        if self.amount_edit.is_empty() {
            return;
        }
        // Check address to send over Tor if enabled.
        let addr_str = self.address_edit.as_str();
        if let Ok(r) = SlatepackAddress::try_from(addr_str.trim()) {
            if let Ok(a) = amount_from_hr_string(self.amount_edit.as_str()) {
                wallet.task(WalletTask::Send(a, Some(r)));
                Modal::close();
            }
        } else if !addr_str.is_empty() {
            self.address_error = true;
        } else if let Ok(a) = amount_from_hr_string(self.amount_edit.as_str()) {
            wallet.task(WalletTask::Send(a, None));
            Modal::close();
        }
    }

    /// Close modal and clear data.
    fn close(&mut self) {
        self.amount_edit = "".to_string();
        self.address_edit = "".to_string();
        self.address_scan_content = None;
        Modal::close();
    }
}