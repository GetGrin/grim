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

use egui::{Margin, RichText, TextStyle, Widget};
use egui_extras::{Size, StripBuilder};

use crate::gui::Colors;
use crate::gui::icons::{EYE, EYE_SLASH, PLUS_CIRCLE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, ModalPosition, View};
use crate::gui::views::wallets::creation::{ConnectionSetup, MnemonicSetup, StepControl};
use crate::gui::views::wallets::creation::mnemonic::PhraseMode;

/// Wallet creation step.
enum Step {
    /// Mnemonic phrase input.
    EnterMnemonic,
    /// Mnemonic phrase confirmation for [`Mnemonic`].
    ConfirmMnemonic,
    /// Wallet connection setup.
    SetupConnection
}

/// Wallet creation content.
pub struct WalletCreation {
    /// Wallet creation ui step.
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
}

impl Default for WalletCreation {
    fn default() -> Self {
        Self {
            step: None,
            modal_just_opened: true,
            name_edit: "".to_string(),
            pass_edit: "".to_string(),
            hide_pass: true,
            mnemonic_setup: MnemonicSetup::default(),
            network_setup: ConnectionSetup::default(),
        }
    }
}


impl StepControl for WalletCreation {
    /// Go to next wallet creation [`Step`].
    fn next_step(&mut self) {
        self.step = match &self.step {
            None => Some(Step::EnterMnemonic),
            Some(step) => {
                match step {
                    Step::EnterMnemonic => {
                        if self.mnemonic_setup.get_mnemonic_mode() == &PhraseMode::Generate {
                            Some(Step::SetupConnection)
                        } else {
                            Some(Step::ConfirmMnemonic)
                        }
                    }
                    Step::ConfirmMnemonic => Some(Step::SetupConnection),
                    Step::SetupConnection => {
                        //TODO: Confirm mnemonic
                        None
                    }
                }
            }
        }
    }

    /// Go to previous wallet creation [`Step`].
    fn prev_step(&mut self) {
        match &self.step {
            None => {}
            Some(step) => {
                match step {
                    Step::EnterMnemonic => {
                        // Clear values if it needs to go back on first step.
                        self.step = None;
                        self.name_edit = "".to_string();
                        self.pass_edit = "".to_string();
                        self.mnemonic_setup.reset();
                    }
                    Step::ConfirmMnemonic => self.step = Some(Step::EnterMnemonic),
                    Step::SetupConnection => self.step = Some(Step::ConfirmMnemonic)
                }
            }
        }
    }
}

impl WalletCreation {
    /// Wallet name/password input modal identifier.
    pub const MODAL_ID: &'static str = "create_wallet_modal";

    pub fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        // Show wallet creation step content.
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
                self.step_ui(ui, cb);
            });
    }

    /// Draw wallet creation [`Step`] content.
    fn step_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        match &self.step {
            None => {
                // Show wallet creation message if step is empty.
                View::center_content(ui, 124.0 + View::get_bottom_inset(), |ui| {
                    let text = t!("wallets.create_desc");
                    ui.label(RichText::new(text)
                        .size(16.0)
                        .color(Colors::INACTIVE_TEXT)
                    );
                    ui.add_space(8.0);
                    let add_text = format!("{} {}", PLUS_CIRCLE, t!("wallets.add"));
                    View::button(ui, add_text, Colors::BUTTON, || {
                        Self::show_modal();
                    });
                });
            }
            Some(step) => {
                match step {
                    Step::EnterMnemonic => {
                        self.mnemonic_setup.ui(ui, self, cb);
                    }
                    Step::ConfirmMnemonic => {}
                    Step::SetupConnection => {}
                }
            }
        }
    }

    /// Check if it's possible to go back for current step.
    pub fn can_go_back(&self) -> bool {
        self.step.is_some()
    }

    /// Back button key event handling.
    pub fn go_back(&mut self) {
        self.prev_step();
    }

    /// Start wallet creation from showing [`Modal`] to enter name and password.
    pub fn show_modal() {
        Modal::show(Modal::new(Self::MODAL_ID)
            .position(ModalPosition::CenterTop)
            .title(t!("wallets.add")));
    }

    /// Callback to go to next step for wallet creation from [`Modal`].
    fn on_modal_confirmation(&mut self, modal: &Modal, cb: &dyn PlatformCallbacks) {
        // Check if input values are not empty.
        if self.name_edit.is_empty() || self.pass_edit.is_empty() {
            return;
        }
        self.step = Some(Step::EnterMnemonic);
        cb.hide_keyboard();
        modal.close();
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
                        // Clear values.
                        self.hide_pass = false;
                        self.modal_just_opened = true;
                        self.name_edit = "".to_string();
                        self.pass_edit = "".to_string();
                        // Close modal.
                        cb.hide_keyboard();
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("continue"), Colors::WHITE, || {
                        self.on_modal_confirmation(modal, cb);
                    });
                });
            });
            ui.add_space(6.0);
        });
    }
}
