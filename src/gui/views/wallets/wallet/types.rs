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
use crate::wallet::Wallet;

/// GRIN coin symbol.
pub const GRIN: &str = "ツ";
/// Hint for Slatepack message input.
pub const SLATEPACK_MESSAGE_HINT: &'static str = "BEGINSLATEPACK.\n...\n...\n...\nENDSLATEPACK.";

/// Wallet tab content interface.
pub trait WalletTab {
    fn get_type(&self) -> WalletTabType;
    fn ui(&mut self,
          ui: &mut egui::Ui,
          wallet: &mut Wallet,
          cb: &dyn PlatformCallbacks);
}

/// Type of [`WalletTab`] content.
#[derive(PartialEq)]
pub enum WalletTabType {
    Txs,
    Messages,
    Transport,
    Settings
}

impl WalletTabType {
    /// Name of wallet tab to show at ui.
    pub fn name(&self) -> String {
        match *self {
            WalletTabType::Txs => t!("wallets.txs"),
            WalletTabType::Messages => t!("wallets.messages"),
            WalletTabType::Transport => t!("wallets.transport"),
            WalletTabType::Settings => t!("wallets.settings")
        }
    }
}