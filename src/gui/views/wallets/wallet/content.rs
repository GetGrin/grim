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

use std::time::Duration;

use egui::{Margin, RichText};
use grin_chain::SyncStatus;

use crate::AppConfig;
use crate::gui::Colors;
use crate::gui::icons::{DOWNLOAD, GEAR_FINE, POWER, REPEAT, UPLOAD, WALLET};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Root, View};
use crate::gui::views::wallets::{WalletInfo, WalletReceive, WalletSend, WalletSettings};
use crate::gui::views::wallets::types::{WalletTab, WalletTabType};
use crate::node::Node;
use crate::wallet::Wallet;

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
        // Show wallet tabs panel.
        let not_show_tabs =
            wallet.is_closing() || (wallet.get_data().is_none() && !wallet.load_error());
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
            .show_animated_inside(ui, !not_show_tabs, |ui| {
                ui.vertical_centered(|ui| {
                    // Setup tabs width.
                    let available_width = ui.available_width();
                    if available_width == 0.0 {
                        return;
                    }
                    let mut rect = ui.available_rect_before_wrap();
                    let width = f32::min(available_width, Root::SIDE_PANEL_WIDTH * 1.3);
                    rect.set_width(width);

                    // Draw wallet tabs.
                    ui.allocate_ui(rect.size(), |ui| {
                        self.tabs_ui(ui);
                    });
                });
            });

        // Show tab content panel.
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
                ui.vertical_centered(|ui| {
                    // Setup tab content width.
                    let available_width = ui.available_width();
                    if available_width == 0.0 {
                        return;
                    }
                    let mut rect = ui.available_rect_before_wrap();
                    let width = f32::min(available_width, Root::SIDE_PANEL_WIDTH * 1.3);
                    rect.set_width(width);

                    // Draw current tab content.
                    ui.allocate_ui(rect.size(), |ui| {
                        self.current_tab.ui(ui, frame, wallet, cb);
                    });
                });
            });

        // Refresh content after 1 second for loaded wallet.
        if wallet.get_data().is_some() {
            ui.ctx().request_repaint_after(Duration::from_millis(1000));
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
                    View::tab_button(ui, GEAR_FINE, current_type == WalletTabType::Settings, || {
                        self.current_tab = Box::new(WalletSettings::default());
                    });
                });
            });
        });
    }

    /// Content to draw when wallet is loading, returns `true` if wallet is not ready.
    pub fn loading_ui(ui: &mut egui::Ui, frame: &mut eframe::Frame, wallet: &Wallet) -> bool {
        if wallet.is_closing() {
            Self::loading_progress_ui(ui, wallet);
            return true;
        } else if wallet.config.ext_conn_id.is_none() {
            if !Node::is_running() || Node::is_stopping() {
                let dual_panel_root = Root::is_dual_panel_mode(frame);
                View::center_content(ui, 108.0, |ui| {
                    let text = t!("wallets.enable_node", "settings" => GEAR_FINE);
                    ui.label(RichText::new(text).size(16.0).color(Colors::INACTIVE_TEXT));
                    ui.add_space(8.0);
                    // Show button to enable integrated node at non-dual root panel mode
                    // or when network connections are not showing and node is not stopping
                    if (!dual_panel_root || AppConfig::show_connections_network_panel())
                        && !Node::is_stopping() {
                        let enable_node_text = format!("{} {}", POWER, t!("network.enable_node"));
                        View::button(ui, enable_node_text, Colors::GOLD, || {
                            Node::start();
                        });
                    }
                });
                return true
            } else if wallet.load_error()
                && Node::get_sync_status() == Some(SyncStatus::NoSync) {
                Self::loading_error_ui(ui, wallet);
                return true;
            } else if wallet.get_data().is_none() {
                Self::loading_progress_ui(ui, wallet);
                return true;
            }
        } else if wallet.get_data().is_none() {
            if wallet.load_error() {
                Self::loading_error_ui(ui, wallet);
            } else {
                Self::loading_progress_ui(ui, wallet);
            }
            return true;
        }
        false
    }

    /// Draw wallet loading error content.
    fn loading_error_ui(ui: &mut egui::Ui, wallet: &Wallet) {
        View::center_content(ui, 108.0, |ui| {
            let text = t!("wallets.wallet_loading_err", "settings" => GEAR_FINE);
            ui.label(RichText::new(text).size(16.0).color(Colors::INACTIVE_TEXT));
            ui.add_space(8.0);
            let retry_text = format!("{} {}", REPEAT, t!("retry"));
            View::button(ui, retry_text, Colors::GOLD, || {
                wallet.set_load_error(false);
            });
        });
    }

    /// Draw wallet loading progress content.
    pub fn loading_progress_ui(ui: &mut egui::Ui, wallet: &Wallet) {
        View::center_content(ui, 162.0, |ui| {
            View::big_loading_spinner(ui);
            ui.add_space(18.0);
            // Setup loading progress text.
            let text = {
                let info_progress = wallet.info_load_progress();
                if wallet.is_closing() {
                    t!("wallets.wallet_closing")
                } else if info_progress != 100 {
                    if info_progress == 0 {
                        t!("wallets.wallet_loading")
                    } else {
                        format!("{}: {}%", t!("wallets.wallet_loading"), info_progress)
                    }
                } else {
                    let tx_progress = wallet.txs_load_progress();
                    if tx_progress == 0 {
                        t!("wallets.tx_loading")
                    } else {
                        format!("{}: {}%", t!("wallets.tx_loading"), tx_progress)
                    }
                }
            };
            ui.label(RichText::new(text).size(16.0).color(Colors::INACTIVE_TEXT));
        });
    }
}