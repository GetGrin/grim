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

use egui::{Align, Id, Layout, Margin, RichText, Rounding, ScrollArea};
use egui::scroll_area::ScrollBarVisibility;
use grin_core::core::amount_to_hr_string;
use grin_wallet_libwallet::{TxLogEntry, TxLogEntryType};

use crate::gui::Colors;
use crate::gui::icons::{ARROW_CIRCLE_DOWN, ARROW_CIRCLE_UP, BRIDGE, CALENDAR_CHECK, CHAT_CIRCLE_TEXT, CHECK_CIRCLE, DOTS_THREE_CIRCLE, FILE_TEXT, GEAR_FINE, PROHIBIT, X_CIRCLE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Root, View};
use crate::gui::views::wallets::types::WalletTab;
use crate::gui::views::wallets::wallet::types::WalletTabType;
use crate::gui::views::wallets::wallet::WalletContent;
use crate::wallet::Wallet;

/// Wallet info tab content.
#[derive(Default)]
pub struct WalletInfo;

impl WalletTab for WalletInfo {
    fn get_type(&self) -> WalletTabType {
        WalletTabType::Txs
    }

    fn ui(&mut self,
          ui: &mut egui::Ui,
          _: &mut eframe::Frame,
          wallet: &mut Wallet,
          _: &dyn PlatformCallbacks) {
        if WalletContent::sync_ui(ui, wallet) {
            return;
        }

        // Show wallet transactions panel.
        egui::CentralPanel::default()
            .frame(egui::Frame {
                stroke: View::ITEM_STROKE,
                fill: Colors::BUTTON,
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
                    self.txs_ui(ui, wallet);
                });
            });
    }
}

impl WalletInfo {
    /// Draw transactions content.
    fn txs_ui(&self, ui: &mut egui::Ui, wallet: &mut Wallet) {
        let data = wallet.get_data().unwrap();
        let txs_size = data.txs.len();

        // Show transactions info.
        View::max_width_ui(ui, Root::SIDE_PANEL_WIDTH * 1.35, |ui| {
            let amount_awaiting_conf = data.info.amount_awaiting_confirmation;
            let amount_awaiting_fin = data.info.amount_awaiting_finalization;
            let amount_locked = data.info.amount_locked;

            // Show non-zero awaiting confirmation amount.
            if amount_awaiting_conf != 0 {
                let awaiting_conf = amount_to_hr_string(amount_awaiting_conf, true);
                let rounding = if amount_awaiting_fin != 0 || amount_locked != 0 {
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
            if amount_awaiting_fin != 0 {
                let awaiting_conf = amount_to_hr_string(amount_awaiting_fin, true);
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

            // Show message when wallet txs are empty.
            if txs_size == 0 {
                View::center_content(ui, 96.0, |ui| {
                    let empty_text = t!(
                            "wallets.txs_empty",
                            "message" => CHAT_CIRCLE_TEXT,
                            "transport" => BRIDGE,
                            "settings" => GEAR_FINE
                        );
                    ui.label(RichText::new(empty_text).size(16.0).color(Colors::INACTIVE_TEXT));
                });
                return;
            }
        });

        // Show list of transactions.
        ui.add_space(3.0);
        ScrollArea::vertical()
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
            .id_source(Id::from("txs_content").with(wallet.get_config().id))
            .auto_shrink([false; 2])
            .show_rows(ui, TX_ITEM_HEIGHT, txs_size, |ui, row_range| {
                ui.add_space(4.0);
                View::max_width_ui(ui, Root::SIDE_PANEL_WIDTH * 1.3, |ui| {
                    for index in row_range {
                        let tx = data.txs.get(index).unwrap();
                        // Setup item rounding.
                        let item_rounding = View::item_rounding(index, txs_size, false);
                        // Show transaction item.
                        tx_item_ui(ui, tx, item_rounding, data.info.last_confirmed_height, wallet);
                    }
                });
                ui.add_space(2.0);
            });

        // for tx in &data.txs {
        //     if tx.tx_type != TxLogEntryType::TxReceivedCancelled && tx.tx_type != TxLogEntryType::TxSentCancelled {
        //         println!("tx: {}", serde_json::to_string::<TxLogEntry>(tx).unwrap());
        //     }
        // }
    }
}

/// Height of transaction list item.
const TX_ITEM_HEIGHT: f32 = 76.0;

/// Draw transaction item.
fn tx_item_ui(ui: &mut egui::Ui,
              tx: &TxLogEntry,
              mut rounding: Rounding,
              last_height: u64,
              wallet: &mut Wallet) {
    // Setup layout size.
    let mut rect = ui.available_rect_before_wrap();
    rect.min += egui::vec2(6.0, 0.0);
    rect.set_height(TX_ITEM_HEIGHT);

    // Draw round background.
    let bg_rect = rect.clone();
    ui.painter().rect(bg_rect, rounding, Colors::BUTTON, View::ITEM_STROKE);

    // Setup transaction flags.
    let is_canceled = tx.tx_type == TxLogEntryType::TxSentCancelled
        || tx.tx_type == TxLogEntryType::TxReceivedCancelled;
    let is_cancelling = wallet.is_cancelling(&tx.id);

    ui.vertical(|ui| {
        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(-6.0);
            // Draw button to show transaction info.
            rounding.nw = 0.0;
            rounding.sw = 0.0;
            View::item_button(ui, rounding, FILE_TEXT, None, || {
                //TODO: Show tx info
            });

            if !is_cancelling && !tx.confirmed && tx.tx_type != TxLogEntryType::TxReceivedCancelled
                && tx.tx_type != TxLogEntryType::TxSentCancelled {
                View::item_button(ui, Rounding::default(), PROHIBIT, Some(Colors::RED), || {
                    wallet.cancel(tx.id);
                });
            }

            let layout_size = ui.available_size();
            ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    // Setup transaction amount.
                    ui.add_space(3.0);
                    let amount_text = if tx.amount_credited > tx.amount_debited {
                        format!("+{}",
                                amount_to_hr_string(tx.amount_credited - tx.amount_debited, true))
                    } else {
                        format!("-{}",
                                amount_to_hr_string(tx.amount_debited - tx.amount_credited, true))
                    };

                    // Setup amount color.
                    let amount_color = match tx.tx_type {
                        TxLogEntryType::ConfirmedCoinbase => Colors::BLACK,
                        TxLogEntryType::TxReceived => Colors::BLACK,
                        TxLogEntryType::TxSent => Colors::BLACK,
                        TxLogEntryType::TxReceivedCancelled => Colors::TEXT,
                        TxLogEntryType::TxSentCancelled => Colors::TEXT,
                        TxLogEntryType::TxReverted => Colors::TEXT
                    };
                    View::ellipsize_text(ui, amount_text, 18.0, amount_color);
                    ui.add_space(-2.0);

                    // Setup transaction status text.
                    let status_text = if !tx.confirmed {
                        if wallet.is_cancelling(&tx.id) {
                            format!("{} {}", DOTS_THREE_CIRCLE, t!("wallets.tx_cancelling"))
                        } else if is_canceled {
                            format!("{} {}", X_CIRCLE, t!("wallets.tx_canceled"))
                        } else {
                            match tx.tx_type {
                                TxLogEntryType::TxReceived => {
                                    format!("{} {}", DOTS_THREE_CIRCLE, t!("wallets.tx_receiving"))
                                },
                                TxLogEntryType::TxSent => {
                                    format!("{} {}", DOTS_THREE_CIRCLE, t!("wallets.tx_sending"))
                                },
                                _ => {
                                    format!("{} {}", DOTS_THREE_CIRCLE, t!("wallets.tx_confirming"))
                                }
                            }
                        }
                    } else {
                        let tx_height = tx.kernel_lookup_min_height.unwrap_or(0);
                        let min_confirmations = wallet.get_config().min_confirmations;
                        match tx.tx_type {
                            TxLogEntryType::ConfirmedCoinbase => {
                                format!("{} {}", CHECK_CIRCLE, t!("wallets.tx_confirmed"))
                            },
                            TxLogEntryType::TxReceived => {
                                if last_height - tx_height > min_confirmations {
                                    format!("{} {}", ARROW_CIRCLE_DOWN, t!("wallets.tx_received"))
                                } else {
                                    format!("{} {}",
                                            DOTS_THREE_CIRCLE,
                                            t!("wallets.tx_confirming"))
                                }
                            },
                            TxLogEntryType::TxSent => {
                                if last_height - tx_height > min_confirmations {
                                    format!("{} {}", ARROW_CIRCLE_UP, t!("wallets.tx_sent"))
                                } else {
                                    format!("{} {}", DOTS_THREE_CIRCLE, t!("wallets.tx_confirming"))
                                }
                            },
                            _ => format!("{} {}", X_CIRCLE, t!("wallets.canceled"))
                        }
                    };

                    // Setup status text color.
                    let status_color = match tx.tx_type {
                        TxLogEntryType::ConfirmedCoinbase => Colors::TEXT,
                        TxLogEntryType::TxReceived => if tx.confirmed {
                            Colors::GREEN
                        } else {
                            Colors::TEXT
                        },
                        TxLogEntryType::TxSent => if tx.confirmed {
                            Colors::RED
                        } else {
                            Colors::TEXT
                        },
                        TxLogEntryType::TxReceivedCancelled => Colors::INACTIVE_TEXT,
                        TxLogEntryType::TxSentCancelled => Colors::INACTIVE_TEXT,
                        TxLogEntryType::TxReverted => Colors::INACTIVE_TEXT,
                    };
                    ui.label(RichText::new(status_text).size(15.0).color(status_color));

                    // Setup transaction time.
                    let tx_time = View::format_time(tx.creation_ts.timestamp());
                    let tx_time_text = format!("{} {}", CALENDAR_CHECK, tx_time);
                    ui.label(RichText::new(tx_time_text).size(15.0).color(Colors::GRAY));
                });
            });
        });
    });
}