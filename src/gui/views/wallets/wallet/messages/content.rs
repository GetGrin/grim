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
use egui::{Id, Margin, RichText, ScrollArea};
use egui::scroll_area::ScrollBarVisibility;
use grin_core::core::amount_to_hr_string;
use grin_wallet_libwallet::{Error, Slate, SlateState};
use parking_lot::RwLock;

use crate::gui::Colors;
use crate::gui::icons::{BROOM, CLIPBOARD_TEXT, DOWNLOAD_SIMPLE, SCAN, UPLOAD_SIMPLE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{FilePickButton, Modal, Content, View, CameraContent};
use crate::gui::views::types::{ModalPosition, QrScanResult};
use crate::gui::views::wallets::wallet::messages::request::MessageRequestModal;
use crate::gui::views::wallets::wallet::types::{SLATEPACK_MESSAGE_HINT, WalletTab, WalletTabType};
use crate::gui::views::wallets::wallet::{WalletContent, WalletTransactionModal};
use crate::wallet::types::WalletTransaction;
use crate::wallet::Wallet;

/// Slatepack messages interaction tab content.
pub struct WalletMessages {
    /// Flag to check if it's first content draw.
    first_draw: bool,

    /// Slatepacks message input text.
    message_edit: String,
    /// Flag to check if message request is loading.
    message_loading: bool,
    /// Error on finalization, parse or response creation.
    message_error: String,
    /// Parsed message result.
    message_result: Arc<RwLock<Option<(Slate, Result<WalletTransaction, Error>)>>>,

    /// Wallet transaction [`Modal`] content.
    tx_info_content: Option<WalletTransactionModal>,

    /// Invoice or sending request creation [`Modal`] content.
    request_modal_content: Option<MessageRequestModal>,

    /// Camera content for Slatepack message QR code scanning [`Modal`].
    message_camera_content: CameraContent,
    /// Flag to check if there is an error on scanning Slatepack message QR code at [`Modal`].
    message_scan_error: bool,

    /// Button to parse picked file content.
    file_pick_button: FilePickButton,
}

/// Identifier for amount input [`Modal`] to create invoice or sending request.
const REQUEST_MODAL: &'static str = "messages_request";

/// Identifier for [`Modal`] modal to show transaction information.
const TX_INFO_MODAL: &'static str = "messages_tx_info";

/// Identifier for [`Modal`] to scan Slatepack message from QR code.
const SCAN_QR_MESSAGE_MODAL: &'static str = "qr_slatepack_message_scan_modal";

impl WalletTab for WalletMessages {
    fn get_type(&self) -> WalletTabType {
        WalletTabType::Messages
    }

    fn ui(&mut self, ui: &mut egui::Ui, wallet: &mut Wallet, cb: &dyn PlatformCallbacks) {
        if WalletContent::sync_ui(ui, wallet) {
            return;
        }

        // Show modal content for this ui container.
        self.modal_content_ui(ui, wallet, cb);

        egui::CentralPanel::default()
            .frame(egui::Frame {
                stroke: View::item_stroke(),
                fill: Colors::white_or_black(false),
                inner_margin: Margin {
                    left: View::far_left_inset_margin(ui) + 4.0,
                    right: View::get_right_inset() + 4.0,
                    top: 3.0,
                    bottom: 4.0,
                },
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                ScrollArea::vertical()
                    .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                    .id_source(Id::from("wallet_messages").with(wallet.get_config().id))
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            View::max_width_ui(ui, Content::SIDE_PANEL_WIDTH * 1.3, |ui| {
                                self.ui(ui, wallet, cb);
                            });
                        });
                    });
            });
    }
}

impl WalletMessages {
    /// Create new content instance, put message into input if provided.
    pub fn new(message: Option<String>) -> Self {
        Self {
            first_draw: true,
            message_edit: message.unwrap_or("".to_string()),
            message_loading: false,
            message_error: "".to_string(),
            message_result: Arc::new(Default::default()),
            tx_info_content: None,
            request_modal_content: None,
            message_camera_content: Default::default(),
            message_scan_error: false,
            file_pick_button: FilePickButton::default(),
        }
    }

    /// Draw manual wallet transaction interaction content.
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              wallet: &mut Wallet,
              cb: &dyn PlatformCallbacks) {
        if self.first_draw {
            // Parse provided message on first draw.
            if !self.message_edit.is_empty() {
                self.parse_message(wallet);
            }
            self.first_draw = false;
        }

        ui.add_space(3.0);

        // Show creation of request to send or receive funds.
        self.request_ui(ui, wallet, cb);

        ui.add_space(12.0);
        View::horizontal_line(ui, Colors::item_stroke());
        ui.add_space(6.0);

        // Show Slatepack message input field.
        self.input_slatepack_ui(ui, wallet, cb);

        ui.add_space(6.0);
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
                    REQUEST_MODAL => {
                        if let Some(content) = self.request_modal_content.as_mut() {
                            Modal::ui(ui.ctx(), |ui, modal| {
                                content.ui(ui, wallet, modal, cb);
                            });
                        }
                    }
                    TX_INFO_MODAL => {
                        if let Some(content) = self.tx_info_content.as_mut() {
                            Modal::ui(ui.ctx(), |ui, modal| {
                                content.ui(ui, wallet, modal, cb);
                            });
                        }
                    }
                    SCAN_QR_MESSAGE_MODAL => {
                        Modal::ui(ui.ctx(), |ui, modal| {
                            self.qr_message_scan_modal_ui(ui, modal, wallet, cb);
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    /// Draw creation of request to send or receive funds.
    fn request_ui(&mut self,
                  ui: &mut egui::Ui,
                  wallet: &Wallet,
                  cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("wallets.create_request_desc"))
            .size(16.0)
            .color(Colors::inactive_text()));
        ui.add_space(7.0);

        // Show send button only if balance is not empty.
        let data = wallet.get_data().unwrap();
        if data.info.amount_currently_spendable > 0 {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    let send_text = format!("{} {}", UPLOAD_SIMPLE, t!("wallets.send"));
                    View::colored_text_button(ui, send_text, Colors::red(), Colors::button(), || {
                        self.show_request_modal(false, cb);
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    self.receive_button_ui(ui, cb);
                });
            });
        } else {
            self.receive_button_ui(ui, cb);
        }
    }

    /// Draw invoice request creation button.
    fn receive_button_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        let receive_text = format!("{} {}", DOWNLOAD_SIMPLE, t!("wallets.receive"));
        View::colored_text_button(ui, receive_text, Colors::green(), Colors::button(), || {
            self.show_request_modal(true, cb);
        });
    }

    /// Show [`Modal`] to create invoice or sending request.
    fn show_request_modal(&mut self, invoice: bool, cb: &dyn PlatformCallbacks) {
        self.request_modal_content = Some(MessageRequestModal::new(invoice));
        let title = if invoice {
            t!("wallets.receive")
        } else {
            t!("wallets.send")
        };
        Modal::new(REQUEST_MODAL).position(ModalPosition::CenterTop).title(title).show();
        cb.show_keyboard();
    }

    /// Draw Slatepack message input content.
    fn input_slatepack_ui(&mut self,
                          ui: &mut egui::Ui,
                          wallet: &mut Wallet,
                          cb: &dyn PlatformCallbacks) {
        // Setup description text.
        if !self.message_error.is_empty() {
            ui.label(RichText::new(&self.message_error).size(16.0).color(Colors::red()));
        } else {
            ui.label(RichText::new(t!("wallets.input_slatepack_desc"))
                .size(16.0)
                .color(Colors::inactive_text()));
        }
        ui.add_space(6.0);

        View::horizontal_line(ui, Colors::item_stroke());
        ui.add_space(3.0);

        // Save message to check for changes.
        let message_before = self.message_edit.clone();

        let scroll_id = Id::from("message_input_scroll").with(wallet.get_config().id);
        ScrollArea::vertical()
            .id_source(scroll_id)
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .max_height(128.0)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(7.0);
                let input_id = scroll_id.with("_input");
                let resp = egui::TextEdit::multiline(&mut self.message_edit)
                    .id(input_id)
                    .font(egui::TextStyle::Small)
                    .desired_rows(5)
                    .interactive(!self.message_loading)
                    .hint_text(SLATEPACK_MESSAGE_HINT)
                    .desired_width(f32::INFINITY)
                    .show(ui)
                    .response;
                // Show soft keyboard on click.
                if resp.clicked() {
                    resp.request_focus();
                    cb.show_keyboard();
                }
                if resp.has_focus() {
                    // Apply text from input on Android as temporary fix for egui.
                    View::on_soft_input(ui, input_id, &mut self.message_edit);
                }
                ui.add_space(6.0);
            });
        ui.add_space(2.0);
        View::horizontal_line(ui, Colors::item_stroke());
        ui.add_space(10.0);

        // Parse message if input field was changed.
        if message_before != self.message_edit {
            self.parse_message(wallet);
        }

        if self.message_loading {
            View::small_loading_spinner(ui);
            // Check loading result.
            let has_tx = {
                let r_res = self.message_result.read();
                r_res.is_some()
            };
            if has_tx {
                let mut w_res = self.message_result.write();
                let tx_res = w_res.as_ref().unwrap();
                let slate = &tx_res.0;
                match &tx_res.1 {
                    Ok(tx) => {
                        self.message_edit.clear();
                        // Show transaction modal on success.
                        self.tx_info_content = Some(WalletTransactionModal::new(wallet, tx, false));
                        Modal::new(TX_INFO_MODAL)
                            .position(ModalPosition::CenterTop)
                            .title(t!("wallets.tx"))
                            .show();
                        *w_res = None;
                    }
                    Err(err) => {
                        match err {
                            // Set already canceled transaction error message.
                            Error::TransactionWasCancelled {..} => {
                                self.message_error = t!("wallets.resp_canceled_err");
                            }
                            // Set an error when there is not enough funds to pay.
                            Error::NotEnoughFunds {..} => {
                                let m = t!(
                                    "wallets.pay_balance_error",
                                    "amount" => amount_to_hr_string(slate.amount, true)
                                );
                                self.message_error = m;
                            }
                            // Set default error message.
                            _ => {
                                let finalize = slate.state == SlateState::Standard2 ||
                                    slate.state == SlateState::Invoice2;
                                self.message_error = if finalize {
                                    t!("wallets.finalize_slatepack_err")
                                } else {
                                    t!("wallets.resp_slatepack_err")
                                };
                            }
                        }
                    }
                }
                self.message_loading = false;
            }
            return;
        }

        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    // Draw button to scan Slatepack message QR code.
                    let scan_text = format!("{} {}", SCAN, t!("scan"));
                    View::button(ui, scan_text, Colors::button(), || {
                        self.message_edit.clear();
                        self.message_error.clear();
                        self.show_qr_message_scan_modal(cb);
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    // Draw button to paste text from clipboard.
                    let paste = format!("{} {}", CLIPBOARD_TEXT, t!("paste"));
                    View::button(ui, paste, Colors::button(), || {
                        let buf = cb.get_string_from_buffer();
                        let previous = self.message_edit.clone();
                        self.message_edit = buf.clone().trim().to_string();
                        // Parse Slatepack message resetting message error.
                        if buf != previous {
                            self.parse_message(wallet);
                            self.parse_message(wallet);
                        }
                    });
                });
            });
            ui.add_space(10.0);
        });

        if self.message_edit.is_empty() {
            // Draw button to choose file.
            let mut parsed_text = "".to_string();
            self.file_pick_button.ui(ui, cb, |text| {
                parsed_text = text;
            });
            self.message_edit = parsed_text;
            self.parse_message(wallet);
        } else {
            // Draw button to clear message input.
            let clear_text = format!("{} {}", BROOM, t!("clear"));
            View::button(ui, clear_text, Colors::button(), || {
                self.message_edit.clear();
                self.message_error.clear();
            });
        }
    }

    /// Parse message input making operation based on incoming status.
    pub fn parse_message(&mut self, wallet: &Wallet) {
        self.message_error.clear();
        self.message_edit = self.message_edit.trim().to_string();
        if self.message_edit.is_empty() {
            return;
        }
        if let Ok(mut slate) = wallet.parse_slatepack(&self.message_edit) {
            // Try to setup empty amount from transaction by id.
            if slate.amount == 0 {
                let _ = wallet.get_data().unwrap().txs.as_ref().unwrap().iter().map(|tx| {
                    if tx.data.tx_slate_id == Some(slate.id) {
                        if slate.amount == 0 {
                            slate.amount = tx.amount;
                        }
                    }
                    tx
                }).collect::<Vec<&WalletTransaction>>();
            }

            // Check if message with same id and state already exists to show tx modal.
            let exists = wallet.read_slatepack(&slate).is_some();
            if exists {
                if let Some(tx) = wallet.tx_by_slate(&slate).as_ref() {
                    self.message_edit.clear();
                    self.tx_info_content = Some(WalletTransactionModal::new(wallet, tx, false));
                    Modal::new(TX_INFO_MODAL)
                        .position(ModalPosition::CenterTop)
                        .title(t!("wallets.tx"))
                        .show();
                } else {
                    self.message_error = t!("wallets.parse_slatepack_err");
                }
                return;
            }

            // Create response or finalize at separate thread.
            let sl = slate.clone();
            let message = self.message_edit.clone();
            let message_result = self.message_result.clone();
            let wallet = wallet.clone();

            self.message_loading = true;
            thread::spawn(move || {
                let result = match slate.state {
                    SlateState::Standard1 | SlateState::Invoice1 => {
                        if sl.state != SlateState::Standard1 {
                            wallet.pay(&message)
                        } else {
                            wallet.receive(&message)
                        }
                    }
                    SlateState::Standard2 | SlateState::Invoice2 => {
                        wallet.finalize(&message)
                    }
                    _ => {
                        if let Some(tx) = wallet.tx_by_slate(&slate) {
                            Ok(tx)
                        } else {
                            Err(Error::GenericError(t!("wallets.parse_slatepack_err")))
                        }
                    }
                };
                let mut w_res = message_result.write();
                *w_res = Some((slate, result));
            });
        } else {
            self.message_error = t!("wallets.parse_slatepack_err");
        }
    }

    /// Show QR code Slatepack message scanner [`Modal`].
    pub fn show_qr_message_scan_modal(&mut self, cb: &dyn PlatformCallbacks) {
        self.message_scan_error = false;
        // Show QR code scan modal.
        Modal::new(SCAN_QR_MESSAGE_MODAL)
            .position(ModalPosition::CenterTop)
            .title(t!("scan_qr"))
            .closeable(false)
            .show();
        cb.start_camera();
    }

    /// Draw QR code scanner [`Modal`] content.
    fn qr_message_scan_modal_ui(&mut self,
                                ui: &mut egui::Ui,
                                modal: &Modal,
                                wallet: &Wallet,
                                cb: &dyn PlatformCallbacks) {
        if self.message_scan_error {
            ui.add_space(6.0);
            ui.vertical_centered(|ui| {
                let err_text = format!("{}", t!("wallets.parse_slatepack_err")).replace(":", ".");
                ui.label(RichText::new(err_text)
                    .size(17.0)
                    .color(Colors::red()));
            });
            ui.add_space(12.0);

            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("close"), Colors::white_or_black(false), || {
                        self.message_scan_error = false;
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("repeat"), Colors::white_or_black(false), || {
                        Modal::set_title(t!("scan_qr"));
                        self.message_scan_error = false;
                        cb.start_camera();
                    });
                });
            });
            ui.add_space(6.0);
            return;
        } else if let Some(result) = self.message_camera_content.qr_scan_result() {
            cb.stop_camera();
            self.message_camera_content.clear_state();
            match &result {
                QrScanResult::Slatepack(text) => {
                    self.message_edit = text.to_string();
                    self.parse_message(wallet);
                    modal.close();
                }
                _ => {
                    self.message_scan_error = true;
                }
            }
        } else {
            ui.add_space(6.0);
            self.message_camera_content.ui(ui, cb);
            ui.add_space(8.0);
        }

        ui.vertical_centered_justified(|ui| {
            View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                cb.stop_camera();
                modal.close();
            });
        });
        ui.add_space(6.0);
    }
}