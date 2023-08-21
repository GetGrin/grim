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

use egui::{Id, Margin, RichText, ScrollArea, Widget};

use crate::gui::Colors;
use crate::gui::icons::{ARCHIVE_BOX, BROOM, CLIPBOARD_TEXT, COPY, HAND_COINS};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Root, View};
use crate::gui::views::wallets::wallet::types::{WalletTab, WalletTabType};
use crate::gui::views::wallets::wallet::WalletContent;
use crate::wallet::Wallet;

/// Receiving tab content.
pub struct WalletReceive {
    /// Slatepack text from sender to create response.
    message_edit: String,
    /// Generated Slatepack response.
    response_edit: String,
    /// Flag to check if there is an error happened on receive.
    receive_error: bool,
    /// Flag to check if response was copied to the clipboard.
    response_copied: bool,
}

impl Default for WalletReceive {
    fn default() -> Self {
        Self {
            message_edit: "".to_string(),
            response_edit: "".to_string(),
            receive_error: false,
            response_copied: false,
        }
    }
}

impl WalletTab for WalletReceive {
    fn get_type(&self) -> WalletTabType {
        WalletTabType::Receive
    }

    fn ui(&mut self,
          ui: &mut egui::Ui,
          frame: &mut eframe::Frame,
          wallet: &mut Wallet,
          cb: &dyn PlatformCallbacks) {
        if WalletContent::sync_ui(ui, frame, wallet) {
            return;
        }

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
                ui.vertical_centered(|ui| {
                    View::max_width_ui(ui, Root::SIDE_PANEL_WIDTH * 1.3, |ui| {
                        self.receive_ui(ui, wallet, cb);
                    });
                });
            });
    }
}

/// Hint for Slatepack Message input.
const RECEIVE_SLATEPACK_HINT: &'static str = "BEGINSLATEPACK.\n...\n...\n...\nENDSLATEPACK.";

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

        // Setup manual sending description.
        let response_empty = self.response_edit.is_empty();
        let desc_text = if response_empty {
            t!("wallets.receive_paste_slatepack")
        } else {
            t!("wallets.receive_send_slatepack")
        };
        ui.label(RichText::new(desc_text).size(16.0).color(Colors::INACTIVE_TEXT));
        ui.add_space(3.0);

        // Show Slatepack text input.
        let message = if response_empty {
            &mut self.message_edit
        } else {
            &mut self.response_edit
        };

        View::horizontal_line(ui, Colors::ITEM_STROKE);
        ui.add_space(3.0);
        ScrollArea::vertical()
            .max_height(128.0)
            .id_source(Id::from("receive_input").with(wallet.config.id))
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(7.0);
                let message_before = message.clone();
                egui::TextEdit::multiline(message)
                    .font(egui::TextStyle::Small)
                    .desired_rows(5)
                    .interactive(response_empty)
                    .hint_text(RECEIVE_SLATEPACK_HINT)
                    .desired_width(f32::INFINITY)
                    .show(ui);
                // Clear an error when message changed.
                if &message_before != message {
                    self.receive_error = false;
                }
                ui.add_space(6.0);
            });
        ui.add_space(2.0);
        View::horizontal_line(ui, Colors::ITEM_STROKE);
        ui.add_space(10.0);

        // Show receiving input control buttons.
        self.receive_buttons_ui(ui, wallet, cb);
    }

    /// Draw manual receiving input control buttons.
    fn receive_buttons_ui(&mut self,
                          ui: &mut egui::Ui,
                          wallet: &mut Wallet,
                          cb: &dyn PlatformCallbacks) {
        let field_is_empty = self.message_edit.is_empty() && self.response_edit.is_empty();
        let columns_num = if !field_is_empty { 2 } else { 1 };

        // Draw buttons to clear/copy/paste.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(columns_num, |columns| {
                let first_column_content = |ui: &mut egui::Ui| {
                    if !field_is_empty {
                        let clear_text = format!("{} {}", BROOM, t!("clear"));
                        View::button(ui, clear_text, Colors::BUTTON, || {
                            self.receive_error = false;
                            self.response_copied = false;
                            self.message_edit.clear();
                            self.response_edit.clear();
                        });
                    } else if self.message_edit.is_empty() {
                        let paste_text = format!("{} {}", CLIPBOARD_TEXT, t!("paste"));
                        View::button(ui, paste_text, Colors::BUTTON, || {
                            self.message_edit = cb.get_string_from_buffer();
                            self.receive_error = false;
                        });
                    }
                };
                if columns_num == 1 {
                    columns[0].vertical_centered(first_column_content);
                } else {
                    columns[0].vertical_centered_justified(first_column_content);
                }
                if !field_is_empty {
                    columns[1].vertical_centered_justified(|ui| {
                        if !self.message_edit.is_empty() {
                            let paste_text = format!("{} {}", CLIPBOARD_TEXT, t!("paste"));
                            View::button(ui, paste_text, Colors::BUTTON, || {
                                self.message_edit = cb.get_string_from_buffer();
                                self.receive_error = false;
                            });
                        } else if !self.response_edit.is_empty() {
                            let copy_text = format!("{} {}", COPY, t!("copy"));
                            View::button(ui, copy_text, Colors::BUTTON, || {
                                cb.copy_string_to_buffer(self.response_edit.clone());
                                self.response_copied = true;
                            });
                        }
                    });
                }
            });
        });

        // Draw button to create response.
        if !self.message_edit.is_empty() && !self.receive_error {
            ui.add_space(8.0);
            let create_text = format!("{} {}", ARCHIVE_BOX,  t!("wallets.create_response"));
            View::button(ui, create_text, Colors::GOLD, || {
                match wallet.receive(self.message_edit.clone()) {
                    Ok(response) => {
                        self.response_edit = response.trim().to_string();
                        self.message_edit.clear();
                        // Copy response to clipboard.
                        cb.copy_string_to_buffer(response);
                        self.response_copied = true;
                    },
                    Err(_) => self.receive_error = true
                }
            });
            ui.add_space(8.0);
        } else if self.receive_error {
            ui.add_space(8.0);
            ui.label(RichText::new(t!("wallets.receive_slatepack_err"))
                .size(16.0)
                .color(Colors::RED));
            ui.add_space(8.0);
        } else if self.response_copied {
            ui.add_space(8.0);
            ui.label(RichText::new(t!("wallets.response_copied"))
                .size(16.0)
                .color(Colors::GREEN));
            ui.add_space(8.0);
        }
    }
}