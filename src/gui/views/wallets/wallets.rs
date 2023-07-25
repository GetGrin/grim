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

use egui::{Align2, Margin, Vec2};

use crate::gui::Colors;
use crate::gui::icons::{ARROW_LEFT, GEAR, GLOBE, PLUS};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, ModalContainer, Root, TitlePanel, TitleType, View};
use crate::gui::views::wallets::creation::{MnemonicSetup, WalletCreation};
use crate::gui::views::wallets::setup::ConnectionSetup;
use crate::gui::views::wallets::wallet::WalletContent;
use crate::wallet::{Wallet, WalletList};

/// Wallets content.
pub struct WalletsContent {
    /// List of wallets.
    list: Vec<Wallet>,

    /// Selected list item content.
    item_content: Option<WalletContent>,
    /// Wallet creation content.
    creation_content: WalletCreation,

    /// [`Modal`] ids allowed at this ui container.
    modal_ids: Vec<&'static str>
}

impl Default for WalletsContent {
    fn default() -> Self {
        Self {
            list: WalletList::list(),
            item_content: None,
            creation_content: WalletCreation::default(),
            modal_ids: vec![
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
    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, cb: &dyn PlatformCallbacks) {
        // Show modal content for current ui container.
        if self.can_draw_modal() {
            Modal::ui(ui, |ui, modal| {
                match modal.id {
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

        // Show title panel.
        self.title_ui(ui, frame);

        let is_wallet_panel_open = Self::is_dual_panel_mode(ui, frame) || self.list.is_empty();
        let wallet_panel_width = self.wallet_panel_width(ui, frame);
        // Show wallet content.
        egui::SidePanel::right("wallet_panel")
            .resizable(false)
            .min_width(wallet_panel_width)
            .frame(egui::Frame {
                fill: if self.list.is_empty() { Colors::FILL_DARK } else { Colors::WHITE },
                ..Default::default()
            })
            .show_animated_inside(ui, is_wallet_panel_open, |ui| {
                self.wallet_content_ui(ui, frame, cb);
            });

        // Show list of wallets.
        if !self.list.is_empty() {
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
                    //TODO: wallets list
                });
            // Show wallet creation button if wallet panel is not open.
            if !is_wallet_panel_open {
                self.create_wallet_btn_ui(ui);
            }
        }
    }

    /// Draw title content.
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
             if self.creation_content.can_go_back() {
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

    /// Draw [`WalletContent`] ui.
    fn wallet_content_ui(&mut self,
                          ui: &mut egui::Ui,
                          frame: &mut eframe::Frame,
                          cb: &dyn PlatformCallbacks) {
        if self.list.is_empty() || self.item_content.is_none() {
            self.creation_content.ui(ui, cb)
        } else {
            self.item_content.as_mut().unwrap().ui(ui, frame, cb);
        }
    }

    /// Get [`WalletContent`] panel width.
    fn wallet_panel_width(&self, ui: &mut egui::Ui, frame: &mut eframe::Frame) -> f32 {
        if Self::is_dual_panel_mode(ui, frame) {
            let min_width = (Root::SIDE_PANEL_MIN_WIDTH + View::get_right_inset()) as i64;
            let available_width = if self.list.is_empty() {
                ui.available_width()
            } else {
                ui.available_width() - Root::SIDE_PANEL_MIN_WIDTH
            } as i64;
            max(min_width, available_width) as f32
        } else {
            let dual_panel_root = Root::is_dual_panel_mode(frame);
            if dual_panel_root {
                ui.available_width()
            } else {
                frame.info().window_info.size.x
            }
        }
    }

    /// Check if ui can show [`WalletsContent`] list and [`WalletContent`] content at same time.
    fn is_dual_panel_mode(ui: &mut egui::Ui, frame: &mut eframe::Frame) -> bool {
        let dual_panel_root = Root::is_dual_panel_mode(frame);
        let max_width = ui.available_width();
        dual_panel_root && max_width >= (Root::SIDE_PANEL_MIN_WIDTH * 2.0) + View::get_right_inset()
    }

    /// Draw floating button to create the wallet.
    fn create_wallet_btn_ui(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("create_wallet_button")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .anchor(Align2::RIGHT_BOTTOM, Vec2::new(-8.0, -8.0))
            .frame(egui::Frame::default())
            .show(ui.ctx(), |ui| {
                View::round_button(ui, PLUS, || {
                    self.creation_content.show_name_pass_modal();
                });
            });
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