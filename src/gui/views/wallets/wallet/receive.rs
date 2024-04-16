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

use crate::gui::Colors;
use crate::gui::icons::{BROOM, CLIPBOARD_TEXT, COPY, HAND_COINS, RECEIPT};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, Root, View};
use crate::gui::views::types::{ModalPosition, TextEditOptions};
use crate::gui::views::wallets::wallet::types::{SLATEPACK_MESSAGE_HINT, WalletTab, WalletTabType};
use crate::gui::views::wallets::wallet::WalletContent;
use crate::wallet::Wallet;

/// Receiving tab content.
pub struct WalletReceive {
    /// Flag to check if there is invoice transaction type.
    is_invoice: bool,

    /// Slatepack message from sender to create response message.
    message_edit: String,
    /// Generated Slatepack response message for sender.
    response_edit: String,
    /// Flag to check if there is an error happened on response creation.
    response_error: bool,

    /// Amount to receive for invoice transaction type.
    amount_edit: String,
    /// Generated Slatepack message for invoice transaction.
    request_edit: String,
    /// Flag to check if there is an error happened on invoice creation.
    request_error: bool,
    /// Slatepack message from sender to finalize transaction.
    finalization_edit: String,
    /// Flag to check if there is an error happened on transaction finalization.
    finalization_error: bool,
}

/// Identifier for invoice amount [`Modal`].
const INVOICE_AMOUNT_MODAL: &'static str = "invoice_amount_modal";

impl Default for WalletReceive {
    fn default() -> Self {
        Self {
            is_invoice: false,
            message_edit: "".to_string(),
            response_edit: "".to_string(),
            response_error: false,
            amount_edit: "".to_string(),
            request_edit: "".to_string(),
            request_error: false,
            finalization_edit: "".to_string(),
            finalization_error: false,
        }
    }
}

impl WalletTab for WalletReceive {
    fn get_type(&self) -> WalletTabType {
        WalletTabType::Receive
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

        // Show receiving content panel.
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
                    .id_source(Id::from("wallet_receive").with(wallet.get_config().id))
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            View::max_width_ui(ui, Root::SIDE_PANEL_WIDTH * 1.3, |ui| {
                                self.receive_ui(ui, wallet, cb);
                            });
                        });
                    });
            });
    }
}

impl WalletReceive {
    /// Draw receiving content.
    pub fn receive_ui(&mut self,
                      ui: &mut egui::Ui,
                      wallet: &mut Wallet,
                      cb: &dyn PlatformCallbacks) {
        ui.add_space(2.0);
        View::sub_title(ui, format!("{} {}", HAND_COINS, t!("wallets.manually")));
        View::horizontal_line(ui, Colors::ITEM_STROKE);
        ui.add_space(3.0);
        // Show manual receiving content.
        self.manual_ui(ui, wallet, cb);
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
                    INVOICE_AMOUNT_MODAL => {
                        Modal::ui(ui.ctx(), |ui, modal| {
                            self.invoice_amount_modal(ui, wallet, modal, cb);
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    /// Draw manual receiving content.
    fn manual_ui(&mut self, ui: &mut egui::Ui, wallet: &mut Wallet, cb: &dyn PlatformCallbacks) {
        ui.add_space(10.0);
        ui.columns(2, |columns| {
            let mut is_invoice = self.is_invoice;
            columns[0].vertical_centered(|ui| {
                View::radio_value(ui, &mut is_invoice, false, t!("wallets.receive"));
            });
            columns[1].vertical_centered(|ui| {
                View::radio_value(ui, &mut is_invoice, true, t!("wallets.invoice"));
            });
            if is_invoice != self.is_invoice {
                self.is_invoice = is_invoice;
                // Reset fields to default values on mode change.
                if is_invoice {
                    self.amount_edit = "".to_string();
                    self.request_edit = "".to_string();
                    self.request_error = false;
                    self.finalization_edit = "".to_string();
                    self.finalization_error = false;
                } else {
                    self.message_edit = "".to_string();
                    self.response_edit = "".to_string();
                    self.response_error = false;
                }
            }
        });
        ui.add_space(10.0);

        if self.is_invoice {
            // Show invoice creation content.
            self.manual_invoice_ui(ui, wallet, cb);
        } else {
            // Show manual transaction receiving content.
            self.manual_receive_ui(ui, wallet, cb);
        }
    }

    /// Draw manual receiving content.
    fn manual_receive_ui(&mut self,
                         ui: &mut egui::Ui,
                         wallet: &mut Wallet,
                         cb: &dyn PlatformCallbacks) {
        // Setup description.
        let response_empty = self.response_edit.is_empty();

        if self.response_error {
            ui.label(RichText::new(t!("wallets.receive_slatepack_err"))
                .size(16.0)
                .color(Colors::RED));
        } else {
            let desc_text = if response_empty {
                t!("wallets.receive_slatepack_desc")
            } else {
                t!("wallets.receive_send_slatepack")
            };
            ui.label(RichText::new(desc_text).size(16.0).color(Colors::INACTIVE_TEXT));
        }
        ui.add_space(6.0);

        // Setup Slatepack message text input.
        let message = if response_empty {
            &mut self.message_edit
        } else {
            &mut self.response_edit
        };

        View::horizontal_line(ui, Colors::ITEM_STROKE);
        ui.add_space(3.0);
        ScrollArea::vertical()
            .max_height(128.0)
            .id_source(Id::from("receive_input").with(wallet.get_config().id))
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(7.0);
                let message_before = message.clone();
                egui::TextEdit::multiline(message)
                    .font(egui::TextStyle::Small)
                    .desired_rows(5)
                    .interactive(response_empty)
                    .hint_text(SLATEPACK_MESSAGE_HINT)
                    .desired_width(f32::INFINITY)
                    .show(ui);
                // Clear an error when message changed.
                if &message_before != message {
                    self.response_error = false;
                }
                ui.add_space(6.0);
            });
        ui.add_space(2.0);
        View::horizontal_line(ui, Colors::ITEM_STROKE);
        ui.add_space(10.0);

        // Draw buttons to clear/copy/paste.
        let field_is_empty = self.message_edit.is_empty() && self.response_edit.is_empty();
        let columns_num = if !field_is_empty { 2 } else { 1 };
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(columns_num, |columns| {
                let first_column_content = |ui: &mut egui::Ui| {
                    if !self.response_edit.is_empty() && !self.response_error {
                        let clear_text = format!("{} {}", BROOM, t!("clear"));
                        View::button(ui, clear_text, Colors::BUTTON, || {
                            self.response_error = false;
                            self.message_edit.clear();
                            self.response_edit.clear();
                        });
                    } else {
                        let paste_text = format!("{} {}", CLIPBOARD_TEXT, t!("paste"));
                        View::button(ui, paste_text, Colors::BUTTON, || {
                            self.message_edit = cb.get_string_from_buffer();
                            self.response_error = false;
                        });
                    }
                };
                if columns_num == 1 {
                    columns[0].vertical_centered(first_column_content);
                } else {
                    columns[0].vertical_centered_justified(first_column_content);
                    columns[1].vertical_centered_justified(|ui| {
                        if self.response_error {
                            let clear_text = format!("{} {}", BROOM, t!("clear"));
                            View::button(ui, clear_text, Colors::BUTTON, || {
                                self.response_error = false;
                                self.message_edit.clear();
                                self.response_edit.clear();
                            });
                        } else if !self.response_edit.is_empty() {
                            let copy_text = format!("{} {}", COPY, t!("copy"));
                            View::button(ui, copy_text, Colors::BUTTON, || {
                                cb.copy_string_to_buffer(self.response_edit.clone());
                            });
                        } else {
                            View::button(ui, t!("wallets.create_response"), Colors::GOLD, || {
                                match wallet.receive(self.message_edit.clone()) {
                                    Ok(response) => {
                                        self.response_edit = response.trim().to_string();
                                        self.message_edit.clear();
                                        cb.copy_string_to_buffer(response);
                                    },
                                    Err(e) => {
                                        wallet.sync();
                                        println!("error {}", e);
                                        self.response_error = true
                                    }
                                }
                            });
                        }
                    });
                }
            });
        });
    }

    /// Draw invoice creation content.
    fn manual_invoice_ui(&mut self,
                         ui: &mut egui::Ui,
                         wallet: &mut Wallet,
                         cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("wallets.issue_invoice_desc"))
            .size(16.0)
            .color(Colors::INACTIVE_TEXT));
        ui.add_space(6.0);

        // Draw invoice creation button.
        let invoice_text = format!("{} {}", RECEIPT, t!("wallets.issue_invoice"));
        View::button(ui, invoice_text, Colors::BUTTON, || {
            // Reset modal values.
            self.amount_edit = "".to_string();
            self.request_error = false;
            // Show invoice amount modal.
            Modal::new(INVOICE_AMOUNT_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("wallets.issue_invoice"))
                .show();
            cb.show_keyboard();
        });

        ui.add_space(12.0);
        View::horizontal_line(ui, Colors::ITEM_STROKE);
        ui.add_space(6.0);
        ui.label(RichText::new(t!("wallets.receive_slatepack_desc"))
            .size(16.0)
            .color(Colors::INACTIVE_TEXT));
        ui.add_space(6.0);
        View::horizontal_line(ui, Colors::ITEM_STROKE);
        ui.add_space(3.0);

        // Draw invoice finalization text input.
        ScrollArea::vertical()
            .max_height(128.0)
            .id_source(Id::from("receive_input").with(wallet.get_config().id))
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(7.0);
                let finalization_before = self.finalization_edit.clone();
                egui::TextEdit::multiline(&mut self.finalization_edit)
                    .font(egui::TextStyle::Small)
                    .desired_rows(5)
                    .interactive(true)
                    .hint_text(SLATEPACK_MESSAGE_HINT)
                    .desired_width(f32::INFINITY)
                    .show(ui);
                // Clear an error when message changed.
                if finalization_before != self.finalization_edit {
                    self.finalization_error = false;
                }
                ui.add_space(6.0);
            });
        ui.add_space(2.0);
        View::horizontal_line(ui, Colors::ITEM_STROKE);
        ui.add_space(10.0);

        // Draw buttons to clear/paste.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    let paste_text = format!("{} {}", CLIPBOARD_TEXT, t!("paste"));
                    View::button(ui, paste_text, Colors::BUTTON, || {
                        self.finalization_edit = cb.get_string_from_buffer();
                        self.response_error = false;
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("wallets.finalize"), Colors::GOLD, || {
                        wallet.finalize();
                        //TODO: finalize
                    });
                });
            });
        });

        if self.finalization_error {
            ui.add_space(8.0);
            ui.label(RichText::new(t!("wallets.finalize_slatepack_err"))
                .size(16.0)
                .color(Colors::RED));
        }
        ui.add_space(8.0);
    }

    /// Draw invoice amount [`Modal`] content.
    fn invoice_amount_modal(&mut self,
                            ui: &mut egui::Ui,
                            wallet: &mut Wallet,
                            modal: &Modal,
                            cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        if self.request_edit.is_empty() {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(t!("wallets.enter_amount"))
                    .size(17.0)
                    .color(Colors::GRAY));
            });
            ui.add_space(8.0);

            // Draw invoice amount text edit.
            let amount_edit_id = Id::from(modal.id).with(wallet.get_config().id);
            let amount_edit_opts = TextEditOptions::new(amount_edit_id).h_center();
            let mut amount_edit = self.amount_edit.clone();
            View::text_edit(ui, cb, &mut amount_edit, amount_edit_opts);
            if amount_edit != self.amount_edit {
                self.request_error = false;
                match amount_from_hr_string(amount_edit.as_str()) {
                    Ok(_) => {
                        self.amount_edit = amount_edit;
                    }
                    Err(_) => {}
                }
            }

            // Show invoice creation error.
            if self.request_error {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("wallets.invoice_slatepack_err"))
                    .size(17.0)
                    .color(Colors::RED));
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
                            self.request_error = false;
                            modal.close();
                        });
                    });
                    columns[1].vertical_centered_justified(|ui| {
                        View::button(ui, t!("continue"), Colors::WHITE, || {
                            match amount_from_hr_string(self.amount_edit.as_str()) {
                                Ok(amount) => {
                                    match wallet.issue_invoice(amount) {
                                        Ok(message) => {
                                            self.request_edit = message;
                                            cb.hide_keyboard();
                                        }
                                        Err(_) => {
                                            self.request_error = true;
                                        }
                                    }
                                }
                                Err(_) => {
                                    self.request_error = true;
                                }
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
                let desc_text = t!("wallets.invoice_desc","amount" => amount_format);
                ui.label(RichText::new(desc_text).size(16.0).color(Colors::INACTIVE_TEXT));
                ui.add_space(6.0);
                View::horizontal_line(ui, Colors::ITEM_STROKE);
                ui.add_space(3.0);

                // Draw invoice request text.
                ScrollArea::vertical()
                    .max_height(128.0)
                    .id_source(Id::from("receive_input").with(wallet.get_config().id))
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