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

use egui::scroll_area::ScrollBarVisibility;
use egui::{Align, CornerRadius, Id, Layout, RichText, ScrollArea, StrokeKind};
use grin_core::core::amount_to_hr_string;
use grin_util::ToHex;
use grin_wallet_libwallet::TxLogEntryType;
use std::fs;

use crate::gui::icons::{CIRCLE_HALF, COPY, CUBE, FILE_ARCHIVE, FILE_TEXT, HASH_STRAIGHT, PROHIBIT, QR_CODE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::wallets::wallet::txs::WalletTransactionsContent;
use crate::gui::views::{Modal, QrCodeContent, View};
use crate::gui::Colors;
use crate::wallet::types::{WalletTask, WalletTransaction};
use crate::wallet::Wallet;

/// Transaction information [`Modal`] content.
pub struct WalletTransactionContent {
    /// Transaction identifier.
    tx_id: u32,
    /// Slatepack message text.
    message: Option<String>,

    /// QR code Slatepack message image content.
    qr_code_content: Option<QrCodeContent>,
}

impl WalletTransactionContent {
    /// Create new content instance with [`Wallet`] from provided [`WalletTransaction`].
    pub fn new(id: u32) -> Self {
        Self {
            tx_id: id,
            message: None,
            qr_code_content: None,
        }
    }

    /// Draw [`Modal`] content.
    pub fn ui(&mut self, ui: &mut egui::Ui, wallet: &Wallet, cb: &dyn PlatformCallbacks) {
        // Check values and setup transaction data.
        let wallet_data = wallet.get_data();
        if wallet_data.is_none() {
            Modal::close();
            return;
        }
        let data = wallet_data.unwrap();
        let data_txs = data.txs.clone().unwrap();
        let txs = data_txs.into_iter()
            .filter(|tx| tx.data.id == self.tx_id)
            .collect::<Vec<WalletTransaction>>();
        if txs.is_empty() {
            Modal::close();
            return;
        }
        let tx = txs.get(0).unwrap();

        if let Some(content) = self.qr_code_content.as_mut() {
            ui.add_space(6.0);
            content.ui(ui, cb);

            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            // Show buttons to close modal or come back to text request content.
            ui.columns(2, |cols| {
                cols[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("close"), Colors::white_or_black(false), || {
                        self.qr_code_content = None;
                        Modal::close();
                    });
                });
                cols[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("back"), Colors::white_or_black(false), || {
                        self.qr_code_content = None;
                    });
                });
            });
        } else {
            // Show transaction information.
            self.info_ui(ui, tx, wallet, cb);

            // Show transaction sharing content when can cancel or finalized.
            if tx.can_cancel() && !tx.finalized() {
                self.share_ui(ui, wallet, tx, cb);
            }

            ui.add_space(8.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(8.0);

            // Show button to close modal.
            ui.vertical_centered_justified(|ui| {
                View::button(ui, t!("close"), Colors::white_or_black(false), || {
                    Modal::close();
                });
            });
        }
        ui.add_space(6.0);
    }

    /// Draw transaction sharing content.
    fn share_ui(&mut self,
                ui: &mut egui::Ui,
                wallet: &Wallet,
                tx: &WalletTransaction,
                cb: &dyn PlatformCallbacks) {
        if self.message.is_none() {
            let slatepack_path = wallet.get_config().get_tx_slate_path(tx);
            self.message = Some(fs::read_to_string(slatepack_path).unwrap_or("".to_string()));
        }
        if let Some(m) = &self.message {
            if m.is_empty() {
                return;
            }
            let amount = amount_to_hr_string(tx.amount, true);
            let desc_text = if tx.can_finalize() {
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
            ui.add_space(6.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(desc_text).size(16.0).color(Colors::inactive_text()));
            });
            ui.add_space(6.0);

            let mut message = m.clone();
            // Draw slatepack message content.
            ui.vertical_centered(|ui| {
                let scroll_id = Id::from("tx_info_message_request").with(tx.data.id);
                View::horizontal_line(ui, Colors::item_stroke());
                ui.add_space(3.0);
                ScrollArea::vertical()
                    .id_salt(scroll_id)
                    .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                    .max_height(128.0)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.add_space(7.0);
                        let input_id = scroll_id.with("_input");
                        egui::TextEdit::multiline(&mut message)
                            .id(input_id)
                            .font(egui::TextStyle::Small)
                            .desired_rows(5)
                            .interactive(false)
                            .desired_width(f32::INFINITY)
                            .show(ui).response;
                        ui.add_space(6.0);
                    });
            });

            ui.add_space(2.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(8.0);

            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    // Draw button to show Slatepack message as QR code.
                    let qr_text = format!("{} {}", QR_CODE, t!("qr_code"));
                    View::button(ui, qr_text, Colors::white_or_black(false), || {
                        self.qr_code_content = Some(QrCodeContent::new(message, true));
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    // Draw copy button.
                    let copy_text = format!("{} {}", COPY, t!("copy"));
                    View::button(ui, copy_text, Colors::white_or_black(false), || {
                        cb.copy_string_to_buffer(m.clone());
                        Modal::close();
                    });
                });
            });

            // Draw button to share response as file.
            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                let share_text = format!("{} {}", FILE_TEXT, t!("share"));
                View::colored_text_button(ui,
                                          share_text,
                                          Colors::blue(),
                                          Colors::white_or_black(false), || {
                        if let Some(slate_id) = tx.data.tx_slate_id {
                            let name = format!("{}.{}.slatepack", slate_id, tx.state);
                            let data = m.as_bytes().to_vec();
                            cb.share_data(name, data).unwrap_or_default();
                            Modal::close();
                        }
                    });
            });
        }
    }

    /// Draw transaction information content.
    fn info_ui(&mut self,
               ui: &mut egui::Ui,
               tx: &WalletTransaction,
               wallet: &Wallet,
               cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);

        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(WalletTransactionsContent::TX_ITEM_HEIGHT);

        // Draw tx item background.
        let p = ui.painter();
        let r = View::item_rounding(0, 2, false);
        p.rect(rect, r, Colors::TRANSPARENT, View::item_stroke(), StrokeKind::Middle);

        // Show transaction amount status and time.
        let data = wallet.get_data().unwrap();
        WalletTransactionsContent::tx_item_ui(ui, tx, rect, &data, |ui| {
            // Show block height or buttons.
            if let Some(h) = tx.height {
                if h != 0 {
                    ui.add_space(6.0);
                    let height = format!("{} {}", CUBE, h.to_string());
                    ui.with_layout(Layout::bottom_up(Align::Max), |ui| {
                        ui.add_space(3.0);
                        ui.label(RichText::new(height)
                            .size(15.0)
                            .color(Colors::text(false)));
                    });
                }
                return;
            }

            if wallet.synced_from_node() && !tx.cancelled() && !tx.cancelling() && !tx.posting() {
                let rebroadcast = tx.broadcasting_timed_out(&wallet);

                // Draw button to cancel transaction.
                if tx.can_cancel() || rebroadcast {
                    let r = View::item_rounding(0, 2, true);
                    View::item_button(ui, r, PROHIBIT, Some(Colors::red()), || {
                        wallet.task(WalletTask::Cancel(tx.clone()));
                        Modal::close();
                    });
                }

                // Draw button to repeat transaction action.
                if tx.can_repeat_action() || rebroadcast {
                    let r = if tx.can_finalize() || tx.can_cancel() {
                        CornerRadius::default()
                    } else {
                        View::item_rounding(0, 2, true)
                    };
                    WalletTransactionsContent::tx_repeat_button_ui(ui, r, tx, wallet, rebroadcast);
                }
            }
        });

        // Show identifier.
        if let Some(id) = tx.data.tx_slate_id {
            let label = format!("{} {}", HASH_STRAIGHT, t!("id"));
            info_item_ui(ui, id.to_string(), label, true, cb);
        }
        // Show kernel.
        if let Some(kernel) = tx.data.kernel_excess {
            let label = format!("{} {}", FILE_ARCHIVE, t!("kernel"));
            info_item_ui(ui, kernel.0.to_hex(), label, true, cb);
        }
        // Show receiver address.
        if let Some(rec) = &tx.receiver {
            let label = format!("{} {}", CIRCLE_HALF, t!("network_mining.address"));
            info_item_ui(ui, rec.to_string(), label, true, cb);
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

    ui.painter().rect(bg_rect, rounding, Colors::fill(), View::item_stroke(), StrokeKind::Middle);

    ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
        // Draw button to copy transaction info value.
        if copy {
            rounding.nw = 0.0 as u8;
            rounding.sw = 0.0 as u8;
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