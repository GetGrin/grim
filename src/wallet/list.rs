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

use crate::AppConfig;
use crate::wallet::{Wallet, WalletConfig};

/// [`Wallet`] list container.
#[derive(Clone)]
pub struct WalletList {
    /// List of wallets for [`ChainTypes::Mainnet`].
    pub main_list: Vec<Wallet>,
    /// List of wallets for [`ChainTypes::Testnet`].
    pub test_list: Vec<Wallet>,

    /// Selected wallet id.
    selected: Option<i64>,
}

impl Default for WalletList {
    fn default() -> Self {
        let (main_list, test_list) = Self::init();
        Self {
            main_list,
            test_list,
            selected: None
        }
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
        main_wallets.sort_by_key(|w| -w.get_config().id);
        test_wallets.sort_by_key(|w| -w.get_config().id);
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
        let list = self.mut_list();
        list.insert(0, wallet);
    }

    /// Remove [`Wallet`] with provided identifier.
    pub fn remove(&mut self, id: i64) {
        let list = self.mut_list();
        for (index, wallet) in list.iter().enumerate() {
            if wallet.get_config().id == id {
                list.remove(index);
                return;
            }
        }
    }

    /// Select wallet.
    pub fn select(&mut self, id: Option<i64>) {
        self.selected = id;
    }

    /// Get selected wallet.
    pub fn selected(&self) -> Box<Option<&Wallet>> {
        if self.selected.is_none() {
            return Box::new(None);
        }
        let list = self.list();
        for wallet in list {
            if wallet.get_config().id == self.selected.unwrap() {
                return Box::new(Some(wallet));
            }
        }
        Box::new(None)
    }
}