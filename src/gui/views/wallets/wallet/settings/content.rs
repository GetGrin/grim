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

use egui::{Id, Margin, ScrollArea};
use egui::scroll_area::ScrollBarVisibility;

use crate::gui::Colors;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Content, View};
use crate::gui::views::wallets::settings::{CommonSettings, ConnectionSettings, RecoverySettings};
use crate::gui::views::wallets::types::{WalletTab, WalletTabType};
use crate::gui::views::wallets::WalletContent;
use crate::wallet::Wallet;

/// Wallet settings tab content.
pub struct WalletSettings {
    /// Common setup content.
    common_setup: CommonSettings,
    /// Connection setup content.
    conn_setup: ConnectionSettings,
    /// Recovery setup content.
    recovery_setup: RecoverySettings
}

impl Default for WalletSettings {
    fn default() -> Self {
        Self {
            common_setup: CommonSettings::default(),
            conn_setup: ConnectionSettings::default(),
            recovery_setup: RecoverySettings::default()
        }
    }
}

impl WalletTab for WalletSettings {
    fn get_type(&self) -> WalletTabType {
        WalletTabType::Settings
    }

    fn ui(&mut self,
          ui: &mut egui::Ui,
          wallet: &mut Wallet,
          cb: &dyn PlatformCallbacks) {
        // Show loading progress if navigation is blocked.
        if WalletContent::block_navigation_on_sync(wallet) {
            WalletContent::sync_progress_ui(ui, wallet);
            return;
        }

        // Show settings content panel.
        egui::CentralPanel::default()
            .frame(egui::Frame {
                stroke: View::item_stroke(),
                fill: Colors::white_or_black(false),
                inner_margin: Margin {
                    left: View::far_left_inset_margin(ui) + 4.0,
                    right: View::get_right_inset() + 4.0,
                    top: 3.0,
                    bottom: 4.0,
                },
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                ScrollArea::vertical()
                    .id_source(Id::from("wallet_settings_scroll").with(wallet.get_config().id))
                    .auto_shrink([false; 2])
                    .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            View::max_width_ui(ui, Content::SIDE_PANEL_WIDTH * 1.3, |ui| {
                                // Show common wallet setup.
                                self.common_setup.ui(ui, wallet, cb);
                                // Show wallet connections setup.
                                self.conn_setup.wallet_ui(ui, wallet, cb);
                                // Show wallet recovery setup.
                                self.recovery_setup.ui(ui, wallet, cb);
                            });
                        });
                    });
            });
    }
}