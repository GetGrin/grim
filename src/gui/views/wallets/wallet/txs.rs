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
use grin_wallet_libwallet::{Slate, SlateState, TxLogEntryType};

use crate::gui::Colors;
use crate::gui::icons::{ARROW_CIRCLE_DOWN, ARROW_CIRCLE_UP, ARROW_CLOCKWISE, ARROWS_CLOCKWISE, BRIDGE, CALENDAR_CHECK, CHAT_CIRCLE_TEXT, CHECK_CIRCLE, DOTS_THREE_CIRCLE, FILE_TEXT, GEAR_FINE, PROHIBIT, X_CIRCLE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Root, View};
use crate::gui::views::wallets::types::WalletTab;
use crate::gui::views::wallets::wallet::types::{GRIN, WalletTabType};
use crate::gui::views::wallets::wallet::WalletContent;
use crate::wallet::types::{WalletData, WalletTransaction};
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
        View::max_width_ui(ui, Root::SIDE_PANEL_WIDTH * 1.25, |ui| {
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
        ui.add_space(4.0);
        ScrollArea::vertical()
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
            .id_source(Id::from("txs_content").with(wallet.get_config().id))
            .auto_shrink([false; 2])
            .show_rows(ui, TX_ITEM_HEIGHT, txs_size, |ui, row_range| {
                ui.add_space(3.0);
                View::max_width_ui(ui, Root::SIDE_PANEL_WIDTH * 1.3, |ui| {
                    for index in row_range {
                        let tx = data.txs.get(index).unwrap();
                        // Setup item rounding.
                        let item_rounding = View::item_rounding(index, txs_size, false);
                        // Show transaction item.
                        tx_item_ui(ui, tx, item_rounding, &data, wallet);
                    }
                });
            });
    }
}

/// Height of transaction list item.
const TX_ITEM_HEIGHT: f32 = 76.0;

/// Draw transaction item.
fn tx_item_ui(ui: &mut egui::Ui,
              tx: &WalletTransaction,
              mut rounding: Rounding,
              data: &WalletData,
              wallet: &mut Wallet) {
    // Setup layout size.
    let mut rect = ui.available_rect_before_wrap();
    rect.set_height(TX_ITEM_HEIGHT);

    // Draw round background.
    let bg_rect = rect.clone();
    ui.painter().rect(bg_rect, rounding, Colors::BUTTON, View::ITEM_STROKE);

    ui.vertical(|ui| {
        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            // Draw button to show transaction info.
            rounding.nw = 0.0;
            rounding.sw = 0.0;
            View::item_button(ui, rounding, FILE_TEXT, None, || {
                //TODO: Show tx info
            });

            // Setup flag to repost unconfirmed posting transaction after min confirmation time.
            let last_height = data.info.last_confirmed_height;
            let min_conf = data.info.minimum_confirmations;
            let can_repost = tx.posting && tx.repost_height.is_some() &&
                    last_height - tx.repost_height.unwrap() > min_conf;

            // Draw cancel button for txs to repost or also non-cancelled, non-posting.
            if can_repost || (!tx.posting && !tx.data.confirmed &&
                tx.data.tx_type != TxLogEntryType::TxReceivedCancelled
                && tx.data.tx_type != TxLogEntryType::TxSentCancelled) {
                View::item_button(ui, Rounding::default(), PROHIBIT, Some(Colors::RED), || {
                    wallet.cancel(tx.data.id);
                });
            }

            // Draw button to repost transaction.
            if can_repost {
                View::item_button(ui,
                                  Rounding::default(),
                                  ARROW_CLOCKWISE,
                                  Some(Colors::GREEN), || {
                    // Create slate to check existing file.
                    let mut slate = Slate::blank(1, false);
                    slate.id = tx.data.tx_slate_id.unwrap();
                    slate.state = match tx.data.tx_type {
                        TxLogEntryType::TxReceived => SlateState::Invoice3,
                        _ => SlateState::Standard3
                    };
                    // Post tx after getting slate from slatepack file.
                    if let Some(sp) = wallet.read_slatepack(&slate) {
                        if let Ok(s) = wallet.parse_slatepack(sp) {
                            let _ = wallet.post(&s, wallet.can_use_dandelion());
                        }
                    }
                });
            }

            let layout_size = ui.available_size();
            ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                ui.add_space(8.0);
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
                    let status_text = if !tx.data.confirmed {
                        let is_canceled = tx.data.tx_type == TxLogEntryType::TxSentCancelled
                            || tx.data.tx_type == TxLogEntryType::TxReceivedCancelled;
                        if is_canceled {
                            format!("{} {}", X_CIRCLE, t!("wallets.tx_canceled"))
                        } else if tx.posting {
                            format!("{} {}", DOTS_THREE_CIRCLE, t!("wallets.tx_finalizing"))
                        } else {
                            match tx.data.tx_type {
                                TxLogEntryType::TxReceived => {
                                    format!("{} {}", DOTS_THREE_CIRCLE, t!("wallets.tx_receiving"))
                                },
                                TxLogEntryType::TxSent => {
                                    format!("{} {}", DOTS_THREE_CIRCLE, t!("wallets.tx_sending"))
                                },
                                _ => {
                                    format!("{} {}", DOTS_THREE_CIRCLE, t!("wallets.tx_confirmed"))
                                }
                            }
                        }
                    } else {
                        let tx_height = tx.data.kernel_lookup_min_height.unwrap_or(0);
                        match tx.data.tx_type {
                            TxLogEntryType::ConfirmedCoinbase => {
                                format!("{} {}", CHECK_CIRCLE, t!("wallets.tx_confirmed"))
                            },
                            TxLogEntryType::TxSent | TxLogEntryType::TxReceived => {
                                if data.info.last_confirmed_height - tx_height > min_conf {
                                    let (icon, text) = if tx.data.tx_type == TxLogEntryType::TxSent {
                                        (ARROW_CIRCLE_UP, t!("wallets.tx_sent"))
                                    } else {
                                        (ARROW_CIRCLE_DOWN, t!("wallets.tx_received"))
                                    };
                                    format!("{} {}", icon, text)
                                } else {
                                    let h = data.info.last_confirmed_height;
                                    let left_conf = h - tx_height;
                                    let conf_info = if h >= tx_height && left_conf <= min_conf {
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
                        TxLogEntryType::ConfirmedCoinbase => Colors::TEXT,
                        TxLogEntryType::TxReceived => if tx.data.confirmed {
                            Colors::GREEN
                        } else {
                            Colors::TEXT
                        },
                        TxLogEntryType::TxSent => if tx.data.confirmed {
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
                    let tx_time = View::format_time(tx.data.creation_ts.timestamp());
                    let tx_time_text = format!("{} {}", CALENDAR_CHECK, tx_time);
                    ui.label(RichText::new(tx_time_text).size(15.0).color(Colors::GRAY));
                });
            });
        });
    });
}