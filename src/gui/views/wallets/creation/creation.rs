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

use egui::{Margin, RichText, TextStyle, vec2, Widget};
use egui_extras::{RetainedImage, Size, StripBuilder};

use crate::built_info;
use crate::gui::Colors;
use crate::gui::icons::{CHECK, EYE, EYE_SLASH, PLUS_CIRCLE, SHARE_FAT};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, ModalPosition, View};
use crate::gui::views::wallets::creation::MnemonicSetup;
use crate::gui::views::wallets::creation::types::{PhraseMode, Step};
use crate::gui::views::wallets::setup::ConnectionSetup;
use crate::wallet::WalletList;

/// Wallet creation content.
pub struct WalletCreation {
    /// Wallet creation step.
    step: Option<Step>,

    /// Flag to check if [`Modal`] just was opened to focus on first field.
    modal_just_opened: bool,
    /// Wallet name value.
    name_edit: String,
    /// Password to encrypt created wallet.
    pass_edit: String,
    /// Flag to show/hide password at [`egui::TextEdit`] field.
    hide_pass: bool,

    /// Mnemonic phrase setup content.
    pub(crate) mnemonic_setup: MnemonicSetup,
    /// Network setup content.
    pub(crate) network_setup: ConnectionSetup,

    /// App logo image.
    logo: RetainedImage,
}

impl Default for WalletCreation {
    fn default() -> Self {
        Self {
            step: None,
            modal_just_opened: true,
            name_edit: String::from(""),
            pass_edit: String::from(""),
            hide_pass: true,
            mnemonic_setup: MnemonicSetup::default(),
            network_setup: ConnectionSetup::default(),
            logo: RetainedImage::from_image_bytes(
                "logo.png",
                include_bytes!("../../../../../img/logo.png"),
            ).unwrap(),
        }
    }
}

impl WalletCreation {
    /// Wallet name/password input modal identifier.
    pub const NAME_PASS_MODAL: &'static str = "name_pass_modal";

    pub fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        // Show wallet creation step description and confirmation panel.
        if self.step.is_some() {
            egui::TopBottomPanel::bottom("wallet_creation_step_panel")
                .frame(egui::Frame {
                    stroke: View::DEFAULT_STROKE,
                    fill: Colors::FILL_DARK,
                    inner_margin: Margin {
                        left: View::far_left_inset_margin(ui) + 4.0,
                        right: View::get_right_inset() + 4.0,
                        top: 4.0,
                        bottom: View::get_bottom_inset() + 4.0,
                    },
                    ..Default::default()
                })
                .show_inside(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        self.step_control_ui(ui);
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
                self.step_content_ui(ui, cb);
            });
    }

    /// Draw [`Step`] description and confirmation control.
    fn step_control_ui(&mut self, ui: &mut egui::Ui) {
        if let Some(step) = &self.step {
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
            ui.label(RichText::new(step_text).size(16.0).color(Colors::GRAY));

            // Show error if entered phrase is not valid.
            if !self.mnemonic_setup.valid_phrase {
                step_available = false;
                ui.label(RichText::new(t!("wallets.not_valid_phrase"))
                    .size(16.0)
                    .color(Colors::RED));
                ui.add_space(2.0);
            }

            // Show next step button if there are no empty words.
            if step_available {
                // Setup button text.
                let (next_text, color) = if step == &Step::SetupConnection {
                    (format!("{} {}", CHECK, t!("complete")), Colors::GOLD)
                } else {
                    let text = format!("{} {}", SHARE_FAT, t!("continue"));
                    (text, Colors::WHITE)
                };

                ui.add_space(4.0);
                // Show button.
                View::button(ui, next_text.to_uppercase(), color, || {
                    self.forward();
                });
                ui.add_space(4.0);
            }
        }
    }

    /// Draw wallet creation [`Step`] content.
    fn step_content_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        match &self.step {
            None => {
                // Show wallet creation message if step is empty.
                View::center_content(ui, 415.0 + View::get_bottom_inset(), |ui| {
                    ui.add(
                        egui::Image::new(self.logo.texture_id(ui.ctx()), vec2(200.0, 200.0))
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
                    let add_text = format!("{} {}", PLUS_CIRCLE, t!("wallets.add"));
                    View::button(ui, add_text, Colors::BUTTON, || {
                        self.show_name_pass_modal();
                    });
                });
            }
            Some(step) => {
                match step {
                    Step::EnterMnemonic => self.mnemonic_setup.ui(ui),
                    Step::ConfirmMnemonic => self.mnemonic_setup.confirm_ui(ui),
                    Step::SetupConnection => self.network_setup.ui(ui, cb)
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
                        // Clear values if it needs to go back on first step.
                        self.step = None;
                        self.name_edit = String::from("");
                        self.pass_edit = String::from("");
                        self.mnemonic_setup.reset();
                    }
                    Step::ConfirmMnemonic => self.step = Some(Step::EnterMnemonic),
                    Step::SetupConnection => self.step = Some(Step::EnterMnemonic)
                }
            }
        }
    }

    /// Go to the next wallet creation [`Step`].
    fn forward(&mut self) {
        self.step = match &self.step {
            None => Some(Step::EnterMnemonic),
            Some(step) => {
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
                    Step::ConfirmMnemonic => Some(Step::SetupConnection),
                    Step::SetupConnection => {
                        // Create wallet at last step.
                        WalletList::create_wallet(
                            self.name_edit.clone(),
                            self.pass_edit.clone(),
                            self.mnemonic_setup.mnemonic.get_phrase(),
                            self.network_setup.get_ext_conn_url()
                        ).unwrap();
                        None
                    }
                }
            }
        }
    }

    /// Start wallet creation from showing [`Modal`] to enter name and password.
    pub fn show_name_pass_modal(&mut self) {
        // Reset modal values.
        self.hide_pass = true;
        self.modal_just_opened = true;
        self.name_edit = String::from("");
        self.pass_edit = String::from("");
        // Show modal.
        Modal::new(Self::NAME_PASS_MODAL)
            .position(ModalPosition::CenterTop)
            .title(t!("wallets.add"))
            .show();
    }

    /// Draw wallet creation [`Modal`] content.
    pub fn modal_ui(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("wallets.name"))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Show wallet name text edit.
            let name_resp = egui::TextEdit::singleline(&mut self.name_edit)
                .id(ui.id().with("wallet_name_edit"))
                .font(TextStyle::Heading)
                .desired_width(ui.available_width())
                .cursor_at_end(true)
                .ui(ui);
            ui.add_space(8.0);
            if name_resp.clicked() {
                cb.show_keyboard();
            }

            // Check if modal was just opened to show focus on name text input.
            if self.modal_just_opened {
                self.modal_just_opened = false;
                cb.show_keyboard();
                name_resp.request_focus();
            }

            ui.label(RichText::new(t!("wallets.pass"))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            StripBuilder::new(ui)
                .size(Size::exact(34.0))
                .vertical(|mut strip| {
                    strip.strip(|builder| {
                        builder
                            .size(Size::remainder())
                            .size(Size::exact(48.0))
                            .horizontal(|mut strip| {
                                strip.cell(|ui| {
                                    ui.add_space(2.0);
                                    // Draw wallet password text edit.
                                    let pass_resp = egui::TextEdit::singleline(&mut self.pass_edit)
                                        .id(ui.id().with("wallet_pass_edit"))
                                        .font(TextStyle::Heading)
                                        .desired_width(ui.available_width())
                                        .cursor_at_end(true)
                                        .password(self.hide_pass)
                                        .ui(ui);
                                    if pass_resp.clicked() {
                                        cb.show_keyboard();
                                    }

                                    // Hide keyboard if input fields has no focus.
                                    if !pass_resp.has_focus() && !name_resp.has_focus() {
                                        cb.hide_keyboard();
                                    }
                                });
                                strip.cell(|ui| {
                                    ui.vertical_centered(|ui| {
                                        // Draw button to show/hide password.
                                        let eye_icon = if self.hide_pass { EYE } else { EYE_SLASH };
                                        View::button(ui, eye_icon.to_string(), Colors::WHITE, || {
                                            self.hide_pass = !self.hide_pass;
                                        });
                                    });
                                });
                            });
                    })
                });

            // Show information when specified values are empty.
            if self.name_edit.is_empty() {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("wallets.name_empty"))
                    .size(17.0)
                    .color(Colors::INACTIVE_TEXT));
            } else if self.pass_edit.is_empty() {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("wallets.pass_empty"))
                    .size(17.0)
                    .color(Colors::INACTIVE_TEXT));
            }
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
                    View::button(ui, t!("continue"), Colors::WHITE, || {
                        // Check if input values are not empty.
                        if self.name_edit.is_empty() || self.pass_edit.is_empty() {
                            return;
                        }
                        self.forward();
                        cb.hide_keyboard();
                        modal.close();
                    });
                });
            });
            ui.add_space(6.0);
        });
    }
}
