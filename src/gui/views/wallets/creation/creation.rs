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
use grin_util::ZeroingString;

use crate::gui::Colors;
use crate::gui::icons::{CHECK, CLIPBOARD_TEXT, COPY, SCAN};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, Content, View, CameraScanModal};
use crate::gui::views::types::{LinePosition, ModalContainer, ModalPosition, QrScanResult};
use crate::gui::views::wallets::creation::MnemonicSetup;
use crate::gui::views::wallets::creation::types::Step;
use crate::gui::views::wallets::ConnectionSettings;
use crate::node::Node;
use crate::wallet::{ExternalConnection, Wallet};
use crate::wallet::types::PhraseMode;

/// Wallet creation content.
pub struct WalletCreation {
    /// Wallet name.
    pub name: String,
    /// Wallet password.
    pub pass: ZeroingString,

    /// Wallet creation step.
    step: Step,

    /// QR code scanning [`Modal`] content.
    scan_modal_content: Option<CameraScanModal>,

    /// Mnemonic phrase setup content.
    mnemonic_setup: MnemonicSetup,
    /// Network setup content.
    network_setup: ConnectionSettings,

    /// Flag to check if an error occurred during wallet creation.
    creation_error: Option<String>,

    /// [`Modal`] identifiers allowed at this ui container.
    modal_ids: Vec<&'static str>
}

const QR_CODE_PHRASE_SCAN_MODAL: &'static str = "qr_code_rec_phrase_scan_modal";

impl ModalContainer for WalletCreation {
    fn modal_ids(&self) -> &Vec<&'static str> {
        &self.modal_ids
    }

    fn modal_ui(&mut self,
                ui: &mut egui::Ui,
                modal: &Modal,
                cb: &dyn PlatformCallbacks) {
        match modal.id {
            QR_CODE_PHRASE_SCAN_MODAL => {
                if let Some(content) = self.scan_modal_content.as_mut() {
                    content.ui(ui, modal, cb, |result| {
                        match result {
                            QrScanResult::Text(text) => {
                                self.mnemonic_setup.mnemonic.import(&text);
                                modal.close();
                            }
                            QrScanResult::SeedQR(text) => {
                                self.mnemonic_setup.mnemonic.import(&text);
                                modal.close();
                            }
                            _ => {}
                        }
                    });
                }
            },

            _ => {}
        }
    }
}

impl WalletCreation {
    /// Create new wallet creation instance from name and password.
    pub fn new(name: String, pass: ZeroingString) -> Self {
        Self {
            name,
            pass,
            step: Step::EnterMnemonic,
            scan_modal_content: None,
            mnemonic_setup: MnemonicSetup::default(),
            network_setup: ConnectionSettings::default(),
            creation_error: None,
            modal_ids: vec![
                QR_CODE_PHRASE_SCAN_MODAL
            ],
        }
    }

    /// Draw wallet creation content.
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              cb: &dyn PlatformCallbacks,
              on_create: impl FnMut(Wallet)) {
        self.current_modal_ui(ui, cb);

        egui::TopBottomPanel::bottom("wallet_creation_step_panel")
            .frame(egui::Frame {
                inner_margin: Margin {
                    left: (View::far_left_inset_margin(ui) + View::TAB_ITEMS_PADDING) as i8,
                    right: (View::get_right_inset() + View::TAB_ITEMS_PADDING) as i8,
                    top: View::TAB_ITEMS_PADDING as i8,
                    bottom: (View::get_bottom_inset() + View::TAB_ITEMS_PADDING) as i8,
                },
                fill: Colors::fill_deep(),
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                // Draw divider line.
                let rect = {
                    let mut r = ui.available_rect_before_wrap();
                    r.min.y -= View::TAB_ITEMS_PADDING;
                    r.min.x -= View::far_left_inset_margin(ui) + View::TAB_ITEMS_PADDING;
                    r.max.x += View::get_right_inset() + View::TAB_ITEMS_PADDING;
                    r
                };
                View::line(ui, LinePosition::TOP, &rect, Colors::item_stroke());
                // Show step control content.
                View::max_width_ui(ui, Content::SIDE_PANEL_WIDTH * 1.3, |ui| {
                    self.step_control_ui(ui, on_create, cb);
                });
            });

        // Show wallet creation step content panel.
        egui::CentralPanel::default()
            .frame(egui::Frame {
                inner_margin: Margin {
                    left: (View::far_left_inset_margin(ui) + 4.0) as i8,
                    right: (View::get_right_inset() + 4.0) as i8,
                    top: 3.0 as i8,
                    bottom: 4.0 as i8,
                },
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                ScrollArea::vertical()
                    .id_salt(Id::from(format!("creation_step_scroll_{}", self.step.name())))
                    .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        let max_width = if self.step == Step::SetupConnection {
                            Content::SIDE_PANEL_WIDTH * 1.3
                        } else {
                            Content::SIDE_PANEL_WIDTH * 2.0
                        };
                        View::max_width_ui(ui, max_width, |ui| {
                            self.step_content_ui(ui, cb);
                        });
                    });
            });
    }

    /// Draw [`Step`] description and confirmation control.
    fn step_control_ui(&mut self,
                       ui: &mut egui::Ui,
                       on_create: impl FnOnce(Wallet),
                       cb: &dyn PlatformCallbacks) {
        let step = &self.step;
        // Setup description and next step availability.
        let (step_text, mut next) = match step {
            Step::EnterMnemonic => {
                let mode = &self.mnemonic_setup.mnemonic.mode();
                let (text, available) = match mode {
                    PhraseMode::Generate => (t!("wallets.create_phrase_desc"), true),
                    PhraseMode::Import => {
                        let available = !self.mnemonic_setup.mnemonic.has_empty_or_invalid();
                        (t!("wallets.restore_phrase_desc"), available)
                    }
                };
                (text, available)
            }
            Step::ConfirmMnemonic => {
                let text = t!("wallets.restore_phrase_desc");
                let available = !self.mnemonic_setup.mnemonic.has_empty_or_invalid();
                (text, available)
            }
            Step::SetupConnection => {
                (t!("wallets.setup_conn_desc"), self.creation_error.is_none())
            }
        };

        // Show step description or error.
        let generate_step = step == &Step::EnterMnemonic &&
            self.mnemonic_setup.mnemonic.mode() == PhraseMode::Generate;
        if (self.mnemonic_setup.mnemonic.valid() && self.creation_error.is_none()) ||
            generate_step {
            ui.label(RichText::new(step_text).size(16.0).color(Colors::gray()));
            ui.add_space(6.0);
        } else {
            next = false;
            // Show error text.
            if let Some(err) = &self.creation_error {
                ui.add_space(10.0);
                ui.label(RichText::new(err)
                    .size(16.0)
                    .color(Colors::red()));
                ui.add_space(10.0);
            } else {
                ui.label(RichText::new(&t!("wallets.not_valid_phrase"))
                    .size(16.0)
                    .color(Colors::red()));
                ui.add_space(4.0);
            };
        }

        // Setup spacing between buttons.
        ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 0.0);
        // Setup vertical padding inside button.
        ui.style_mut().spacing.button_padding = egui::vec2(10.0, 7.0);

        match step {
            Step::EnterMnemonic => {
                ui.columns(2, |columns| {
                    // Show copy or paste button for mnemonic phrase step.
                    columns[0].vertical_centered_justified(|ui| {
                        match self.mnemonic_setup.mnemonic.mode() {
                            PhraseMode::Generate => {
                                let c_t = format!("{} {}",
                                                  COPY,
                                                  t!("copy").to_uppercase());
                                View::button(ui, c_t, Colors::white_or_black(false), || {
                                    cb.copy_string_to_buffer(self.mnemonic_setup
                                        .mnemonic
                                        .get_phrase());
                                });
                            }
                            PhraseMode::Import => {
                                let p_t = format!("{} {}",
                                                  CLIPBOARD_TEXT,
                                                  t!("paste").to_uppercase());
                                View::button(ui, p_t, Colors::white_or_black(false), || {
                                    let data = ZeroingString::from(cb.get_string_from_buffer());
                                    self.mnemonic_setup.mnemonic.import(&data);
                                });
                            }
                        }
                    });
                    // Show next step or QR code scan button.
                    columns[1].vertical_centered_justified(|ui| {
                        if next {
                            self.next_step_button_ui(ui, on_create);
                        } else {
                            let scan_text = format!("{} {}",
                                                    SCAN,
                                                    t!("scan").to_uppercase());
                            View::button(ui, scan_text, Colors::white_or_black(false), || {
                                self.scan_modal_content = Some(CameraScanModal::default());
                                // Show QR code scan modal.
                                Modal::new(QR_CODE_PHRASE_SCAN_MODAL)
                                    .position(ModalPosition::CenterTop)
                                    .title(t!("scan_qr"))
                                    .closeable(false)
                                    .show();
                                cb.start_camera();
                            });
                        }
                    });
                });
            }
            Step::ConfirmMnemonic => {
                // Show next step or paste button.
                if next {
                    self.next_step_button_ui(ui, on_create);
                } else {
                    let paste_text = format!("{} {}", CLIPBOARD_TEXT, t!("paste").to_uppercase());
                    View::button(ui, paste_text, Colors::white_or_black(false), || {
                        let data = ZeroingString::from(cb.get_string_from_buffer());
                        self.mnemonic_setup.mnemonic.import(&data);
                    });
                }
            }
            Step::SetupConnection => {
                if next {
                    self.next_step_button_ui(ui, on_create);
                    ui.add_space(2.0);
                }
            }
        }
    }

    /// Draw button to go to next [`Step`].
    fn next_step_button_ui(&mut self,
                           ui: &mut egui::Ui,
                           on_create: impl FnOnce(Wallet)) {
        // Setup button text.
        let (next_text, text_color, bg_color) = if self.step == Step::SetupConnection {
            (format!("{} {}", CHECK, t!("complete")), Colors::title(true), Colors::gold())
        } else {
            (t!("continue"), Colors::green(), Colors::white_or_black(false))
        };

        // Show next step button.
        View::colored_text_button_ui(ui, next_text.to_uppercase(), text_color, bg_color, |ui| {
            self.step = match self.step {
                Step::EnterMnemonic => {
                    if self.mnemonic_setup.mnemonic.mode() == PhraseMode::Generate {
                        Step::ConfirmMnemonic
                    } else {
                        Step::SetupConnection
                    }
                }
                Step::ConfirmMnemonic => {
                    Step::SetupConnection
                },
                Step::SetupConnection => {
                    // Create wallet at last step.
                    match Wallet::create(&self.name,
                                         &self.pass,
                                         &self.mnemonic_setup.mnemonic,
                                         &self.network_setup.method) {
                        Ok(w) => {
                            self.mnemonic_setup.reset();
                            // Pass created wallet to callback.
                            (on_create)(w);
                            Step::EnterMnemonic
                        }
                        Err(e) => {
                            self.creation_error = Some(format!("{:?}", e));
                            Step::SetupConnection
                        }
                    }
                }
            };

            // Check external connections availability on connection setup.
            if self.step == Step::SetupConnection {
                ExternalConnection::check(None, ui.ctx());
            }
        });
    }

    /// Draw wallet creation [`Step`] content.
    fn step_content_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        match &self.step {
            Step::EnterMnemonic => self.mnemonic_setup.ui(ui, cb),
            Step::ConfirmMnemonic => self.mnemonic_setup.confirm_ui(ui, cb),
            Step::SetupConnection => {
                // Redraw if node is running.
                if Node::is_running() && !Content::is_dual_panel_mode(ui.ctx()) {
                    ui.ctx().request_repaint_after(Node::STATS_UPDATE_DELAY);
                }
                self.network_setup.create_ui(ui, cb)
            }
        }
    }

    /// Back to previous wallet creation [`Step`], return `true` to close creation.
    pub fn on_back(&mut self) -> bool {
        match &self.step {
            Step::ConfirmMnemonic => {
                self.step = Step::EnterMnemonic;
                false
            },
            Step::SetupConnection => {
                self.creation_error = None;
                self.step = Step::EnterMnemonic;
                false
            }
            _ => true
        }
    }
}