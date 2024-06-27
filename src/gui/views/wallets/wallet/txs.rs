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

use std::sync::Arc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use egui::{Align, Id, Layout, Margin, RichText, Rounding, ScrollArea};
use egui::scroll_area::ScrollBarVisibility;
use grin_core::core::amount_to_hr_string;
use grin_util::ToHex;
use grin_wallet_libwallet::{Error, Slate, SlateState, TxLogEntryType};
use parking_lot::RwLock;

use crate::gui::Colors;
use crate::gui::icons::{ARROW_CIRCLE_DOWN, ARROW_CIRCLE_UP, ARROW_CLOCKWISE, BRIDGE, BROOM, CALENDAR_CHECK, CHAT_CIRCLE_TEXT, CHECK, CHECK_CIRCLE, CLIPBOARD_TEXT, COPY, DOTS_THREE_CIRCLE, FILE_ARCHIVE, FILE_TEXT, GEAR_FINE, HASH_STRAIGHT, PROHIBIT, QR_CODE, SCAN, X_CIRCLE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{CameraContent, FilePickButton, Modal, PullToRefresh, QrCodeContent, Root, View};
use crate::gui::views::types::ModalPosition;
use crate::gui::views::wallets::types::WalletTab;
use crate::gui::views::wallets::wallet::types::{GRIN, SLATEPACK_MESSAGE_HINT, WalletTabType};
use crate::gui::views::wallets::wallet::WalletContent;
use crate::wallet::types::{WalletData, WalletTransaction};
use crate::wallet::Wallet;

/// Wallet transactions tab content.
pub struct WalletTransactions {
    /// Transaction identifier to use at [`Modal`].
    tx_info_id: Option<u32>,
    /// Identifier for [`Slate`] to use at [`Modal`].
    tx_info_slate_id: Option<String>,
    /// Response Slatepack message input value at [`Modal`].
    tx_info_response_edit: String,
    /// Finalization Slatepack message input value at [`Modal`].
    tx_info_finalize_edit: String,
    /// Flag to check if error happened during transaction finalization at [`Modal`].
    tx_info_finalize_error: bool,
    /// Flag to check if tx finalization requested at [`Modal`].
    tx_info_finalize: bool,
    /// Flag to check if tx is finalizing at [`Modal`].
    tx_info_finalizing: bool,
    /// Transaction finalization result for [`Modal`].
    tx_info_final_result: Arc<RwLock<Option<Result<Slate, Error>>>>,
    /// Flag to check if QR code is showing at [`Modal`].
    tx_info_show_qr: bool,
    /// QR code Slatepack message image [`Modal`] content.
    tx_info_qr_code_content: QrCodeContent,
    /// Flag to check if QR code scanner is showing at [`Modal`].
    tx_info_show_scanner: bool,
    /// QR code scanner [`Modal`] content.
    tx_info_scanner_content: CameraContent,
    /// Button to parse picked file content at [`Modal`].
    tx_info_file_pick_button: FilePickButton,

    /// Transaction identifier to use at confirmation [`Modal`].
    confirm_cancel_tx_id: Option<u32>,

    /// Flag to check if sync of wallet was initiated manually at time.
    manual_sync: Option<u128>
}

impl Default for WalletTransactions {
    fn default() -> Self {
        Self {
            tx_info_id: None,
            tx_info_slate_id: None,
            tx_info_response_edit: "".to_string(),
            tx_info_finalize_edit: "".to_string(),
            tx_info_finalize_error: false,
            tx_info_finalize: false,
            tx_info_finalizing: false,
            tx_info_final_result: Arc::new(RwLock::new(None)),
            tx_info_show_qr: false,
            tx_info_qr_code_content: QrCodeContent::new("".to_string(), true),
            tx_info_show_scanner: false,
            tx_info_scanner_content: CameraContent::default(),
            tx_info_file_pick_button: FilePickButton::default(),
            confirm_cancel_tx_id: None,
            manual_sync: None,
        }
    }
}

impl WalletTab for WalletTransactions {
    fn get_type(&self) -> WalletTabType {
        WalletTabType::Txs
    }

    fn ui(&mut self, ui: &mut egui::Ui, wallet: &mut Wallet, cb: &dyn PlatformCallbacks) {
        if WalletContent::sync_ui(ui, wallet) {
            return;
        }

        // Show modal content for this ui container.
        self.modal_content_ui(ui, wallet, cb);

        // Show wallet transactions content.
        egui::CentralPanel::default()
            .frame(egui::Frame {
                stroke: View::item_stroke(),
                fill: Colors::button(),
                inner_margin: Margin {
                    left: View::far_left_inset_margin(ui) + 4.0,
                    right: View::get_right_inset() + 4.0,
                    top: 0.0,
                    bottom: 4.0,
                },
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    let data = wallet.get_data().unwrap();
                    self.txs_ui(ui, wallet, &data, cb);
                });
            });
    }
}

/// Identifier for transaction information [`Modal`].
const TX_INFO_MODAL: &'static str = "tx_info_modal";

/// Identifier for transaction cancellation confirmation [`Modal`].
const CANCEL_TX_CONFIRMATION_MODAL: &'static str = "cancel_tx_conf_modal";

/// Height of transaction list item.
const TX_ITEM_HEIGHT: f32 = 76.0;

impl WalletTransactions {
    /// Draw transactions content.
    fn txs_ui(&mut self,
              ui: &mut egui::Ui,
              wallet: &mut Wallet,
              data: &WalletData,
              cb: &dyn PlatformCallbacks) {
        let amount_conf = data.info.amount_awaiting_confirmation;
        let amount_fin = data.info.amount_awaiting_finalization;
        let amount_locked = data.info.amount_locked;
        View::max_width_ui(ui, Root::SIDE_PANEL_WIDTH * 1.3, |ui| {
            // Show non-zero awaiting confirmation amount.
            if amount_conf != 0 {
                let awaiting_conf = amount_to_hr_string(amount_conf, true);
                let rounding = if amount_fin != 0 || amount_locked != 0 {
                    [false, false, false, false]
                } else {
                    [false, false, true, true]
                };
                View::rounded_box(ui,
                                  format!("{} ツ", awaiting_conf),
                                  t!("wallets.await_conf_amount"),
                                  rounding);
            }
            // Show non-zero awaiting finalization amount.
            if amount_fin != 0 {
                let awaiting_conf = amount_to_hr_string(amount_fin, true);
                let rounding = if amount_locked != 0 {
                    [false, false, false, false]
                } else {
                    [false, false, true, true]
                };
                View::rounded_box(ui,
                                  format!("{} ツ", awaiting_conf),
                                  t!("wallets.await_fin_amount"),
                                  rounding);
            }
            // Show non-zero locked amount.
            if amount_locked != 0 {
                let awaiting_conf = amount_to_hr_string(amount_locked, true);
                View::rounded_box(ui,
                                  format!("{} ツ", awaiting_conf),
                                  t!("wallets.locked_amount"),
                                  [false, false, true, true]);
            }

            // Show message when txs are empty.
            if let Some(txs) = data.txs.as_ref() {
                if txs.is_empty() {
                    View::center_content(ui, 96.0, |ui| {
                        let empty_text = t!(
                            "wallets.txs_empty",
                            "message" => CHAT_CIRCLE_TEXT,
                            "transport" => BRIDGE,
                            "settings" => GEAR_FINE
                        );
                        ui.label(RichText::new(empty_text).size(16.0).color(Colors::inactive_text()));
                    });
                    return;
                }
            }
        });

        // Show loader when txs are not loaded.
        if data.txs.is_none() {
            ui.centered_and_justified(|ui| {
                View::big_loading_spinner(ui);
            });
            return;
        }

        ui.add_space(4.0);

        // Show list of transactions.
        let txs = data.txs.as_ref().unwrap();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let refresh = self.manual_sync.unwrap_or(0) + 1600 > now;
        let refresh_resp = PullToRefresh::new(refresh)
            .can_refresh(!refresh && !wallet.syncing())
            .min_refresh_distance(70.0)
            .scroll_area_ui(ui, |ui| {
                ScrollArea::vertical()
                    .id_source(Id::from("txs_content").with(wallet.get_config().id))
                    .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                    .auto_shrink([false; 2])
                    .show_rows(ui, TX_ITEM_HEIGHT, txs.len(), |ui, row_range| {
                        ui.add_space(1.0);
                        View::max_width_ui(ui, Root::SIDE_PANEL_WIDTH * 1.3, |ui| {
                            let padding = amount_conf != 0 || amount_fin != 0 || amount_locked != 0;
                            for index in row_range {
                                let tx = txs.get(index).unwrap();
                                let r = View::item_rounding(index, txs.len(), false);
                                self.tx_item_ui(ui, tx, r, padding, true, &data, wallet, cb);
                            }
                        });
                    })
            });

        // Sync wallet on refresh.
        if refresh_resp.should_refresh() {
            self.manual_sync = Some(now);
            if !wallet.syncing() {
                wallet.sync(true);
            }
        }
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
                    TX_INFO_MODAL => {
                        Modal::ui(ui.ctx(), |ui, modal| {
                            self.tx_info_modal_ui(ui, wallet, modal, cb);
                        });
                    }
                    CANCEL_TX_CONFIRMATION_MODAL => {
                        Modal::ui(ui.ctx(), |ui, modal| {
                            self.cancel_confirmation_modal(ui, wallet, modal);
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    /// Draw transaction item.
    fn tx_item_ui(&mut self,
                  ui: &mut egui::Ui,
                  tx: &WalletTransaction,
                  mut rounding: Rounding,
                  extra_padding: bool,
                  can_show_info: bool,
                  data: &WalletData,
                  wallet: &mut Wallet,
                  cb: &dyn PlatformCallbacks) {
        // Setup layout size.
        let mut rect = ui.available_rect_before_wrap();
        if extra_padding {
            rect.min += egui::emath::vec2(6.0, 0.0);
            rect.max -= egui::emath::vec2(6.0, 0.0);
        }
        rect.set_height(TX_ITEM_HEIGHT);

        // Draw round background.
        let bg_rect = rect.clone();
        let color = if can_show_info {
            Colors::button()
        } else {
            Colors::fill()
        };
        ui.painter().rect(bg_rect, rounding, color, View::item_stroke());

        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Max), |ui| {
            ui.horizontal_centered(|ui| {
                // Draw button to show transaction info.
                if can_show_info && tx.data.tx_slate_id.is_some() {
                    rounding.nw = 0.0;
                    rounding.sw = 0.0;
                    View::item_button(ui, rounding, FILE_TEXT, None, || {
                        self.tx_info_finalize = false;
                        self.show_tx_info_modal(wallet, tx);
                    });
                }

                // Draw finalization button for tx that can be finalized.
                let finalize = ((!can_show_info && !self.tx_info_finalizing) || can_show_info)
                    && tx.can_finalize;
                if finalize {
                    let (icon, color) = if !can_show_info && self.tx_info_finalize {
                        (FILE_TEXT, None)
                    } else {
                        (CHECK, Some(Colors::green()))
                    };
                    let final_rounding = if can_show_info {
                        Rounding::default()
                    } else {
                        rounding.nw = 0.0;
                        rounding.sw = 0.0;
                        rounding
                    };
                    View::item_button(ui, final_rounding, icon, color, || {
                        cb.hide_keyboard();
                        if !can_show_info && self.tx_info_finalize {
                            self.tx_info_finalize = false;
                            return;
                        }
                        self.tx_info_finalize = true;
                        // Show transaction information modal.
                        if can_show_info {
                            self.show_tx_info_modal(wallet, tx);
                        }
                    });
                }

                // Draw cancel button for tx that can be reposted and canceled.
                let wallet_loaded = wallet.foreign_api_port().is_some();
                if wallet_loaded && ((!can_show_info && !self.tx_info_finalizing) || can_show_info) &&
                    (tx.can_repost(data) || tx.can_cancel()) {
                    View::item_button(ui, Rounding::default(), PROHIBIT, Some(Colors::red()), || {
                        if can_show_info {
                            self.confirm_cancel_tx_id = Some(tx.data.id);
                            // Show transaction cancellation confirmation modal.
                            Modal::new(CANCEL_TX_CONFIRMATION_MODAL)
                                .position(ModalPosition::Center)
                                .title(t!("modal.confirmation"))
                                .show();
                        } else {
                            cb.hide_keyboard();
                            wallet.cancel(tx.data.id);
                        }
                    });
                }

                // Draw button to repost transaction.
                if ((!can_show_info && !self.tx_info_finalizing) || can_show_info) &&
                    tx.can_repost(data) {
                    let r = if finalize || can_show_info {
                        Rounding::default()
                    } else {
                        rounding.nw = 0.0;
                        rounding.sw = 0.0;
                        rounding
                    };
                    View::item_button(ui, r, ARROW_CLOCKWISE, Some(Colors::green()), || {
                        cb.hide_keyboard();
                        // Post tx after getting slate from slatepack file.
                        if let Some((s, _)) = wallet.read_slate_by_tx(tx) {
                            let _ = wallet.post(&s, wallet.can_use_dandelion());
                        }
                    });
                }
            });

            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                ui.add_space(6.0);
                ui.vertical(|ui| {
                    ui.add_space(3.0);

                    // Setup transaction amount.
                    let mut amount_text = if tx.data.tx_type == TxLogEntryType::TxSent ||
                        tx.data.tx_type == TxLogEntryType::TxSentCancelled {
                        "-"
                    } else if tx.data.tx_type == TxLogEntryType::TxReceived ||
                        tx.data.tx_type == TxLogEntryType::TxReceivedCancelled {
                        "+"
                    } else {
                        ""
                    }.to_string();
                    amount_text = format!("{}{} {}",
                                          amount_text,
                                          amount_to_hr_string(tx.amount, true),
                                          GRIN);

                    // Setup amount color.
                    let amount_color = match tx.data.tx_type {
                        TxLogEntryType::ConfirmedCoinbase => Colors::white_or_black(true),
                        TxLogEntryType::TxReceived => Colors::white_or_black(true),
                        TxLogEntryType::TxSent => Colors::white_or_black(true),
                        TxLogEntryType::TxReceivedCancelled => Colors::text(false),
                        TxLogEntryType::TxSentCancelled => Colors::text(false),
                        TxLogEntryType::TxReverted => Colors::text(false)
                    };
                    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                        ui.add_space(1.0);
                        View::ellipsize_text(ui, amount_text, 18.0, amount_color);
                    });
                    ui.add_space(-2.0);

                    // Setup transaction status text.
                    let status_text = if !tx.data.confirmed {
                        let is_canceled = tx.data.tx_type == TxLogEntryType::TxSentCancelled
                            || tx.data.tx_type == TxLogEntryType::TxReceivedCancelled;
                        if is_canceled {
                            format!("{} {}", X_CIRCLE, t!("wallets.tx_canceled"))
                        } else if tx.posting {
                            format!("{} {}", DOTS_THREE_CIRCLE, t!("wallets.tx_finalizing"))
                        } else {
                            if tx.cancelling {
                                format!("{} {}", DOTS_THREE_CIRCLE, t!("wallets.tx_cancelling"))
                            } else {
                                match tx.data.tx_type {
                                    TxLogEntryType::TxReceived => {
                                        format!("{} {}",
                                                DOTS_THREE_CIRCLE,
                                                t!("wallets.tx_receiving"))
                                    },
                                    TxLogEntryType::TxSent => {
                                        format!("{} {}",
                                                DOTS_THREE_CIRCLE,
                                                t!("wallets.tx_sending"))
                                    },
                                    _ => {
                                        format!("{} {}",
                                                DOTS_THREE_CIRCLE,
                                                t!("wallets.tx_confirmed"))
                                    }
                                }
                            }
                        }
                    } else {
                        match tx.data.tx_type {
                            TxLogEntryType::ConfirmedCoinbase => {
                                format!("{} {}", CHECK_CIRCLE, t!("wallets.tx_confirmed"))
                            },
                            TxLogEntryType::TxSent | TxLogEntryType::TxReceived => {
                                let height = data.info.last_confirmed_height;
                                let min_conf = data.info.minimum_confirmations;
                                if tx.conf_height.is_none() || (tx.conf_height.unwrap() != 0 &&
                                    height - tx.conf_height.unwrap() > min_conf - 1) {
                                    let (i, t) = if tx.data.tx_type == TxLogEntryType::TxSent {
                                        (ARROW_CIRCLE_UP, t!("wallets.tx_sent"))
                                    } else {
                                        (ARROW_CIRCLE_DOWN, t!("wallets.tx_received"))
                                    };
                                    format!("{} {}", i, t)
                                } else {
                                    let tx_height = tx.conf_height.unwrap() - 1;
                                    let left_conf = height - tx_height;
                                    let conf_info = if tx_height != 0 && height >= tx_height &&
                                        left_conf < min_conf {
                                        format!("{}/{}", left_conf, min_conf)
                                    } else {
                                        "".to_string()
                                    };
                                    format!("{} {} {}",
                                            DOTS_THREE_CIRCLE,
                                            t!("wallets.tx_confirming"),
                                            conf_info
                                    )
                                }
                            },
                            _ => format!("{} {}", X_CIRCLE, t!("wallets.canceled"))
                        }
                    };

                    // Setup status text color.
                    let status_color = match tx.data.tx_type {
                        TxLogEntryType::ConfirmedCoinbase => Colors::text(false),
                        TxLogEntryType::TxReceived => if tx.data.confirmed {
                            Colors::green()
                        } else {
                            Colors::text(false)
                        },
                        TxLogEntryType::TxSent => if tx.data.confirmed {
                            Colors::red()
                        } else {
                            Colors::text(false)
                        },
                        TxLogEntryType::TxReceivedCancelled => Colors::inactive_text(),
                        TxLogEntryType::TxSentCancelled => Colors::inactive_text(),
                        TxLogEntryType::TxReverted => Colors::inactive_text(),
                    };
                    ui.label(RichText::new(status_text).size(15.0).color(status_color));

                    // Setup transaction time.
                    let tx_time = View::format_time(tx.data.creation_ts.timestamp());
                    let tx_time_text = format!("{} {}", CALENDAR_CHECK, tx_time);
                    ui.label(RichText::new(tx_time_text).size(15.0).color(Colors::gray()));
                    ui.add_space(3.0);
                });
            });
        });
    }

    /// Show transaction information [`Modal`].
    fn show_tx_info_modal(&mut self, wallet: &Wallet, tx: &WalletTransaction) {
        self.tx_info_response_edit = "".to_string();
        self.tx_info_finalize_edit = "".to_string();
        self.tx_info_finalize_error = false;
        self.tx_info_id = Some(tx.data.id);
        self.tx_info_show_qr = false;
        self.tx_info_slate_id = if let Some(id) = tx.data.tx_slate_id {
            Some(id.to_string())
        } else {
            None
        };

        // Setup slate and message from transaction.
        self.tx_info_response_edit = if !tx.data.confirmed && tx.data.tx_slate_id.is_some() &&
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
        };

        // Show transaction information modal.
        Modal::new(TX_INFO_MODAL)
            .position(ModalPosition::CenterTop)
            .title(t!("wallets.tx"))
            .show();
    }

    /// Draw transaction info [`Modal`] content.
    fn tx_info_modal_ui(&mut self,
                       ui: &mut egui::Ui,
                       wallet: &mut Wallet,
                       modal: &Modal,
                       cb: &dyn PlatformCallbacks) {
        // Check values and setup transaction data.
        let wallet_data = wallet.get_data();
        if wallet_data.is_none() {
            modal.close();
            return;
        }
        let data = wallet_data.unwrap();
        let tx_id = self.tx_info_id.unwrap();
        let data_txs = data.txs.clone().unwrap();
        let txs = data_txs.into_iter()
            .filter(|tx| tx.data.id == tx_id)
            .collect::<Vec<WalletTransaction>>();
        if txs.is_empty() {
            cb.hide_keyboard();
            modal.close();
            return;
        }
        let tx = txs.get(0).unwrap();

        if !self.tx_info_show_qr && !self.tx_info_show_scanner {
            ui.add_space(6.0);

            // Show transaction amount status and time.
            let rounding = View::item_rounding(0, 2, false);
            self.tx_item_ui(ui, tx, rounding, false, false, &data, wallet, cb);

            // Show transaction ID info.
            if let Some(id) = tx.data.tx_slate_id {
                let label = format!("{} {}", HASH_STRAIGHT, t!("id"));
                Self::tx_info_modal_item_ui(ui, id.to_string(), label, true, cb);
            }
            // Show transaction kernel info.
            if let Some(kernel) = tx.data.kernel_excess {
                let label = format!("{} {}", FILE_ARCHIVE, t!("kernel"));
                Self::tx_info_modal_item_ui(ui, kernel.0.to_hex(), label, true, cb);
            }
        }

        // Show Slatepack message or reset flag to show QR if not available.
        if !tx.posting && !tx.data.confirmed && !tx.cancelling &&
            (tx.data.tx_type == TxLogEntryType::TxSent ||
            tx.data.tx_type == TxLogEntryType::TxReceived) {
            self.tx_info_modal_slate_ui(ui, tx, wallet, modal, cb);
        } else if self.tx_info_show_qr {
            self.tx_info_qr_code_content.clear_state();
            self.tx_info_show_qr = false;
        }

        if !self.tx_info_finalizing {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            if self.tx_info_show_qr {
                // Show buttons to close modal or come back to text request content.
                ui.columns(2, |cols| {
                    cols[0].vertical_centered_justified(|ui| {
                        View::button(ui, t!("close"), Colors::white_or_black(false), || {
                            self.tx_info_qr_code_content.clear_state();
                            self.tx_info_show_qr = false;
                            modal.close();
                        });
                    });
                    cols[1].vertical_centered_justified(|ui| {
                        View::button(ui, t!("back"), Colors::white_or_black(false), || {
                            self.tx_info_qr_code_content.clear_state();
                            self.tx_info_show_qr = false;
                        });
                    });
                });
            } else if self.tx_info_show_scanner {
                ui.add_space(8.0);
                // Show buttons to close modal or scanner.
                ui.columns(2, |cols| {
                    cols[0].vertical_centered_justified(|ui| {
                        View::button(ui, t!("close"), Colors::white_or_black(false), || {
                            cb.stop_camera();
                            self.tx_info_scanner_content.clear_state();
                            self.tx_info_show_scanner = false;
                            modal.close();
                        });
                    });
                    cols[1].vertical_centered_justified(|ui| {
                        View::button(ui, t!("back"), Colors::white_or_black(false), || {
                            cb.stop_camera();
                            self.tx_info_scanner_content.clear_state();
                            self.tx_info_show_scanner = false;
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
                        self.tx_info_id = None;
                        self.tx_info_finalize = false;
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
                let r_res = self.tx_info_final_result.read();
                r_res.is_some()
            };
            if has_res {
                let res = {
                    let r_res = self.tx_info_final_result.read();
                    r_res.as_ref().unwrap().clone()
                };
                if let Ok(_) = res {
                    self.tx_info_finalize = false;
                    self.tx_info_finalize_edit = "".to_string();
                } else {
                    self.tx_info_finalize_error = true;
                }
                // Clear status and result.
                {
                    let mut w_res = self.tx_info_final_result.write();
                    *w_res = None;
                }
                self.tx_info_finalizing = false;
                modal.enable_closing();
            }
        }
    }

    /// Draw transaction information [`Modal`] item content.
    fn tx_info_modal_item_ui(ui: &mut egui::Ui,
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

    /// Draw Slate content to show response or generate payment proof.
    fn tx_info_modal_slate_ui(&mut self,
                              ui: &mut egui::Ui,
                              tx: &WalletTransaction,
                              wallet: &Wallet,
                              modal: &Modal,
                              cb: &dyn PlatformCallbacks) {
        if self.tx_info_slate_id.is_none() {
            cb.hide_keyboard();
            modal.close();
            return;
        }
        ui.add_space(6.0);

        // Draw QR code scanner content if requested.
        if self.tx_info_show_scanner {
            if let Some(result) = self.tx_info_scanner_content.qr_scan_result() {
                cb.stop_camera();
                self.tx_info_scanner_content.clear_state();

                // Setup value to finalization input field.
                self.tx_info_finalize_edit = result.text();
                self.on_finalization_input_change(tx, wallet, modal, cb);

                modal.enable_closing();
                self.tx_info_scanner_content.clear_state();
                self.tx_info_show_scanner = false;
            } else {
                self.tx_info_scanner_content.ui(ui, cb);
            }
            return;
        }

        let amount = amount_to_hr_string(tx.amount, true);

        // Draw Slatepack message description text.
        ui.vertical_centered(|ui| {
            if self.tx_info_finalize {
                let desc_text = if self.tx_info_finalize_error {
                    t!("wallets.finalize_slatepack_err")
                } else {
                    if tx.data.tx_type == TxLogEntryType::TxSent {
                        t!("wallets.parse_s2_slatepack_desc", "amount" => amount)
                    } else {
                        t!("wallets.parse_i2_slatepack_desc", "amount" => amount)
                    }
                };
                let desc_color = if self.tx_info_finalize_error {
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
        let message_edit = if self.tx_info_finalize {
            &mut self.tx_info_finalize_edit
        }  else {
            &mut self.tx_info_response_edit
        };
        let message_before = message_edit.clone();

        // Draw QR code content if requested.
        if self.tx_info_show_qr {
            let text = message_edit.clone();
            if text.is_empty() {
                self.tx_info_qr_code_content.clear_state();
                self.tx_info_show_qr = false;
            } else {
                // Draw QR code content.
                self.tx_info_qr_code_content.ui(ui, text.clone(), cb);
                return;
            }
        }

        // Draw Slatepack message finalization input or request text.
        ui.vertical_centered(|ui| {
            let scroll_id = if self.tx_info_finalize {
                Id::from("tx_info_message_finalize")
            } else {
                Id::from("tx_info_message_request")
            }.with(self.tx_info_slate_id.clone().unwrap()).with(tx.data.id);
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
                        .interactive(self.tx_info_finalize && !self.tx_info_finalizing)
                        .hint_text(SLATEPACK_MESSAGE_HINT)
                        .desired_width(f32::INFINITY)
                        .show(ui).response;
                    // Show soft keyboard on click.
                    if self.tx_info_finalize && resp.clicked() {
                        resp.request_focus();
                        cb.show_keyboard();
                    }
                    if self.tx_info_finalize && resp.has_focus() {
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
        if self.tx_info_finalizing {
            return;
        }

        // Setup spacing between buttons.
        ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

        if self.tx_info_finalize {
            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    // Draw button to scan Slatepack message QR code.
                    let qr_text = format!("{} {}", SCAN, t!("scan"));
                    View::button(ui, qr_text, Colors::button(), || {
                        cb.hide_keyboard();
                        modal.disable_closing();
                        cb.start_camera();
                        self.tx_info_show_scanner = true;
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    // Draw button to paste data from clipboard.
                    let paste_text = format!("{} {}", CLIPBOARD_TEXT, t!("paste"));
                    View::button(ui, paste_text, Colors::button(), || {
                        self.tx_info_finalize_edit = cb.get_string_from_buffer();
                    });
                });
            });
            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                if self.tx_info_finalize_error {
                    // Draw button to clear message input.
                    let clear_text = format!("{} {}", BROOM, t!("clear"));
                    View::button(ui, clear_text, Colors::button(), || {
                        self.tx_info_finalize_edit.clear();
                        self.tx_info_finalize_error = false;
                    });
                } else {
                    // Draw button to choose file.
                    self.tx_info_file_pick_button.ui(ui, cb, |text| {
                        self.tx_info_finalize_edit = text;
                    });
                }
            });

            // Callback on finalization message input change.
            if message_before != self.tx_info_finalize_edit {
                self.on_finalization_input_change(tx, wallet, modal, cb);
            }
        } else {
            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    // Draw button to show Slatepack message as QR code.
                    let qr_text = format!("{} {}", QR_CODE, t!("qr_code"));
                    View::button(ui, qr_text, Colors::button(), || {
                        cb.hide_keyboard();
                        self.tx_info_show_qr = true;
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    // Draw copy button.
                    let copy_text = format!("{} {}", COPY, t!("copy"));
                    View::button(ui, copy_text, Colors::button(), || {
                        cb.copy_string_to_buffer(self.tx_info_response_edit.clone());
                        self.tx_info_finalize_edit = "".to_string();
                        if tx.can_finalize {
                            self.tx_info_finalize = true;
                        } else {
                            cb.hide_keyboard();
                            modal.close();
                        }
                    });
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
        let message = &self.tx_info_finalize_edit;
        if message.is_empty() {
            self.tx_info_finalize_error = false;
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
                    let final_res = self.tx_info_final_result.clone();
                    // Finalize transaction at separate thread.
                    cb.hide_keyboard();
                    self.tx_info_finalizing = true;
                    modal.disable_closing();
                    thread::spawn(move || {
                        let res = wallet.finalize(&message, wallet.can_use_dandelion());
                        let mut w_res = final_res.write();
                        *w_res = Some(res);
                    });
                } else {
                    self.tx_info_finalize_error = true;
                }
            } else {
                self.tx_info_finalize_error = true;
            }
        }
    }

    /// Confirmation [`Modal`] to cancel transaction.
    fn cancel_confirmation_modal(&mut self, ui: &mut egui::Ui, wallet: &mut Wallet, modal: &Modal) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            // Setup confirmation text.
            let data = wallet.get_data().unwrap();
            let data_txs = data.txs.unwrap();
            let txs = data_txs.into_iter()
                .filter(|tx| tx.data.id == self.confirm_cancel_tx_id.unwrap())
                .collect::<Vec<WalletTransaction>>();
            if txs.is_empty() {
                modal.close();
                return;
            }
            let tx = txs.get(0).unwrap();
            let amount = amount_to_hr_string(tx.amount, true);
            let text = match tx.data.tx_type {
                TxLogEntryType::TxReceived => {
                    t!("wallets.tx_receive_cancel_conf", "amount" => amount)
                },
                _ => {
                    t!("wallets.tx_send_cancel_conf", "amount" => amount)
                }
            };
            ui.label(RichText::new(text)
                .size(17.0)
                .color(Colors::text(false)));
            ui.add_space(8.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                        self.confirm_cancel_tx_id = None;
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, "OK".to_string(), Colors::white_or_black(false), || {
                        wallet.cancel(self.confirm_cancel_tx_id.unwrap());
                        self.confirm_cancel_tx_id = None;
                        modal.close();
                    });
                });
            });
            ui.add_space(6.0);
        });
    }
}