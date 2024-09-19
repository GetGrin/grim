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
use egui::{Align, Id, Layout, RichText, Rounding, ScrollArea};
use egui::scroll_area::ScrollBarVisibility;
use grin_core::core::amount_to_hr_string;
use grin_util::ToHex;
use grin_wallet_libwallet::{Error, Slate, SlateState, TxLogEntryType};
use parking_lot::RwLock;
use crate::gui::Colors;
use crate::gui::icons::{BROOM, CHECK, CLIPBOARD_TEXT, COPY, CUBE, FILE_ARCHIVE, FILE_TEXT, HASH_STRAIGHT, PROHIBIT, QR_CODE, SCAN};
use crate::gui::platform::PlatformCallbacks;

use crate::gui::views::{CameraContent, FilePickButton, Modal, QrCodeContent, View};
use crate::gui::views::wallets::wallet::txs::WalletTransactions;
use crate::gui::views::wallets::wallet::types::SLATEPACK_MESSAGE_HINT;
use crate::wallet::types::WalletTransaction;
use crate::wallet::Wallet;

/// Transaction information [`Modal`] content.
pub struct WalletTransactionModal {
    /// Transaction identifier.
    tx_id: u32,
    /// Identifier for [`Slate`].
    slate_id: Option<String>,

    /// Response Slatepack message input value.
    response_edit: String,

    /// Flag to show transaction finalization input.
    show_finalization: bool,
    /// Finalization Slatepack message input value.
    finalize_edit: String,
    /// Flag to check if error happened during transaction finalization.
    finalize_error: bool,
    /// Flag to check if transaction is finalizing.
    finalizing: bool,
    /// Transaction finalization result.
    final_result: Arc<RwLock<Option<Result<WalletTransaction, Error>>>>,

    /// QR code Slatepack message image content.
    qr_code_content: Option<QrCodeContent>,

    /// QR code scanner content.
    qr_scan_content: Option<CameraContent>,

    /// Button to parse picked file content.
    file_pick_button: FilePickButton,
}

impl WalletTransactionModal {
    /// Create new content instance with [`Wallet`] from provided [`WalletTransaction`].
    pub fn new(wallet: &Wallet, tx: &WalletTransaction, show_finalization: bool) -> Self {
        Self {
            tx_id: tx.data.id,
            slate_id: match tx.data.tx_slate_id {
                None => None,
                Some(id) => Some(id.to_string())
            },
            response_edit: if !tx.data.confirmed && tx.data.tx_slate_id.is_some() &&
                (tx.data.tx_type == TxLogEntryType::TxSent ||
                    tx.data.tx_type == TxLogEntryType::TxReceived) {
                let mut slate = Slate::blank(1, false);
                slate.state = if tx.can_finalize {
                    if tx.data.tx_type == TxLogEntryType::TxSent {
                        SlateState::Standard1
                    } else {
                        SlateState::Invoice1
                    }
                } else {
                    if tx.data.tx_type == TxLogEntryType::TxReceived {
                        SlateState::Standard2
                    } else {
                        SlateState::Invoice2
                    }
                };
                slate.id = tx.data.tx_slate_id.unwrap();
                wallet.read_slatepack(&slate).unwrap_or("".to_string())
            } else {
                "".to_string()
            },
            finalize_edit: "".to_string(),
            finalize_error: false,
            show_finalization,
            finalizing: false,
            final_result: Arc::new(RwLock::new(None)),
            qr_code_content: None,
            qr_scan_content: None,
            file_pick_button: FilePickButton::default(),
        }
    }

    /// Draw [`Modal`] content.
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              wallet: &Wallet,
              modal: &Modal,
              cb: &dyn PlatformCallbacks) {
        // Check values and setup transaction data.
        let wallet_data = wallet.get_data();
        if wallet_data.is_none() {
            modal.close();
            return;
        }
        let data = wallet_data.unwrap();
        let data_txs = data.txs.clone().unwrap();
        let txs = data_txs.into_iter()
            .filter(|tx| tx.data.id == self.tx_id)
            .collect::<Vec<WalletTransaction>>();
        if txs.is_empty() {
            cb.hide_keyboard();
            modal.close();
            return;
        }
        let tx = txs.get(0).unwrap();

        if self.qr_code_content.is_none() && self.qr_scan_content.is_none() {
            ui.add_space(6.0);

            // Show transaction amount status and time.
            let r = View::item_rounding(0, 2, false);
            let mut rect = ui.available_rect_before_wrap();
            rect.set_height(WalletTransactions::TX_ITEM_HEIGHT);
            WalletTransactions::tx_item_ui(ui, tx, rect, r, &data, |ui| {
                // Do not show buttons on finalizing.
                if self.finalizing {
                    return;
                }

                let wallet_loaded = wallet.foreign_api_port().is_some();

                // Draw button to show transaction finalization or transaction info.
                if wallet_loaded && tx.can_finalize {
                    let (icon, color) = if self.show_finalization {
                        (FILE_TEXT, None)
                    } else {
                        (CHECK, Some(Colors::green()))
                    };
                    let mut r = r.clone();
                    r.nw = 0.0;
                    r.sw = 0.0;
                    View::item_button(ui, r, icon, color, || {
                        cb.hide_keyboard();
                        if self.show_finalization {
                            self.show_finalization = false;
                            return;
                        }
                        self.show_finalization = true;
                    });
                }

                // Draw button to cancel transaction.
                if wallet_loaded && tx.can_cancel() {
                    View::item_button(ui, Rounding::default(), PROHIBIT, Some(Colors::red()), || {
                        cb.hide_keyboard();
                        wallet.cancel(tx.data.id);
                    });
                }
            });

            // Show identifier.
            if let Some(id) = tx.data.tx_slate_id {
                let label = format!("{} {}", HASH_STRAIGHT, t!("id"));
                Self::info_item_ui(ui, id.to_string(), label, true, cb);
            }
            // Show kernel.
            if let Some(kernel) = tx.data.kernel_excess {
                let label = format!("{} {}", FILE_ARCHIVE, t!("kernel"));
                Self::info_item_ui(ui, kernel.0.to_hex(), label, true, cb);
            }
            // Show block height.
            if let Some(height) = tx.height {
                if height != 0 {
                    let label = format!("{} {}", CUBE, t!("network_node.block"));
                    Self::info_item_ui(ui, height.to_string(), label, true, cb);
                }
            }
        }

        // Show Slatepack message or reset QR code state if not available.
        if !tx.finalizing && !tx.data.confirmed && !tx.cancelling &&
            (tx.data.tx_type == TxLogEntryType::TxSent ||
                tx.data.tx_type == TxLogEntryType::TxReceived) && !self.response_edit.is_empty() {
            self.message_ui(ui, tx, wallet, modal, cb);
        } else if let Some(qr_content) = self.qr_code_content.as_mut() {
            qr_content.clear_state();
        }

        if !self.finalizing {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            if self.qr_code_content.is_some() {
                // Show buttons to close modal or come back to text request content.
                ui.columns(2, |cols| {
                    cols[0].vertical_centered_justified(|ui| {
                        View::button(ui, t!("close"), Colors::white_or_black(false), || {
                            self.qr_code_content = None;
                            modal.close();
                        });
                    });
                    cols[1].vertical_centered_justified(|ui| {
                        View::button(ui, t!("back"), Colors::white_or_black(false), || {
                            self.qr_code_content = None;
                        });
                    });
                });
            } else if self.qr_scan_content.is_some() {
                ui.add_space(8.0);
                // Show buttons to close modal or scanner.
                ui.columns(2, |cols| {
                    cols[0].vertical_centered_justified(|ui| {
                        View::button(ui, t!("close"), Colors::white_or_black(false), || {
                            cb.stop_camera();
                            self.qr_scan_content = None;
                            modal.close();
                        });
                    });
                    cols[1].vertical_centered_justified(|ui| {
                        View::button(ui, t!("back"), Colors::white_or_black(false), || {
                            cb.stop_camera();
                            self.qr_scan_content = None;
                            modal.enable_closing();
                        });
                    });
                });
            } else {
                ui.add_space(8.0);
                View::horizontal_line(ui, Colors::item_stroke());
                ui.add_space(8.0);

                // Show button to close modal.
                ui.vertical_centered_justified(|ui| {
                    View::button(ui, t!("close"), Colors::white_or_black(false), || {
                        cb.hide_keyboard();
                        modal.close();
                    });
                });
            }
            ui.add_space(6.0);
        } else {
            // Show loader on finalizing.
            ui.vertical_centered(|ui| {
                View::small_loading_spinner(ui);
                ui.add_space(16.0);
            });
            // Check finalization result.
            let has_res = {
                let r_res = self.final_result.read();
                r_res.is_some()
            };
            if has_res {
                let res = {
                    let r_res = self.final_result.read();
                    r_res.as_ref().unwrap().clone()
                };
                if let Ok(_) = res {
                    self.show_finalization = false;
                    self.finalize_edit = "".to_string();
                } else {
                    self.finalize_error = true;
                }
                // Clear status and result.
                {
                    let mut w_res = self.final_result.write();
                    *w_res = None;
                }
                self.finalizing = false;
                modal.enable_closing();
            }
        }
    }

    /// Draw transaction information item content.
    fn info_item_ui(ui: &mut egui::Ui,
                    value: String,
                    label: String,
                    copy: bool,
                    cb: &dyn PlatformCallbacks) {
        // Setup layout size.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(50.0);

        // Draw round background.
        let bg_rect = rect.clone();
        let mut rounding = View::item_rounding(1, 3, false);

        ui.painter().rect(bg_rect, rounding, Colors::fill(), View::item_stroke());

        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            // Draw button to copy transaction info value.
            if copy {
                rounding.nw = 0.0;
                rounding.sw = 0.0;
                View::item_button(ui, rounding, COPY, None, || {
                    cb.copy_string_to_buffer(value.clone());
                });
            }

            // Draw value information.
            let layout_size = ui.available_size();
            ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                ui.add_space(6.0);
                ui.vertical(|ui| {
                    ui.add_space(3.0);
                    View::ellipsize_text(ui, value, 15.0, Colors::title(false));
                    ui.label(RichText::new(label).size(15.0).color(Colors::gray()));
                    ui.add_space(3.0);
                });
            });
        });
    }

    /// Draw Slatepack message content.
    fn message_ui(&mut self,
                  ui: &mut egui::Ui,
                  tx: &WalletTransaction,
                  wallet: &Wallet,
                  modal: &Modal,
                  cb: &dyn PlatformCallbacks) {
        if self.slate_id.is_none() {
            cb.hide_keyboard();
            modal.close();
            return;
        }
        ui.add_space(6.0);

        // Draw QR code scanner content if requested.
        if let Some(qr_scan_content) = self.qr_scan_content.as_mut() {
            if let Some(result) = qr_scan_content.qr_scan_result() {
                cb.stop_camera();

                // Setup value to finalization input field.
                self.finalize_edit = result.text();
                self.on_finalization_input_change(tx, wallet, modal, cb);

                modal.enable_closing();
                self.qr_scan_content = None;
            } else {
                qr_scan_content.ui(ui, cb);
            }
            return;
        }

        let amount = amount_to_hr_string(tx.amount, true);

        // Draw Slatepack message description text.
        ui.vertical_centered(|ui| {
            if self.show_finalization {
                let desc_text = if self.finalize_error {
                    t!("wallets.finalize_slatepack_err")
                } else {
                    if tx.data.tx_type == TxLogEntryType::TxSent {
                        t!("wallets.parse_s2_slatepack_desc", "amount" => amount)
                    } else {
                        t!("wallets.parse_i2_slatepack_desc", "amount" => amount)
                    }
                };
                let desc_color = if self.finalize_error {
                    Colors::red()
                } else {
                    Colors::gray()
                };
                ui.label(RichText::new(desc_text).size(16.0).color(desc_color));
            } else {
                let desc_text = if tx.can_finalize {
                    if tx.data.tx_type == TxLogEntryType::TxSent {
                        t!("wallets.send_request_desc", "amount" => amount)
                    } else {
                        t!("wallets.invoice_desc", "amount" => amount)
                    }
                } else {
                    if tx.data.tx_type == TxLogEntryType::TxSent {
                        t!("wallets.parse_i1_slatepack_desc", "amount" => amount)
                    } else {
                        t!("wallets.parse_s1_slatepack_desc", "amount" => amount)
                    }
                };
                ui.label(RichText::new(desc_text).size(16.0).color(Colors::gray()));
            }
        });
        ui.add_space(6.0);

        // Setup message input value.
        let message_edit = if self.show_finalization {
            &mut self.finalize_edit
        }  else {
            &mut self.response_edit
        };
        let message_before = message_edit.clone();

        // Draw QR code content if requested.
        if let Some(qr_content) = self.qr_code_content.as_mut() {
            qr_content.ui(ui, cb);
            return;
        }

        // Draw Slatepack message finalization input or request text.
        ui.vertical_centered(|ui| {
            let scroll_id = if self.show_finalization {
                Id::from("tx_info_message_finalize")
            } else {
                Id::from("tx_info_message_request")
            }.with(self.slate_id.clone().unwrap()).with(tx.data.id);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(3.0);
            ScrollArea::vertical()
                .id_source(scroll_id)
                .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                .max_height(128.0)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.add_space(7.0);
                    let input_id = scroll_id.with("_input");
                    let resp = egui::TextEdit::multiline(message_edit)
                        .id(input_id)
                        .font(egui::TextStyle::Small)
                        .desired_rows(5)
                        .interactive(self.show_finalization && !self.finalizing)
                        .hint_text(SLATEPACK_MESSAGE_HINT)
                        .desired_width(f32::INFINITY)
                        .show(ui).response;
                    // Show soft keyboard on click.
                    if self.show_finalization && resp.clicked() {
                        resp.request_focus();
                        cb.show_keyboard();
                    }
                    if self.show_finalization && resp.has_focus() {
                        // Apply text from input on Android as temporary fix for egui.
                        View::on_soft_input(ui, input_id, message_edit);
                    }
                    ui.add_space(6.0);
                });
        });

        ui.add_space(2.0);
        View::horizontal_line(ui, Colors::item_stroke());
        ui.add_space(8.0);

        // Do not show buttons on finalization.
        if self.finalizing {
            return;
        }

        // Setup spacing between buttons.
        ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

        if self.show_finalization {
            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    // Draw button to scan Slatepack message QR code.
                    let qr_text = format!("{} {}", SCAN, t!("scan"));
                    View::button(ui, qr_text, Colors::button(), || {
                        cb.hide_keyboard();
                        modal.disable_closing();
                        cb.start_camera();
                        self.qr_scan_content = Some(CameraContent::default());
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    // Draw button to paste data from clipboard.
                    let paste_text = format!("{} {}", CLIPBOARD_TEXT, t!("paste"));
                    View::button(ui, paste_text, Colors::button(), || {
                        self.finalize_edit = cb.get_string_from_buffer();
                    });
                });
            });
            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                if self.finalize_error {
                    // Draw button to clear message input.
                    let clear_text = format!("{} {}", BROOM, t!("clear"));
                    View::button(ui, clear_text, Colors::button(), || {
                        self.finalize_edit.clear();
                        self.finalize_error = false;
                    });
                } else {
                    // Draw button to choose file.
                    self.file_pick_button.ui(ui, cb, |text| {
                        self.finalize_edit = text;
                    });
                }
            });

            // Callback on finalization message input change.
            if message_before != self.finalize_edit {
                self.on_finalization_input_change(tx, wallet, modal, cb);
            }
        } else {
            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    // Draw button to show Slatepack message as QR code.
                    let qr_text = format!("{} {}", QR_CODE, t!("qr_code"));
                    View::button(ui, qr_text.clone(), Colors::button(), || {
                        cb.hide_keyboard();
                        let text = self.response_edit.clone();
                        self.qr_code_content = Some(QrCodeContent::new(text, true));
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    // Draw copy button.
                    let copy_text = format!("{} {}", COPY, t!("copy"));
                    View::button(ui, copy_text, Colors::button(), || {
                        cb.copy_string_to_buffer(self.response_edit.clone());
                        self.finalize_edit = "".to_string();
                        if tx.can_finalize {
                            self.show_finalization = true;
                        } else {
                            cb.hide_keyboard();
                            modal.close();
                        }
                    });
                });
            });

            // Show button to share response as file.
            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                let share_text = format!("{} {}", FILE_TEXT, t!("share"));
                View::colored_text_button(ui,
                                          share_text,
                                          Colors::blue(),
                                          Colors::white_or_black(false), || {
                        if let Some((s, _)) = wallet.read_slate_by_tx(tx) {
                            let name = format!("{}.{}.slatepack", s.id, s.state);
                            let data = self.response_edit.as_bytes().to_vec();
                            cb.share_data(name, data).unwrap_or_default();
                        }
                    });
            });
        }
    }

    /// Parse Slatepack message on transaction finalization input change.
    fn on_finalization_input_change(&mut self,
                                    tx: &WalletTransaction,
                                    wallet: &Wallet,
                                    modal: &Modal,
                                    cb: &dyn PlatformCallbacks) {
        let message = &self.finalize_edit;
        if message.is_empty() {
            self.finalize_error = false;
        } else {
            // Parse input message to finalize.
            if let Ok(slate) = wallet.parse_slatepack(message) {
                let send = slate.state == SlateState::Standard2 &&
                    tx.data.tx_type == TxLogEntryType::TxSent;
                let receive = slate.state == SlateState::Invoice2 &&
                    tx.data.tx_type == TxLogEntryType::TxReceived;
                if Some(slate.id) == tx.data.tx_slate_id && (send || receive) {
                    let message = message.clone();
                    let wallet = wallet.clone();
                    let final_res = self.final_result.clone();
                    // Finalize transaction at separate thread.
                    cb.hide_keyboard();
                    self.finalizing = true;
                    modal.disable_closing();
                    thread::spawn(move || {
                        let res = wallet.finalize(&message);
                        let mut w_res = final_res.write();
                        *w_res = Some(res);
                    });
                } else {
                    self.finalize_error = true;
                }
            } else {
                self.finalize_error = true;
            }
        }
    }
}