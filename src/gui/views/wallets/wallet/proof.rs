use egui::scroll_area::ScrollBarVisibility;
use egui::{Id, RichText, ScrollArea};
use grin_util::ToHex;
use grin_wallet_libwallet::{Error, PaymentProof, TxLogEntryType};

use crate::gui::icons::{BROOM, CLIPBOARD_TEXT, COPY, FILE_TEXT, SEAL_CHECK};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{FilePickContent, FilePickContentType, Modal, View};
use crate::gui::Colors;
use crate::wallet::types::{WalletTask, WalletTx};
use crate::wallet::Wallet;

pub struct PaymentProofContent {
    /// Payment proof text.
    input_edit: String,
    /// Button to pick payment proof file.
    pick_button: FilePickContent,
    /// Flag to check if an error occurred during proof parsing.
    parse_error: bool,
    /// Proof validation result.
    pub validation_result: Option<Result<(u32, bool, bool), Error>>,
}

impl PaymentProofContent {
    /// Create new content to share or validate payment proof.
    pub fn new(proof_text: Option<String>) -> Self {
        Self {
            input_edit: proof_text.unwrap_or("".to_string()),
            pick_button: FilePickContent::new(FilePickContentType::Button(t!("file").into())),
            parse_error: false,
            validation_result: None,
        }
    }

    /// Draw transaction payment proof input.
    pub fn input_ui(&mut self, ui: &mut egui::Ui, wallet: &Wallet, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            if self.parse_error {
                let label_text = t!("wallets.payment_proof_error");
                ui.label(RichText::new(label_text).size(16.0).color(Colors::red()));
            } else if let Some(proof) = self.validation_result.as_ref() {
                match proof {
                    Ok(_) => {
                        let label_text = t!("wallets.payment_proof_valid");
                        ui.label(RichText::new(label_text).size(16.0).color(Colors::green()));
                    }
                    Err(e) => {
                        let error_text = t!("wallets.payment_proof_error");
                        let label_text = format!("{} ({:?})", error_text, e);
                        ui.label(RichText::new(label_text).size(16.0).color(Colors::red()));
                    }
                }
            } else {
                let desc_label = t!("wallets.payment_proof_desc");
                ui.label(RichText::new(desc_label).size(16.0).color(Colors::inactive_text()));
            }
        });
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            let scroll_id = Id::from("tx_info_payment_proof_input");
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(3.0);
            ScrollArea::vertical()
                .id_salt(scroll_id)
                .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                .max_height(128.0)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.add_space(7.0);
                    let input_id = scroll_id.with("edit");
                    let proof_input_before = self.input_edit.clone();
                    let resp = egui::TextEdit::multiline(&mut self.input_edit)
                        .id(input_id)
                        .font(egui::TextStyle::Small)
                        .desired_rows(5)
                        .interactive(!wallet.payment_proof_verifying())
                        .desired_width(f32::INFINITY)
                        .show(ui)
                        .response;
                    if View::is_desktop() {
                        resp.request_focus();
                    }
                    // Parse payment proof on input change.
                    if self.input_edit != proof_input_before {
                        self.on_proof_edit_change(wallet);
                    }
                    ui.add_space(6.0);
                });
        });

        ui.add_space(2.0);
        View::horizontal_line(ui, Colors::item_stroke());
        ui.add_space(8.0);

        if wallet.payment_proof_verifying() {
            ui.vertical_centered(|ui| {
                View::small_loading_spinner(ui);
            });
        } else {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    if self.parse_error || (self.validation_result.is_some() &&
                        self.validation_result.as_ref().unwrap().is_err()) {
                        // Draw button to clear message input.
                        let clear_text = format!("{} {}", BROOM, t!("clear"));
                        View::button(ui, clear_text, Colors::white_or_black(false), || {
                            self.input_edit = "".to_string();
                            self.parse_error = false;
                            self.validation_result = None;
                        });
                    } else {
                        // Draw button to paste proof text.
                        let paste_text = format!("{} {}", CLIPBOARD_TEXT, t!("paste"));
                        View::button(ui, paste_text, Colors::white_or_black(false), || {
                            self.input_edit = cb.get_string_from_buffer();
                            self.on_proof_edit_change(wallet);
                        });
                    }
                });
                columns[1].vertical_centered_justified(|ui| {
                    let mut changed = false;
                    self.pick_button.ui(ui, cb, |data| {
                        self.input_edit = data.clone();
                        changed = true;
                    });
                    if changed {
                        self.on_proof_edit_change(wallet);
                    }
                });
            });
        }
    }

    /// Callback on payment proof input change.
    fn on_proof_edit_change(&mut self, wallet: &Wallet) {
        if wallet.payment_proof_verifying() {
            return;
        }
        if self.input_edit.is_empty() {
            self.parse_error = false;
            return;
        }
        if let Ok(p) = serde_json::from_str::<PaymentProof>(self.input_edit.as_str()) {
            wallet.task(WalletTask::VerifyProof(p, None));
        } else {
            self.parse_error = true;
        }
    }

    /// Draw transaction payment proof content to share.
    pub fn share_ui(&mut self,
                    ui: &mut egui::Ui,
                    tx: &WalletTx,
                    cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            let (desc_text, color) = if tx.data.tx_type == TxLogEntryType::TxReceived {
                (t!("wallets.payment_proof_valid").into(), Colors::green())
            } else {
                (format!("{}:", t!("wallets.payment_proof")), Colors::inactive_text())
            };
            let desc = format!("{} {}", SEAL_CHECK, desc_text);
            ui.label(RichText::new(desc).size(16.0).color(color));
        });
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            let scroll_id = Id::from("tx_info_payment_proof_share");
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(3.0);
            ScrollArea::vertical()
                .id_salt(scroll_id)
                .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                .max_height(128.0)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.add_space(7.0);
                    let input_id = scroll_id.with("edit");
                    egui::TextEdit::multiline(&mut self.input_edit)
                        .id(input_id)
                        .font(egui::TextStyle::Small)
                        .desired_rows(5)
                        .interactive(false)
                        .desired_width(f32::INFINITY)
                        .show(ui);
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
                // Draw copy button.
                let copy_text = format!("{} {}", COPY, t!("copy"));
                View::button(ui, copy_text, Colors::white_or_black(false), || {
                    cb.copy_string_to_buffer(self.input_edit.clone());
                    Modal::close();
                });
            });
            columns[1].vertical_centered_justified(|ui| {
                let share_text = format!("{} {}", FILE_TEXT, t!("share"));
                View::colored_text_button(ui,
                                          share_text,
                                          Colors::blue(),
                                          Colors::white_or_black(false), || {
                        let file_name = format!("{}.txt", tx.data.kernel_excess.unwrap().to_hex());
                        let data = self.input_edit.as_bytes().to_vec();
                        cb.share_data(file_name, data).unwrap_or_default();
                        Modal::close();
                    });
            });
        });
    }
}