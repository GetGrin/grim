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

use egui::{Margin, RichText};

use crate::gui::Colors;
use crate::gui::icons::{DOWNLOAD, UPLOAD, WALLET, WRENCH};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::View;
use crate::gui::views::wallets::{WalletInfo, WalletReceive, WalletSend, WalletSettings};
use crate::gui::views::wallets::types::{WalletTab, WalletTabType};
use crate::node::Node;
use crate::wallet::{Wallet, Wallets};

/// Selected and opened wallet content.
pub struct WalletContent {
    /// Current tab content to show.
    current_tab: Box<dyn WalletTab>,
}

impl Default for WalletContent {
    fn default() -> Self {
        Self { current_tab: Box::new(WalletInfo::default()) }
    }
}

impl WalletContent {
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              frame: &mut eframe::Frame,
              wallet: &mut Wallet,
              cb: &dyn PlatformCallbacks) {
        // Show bottom tabs.
        egui::TopBottomPanel::bottom("wallet_tabs")
            .frame(egui::Frame {
                fill: Colors::FILL,
                inner_margin: Margin {
                    left: View::far_left_inset_margin(ui) + 4.0,
                    right: View::get_right_inset() + 4.0,
                    top: 4.0,
                    bottom: View::get_bottom_inset() + 4.0,
                },
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                self.tabs_ui(ui);
            });

        // Show tab content.
        egui::CentralPanel::default()
            .frame(egui::Frame {
                stroke: View::DEFAULT_STROKE,
                fill: Colors::WHITE,
                inner_margin: Margin {
                    left: View::far_left_inset_margin(ui) + 4.0,
                    right: View::get_right_inset() + 4.0,
                    top: 4.0,
                    bottom: 4.0,
                },
                ..Default::default()
            })
            .show_inside(ui, |ui| {
            });

        // Refresh content after delay for loaded wallet.
        if wallet.is_loaded() {
            ui.ctx().request_repaint_after(Wallets::INFO_UPDATE_DELAY);
        } else {
            ui.ctx().request_repaint();
        }
    }

    /// Draw tab buttons in the bottom of the screen.
    fn tabs_ui(&mut self, ui: &mut egui::Ui) {
        ui.scope(|ui| {
            // Setup spacing between tabs.
            ui.style_mut().spacing.item_spacing = egui::vec2(4.0, 0.0);
            // Setup vertical padding inside tab button.
            ui.style_mut().spacing.button_padding = egui::vec2(0.0, 8.0);

            // Draw tab buttons.
            let current_type = self.current_tab.get_type();
            ui.columns(4, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::tab_button(ui, WALLET, current_type == WalletTabType::Info, || {
                        self.current_tab = Box::new(WalletInfo::default());
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::tab_button(ui, DOWNLOAD, current_type == WalletTabType::Receive, || {
                        self.current_tab = Box::new(WalletReceive::default());
                    });
                });
                columns[2].vertical_centered_justified(|ui| {
                    View::tab_button(ui, UPLOAD, current_type == WalletTabType::Send, || {
                        self.current_tab = Box::new(WalletSend::default());
                    });
                });
                columns[3].vertical_centered_justified(|ui| {
                    View::tab_button(ui, WRENCH, current_type == WalletTabType::Settings, || {
                        self.current_tab = Box::new(WalletSettings::default());
                    });
                });
            });
        });
    }

    /// Content to draw when wallet is loading.
    fn loading_ui(ui: &mut egui::Ui, wallet: &Wallet) {
        if wallet.config.external_node_url.is_none() && !Node::is_running() {

        } else {
            if let Some(error) = &wallet.loading_error {
                // View::center_content(ui, 162.0, |ui| {
                //     let text = t!("wallets.enable_node", "settings" => WRENCH);
                //     View::big_loading_spinner(ui);
                //     ui.add_space(18.0);
                //     let text = if wallet.loading_progress == 0 {
                //         t!("wallet_loading")
                //     } else {
                //         format!("{}: {}%", t!("wallet_loading"), wallet.loading_progress)
                //     };
                //     ui.label(RichText::new(text).size(16.0).color(Colors::INACTIVE_TEXT));
                // });
            } else if !wallet.is_loaded() {
                View::center_content(ui, 162.0, |ui| {
                    View::big_loading_spinner(ui);
                    ui.add_space(18.0);
                    let text = if wallet.loading_progress == 0 {
                        t!("wallets.wallet_loading")
                    } else {
                        format!("{}: {}%", t!("wallets.wallet_loading"), wallet.loading_progress)
                    };
                    ui.label(RichText::new(text).size(16.0).color(Colors::INACTIVE_TEXT));
                });
            }
        }
    }
}