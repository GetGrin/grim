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

use egui::{Id, Margin, RichText, ScrollArea, vec2};
use egui::scroll_area::ScrollBarVisibility;
use grin_util::ZeroingString;

use crate::built_info;
use crate::gui::Colors;
use crate::gui::icons::{CHECK, CLIPBOARD_TEXT, COPY, FOLDER_PLUS, SCAN, SHARE_FAT};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, Root, View};
use crate::gui::views::types::{ModalPosition, TextEditOptions};
use crate::gui::views::wallets::creation::MnemonicSetup;
use crate::gui::views::wallets::creation::types::Step;
use crate::gui::views::wallets::setup::ConnectionSetup;
use crate::node::Node;
use crate::wallet::{ExternalConnection, Wallet};
use crate::wallet::types::PhraseMode;

/// Wallet creation content.
pub struct WalletCreation {
    /// Wallet creation step.
    step: Option<Step>,

    /// Flag to check if wallet creation [`Modal`] was just opened to focus on first field.
    modal_just_opened: bool,
    /// Wallet name value.
    name_edit: String,
    /// Password to encrypt created wallet.
    pass_edit: String,

    /// Mnemonic phrase setup content.
    pub(crate) mnemonic_setup: MnemonicSetup,
    /// Network setup content.
    pub(crate) network_setup: ConnectionSetup
}

impl Default for WalletCreation {
    fn default() -> Self {
        Self {
            step: None,
            modal_just_opened: true,
            name_edit: String::from(""),
            pass_edit: String::from(""),
            mnemonic_setup: MnemonicSetup::default(),
            network_setup: ConnectionSetup::default()
        }
    }
}

impl WalletCreation {
    /// Wallet name/password input modal identifier.
    pub const NAME_PASS_MODAL: &'static str = "name_pass_modal";

    /// Draw wallet creation content.
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              frame: &mut eframe::Frame,
              cb: &dyn PlatformCallbacks,
              on_create: impl FnOnce(Wallet)) {
        // Show wallet creation step description and confirmation panel.
        if self.step.is_some() {
            egui::TopBottomPanel::bottom("wallet_creation_step_panel")
                .frame(egui::Frame {
                    stroke: View::DEFAULT_STROKE,
                    fill: Colors::FILL,
                    inner_margin: Margin {
                        left: View::far_left_inset_margin(ui) + 8.0,
                        right: View::get_right_inset() + 8.0,
                        top: 4.0,
                        bottom: View::get_bottom_inset(),
                    },
                    ..Default::default()
                })
                .show_inside(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.vertical_centered(|ui| {
                            View::max_width_ui(ui, Root::SIDE_PANEL_WIDTH * 2.0, |ui| {
                                self.step_control_ui(ui, on_create, cb);
                            });
                        });

                    });
                });
        }

        // Show wallet creation step content panel.
        egui::CentralPanel::default()
            .frame(egui::Frame {
                stroke: View::DEFAULT_STROKE,
                inner_margin: Margin {
                    left: View::far_left_inset_margin(ui) + 4.0,
                    right: View::get_right_inset() + 4.0,
                    top: 3.0,
                    bottom: 4.0,
                },
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                let id = if let Some(step) = &self.step {
                    format!("creation_step_scroll_{}", step.name())
                } else {
                    "creation_step_scroll".to_owned()
                };
                ScrollArea::vertical()
                    .id_source(id)
                    .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            let max_width = if self.step == Some(Step::SetupConnection) {
                                Root::SIDE_PANEL_WIDTH * 1.3
                            } else {
                                Root::SIDE_PANEL_WIDTH * 2.0
                            };
                            View::max_width_ui(ui, max_width, |ui| {
                                self.step_content_ui(ui, frame, cb);
                            });
                        });
                    });
            });
    }

    /// Draw [`Step`] description and confirmation control.
    fn step_control_ui(&mut self,
                       ui: &mut egui::Ui,
                       on_create: impl FnOnce(Wallet),
                       cb: &dyn PlatformCallbacks) {
        if let Some(step) = self.step.clone() {
            // Setup step description text and availability.
            let (step_text, mut step_available) = match step {
                Step::EnterMnemonic => {
                    let mode = &self.mnemonic_setup.mnemonic.mode;
                    let text = if mode == &PhraseMode::Generate {
                        t!("wallets.create_phrase_desc")
                    } else {
                        t!("wallets.restore_phrase_desc")
                    };
                    let available = !self
                        .mnemonic_setup
                        .mnemonic
                        .words
                        .contains(&String::from(""));
                    (text, available)
                }
                Step::ConfirmMnemonic => {
                    let text = t!("wallets.restore_phrase_desc");
                    let available = !self
                        .mnemonic_setup
                        .mnemonic
                        .confirm_words
                        .contains(&String::from(""));
                    (text, available)
                },
                Step::SetupConnection => (t!("wallets.setup_conn_desc"), true)
            };
            // Show step description.
            ui.add_space(2.0);
            ui.label(RichText::new(step_text).size(16.0).color(Colors::GRAY));
            ui.add_space(2.0);
            // Show error if entered phrase is not valid.
            if !self.mnemonic_setup.valid_phrase {
                step_available = false;
                ui.label(RichText::new(t!("wallets.not_valid_phrase"))
                    .size(16.0)
                    .color(Colors::RED));
                ui.add_space(2.0);
            }
            if step == Step::EnterMnemonic {
                ui.add_space(4.0);

                // Setup spacing between buttons.
                ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

                ui.columns(2, |columns| {
                    // Show copy or paste button for mnemonic phrase step.
                    columns[0].vertical_centered_justified(|ui| {
                        self.copy_or_paste_button_ui(ui, cb);
                    });

                    columns[1].vertical_centered_justified(|ui| {
                        if step_available {
                            // Show next step button if there are no empty words.
                            self.next_step_button_ui(ui, step, on_create);
                        } else {
                            // Show QR code scan button.
                            let scan_text = format!("{} {}", SCAN, t!("scan").to_uppercase());
                            View::button(ui, scan_text, Colors::WHITE, || {
                                self.mnemonic_setup.show_qr_scan_modal(cb);
                            });
                        }
                    });
                });
                ui.add_space(4.0);
            } else {
                if step_available {
                    ui.add_space(4.0);
                    self.next_step_button_ui(ui, step, on_create);
                    ui.add_space(4.0);
                }
            }
            ui.add_space(4.0);
        }
    }

    /// Draw copy or paste button at [`Step::EnterMnemonic`].
    fn copy_or_paste_button_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        match self.mnemonic_setup.mnemonic.mode {
            PhraseMode::Generate => {
                // Show copy button.
                let c_t = format!("{} {}", COPY, t!("copy").to_uppercase());
                View::button(ui, c_t.to_uppercase(), Colors::WHITE, || {
                    cb.copy_string_to_buffer(self.mnemonic_setup.mnemonic.get_phrase());
                });
            }
            PhraseMode::Import => {
                // Show paste button.
                let p_t = format!("{} {}", CLIPBOARD_TEXT, t!("paste").to_uppercase());
                View::button(ui, p_t, Colors::WHITE, || {
                    let data = ZeroingString::from(cb.get_string_from_buffer());
                    self.mnemonic_setup.mnemonic.import_text(&data);
                });
            }
        }
    }

    /// Draw button to go to next [`Step`].
    fn next_step_button_ui(&mut self,
                           ui: &mut egui::Ui,
                           step: Step,
                           on_create: impl FnOnce(Wallet)) {
        // Setup button text.
        let (next_text, color) = if step == Step::SetupConnection {
            (format!("{} {}", CHECK, t!("complete")), Colors::GOLD)
        } else {
            let text = format!("{} {}", SHARE_FAT, t!("continue"));
            (text, Colors::WHITE)
        };

        // Show next step button.
        View::button(ui, next_text.to_uppercase(), color, || {
            self.step = if let Some(step) = &self.step {
                match step {
                    Step::EnterMnemonic => {
                        if self.mnemonic_setup.mnemonic.mode == PhraseMode::Generate {
                            Some(Step::ConfirmMnemonic)
                        } else {
                            // Check if entered phrase was valid.
                            if self.mnemonic_setup.valid_phrase {
                                Some(Step::SetupConnection)
                            } else {
                                Some(Step::EnterMnemonic)
                            }
                        }
                    }
                    Step::ConfirmMnemonic => {
                        Some(Step::SetupConnection)
                    },
                    Step::SetupConnection => {
                        // Create wallet at last step.
                        let name = self.name_edit.clone();
                        let pass = self.pass_edit.clone();
                        let phrase = self.mnemonic_setup.mnemonic.get_phrase();
                        let conn_method = &self.network_setup.method;
                        let mut wallet = Wallet::create(name,
                                                        pass.clone(),
                                                        phrase,
                                                        conn_method).unwrap();
                        // Open created wallet.
                        wallet.open(pass).unwrap();
                        // Pass created wallet to callback.
                        (on_create)(wallet);
                        // Reset input data.
                        self.step = None;
                        self.name_edit = String::from("");
                        self.pass_edit = String::from("");
                        self.mnemonic_setup.reset();
                        None
                    }
                }
            } else {
                Some(Step::EnterMnemonic)
            };

            // Check external connections availability on connection setup.
            if self.step == Some(Step::SetupConnection) {
                ExternalConnection::start_ext_conn_availability_check();
            }
        });
    }

    /// Draw wallet creation [`Step`] content.
    fn step_content_ui(&mut self,
                       ui: &mut egui::Ui,
                       frame: &mut eframe::Frame,
                       cb: &dyn PlatformCallbacks) {
        match &self.step {
            None => {
                // Show wallet creation message if step is empty.
                View::center_content(ui, 350.0 + View::get_bottom_inset(), |ui| {
                    ui.add(
                        egui::Image::new(egui::include_image!("../../../../../img/logo.png"))
                            .fit_to_exact_size(vec2(180.0, 180.0))
                    );
                    ui.add_space(-15.0);
                    ui.label(RichText::new("GRIM")
                        .size(24.0)
                        .color(Colors::BLACK)
                    );
                    ui.label(RichText::new(built_info::PKG_VERSION)
                        .size(16.0)
                        .color(Colors::BLACK)
                    );
                    ui.add_space(4.0);
                    let text = t!("wallets.create_desc");
                    ui.label(RichText::new(text)
                        .size(16.0)
                        .color(Colors::GRAY)
                    );
                    ui.add_space(8.0);
                    let add_text = format!("{} {}", FOLDER_PLUS, t!("wallets.add"));
                    View::button(ui, add_text, Colors::WHITE, || {
                        self.show_name_pass_modal(cb);
                    });
                });
            }
            Some(step) => {
                match step {
                    Step::EnterMnemonic => self.mnemonic_setup.ui(ui, frame, cb),
                    Step::ConfirmMnemonic => self.mnemonic_setup.confirm_ui(ui, frame, cb),
                    Step::SetupConnection => {
                        // Redraw if node is running.
                        if Node::is_running() {
                            ui.ctx().request_repaint_after(Node::STATS_UPDATE_DELAY);
                        }
                        self.network_setup.create_ui(ui, frame, cb)
                    }
                }
            }
        }
    }

    /// Check if it's possible to go back for current step.
    pub fn can_go_back(&self) -> bool {
        self.step.is_some()
    }

    /// Back to previous wallet creation [`Step`].
    pub fn back(&mut self) {
        match &self.step {
            None => {}
            Some(step) => {
                match step {
                    Step::EnterMnemonic => {
                        self.step = None;
                        self.name_edit = String::from("");
                        self.pass_edit = String::from("");
                        self.mnemonic_setup.reset();
                    },
                    Step::ConfirmMnemonic => self.step = Some(Step::EnterMnemonic),
                    Step::SetupConnection => self.step = Some(Step::EnterMnemonic)
                }
            }
        }
    }

    /// Start wallet creation from showing [`Modal`] to enter name and password.
    pub fn show_name_pass_modal(&mut self, cb: &dyn PlatformCallbacks) {
        // Reset modal values.
        self.modal_just_opened = true;
        self.name_edit = t!("wallets.default_wallet");
        self.pass_edit = String::from("");
        // Show modal.
        Modal::new(Self::NAME_PASS_MODAL)
            .position(ModalPosition::CenterTop)
            .title(t!("wallets.add"))
            .show();
        cb.show_keyboard();
    }

    /// Draw creating wallet name/password input [`Modal`] content.
    pub fn name_pass_modal_ui(&mut self,
                              ui: &mut egui::Ui,
                              modal: &Modal,
                              cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("wallets.name"))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Show wallet name text edit.
            let mut name_edit_opts = TextEditOptions::new(Id::from(modal.id).with("name"))
                .no_focus();
            if self.modal_just_opened {
                self.modal_just_opened = false;
                name_edit_opts.focus = true;
            }
            View::text_edit(ui, cb, &mut self.name_edit, name_edit_opts);
            ui.add_space(8.0);

            ui.label(RichText::new(t!("wallets.pass"))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw wallet password text edit.
            let pass_text_edit_opts = TextEditOptions::new(Id::from(modal.id).with("pass"))
                .password()
                .no_focus();
            View::text_edit(ui, cb, &mut self.pass_edit, pass_text_edit_opts);
            ui.add_space(12.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::WHITE, || {
                        // Close modal.
                        cb.hide_keyboard();
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    let mut on_next = || {
                        // Check if input values are not empty.
                        if self.name_edit.is_empty() || self.pass_edit.is_empty() {
                            return;
                        }
                        self.step = Some(Step::EnterMnemonic);
                        cb.hide_keyboard();
                        modal.close();
                    };

                    // Go to next creation step on Enter button press.
                    View::on_enter_key(ui, || {
                        (on_next)();
                    });

                    View::button(ui, t!("continue"), Colors::WHITE, on_next);
                });
            });
            ui.add_space(6.0);
        });
    }
}
