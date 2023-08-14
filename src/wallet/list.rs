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
    /// List of wallets for [`ChainTypes::Mainnet`].
    pub main_list: Vec<Wallet>,
    /// List of wallets for [`ChainTypes::Testnet`].
    pub test_list: Vec<Wallet>,
    /// Selected [`Wallet`] identifier.
    pub selected_id: Option<i64>,
}

impl Default for WalletList {
    fn default() -> Self {
        let (main_list, test_list) = Self::init();
        Self { main_list, test_list, selected_id: None }
    }
}

impl WalletList {
    /// Initialize [`Wallet`] lists for [`ChainTypes::Mainnet`] and [`ChainTypes::Testnet`].
    fn init() -> (Vec<Wallet>, Vec<Wallet>) {
        let mut main_wallets = Vec::new();
        let mut test_wallets = Vec::new();
        let chain_types = vec![ChainTypes::Mainnet, ChainTypes::Testnet];
        for chain in chain_types {
            // initialize wallets from base directory.
            let wallets_dir = WalletConfig::get_base_path(chain);
            for dir in wallets_dir.read_dir().unwrap() {
                let wallet_dir = dir.unwrap().path();
                if wallet_dir.is_dir() {
                    let wallet = Wallet::init(wallet_dir);
                    if let Some(w) = wallet {
                        if chain == ChainTypes::Testnet {
                            test_wallets.push(w);
                        } else if chain == ChainTypes::Mainnet {
                            main_wallets.push(w);
                        }
                    }
                }
            }
        }
        // Sort wallets by id.
        main_wallets.sort_by_key(|w| -w.config.id);
        test_wallets.sort_by_key(|w| -w.config.id);
        (main_wallets, test_wallets)
    }

    /// Get [`Wallet`] list for current [`ChainTypes`].
    pub fn list(&self) -> &Vec<Wallet> {
        if AppConfig::chain_type() == ChainTypes::Mainnet {
            &self.main_list
        } else {
            &self.test_list
        }
    }

    /// Get mutable [`Wallet`] list for current [`ChainTypes`].
    pub fn mut_list(&mut self) -> &mut Vec<Wallet> {
        if AppConfig::chain_type() == ChainTypes::Mainnet {
            &mut self.main_list
        } else {
            &mut self.test_list
        }
    }

    /// Add created [`Wallet`] to the list.
    pub fn add(&mut self, wallet: Wallet) {
        self.selected_id = Some(wallet.config.id);
        let list = self.mut_list();
        list.insert(0, wallet);
    }

    /// Remove [`Wallet`] with provided identifier.
    pub fn remove(&mut self, id: i64) {
        let list = self.mut_list();
        for (index, wallet) in list.iter().enumerate() {
            if wallet.config.id == id {
                list.remove(index);
                return;
            }
        }
    }

    /// Select [`Wallet`] with provided identifier.
    pub fn select(&mut self, id: Option<i64>) {
        self.selected_id = id;
    }

    /// Get selected [`Wallet`] name.
    pub fn selected_name(&self) -> String {
        for w in self.list() {
            if Some(w.config.id) == self.selected_id {
                return w.config.name.to_owned()
            }
        }
        t!("wallets.unlocked")
    }

    /// Check if selected [`Wallet`] is open.
    pub fn is_selected_open(&self) -> bool {
        for w in self.list() {
            if Some(w.config.id) == self.selected_id {
                return w.is_open()
            }
        }
        false
    }

    /// Check if current list is empty.
    pub fn is_current_list_empty(&self) -> bool {
        self.list().is_empty()
    }

    /// Open selected [`Wallet`].
    pub fn open_selected(&mut self, password: String) -> Result<(), Error> {
        let selected_id = self.selected_id.clone();
        for w in self.mut_list() {
            if Some(w.config.id) == selected_id {
                return w.open(password.clone());
            }
        }
        Err(Error::GenericError("Wallet is not selected".to_string()))
    }
}