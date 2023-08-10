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

use grin_core::global::ChainTypes;
use grin_wallet_libwallet::Error;

use crate::AppConfig;
use crate::wallet::{Wallet, WalletConfig};

/// Wrapper for [`Wallet`] list.
pub struct WalletList {
    /// List of wallets.
    list: Vec<Wallet>,
    /// Selected [`Wallet`] identifier.
    selected_id: Option<i64>,
}

impl Default for WalletList {
    fn default() -> Self {
        Self {
            list: Self::init(),
            selected_id: None
        }
    }
}

impl WalletList {
    /// Initialize [`Wallet`] list from base directory.
    fn init() -> Vec<Wallet> {
        let mut wallets = Vec::new();
        let chain_types = vec![ChainTypes::Mainnet, ChainTypes::Testnet];
        for chain in chain_types {
            let wallets_dir = WalletConfig::get_base_path(chain);
            // Load wallets from base directory.
            for dir in wallets_dir.read_dir().unwrap() {
                let wallet_dir = dir.unwrap().path();
                if wallet_dir.is_dir() {
                    let wallet = Wallet::init(wallet_dir);
                    if let Some(w) = wallet {
                        wallets.push(w);
                    }
                }
            }
        }
        // Sort wallets by id.
        wallets.sort_by_key(|w| w.config.id);
        wallets
    }

    /// Get wallet list for current [`ChainTypes`].
    pub fn list(&self) -> Vec<Wallet> {
        let chain_type = AppConfig::chain_type();
        self.list.iter().cloned()
            .filter(|w| w.config.chain_type == chain_type)
            .collect::<Vec<Wallet>>()
    }

    /// Add created [`Wallet`] to the list.
    pub fn add(&mut self, wallet: Wallet) {
        self.selected_id = Some(wallet.config.id);
        self.list.insert(0, wallet);
    }

    /// Select [`Wallet`] with provided identifier.
    pub fn select(&mut self, id: Option<i64>) {
        self.selected_id = id;
    }

    /// Get selected [`Wallet`] name.
    pub fn selected_name(&self) -> String {
        for w in &self.list {
            if Some(w.config.id) == self.selected_id {
                return w.config.name.to_owned()
            }
        }
        t!("wallets.unlocked")
    }

    /// Check if [`Wallet`] is selected for provided identifier.
    pub fn is_selected(&self, id: i64) -> bool {
        return Some(id) == self.selected_id;
    }

    /// Check if selected [`Wallet`] is open.
    pub fn is_selected_open(&self) -> bool {
        for w in &self.list {
            if Some(w.config.id) == self.selected_id {
                return w.is_open()
            }
        }
        false
    }

    /// Open selected [`Wallet`].
    pub fn open_selected(&mut self, password: String) -> Result<(), Error> {
        for w in self.list.iter_mut() {
            if Some(w.config.id) == self.selected_id {
                return w.open(password.clone());
            }
        }
        Err(Error::GenericError("Wallet is not selected".to_string()))
    }
}