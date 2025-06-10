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

use crate::gui::icons::{FOLDER_LOCK, FOLDER_OPEN, SPINNER, WARNING_CIRCLE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::Modal;
use crate::wallet::Wallet;

/// GRIN coin symbol.
pub const GRIN: &str = "ツ";

/// Content container to simplify modals management and navigation.
pub trait WalletContentContainer {
    /// List of allowed [`Modal`] identifiers.
    fn modal_ids(&self) -> Vec<&'static str>;
    /// Draw modal content.
    fn modal_ui(&mut self, ui: &mut egui::Ui, wallet: &Wallet, modal: &Modal, cb: &dyn PlatformCallbacks);
    /// Draw container content.
    fn container_ui(&mut self, ui: &mut egui::Ui, wallet: &Wallet, cb: &dyn PlatformCallbacks);
    /// Draw content, to call by parent container.
    fn ui(&mut self, ui: &mut egui::Ui, wallet: &Wallet, cb: &dyn PlatformCallbacks) {
        // Draw modal content.
        if let Some(id) = Modal::opened() {
            if self.modal_ids().contains(&id) {
                Modal::ui(ui.ctx(), cb, |ui, modal, cb| {
                    self.modal_ui(ui, wallet, modal, cb);
                });
            }
        }
        self.container_ui(ui, wallet, cb);
    }
}

/// Wallet tab content interface.
pub trait WalletTab {
    fn get_type(&self) -> WalletTabType;
    fn ui(&mut self,
          ui: &mut egui::Ui,
          wallet: &Wallet,
          cb: &dyn PlatformCallbacks);
}

/// Type of [`WalletTab`] content.
#[derive(PartialEq)]
pub enum WalletTabType {
    Txs,
    Settings
}

impl WalletTabType {
    /// Name of wallet tab to show at ui.
    pub fn name(&self) -> String {
        match *self {
            WalletTabType::Txs => t!("wallets.txs"),
            WalletTabType::Settings => t!("wallets.settings")
        }
    }
}

/// Get wallet status text.
pub fn wallet_status_text(wallet: &Wallet) -> String {
    if wallet.is_open() {
        if wallet.sync_error() {
            format!("{} {}", WARNING_CIRCLE, t!("error"))
        } else if wallet.is_closing() {
            format!("{} {}", SPINNER, t!("wallets.closing"))
        } else if wallet.is_repairing() {
            let repair_progress = wallet.repairing_progress();
            if repair_progress == 0 {
                format!("{} {}", SPINNER, t!("wallets.checking"))
            } else {
                format!("{} {}: {}%",
                        SPINNER,
                        t!("wallets.checking"),
                        repair_progress)
            }
        } else if wallet.syncing() {
            let info_progress = wallet.info_sync_progress();
            if info_progress == 100 || info_progress == 0 {
                format!("{} {}", SPINNER, t!("wallets.loading"))
            } else {
                format!("{} {}: {}%",
                        SPINNER,
                        t!("wallets.loading"),
                        info_progress)
            }
        } else {
            format!("{} {}", FOLDER_OPEN, t!("wallets.unlocked"))
        }
    } else {
        format!("{} {}", FOLDER_LOCK, t!("wallets.locked"))
    }
}