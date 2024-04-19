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

use egui::{Id, Margin, RichText, ScrollArea};
use egui::scroll_area::ScrollBarVisibility;
use grin_core::core::{amount_from_hr_string, amount_to_hr_string};
use grin_wallet_libwallet::{Slate, SlateState, TxLogEntry};
use log::error;

use crate::gui::Colors;
use crate::gui::icons::{BROOM, CLIPBOARD_TEXT, COPY, DOWNLOAD, UPLOAD};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, Root, View};
use crate::gui::views::types::{ModalPosition, TextEditOptions};
use crate::gui::views::wallets::wallet::types::{SLATEPACK_MESSAGE_HINT, WalletTab, WalletTabType};
use crate::gui::views::wallets::wallet::WalletContent;
use crate::wallet::Wallet;

#[derive(Clone, Eq, PartialEq, Debug, thiserror::Error)]
enum MessageError {
    #[error("{0}")]
    Response(String),
    #[error("{0}")]
    Parse(String),
    #[error("{0}")]
    Finalize(String),
    #[error("{0}")]
    Other(String),
}

impl MessageError {
    pub fn text(&self) -> &String {
        match self {
            MessageError::Response(text) => text,
            MessageError::Parse(text) => text,
            MessageError::Finalize(text) => text,
            MessageError::Other(text) => text
        }
    }
}

/// Slatepacks messages interaction tab content.
pub struct WalletMessages {
    /// Flag to check if send or invoice request was opened.
    send_request: bool,

    /// Slatepack message to create response message.
    message_edit: String,
    /// Parsed Slatepack message.
    message_slate: Option<Slate>,
    /// Slatepack error on finalization, parse and response creation.
    message_error: Option<MessageError>,
    /// Generated Slatepack response message.
    response_edit: String,
    /// Flag to check if Dandelion is needed to finalize transaction.
    use_dandelion: Option<bool>,

    /// Amount to send or receive.
    amount_edit: String,
    /// Generated Slatepack message as request to send or receive funds.
    request_edit: String,
    /// Flag to check if there is an error happened on invoice creation.
    request_error: Option<MessageError>,
}

/// Identifier for invoice amount [`Modal`].
const AMOUNT_MODAL: &'static str = "amount_modal";

impl Default for WalletMessages {
    fn default() -> Self {
        Self {
            send_request: false,
            message_edit: "".to_string(),
            message_slate: None,
            message_error: None,
            response_edit: "".to_string(),
            use_dandelion: None,
            amount_edit: "".to_string(),
            request_edit: "".to_string(),
            request_error: None,
        }
    }
}

impl WalletTab for WalletMessages {
    fn get_type(&self) -> WalletTabType {
        WalletTabType::Messages
    }

    fn ui(&mut self,
          ui: &mut egui::Ui,
          _: &mut eframe::Frame,
          wallet: &mut Wallet,
          cb: &dyn PlatformCallbacks) {
        if WalletContent::sync_ui(ui, wallet) {
            return;
        }

        // Show modal content for this ui container.
        self.modal_content_ui(ui, wallet, cb);

        // Show manual wallet content panel.
        egui::CentralPanel::default()
            .frame(egui::Frame {
                stroke: View::ITEM_STROKE,
                fill: Colors::WHITE,
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
                    .scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
                    .id_source(Id::from("wallet_manual").with(wallet.get_config().id))
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            View::max_width_ui(ui, Root::SIDE_PANEL_WIDTH * 1.3, |ui| {
                                self.ui(ui, wallet, cb);
                            });
                        });
                    });
            });
    }
}

impl WalletMessages {
    /// Draw manual wallet transaction interaction content.
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              wallet: &mut Wallet,
              cb: &dyn PlatformCallbacks) {
        ui.add_space(4.0);

        // Show creation of request to send or receive funds.
        self.request_ui(ui, cb);

        ui.add_space(12.0);
        View::horizontal_line(ui, Colors::ITEM_STROKE);
        ui.add_space(8.0);

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
                    AMOUNT_MODAL => {
                        Modal::ui(ui.ctx(), |ui, modal| {
                            self.amount_modal_ui(ui, wallet, modal, cb);
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    /// Draw Slatepack message input content.
    fn input_slatepack_ui(&mut self,
                      ui: &mut egui::Ui,
                      wallet: &mut Wallet,
                      cb: &dyn PlatformCallbacks) {
        // Setup description.
        let response_empty = self.response_edit.is_empty();
        if let Some(err) = &self.message_error {
            ui.label(RichText::new(err.text()).size(16.0).color(Colors::RED));
        } else {
            let desc_text = if response_empty && self.message_slate.is_none() {
                t!("wallets.input_slatepack_desc")
            } else {
                let slate = self.message_slate.as_ref().unwrap();
                let amount = amount_to_hr_string(slate.amount, true);
                match slate.state {
                    SlateState::Standard1 => {
                        t!("wallets.parse_s1_slatepack_desc","amount" => amount)
                    }
                    SlateState::Standard2 => {
                        t!("wallets.parse_s2_slatepack_desc","amount" => amount)
                    }
                    SlateState::Invoice1 => {
                        t!("wallets.parse_i1_slatepack_desc","amount" => amount)
                    }
                    SlateState::Invoice2 => {
                        t!("wallets.parse_i2_slatepack_desc","amount" => amount)
                    }
                    _ => {
                        t!("wallets.input_slatepack_desc")
                    }
                }
            };
            ui.label(RichText::new(desc_text).size(16.0).color(Colors::INACTIVE_TEXT));
        }
        ui.add_space(7.0);

        // Setup Slatepack message text input.
        let mut message = if response_empty {
            &mut self.message_edit
        } else {
            &mut self.response_edit
        };

        // Save message to check for changes.
        let message_before = message.clone();

        View::horizontal_line(ui, Colors::ITEM_STROKE);
        ui.add_space(3.0);
        ScrollArea::vertical()
            .max_height(128.0)
            .id_source(Id::from(
                if response_empty {
                    "message_input"
                } else {
                    "response_input"
                }).with(wallet.get_config().id))
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(7.0);
                egui::TextEdit::multiline(message)
                    .font(egui::TextStyle::Small)
                    .desired_rows(5)
                    .interactive(response_empty)
                    .hint_text(SLATEPACK_MESSAGE_HINT)
                    .desired_width(f32::INFINITY)
                    .show(ui);
                ui.add_space(6.0);
            });
        ui.add_space(2.0);
        View::horizontal_line(ui, Colors::ITEM_STROKE);
        ui.add_space(10.0);

        // Draw buttons to clear/copy/paste.
        let fields_empty = self.message_edit.is_empty() && self.response_edit.is_empty();
        let columns_num = if fields_empty { 1 } else { 2 };
        let mut show_dandelion = false;
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(columns_num, |columns| {
                let first_column_content = |ui: &mut egui::Ui| {
                    if self.message_slate.is_some() && self.message_error.is_none() {
                        self.clear_message_button_ui(ui);
                    } else {
                        let paste_text = format!("{} {}", CLIPBOARD_TEXT, t!("paste"));
                        View::button(ui, paste_text, Colors::BUTTON, || {
                            self.message_edit = cb.get_string_from_buffer();
                        });
                    }
                };
                if columns_num == 1 {
                    columns[0].vertical_centered(first_column_content);
                } else {
                    columns[0].vertical_centered_justified(first_column_content);
                    columns[1].vertical_centered_justified(|ui| {
                        if self.message_error.is_some() {
                            self.clear_message_button_ui(ui);
                        } else if !self.response_edit.is_empty() {
                            let copy_text = format!("{} {}", COPY, t!("copy"));
                            View::button(ui, copy_text, Colors::BUTTON, || {
                                cb.copy_string_to_buffer(self.response_edit.clone());
                            });
                        } else {
                            show_dandelion = true;
                            View::button(ui, t!("wallets.finalize"), Colors::GOLD, || {
                                let message = self.message_edit.clone();
                                let use_dandelion = self.use_dandelion.unwrap();
                                if let Ok(_) = wallet.finalize(message, use_dandelion) {
                                    self.message_edit.clear();
                                    self.message_slate = None;

                                } else {
                                    self.message_error = Some(
                                        MessageError::Finalize(t!("wallets.finalize_slatepack_err"))
                                    );
                                }
                            });
                        }
                    });
                }
            });
        });

        // Draw setup of ability to post transaction with Dandelion.
        if show_dandelion {
            if self.use_dandelion.is_none() {
                self.use_dandelion = if let Some(u) = wallet.get_config().use_dandelion {
                    Some(u)
                } else {
                    Some(true)
                };
            }
            let use_dandelion = self.use_dandelion.unwrap();
            View::checkbox(ui, use_dandelion, t!("wallets.use_dandelion"), || {
                self.use_dandelion = Some(!use_dandelion);
                wallet.update_use_dandelion(use_dandelion);
            });
        }

        message = if response_empty {
            &mut self.message_edit
        } else {
            &mut self.response_edit
        };

        // Parse Slatepack message if input field was changed, resetting message error.
        if &message_before != message {
            self.message_error = None;
            self.parse_message(wallet);
        }
    }

    /// Parse message input into [`Slate`], making operations like receive or pay to confirm.
    fn parse_message(&mut self, wallet: &mut Wallet) {
        if self.message_edit.is_empty() {
           return;
        }
        if let Ok(mut slate) = wallet.parse_slatepack(self.message_edit.clone()) {
            println!("parse_message: {}", slate);
            // Make operation based on incoming state status.
            match slate.state {
                SlateState::Standard1 => {
                    if let Ok(resp) = wallet.receive(self.message_edit.clone()) {
                        self.response_edit = resp;
                    } else {
                        // Check if tx with same slate id already exists.
                        let mut exists_tx = false;
                        let _ = wallet.get_data().unwrap().txs.clone().iter().map(|tx| {
                            if tx.tx_slate_id == Some(slate.id) {
                                exists_tx= true;
                                self.message_error = Some(
                                    MessageError::Response(t!("wallets.response_exists_err"))
                                );
                            }
                            tx
                        }).collect::<Vec<&TxLogEntry>>();
                        if exists_tx {
                            return;
                        }

                        // Set default response error message.
                        self.message_error = Some(
                            MessageError::Response(t!("wallets.response_slatepack_err"))
                        );
                    }
                }
                SlateState::Invoice1 => {
                    match wallet.pay(self.message_edit.clone()) {
                        Ok(resp) => {
                            self.response_edit = resp;
                        }
                        Err(err) => {
                            match err {
                                grin_wallet_libwallet::Error::NotEnoughFunds {..} => {
                                    let amount = amount_to_hr_string(slate.amount, true);
                                    let a_t = t!("wallets.pay_balance_error", "amount" => amount);
                                    self.message_error = Some(MessageError::Other(a_t));
                                }
                                _ => {
                                    self.message_error = Some(
                                        MessageError::Response(t!("wallets.response_slatepack_err"))
                                    );
                                }
                            }
                        }
                    }
                }
                _ => {}
            }

            // Try to get amount  from transaction by id.
            if slate.amount == 0 {
                let _ = wallet.get_data().unwrap().txs.clone().iter().map(|tx| {
                    if tx.tx_slate_id == Some(slate.id) {
                        if slate.amount == 0 {
                            let amount = if tx.amount_debited > tx.amount_credited {
                                tx.amount_debited - tx.amount_credited
                            } else {
                                tx.amount_credited - tx.amount_debited
                            };
                            slate.amount = amount;
                        }
                    }
                    tx
                }).collect::<Vec<&TxLogEntry>>();
            }
            self.message_slate = Some(slate.clone());
        } else {
            self.message_slate = None;
            self.message_error = Some(MessageError::Parse(t!("wallets.response_slatepack_err")));
        }
    }

    /// Draw button to clear entered message, slate and errors.
    fn clear_message_button_ui(&mut self, ui: &mut egui::Ui) {
        let clear_text = format!("{} {}", BROOM, t!("clear"));
        View::button(ui, clear_text, Colors::BUTTON, || {
            self.message_error = None;
            self.message_edit.clear();
            self.response_edit.clear();
            self.message_slate = None;
        });
    }

    /// Draw creation of request to send or receive funds.
    fn request_ui(&mut self,
                  ui: &mut egui::Ui,
                  cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("wallets.create_request_desc"))
            .size(16.0)
            .color(Colors::INACTIVE_TEXT));
        ui.add_space(7.0);

        // Setup spacing between buttons.
        ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

        ui.columns(2, |columns| {
            columns[0].vertical_centered_justified(|ui| {
                // Draw send request creation button.
                let send_text = format!("{} {}", UPLOAD, t!("wallets.send"));
                View::button(ui, send_text.clone(), Colors::BUTTON, || {
                    // Setup modal values.
                    self.send_request = true;
                    self.amount_edit = "".to_string();
                    self.request_error = None;
                    // Show send amount modal.
                    Modal::new(AMOUNT_MODAL)
                        .position(ModalPosition::CenterTop)
                        .title(t!("wallets.send"))
                        .show();
                    cb.show_keyboard();
                });
            });
            columns[1].vertical_centered_justified(|ui| {
                // Draw invoice request creation button.
                let receive_text = format!("{} {}", DOWNLOAD, t!("wallets.receive"));
                View::button(ui, receive_text.clone(), Colors::BUTTON, || {
                    // Setup modal values.
                    self.send_request = false;
                    self.amount_edit = "".to_string();
                    self.request_error = None;
                    // Show receive amount modal.
                    Modal::new(AMOUNT_MODAL)
                        .position(ModalPosition::CenterTop)
                        .title(t!("wallets.receive"))
                        .show();
                    cb.show_keyboard();
                });
            });
        });
    }

    /// Draw invoice amount [`Modal`] content.
    fn amount_modal_ui(&mut self,
                       ui: &mut egui::Ui,
                       wallet: &mut Wallet,
                       modal: &Modal,
                       cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        if self.request_edit.is_empty() {
            ui.vertical_centered(|ui| {
                let enter_text = if self.send_request {
                    let data = wallet.get_data().unwrap();
                    let amount = amount_to_hr_string(data.info.amount_currently_spendable, true);
                    t!("wallets.enter_amount_send","amount" => amount)
                } else {
                    t!("wallets.enter_amount_receive")
                };
                ui.label(RichText::new(enter_text)
                    .size(17.0)
                    .color(Colors::GRAY));
            });
            ui.add_space(8.0);

            // Draw invoice amount text edit.
            let amount_edit_id = Id::from(modal.id).with(wallet.get_config().id);
            let amount_edit_opts = TextEditOptions::new(amount_edit_id).h_center();
            let mut amount_edit_before = self.amount_edit.clone();
            View::text_edit(ui, cb, &mut amount_edit_before, amount_edit_opts);

            // Check value if input was changed.
            if amount_edit_before != self.amount_edit {
                self.request_error = None;
                match amount_from_hr_string(amount_edit_before.as_str()) {
                    Ok(a) => {
                        if a <= 0 {
                            return;
                        }
                        // Do not input amount more than balance in sending.
                        if self.send_request {
                            let b = wallet.get_data().unwrap().info.amount_currently_spendable;
                            if b < a {
                                return;
                            }
                        }
                        self.amount_edit = amount_edit_before;
                    }
                    Err(_) => {}
                }
            }

            // Show invoice creation error.
            if self.request_error.is_some() {
                ui.add_space(12.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(self.request_error.clone().unwrap().text())
                        .size(17.0)
                        .color(Colors::RED));
                });
            }

            // Show modal buttons.
            ui.add_space(12.0);
            ui.scope(|ui| {
                // Setup spacing between buttons.
                ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 0.0);

                ui.columns(2, |columns| {
                    columns[0].vertical_centered_justified(|ui| {
                        View::button(ui, t!("modal.cancel"), Colors::WHITE, || {
                            self.amount_edit = "".to_string();
                            self.request_error = None;
                            modal.close();
                        });
                    });
                    columns[1].vertical_centered_justified(|ui| {
                        // Button to create Slatepack message for request.
                        View::button(ui, t!("continue"), Colors::WHITE, || {
                            if let Ok(a) = amount_from_hr_string(self.amount_edit.as_str()) {
                                let message = if self.send_request {
                                    wallet.send(a)
                                } else {
                                    wallet.issue_invoice(a)
                                };
                                match message {
                                    Ok(message) => {
                                        self.request_edit = message;
                                        cb.hide_keyboard();
                                    }
                                    Err(err) => {
                                        match err {
                                            grin_wallet_libwallet::Error::NotEnoughFunds { .. } => {
                                                let m = t!(
                                                    "wallets.pay_balance_error",
                                                    "amount" => self.amount_edit
                                                );
                                                self.request_error = Some(MessageError::Other(m));
                                            }
                                            _ => {
                                                let m = t!("wallets.invoice_slatepack_err");
                                                self.request_error = Some(MessageError::Other(m));
                                            }
                                        }
                                    }
                                }
                            } else {
                                self.request_error = Some(
                                    MessageError::Other(t!("wallets.invoice_slatepack_err"))
                                );
                            }
                        });
                    });
                });
            });
            ui.add_space(6.0);
        } else {
            ui.vertical_centered(|ui| {
                let amount = amount_from_hr_string(self.amount_edit.as_str()).unwrap();
                let amount_format = amount_to_hr_string(amount, true);
                let desc_text = if self.send_request {
                    t!("wallets.send_request_desc","amount" => amount_format)
                } else {
                    t!("wallets.invoice_desc","amount" => amount_format)
                };
                ui.label(RichText::new(desc_text).size(16.0).color(Colors::INACTIVE_TEXT));
                ui.add_space(6.0);
                View::horizontal_line(ui, Colors::ITEM_STROKE);
                ui.add_space(3.0);

                // Draw output Slatepack message text.
                let input_id = if self.send_request {
                    Id::from("send_request_output").with(wallet.get_config().id)
                } else {
                    Id::from("receive_request_output").with(wallet.get_config().id)
                };
                ScrollArea::vertical()
                    .max_height(128.0)
                    .id_source(input_id)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.add_space(7.0);
                        egui::TextEdit::multiline(&mut self.request_edit)
                            .font(egui::TextStyle::Small)
                            .desired_rows(5)
                            .interactive(false)
                            .hint_text(SLATEPACK_MESSAGE_HINT)
                            .desired_width(f32::INFINITY)
                            .show(ui);
                        ui.add_space(6.0);
                    });
                ui.add_space(2.0);
                View::horizontal_line(ui, Colors::ITEM_STROKE);
                ui.add_space(10.0);

                // Draw copy button.
                let copy_text = format!("{} {}", COPY, t!("copy"));
                View::button(ui, copy_text, Colors::BUTTON, || {
                    cb.copy_string_to_buffer(self.request_edit.clone());
                });
            });

            // Draw button to close modal.
            ui.add_space(12.0);
            ui.vertical_centered_justified(|ui| {
                View::button(ui, t!("close"), Colors::WHITE, || {
                    self.amount_edit = "".to_string();
                    self.request_edit = "".to_string();
                    modal.close();
                });
            });
            ui.add_space(6.0);
        }
    }
}