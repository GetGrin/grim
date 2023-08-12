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
use crate::gui::views::wallets::types::WalletTab;
use crate::gui::views::wallets::wallet::types::WalletTabType;
use crate::gui::views::wallets::wallet::WalletContent;
use crate::wallet::Wallet;

/// Wallet info tab content.
#[derive(Default)]
pub struct WalletInfo;

impl WalletTab for WalletInfo {
    fn get_type(&self) -> WalletTabType {
        WalletTabType::Info
    }

    fn ui(&mut self,
          ui: &mut egui::Ui,
          frame: &mut eframe::Frame,
          wallet: &mut Wallet,
          cb: &dyn PlatformCallbacks) {
        if WalletContent::loading_ui(ui, frame, wallet) {
            return;
        }
    }
}