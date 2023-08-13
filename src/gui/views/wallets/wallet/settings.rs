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

use egui::{Id, ScrollArea};

use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::Root;
use crate::gui::views::wallets::setup::{CommonSetup, ConnectionSetup, RecoverySetup};
use crate::gui::views::wallets::wallet::types::{WalletTab, WalletTabType};
use crate::gui::views::wallets::wallet::WalletContent;
use crate::wallet::{ExternalConnection, Wallet};

/// Wallet settings tab content.
pub struct WalletSettings {
    /// Common setup content.
    common_setup: CommonSetup,
    /// Connection setup content.
    conn_setup: ConnectionSetup,
    /// Recovery setup content.
    recovery_setup: RecoverySetup
}

impl Default for WalletSettings {
    fn default() -> Self {
        // Check external connections availability on opening.
        ExternalConnection::start_ext_conn_availability_check();
        Self {
            common_setup: CommonSetup::default(),
            conn_setup: ConnectionSetup::default(),
            recovery_setup: RecoverySetup::default()
        }
    }
}

impl WalletTab for WalletSettings {
    fn get_type(&self) -> WalletTabType {
        WalletTabType::Settings
    }

    fn ui(&mut self,
          ui: &mut egui::Ui,
          frame: &mut eframe::Frame,
          wallet: &mut Wallet,
          cb: &dyn PlatformCallbacks) {
        // Show loading progress if navigation is blocked.
        if WalletContent::block_navigation_on_sync(wallet) {
            WalletContent::sync_progress_ui(ui, wallet);
            return;
        }

        ScrollArea::vertical()
            .id_source(Id::from("wallet_settings_scroll").with(wallet.config.id))
            .auto_shrink([false; 2])
            .show(ui, |ui| {
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
                        // Show common wallet setup.
                        self.common_setup.ui(ui, frame, wallet, cb);
                        // Show wallet connections setup.
                        self.conn_setup.wallet_ui(ui, frame, wallet, cb);
                        // Show wallet recovery setup.
                        self.recovery_setup.ui(ui, frame, wallet, cb);
                    });
                });
            });
    }
}