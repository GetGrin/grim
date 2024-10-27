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

use std::ops::Range;
use std::time::{SystemTime, UNIX_EPOCH};
use egui::{Align, Id, Layout, Rect, RichText, Rounding, ScrollArea};
use egui::epaint::RectShape;
use egui::scroll_area::ScrollBarVisibility;
use grin_core::consensus::COINBASE_MATURITY;
use grin_core::core::amount_to_hr_string;
use grin_wallet_libwallet::TxLogEntryType;

use crate::gui::Colors;
use crate::gui::icons::{ARROW_CIRCLE_DOWN, ARROW_CIRCLE_UP, BRIDGE, CALENDAR_CHECK, CHAT_CIRCLE_TEXT, CHECK, DOTS_THREE_CIRCLE, FILE_TEXT, GEAR_FINE, PROHIBIT, X_CIRCLE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, PullToRefresh, Content, View};
use crate::gui::views::types::{LinePosition, ModalPosition};
use crate::gui::views::wallets::types::WalletTab;
use crate::gui::views::wallets::wallet::types::{GRIN, WalletTabType};
use crate::gui::views::wallets::wallet::WalletTransactionModal;
use crate::wallet::types::{WalletData, WalletTransaction};
use crate::wallet::Wallet;

/// Wallet transactions tab content.
pub struct WalletTransactions {
    /// Transaction information [`Modal`] content.
    tx_info_content: Option<WalletTransactionModal>,

    /// Transaction identifier to use at confirmation [`Modal`].
    confirm_cancel_tx_id: Option<u32>,

    /// Flag to check if sync of wallet was initiated manually at time.
    manual_sync: Option<u128>
}

impl Default for WalletTransactions {
    fn default() -> Self {
        Self {
            tx_info_content: None,
            confirm_cancel_tx_id: None,
            manual_sync: None,
        }
    }
}

impl WalletTab for WalletTransactions {
    fn get_type(&self) -> WalletTabType {
        WalletTabType::Txs
    }

    fn ui(&mut self, ui: &mut egui::Ui, wallet: &Wallet, cb: &dyn PlatformCallbacks) {
        self.modal_content_ui(ui, wallet, cb);
        self.txs_ui(ui, wallet, cb);
    }
}

/// Identifier for transaction information [`Modal`].
const TX_INFO_MODAL: &'static str = "tx_info_modal";
/// Identifier for transaction cancellation confirmation [`Modal`].
const CANCEL_TX_CONFIRMATION_MODAL: &'static str = "cancel_tx_conf_modal";

impl WalletTransactions {
    /// Height of transaction list item.
    pub const TX_ITEM_HEIGHT: f32 = 75.0;

    /// Draw transactions content.
    fn txs_ui(&mut self,
              ui: &mut egui::Ui,
              wallet: &Wallet,
              cb: &dyn PlatformCallbacks) {
        let data = wallet.get_data().unwrap();
        if data.txs.is_none() {
            ui.centered_and_justified(|ui| {
                View::big_loading_spinner(ui);
            });
            return;
        }
        let txs = data.txs.as_ref().unwrap();
        let mut awaiting_amount = false;
        View::max_width_ui(ui, Content::SIDE_PANEL_WIDTH * 1.3, |ui| {
            // Show message when txs are empty.
            if txs.is_empty() {
                View::center_content(ui, 96.0, |ui| {
                    let empty_text = t!(
                            "wallets.txs_empty",
                            "message" => CHAT_CIRCLE_TEXT,
                            "transport" => BRIDGE,
                            "settings" => GEAR_FINE
                        );
                    ui.label(RichText::new(empty_text)
                        .size(16.0)
                        .color(Colors::inactive_text()));
                });
                return;
            }
            // Draw awaiting amount info if exists.
            awaiting_amount = self.awaiting_info_ui(ui, &data);
        });
        ui.add_space(4.0);

        // Show list of transactions.
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let refresh = self.manual_sync.unwrap_or(0) + 1600 > now;
        let refresh_resp = PullToRefresh::new(refresh)
            .can_refresh(!refresh && !wallet.syncing())
            .min_refresh_distance(70.0)
            .scroll_area_ui(ui, |ui| {
                ScrollArea::vertical()
                    .id_salt(Id::from("wallet_tx_list_scroll").with(wallet.get_config().id))
                    .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                    .auto_shrink([false; 2])
                    .show_rows(ui, Self::TX_ITEM_HEIGHT, txs.len(), |ui, row_range| {
                        ui.add_space(1.0);
                        View::max_width_ui(ui, Content::SIDE_PANEL_WIDTH * 1.3, |ui| {
                            self.tx_list_ui(ui, awaiting_amount, row_range, wallet, txs, cb);
                        });
                    })
            });

        // Sync wallet on refresh.
        if refresh_resp.should_refresh() {
            self.manual_sync = Some(now);
            if !wallet.syncing() {
                wallet.sync();
            }
        }
    }

    /// Draw transaction list content.
    fn tx_list_ui(&mut self,
                  ui: &mut egui::Ui,
                  awaiting: bool,
                  row_range: Range<usize>,
                  wallet: &Wallet,
                  txs: &Vec<WalletTransaction>,
                  cb: &dyn PlatformCallbacks) {
        for index in row_range {
            let mut rect = if awaiting {
                let mut rect = ui.available_rect_before_wrap();
                rect.min += egui::emath::vec2(6.0, 0.0);
                rect.max -= egui::emath::vec2(6.0, 0.0);
                rect
            } else {
                ui.available_rect_before_wrap()
            };
            rect.set_height(Self::TX_ITEM_HEIGHT);

            // Draw tx item background.
            let mut r = View::item_rounding(index, txs.len(), false);
            let p = ui.painter();
            p.rect(rect, r, Colors::fill_lite(), View::item_stroke());

            let tx = txs.get(index).unwrap();
            let data = wallet.get_data().unwrap();
            Self::tx_item_ui(ui, tx, rect, &data, |ui| {
                // Draw button to show transaction info.
                if tx.data.tx_slate_id.is_some() {
                    r.nw = 0.0;
                    r.sw = 0.0;
                    View::item_button(ui, r, FILE_TEXT, None, || {
                        self.show_tx_info_modal(wallet, tx, false);
                    });
                }

                let wallet_loaded = wallet.foreign_api_port().is_some();

                // Draw button to show transaction finalization.
                if wallet_loaded && tx.can_finalize {
                    let (icon, color) = (CHECK, Some(Colors::green()));
                    View::item_button(ui, Rounding::default(), icon, color, || {
                        cb.hide_keyboard();
                        self.show_tx_info_modal(wallet, tx, true);
                    });
                }

                // Draw button to cancel transaction.
                if wallet_loaded && tx.can_cancel() {
                    let (icon, color) = (PROHIBIT, Some(Colors::red()));
                    View::item_button(ui, Rounding::default(), icon, color, || {
                        self.confirm_cancel_tx_id = Some(tx.data.id);
                        // Show transaction cancellation confirmation modal.
                        Modal::new(CANCEL_TX_CONFIRMATION_MODAL)
                            .position(ModalPosition::Center)
                            .title(t!("confirmation"))
                            .show();
                    });
                }
            });
        }
    }

    /// Draw information about locked, finalizing or confirming balance, return `true` if exists.
    fn awaiting_info_ui(&mut self, ui: &mut egui::Ui, data: &WalletData) -> bool {
        let amount_conf = data.info.amount_awaiting_confirmation;
        let amount_fin = data.info.amount_awaiting_finalization;
        let amount_locked = data.info.amount_locked;
        if amount_conf == 0 && amount_fin == 0 && amount_locked == 0 {
            return false;
        }
        let rect = ui.available_rect_before_wrap();
        // Draw background.
        let mut bg = RectShape::new(rect, Rounding {
            nw: 0.0,
            ne: 0.0,
            sw: 8.0,
            se: 8.0,
        }, Colors::TRANSPARENT, View::item_stroke());
        let bg_idx = ui.painter().add(bg);
        let resp = ui.allocate_ui(rect.size(), |ui| {
            ui.vertical_centered_justified(|ui| {
                // Correct vertical spacing between items.
                ui.style_mut().spacing.item_spacing.y = -3.0;
                if amount_conf != 0 {
                    // Draw awaiting confirmation amount.
                    awaiting_item_ui(ui, amount_conf, t!("wallets.await_conf_amount"));
                }
                if amount_fin != 0 {
                    // Draw awaiting confirmation amount.
                    awaiting_item_ui(ui, amount_fin, t!("wallets.await_fin_amount"));
                }
                if amount_locked != 0 {
                    // Draw locked amount.
                    awaiting_item_ui(ui, amount_locked, t!("wallets.locked_amount"));
                }
            });
        }).response;
        // Setup background size.
        bg.rect = resp.rect;
        ui.painter().set(bg_idx, bg);
        true
    }

    /// Draw [`Modal`] content for this ui container.
    fn modal_content_ui(&mut self,
                        ui: &mut egui::Ui,
                        wallet: &Wallet,
                        cb: &dyn PlatformCallbacks) {
        match Modal::opened() {
            None => {}
            Some(id) => {
                match id {
                    TX_INFO_MODAL => {
                        Modal::ui(ui.ctx(), |ui, modal| {
                            if let Some(content) = self.tx_info_content.as_mut() {
                                content.ui(ui, wallet, modal, cb);
                            }
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
    pub fn tx_item_ui(ui: &mut egui::Ui,
                      tx: &WalletTransaction,
                      rect: Rect,
                      data: &WalletData,
                      buttons_ui: impl FnOnce(&mut egui::Ui)) {
        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Max), |ui| {
            ui.horizontal_centered(|ui| {
                // Draw buttons.
                buttons_ui(ui);
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
                    let height = data.info.last_confirmed_height;
                    let status_text = if !tx.data.confirmed {
                        let is_canceled = tx.data.tx_type == TxLogEntryType::TxSentCancelled
                            || tx.data.tx_type == TxLogEntryType::TxReceivedCancelled;
                        if is_canceled {
                            format!("{} {}", X_CIRCLE, t!("wallets.tx_canceled"))
                        } else if tx.finalizing {
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
                                    TxLogEntryType::ConfirmedCoinbase => {
                                        let tx_h = tx.height.unwrap_or(1) - 1;
                                        if tx_h != 0 {
                                            let left_conf = height - tx_h;
                                            if height >= tx_h && left_conf < COINBASE_MATURITY {
                                                let conf_info = format!("{}/{}",
                                                                        left_conf,
                                                                        COINBASE_MATURITY);
                                                format!("{} {} {}",
                                                        DOTS_THREE_CIRCLE,
                                                        t!("wallets.tx_confirming"),
                                                        conf_info
                                                )
                                            } else {
                                                format!("{} {}",
                                                        DOTS_THREE_CIRCLE,
                                                        t!("wallets.tx_confirming"))
                                            }
                                        } else {
                                            format!("{} {}",
                                                    DOTS_THREE_CIRCLE,
                                                    t!("wallets.tx_confirming"))
                                        }
                                    },
                                    _ => {
                                        format!("{} {}",
                                                DOTS_THREE_CIRCLE,
                                                t!("wallets.tx_confirming"))
                                    }
                                }
                            }
                        }
                    } else {
                        match tx.data.tx_type {
                            TxLogEntryType::ConfirmedCoinbase => {
                                let tx_h = tx.height.unwrap_or(1) - 1;
                                if tx_h != 0 {
                                    let left_conf = height - tx_h;
                                    if height >= tx_h && left_conf < COINBASE_MATURITY {
                                        let conf_info = format!("{}/{}",
                                                                left_conf,
                                                                COINBASE_MATURITY);
                                        format!("{} {} {}",
                                                DOTS_THREE_CIRCLE,
                                                t!("wallets.tx_confirming"),
                                                conf_info
                                        )
                                    } else {
                                        format!("{} {}",
                                                DOTS_THREE_CIRCLE,
                                                t!("wallets.tx_confirmed"))
                                    }
                                } else {
                                    format!("{} {}",
                                            DOTS_THREE_CIRCLE,
                                            t!("wallets.tx_confirmed"))
                                }

                            },
                            TxLogEntryType::TxSent | TxLogEntryType::TxReceived => {
                                let min_conf = data.info.minimum_confirmations;
                                if tx.height.is_none() || (tx.height.unwrap() != 0 &&
                                    height - tx.height.unwrap() > min_conf - 1) {
                                    let (i, t) = if tx.data.tx_type == TxLogEntryType::TxSent {
                                        (ARROW_CIRCLE_UP, t!("wallets.tx_sent"))
                                    } else {
                                        (ARROW_CIRCLE_DOWN, t!("wallets.tx_received"))
                                    };
                                    format!("{} {}", i, t)
                                } else {
                                    let tx_height = tx.height.unwrap() - 1;
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
    fn show_tx_info_modal(&mut self, wallet: &Wallet, tx: &WalletTransaction, finalize: bool) {
        let modal = WalletTransactionModal::new(wallet, tx, finalize);
        self.tx_info_content = Some(modal);
        Modal::new(TX_INFO_MODAL)
            .position(ModalPosition::CenterTop)
            .title(t!("wallets.tx"))
            .show();
    }

    /// Confirmation [`Modal`] to cancel transaction.
    fn cancel_confirmation_modal(&mut self, ui: &mut egui::Ui, wallet: &Wallet, modal: &Modal) {
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

/// Draw awaiting balance item content.
fn awaiting_item_ui(ui: &mut egui::Ui, amount: u64, label: String) {
    let rect = ui.available_rect_before_wrap();
    View::line(ui, LinePosition::TOP, &rect, Colors::item_stroke());
    ui.add_space(4.0);
    let amount_format = amount_to_hr_string(amount, true);
    ui.label(RichText::new(format!("{} ツ", amount_format))
        .color(Colors::white_or_black(true))
        .size(17.0));
    ui.label(RichText::new(label)
        .color(Colors::gray())
        .size(15.0));
    ui.add_space(4.0);
}