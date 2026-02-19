// Copyright 2026 The Grim Developers
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
use egui::{Id, RichText, ScrollArea};

use crate::gui::icons::{BROOM, CLIPBOARD_TEXT, SCAN};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{CameraContent, FilePickContent, FilePickContentType, Modal, View};
use crate::gui::Colors;
use crate::wallet::types::WalletTask;
use crate::wallet::Wallet;

pub struct MessageInputContent {
    /// Slatepack input text.
    message_edit: String,
    /// Flag to check if error happened at Slatepack message parsing.
    parse_error: bool,

    /// QR code scanner content.
    scan_qr_content: Option<CameraContent>,

    /// Button to parse picked file content.
    file_pick_button: FilePickContent,
}

/// Hint for Slatepack message input.
const SLATEPACK_MESSAGE_HINT: &'static str = "BEGINSLATEPACK.\n...\n...\n...\nENDSLATEPACK.";

impl Default for MessageInputContent {
    fn default() -> Self {
        Self {
            message_edit: "".to_string(),
            parse_error: false,
            scan_qr_content: None,
            file_pick_button: FilePickContent::new(FilePickContentType::Button),
        }
    }
}

impl MessageInputContent {
    /// Draw [`Modal`] content.
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              wallet: &Wallet,
              modal: &Modal,
              cb: &dyn PlatformCallbacks) {
        if let Some(scan_content) = self.scan_qr_content.as_mut() {
            if let Some(result) = scan_content.qr_scan_result() {
                cb.stop_camera();
                modal.enable_closing();
                self.scan_qr_content = None;
                // Parse scan result.
                self.on_message_input(result.text(), wallet);
            } else {
                scan_content.ui(ui, cb);
            }
            ui.add_space(8.0);

            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            // Show buttons to close modal or scanner.
            ui.columns(2, |cols| {
                cols[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("close"), Colors::white_or_black(false), || {
                        cb.stop_camera();
                        self.scan_qr_content = None;
                        Modal::close();
                    });
                });
                cols[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("back"), Colors::white_or_black(false), || {
                        cb.stop_camera();
                        self.scan_qr_content = None;
                        modal.enable_closing();
                    });
                });
            });
        } else {
            ui.add_space(6.0);
            ui.vertical_centered(|ui| {
                let (text, color) = if self.parse_error {
                    (t!("wallets.parse_slatepack_err"), Colors::red())
                } else {
                   (t!("wallets.input_slatepack_desc"), Colors::gray())
                };
                ui.label(RichText::new(text).size(16.0).color(color));
            });
            ui.add_space(6.0);

            // Draw slatepack message content.
            let message_before = self.message_edit.clone();
            ui.vertical_centered(|ui| {
                let scroll_id = Id::from("message_input").with(wallet.identifier());
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
                        egui::TextEdit::multiline(&mut self.message_edit)
                            .id(input_id)
                            .font(egui::TextStyle::Small)
                            .desired_rows(5)
                            .interactive(true)
                            .hint_text(SLATEPACK_MESSAGE_HINT)
                            .desired_width(f32::INFINITY)
                            .show(ui).response;
                        ui.add_space(6.0);
                    });
            });
            // Parse message on input change.
            if message_before != self.message_edit {
                self.on_message_input(self.message_edit.clone(), wallet);
            }

            ui.add_space(2.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(8.0);

            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    if self.parse_error {
                        // Draw button to clear message input.
                        let clear_text = format!("{} {}", BROOM, t!("clear"));
                        View::button(ui, clear_text, Colors::white_or_black(false), || {
                            self.message_edit = "".to_string();
                            self.parse_error = false;
                        });
                    } else {
                        // Draw button to scan Slatepack message QR code.
                        let scan_text = format!("{} {}", SCAN, t!("scan"));
                        View::button(ui, scan_text, Colors::white_or_black(false), || {
                            self.scan_qr_content = Some(CameraContent::default());
                            cb.start_camera();
                        });
                    }
                });
                columns[1].vertical_centered_justified(|ui| {
                    // Draw paste button.
                    let paste_text = format!("{} {}", CLIPBOARD_TEXT, t!("paste"));
                    View::button(ui, paste_text, Colors::white_or_black(false), || {
                        self.on_message_input(cb.get_string_from_buffer(), wallet);
                    });
                });
            });

            // Draw button to pick Slatepack message file.
            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                let mut picked_data = None;
                self.file_pick_button.ui(ui, cb, |data| {
                    picked_data = Some(data);
                });
                if let Some(data) = picked_data {
                    self.on_message_input(data, wallet);
                }
            });

            ui.add_space(8.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(8.0);

            // Show button to close modal.
            ui.vertical_centered_justified(|ui| {
                View::button(ui, t!("close"), Colors::white_or_black(false), || {
                    self.message_edit = "".to_string();
                    Modal::close();
                });
            });
        }
        ui.add_space(6.0);
    }

    /// Parse Slatepack message on input change.
    fn on_message_input(&mut self, text: String, wallet: &Wallet) {
        self.parse_error = false;
        self.message_edit = text;
        if self.message_edit.is_empty() {
            return;
        }
        match wallet.parse_slatepack(&self.message_edit) {
            Ok(_) => {
                wallet.task(WalletTask::OpenMessage(self.message_edit.to_string()));
                self.message_edit = "".to_string();
                Modal::close();
            }
            Err(_) => {
                self.parse_error = true;
            }
        }
    }
}