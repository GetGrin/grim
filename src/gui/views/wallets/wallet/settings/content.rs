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

use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::wallets::{CommonSettings, ConnectionSettings, RecoverySettings};
use crate::gui::views::wallets::types::{WalletTab, WalletTabType};
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
          wallet: &Wallet,
          cb: &dyn PlatformCallbacks) {
        // Show common wallet setup.
        self.common_setup.ui(ui, wallet, cb);
        // Show wallet connections setup.
        self.conn_setup.wallet_ui(ui, wallet, cb);
        // Show wallet recovery setup.
        self.recovery_setup.ui(ui, wallet, cb);
    }
}