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

use std::cmp::max;

use egui::{Align2, Margin, RichText, TextStyle, Widget};
use egui_extras::{Size, StripBuilder};

use crate::gui::Colors;
use crate::gui::icons::{ARROW_LEFT, EYE, EYE_SLASH, FOLDER_PLUS, GEAR, GLOBE, PLUS};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, ModalContainer, ModalPosition, Root, TitlePanel, TitleType, View};
use crate::gui::views::wallets::creation::{MnemonicSetup, WalletCreation};
use crate::gui::views::wallets::setup::ConnectionSetup;
use crate::gui::views::wallets::wallet::WalletContent;
use crate::wallet::{Wallet, Wallets};

/// Wallets content.
pub struct WalletsContent {
    /// Password to open wallet for [`Modal`].
    pass_edit: String,
    /// Flag to show/hide password at [`egui::TextEdit`] field.
    hide_pass: bool,
    /// Flag to check if wrong password was entered.
    wrong_pass: bool,

    /// Selected [`Wallet`] content.
    wallet_content: WalletContent,

    /// Wallet creation content.
    creation_content: WalletCreation,

    /// [`Modal`] ids allowed at this ui container.
    modal_ids: Vec<&'static str>
}

impl Default for WalletsContent {
    fn default() -> Self {
        Self {
            pass_edit: "".to_string(),
            hide_pass: true,
            wrong_pass: false,
            wallet_content: WalletContent::default(),
            creation_content: WalletCreation::default(),
            modal_ids: vec![
                Self::OPEN_WALLET_MODAL,
                WalletCreation::NAME_PASS_MODAL,
                MnemonicSetup::WORD_INPUT_MODAL,
                ConnectionSetup::ADD_CONNECTION_URL_MODAL
            ]
        }
    }
}

impl ModalContainer for WalletsContent {
    fn modal_ids(&self) -> &Vec<&'static str> {
        &self.modal_ids
    }
}

impl WalletsContent {
    /// Identifier for wallet opening [`Modal`].
    pub const OPEN_WALLET_MODAL: &'static str = "open_wallet_modal";

    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, cb: &dyn PlatformCallbacks) {
        // Show modal content for current ui container.
        if self.can_draw_modal() {
            Modal::ui(ui, |ui, modal| {
                match modal.id {
                    Self::OPEN_WALLET_MODAL => {
                        self.open_wallet_modal_ui(ui, modal, cb);
                    },
                    WalletCreation::NAME_PASS_MODAL => {
                        self.creation_content.modal_ui(ui, modal, cb);
                    },
                    MnemonicSetup::WORD_INPUT_MODAL => {
                        self.creation_content.mnemonic_setup.modal_ui(ui, modal, cb);
                    }
                    ConnectionSetup::ADD_CONNECTION_URL_MODAL => {
                        self.creation_content.network_setup.modal_ui(ui, modal, cb);
                    }
                    _ => {}
                }
            });
        }

        // Get wallets.
        let wallets = Wallets::list();
        let is_list_empty = wallets.is_empty();
        let selected = wallets.iter().find(|x| Some(x.config.id) == Wallets::selected_id());

        // Show title panel.
        self.title_ui(ui, frame);

        // Setup wallet content flags.
        let is_wallet_creating = self.creation_content.can_go_back();
        let is_wallet_showing = if let Some(id) = Wallets::selected_id() {
            Wallets::is_open(id)
        } else {
            false
        };

        // Setup panels parameters.
        let is_dual_panel = Self::is_dual_panel_mode(ui, frame);
        let is_wallet_panel_open
            = is_dual_panel || is_wallet_showing || is_wallet_creating || is_list_empty;
        let wallet_panel_width
            = self.wallet_panel_width(ui, is_list_empty, is_dual_panel, is_wallet_showing);

        // Show wallet panel content.
        egui::SidePanel::right("wallet_panel")
            .resizable(false)
            .exact_width(wallet_panel_width)
            .frame(egui::Frame {
                fill: if is_list_empty && !is_wallet_creating {
                    Colors::FILL_DARK
                } else {
                    Colors::WHITE
                },
                ..Default::default()
            })
            .show_animated_inside(ui, is_wallet_panel_open, |ui| {
                if is_wallet_showing {
                    self.wallet_content.ui(ui, frame, selected.unwrap(), cb);
                } else {
                    self.creation_content.ui(ui, cb);
                }
            });

        // Show wallets list.
        if !is_list_empty && (!is_wallet_panel_open || is_dual_panel) {
            egui::CentralPanel::default()
                .frame(egui::Frame {
                    stroke: View::DEFAULT_STROKE,
                    fill: Colors::FILL_DARK,
                    inner_margin: Margin {
                        left: View::far_left_inset_margin(ui) + 4.0,
                        right: View::far_right_inset_margin(ui, frame) + 4.0,
                        top: 3.0,
                        bottom: 4.0,
                    },
                    ..Default::default()
                })
                .show_inside(ui, |ui| {
                    self.list_ui(ui, &wallets, cb);
                });

            // Do not show creation button if wallets panel is not showing at root
            // or if wallet is not showing at dual panel mode.
            let root_dual_panel = Root::is_dual_panel_mode(frame);
            let wallets_panel_not_open = !root_dual_panel && Root::is_network_panel_open();
            if wallets_panel_not_open || (is_wallet_panel_open && !is_wallet_showing) {
                return;
            } else {
                self.create_wallet_btn_ui(ui, if is_dual_panel { wallet_panel_width } else { 0.0 });
            }
        }
    }

    /// Draw [`TitlePanel`] content.
    fn title_ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // Setup title text.
        let title_text = if self.creation_content.can_go_back() {
            t!("wallets.add")
        } else {
            t!("wallets.title")
        };
        let title_content = TitleType::Single(title_text.to_uppercase());

        // Draw title panel.
        TitlePanel::ui(title_content, |ui, frame| {
            if Wallets::selected_id().is_some() {
                if !Self::is_dual_panel_mode(ui, frame) {
                    View::title_button(ui, ARROW_LEFT, || {
                        Wallets::select(None);
                    });
                }
            } else if self.creation_content.can_go_back() {
                View::title_button(ui, ARROW_LEFT, || {
                    self.creation_content.back();
                });
            } else if !Root::is_dual_panel_mode(frame) {
                View::title_button(ui, GLOBE, || {
                    Root::toggle_network_panel();
                });
            };
        }, |ui, frame| {
            View::title_button(ui, GEAR, || {
                //TODO: show settings.
            });
        }, ui, frame);
    }

    /// Draw list of wallets.
    fn list_ui(&mut self, ui: &mut egui::Ui, wallets: &Vec<Wallet>, cb: &dyn PlatformCallbacks) {
        for w in wallets {
            ui.label(&w.config.name);

            /// Show open/close button
            let id = w.config.id;
            let is_selected = Some(id) == Wallets::selected_id();
            let is_open = Wallets::is_open(id);
            if !is_open {
                View::button(ui, "open me".to_string(), Colors::GOLD, || {
                    Wallets::select(Some(id));
                    self.show_open_wallet_modal(cb);
                });
            } else if !is_selected {
                View::button(ui, "select me".to_string(), Colors::GOLD, || {
                    Wallets::select(Some(id));
                });
            }

            if Wallets::is_open(id) {
                ui.label("opened!");
            }
        }
    }

    /// Draw floating button to show wallet creation [`Modal`].
    fn create_wallet_btn_ui(&mut self, ui: &mut egui::Ui, right_margin: f32) {
        egui::Window::new("create_wallet_button")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .anchor(Align2::RIGHT_BOTTOM, egui::Vec2::new(-6.0 - right_margin, -6.0))
            .frame(egui::Frame::default())
            .show(ui.ctx(), |ui| {
                View::round_button(ui, PLUS, || {
                    self.creation_content.show_name_pass_modal();
                });
            });
    }

    /// Show [`Modal`] to open selected wallet.
    pub fn show_open_wallet_modal(&mut self, cb: &dyn PlatformCallbacks) {
        // Reset modal values.
        self.hide_pass = true;
        self.pass_edit = String::from("");
        self.wrong_pass = false;
        // Show modal.
        Modal::new(Self::OPEN_WALLET_MODAL)
            .position(ModalPosition::CenterTop)
            .title(t!("wallets.open"))
            .show();
        cb.show_keyboard();
    }

    /// Draw wallet opening [`Modal`] content.
    fn open_wallet_modal_ui(&mut self,
                            ui: &mut egui::Ui,
                            modal: &Modal,
                            cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("wallets.pass"))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(10.0);

            StripBuilder::new(ui)
                .size(Size::exact(34.0))
                .vertical(|mut strip| {
                    strip.strip(|builder| {
                        builder
                            .size(Size::remainder())
                            .size(Size::exact(48.0))
                            .horizontal(|mut strip| {
                                strip.cell(|ui| {
                                    // Draw wallet password text edit.
                                    let pass_resp = egui::TextEdit::singleline(&mut self.pass_edit)
                                        .id(ui.id().with("wallet_pass_edit"))
                                        .font(TextStyle::Heading)
                                        .desired_width(ui.available_width())
                                        .cursor_at_end(true)
                                        .password(self.hide_pass)
                                        .ui(ui);
                                    pass_resp.request_focus();
                                    if pass_resp.clicked() {
                                        cb.show_keyboard();
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

            // Show information when password is empty.
            if self.pass_edit.is_empty() {
                ui.add_space(10.0);
                ui.label(RichText::new(t!("wallets.pass_empty"))
                    .size(17.0)
                    .color(Colors::INACTIVE_TEXT));
            } else if self.wrong_pass {
                ui.add_space(10.0);
                ui.label(RichText::new(t!("wallets.wrong_pass"))
                    .size(17.0)
                    .color(Colors::RED));
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
                        self.pass_edit = "".to_string();
                        self.wrong_pass = false;
                        self.hide_pass = true;
                        // Close modal.
                        cb.hide_keyboard();
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    // Callback for continue button.
                    let mut on_continue = || {
                        if self.pass_edit.is_empty() {
                            return;
                        }
                        let selected_id = Wallets::selected_id().unwrap();
                        match Wallets::open(selected_id, self.pass_edit.clone()) {
                            Ok(_) => {
                                // Clear values.
                                self.pass_edit = "".to_string();
                                self.wrong_pass = false;
                                self.hide_pass = true;
                                // Close modal.
                                cb.hide_keyboard();
                                modal.close();
                            }
                            Err(_) => self.wrong_pass = true
                        }
                    };
                    // Continue on Enter key press.
                    View::on_enter_key(ui, || {
                        (on_continue)();
                    });

                    View::button(ui, t!("continue"), Colors::WHITE, on_continue);
                });
            });
            ui.add_space(6.0);
        });
    }

    /// Calculate [`WalletContent`] panel width.
    fn wallet_panel_width(
        &self,
        ui:&mut egui::Ui,
        is_list_empty: bool,
        is_dual_panel: bool,
        is_wallet_showing: bool
    ) -> f32 {
        let is_wallet_creation = self.creation_content.can_go_back();
        let available_width = if is_list_empty || is_wallet_creation {
            ui.available_width()
        } else {
            ui.available_width() - Root::SIDE_PANEL_MIN_WIDTH
        };
        if is_dual_panel {
            let min_width = (Root::SIDE_PANEL_MIN_WIDTH + View::get_right_inset()) as i64;
            max(min_width, available_width as i64) as f32
        } else {
            if is_wallet_showing {
                ui.available_width()
            } else {
                available_width
            }
        }
    }

    /// Check if it's possible to show [`WalletsContent`] and [`WalletContent`] panels at same time.
    fn is_dual_panel_mode(ui: &mut egui::Ui, frame: &mut eframe::Frame) -> bool {
        let dual_panel_root = Root::is_dual_panel_mode(frame);
        let max_width = ui.available_width();
        dual_panel_root && max_width >= (Root::SIDE_PANEL_MIN_WIDTH * 2.0) + View::get_right_inset()
    }

    /// Handle Back key event.
    /// Return `false` when event was handled.
    pub fn on_back(&mut self) -> bool {
        let can_go_back = self.creation_content.can_go_back();
        if can_go_back {
            self.creation_content.back();
        }
        !can_go_back
    }
}