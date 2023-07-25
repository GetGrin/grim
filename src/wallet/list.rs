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

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use grin_core::global::ChainTypes;

use lazy_static::lazy_static;

use crate::{AppConfig, Settings};
use crate::wallet::Wallet;

lazy_static! {
    /// Global wallets state.
    static ref WALLETS_STATE: Arc<RwLock<WalletList >> = Arc::new(RwLock::new(WalletList::load()));
}

/// List of created wallets.
pub struct WalletList {
    list: Vec<Wallet>
}

/// Base wallets directory name.
pub const BASE_DIR_NAME: &'static str = "wallets";

impl WalletList {
    /// Load list of wallets.
    fn load() -> Self {
        Self { list: Self::load_wallets(&AppConfig::chain_type()) }
    }

    /// Load wallets for provided [`ChainType`].
    fn load_wallets(chain_type: &ChainTypes) -> Vec<Wallet> {
        let mut wallets = Vec::new();
        let wallets_dir = Self::get_wallets_base_dir(chain_type);
        // Load wallets from directory.
        for dir in wallets_dir.read_dir().unwrap() {
            let wallet = Wallet::load(dir.unwrap().path());
            if let Some(w) = wallet {
                wallets.push(w);
            }
            continue;
        }
        wallets
    }

    /// Get wallets base directory for provided [`ChainTypes`].
    pub fn get_wallets_base_dir(chain_type: &ChainTypes) -> PathBuf {
        let mut wallets_path = Settings::get_base_path(Some(chain_type.shortname()));
        wallets_path.push(BASE_DIR_NAME);
        // Create wallets directory if it doesn't exist.
        if !wallets_path.exists() {
            let _ = fs::create_dir_all(wallets_path.clone());
        }
        wallets_path
    }

    /// Get list of wallets.
    pub fn list() -> Vec<Wallet> {
        let r_state = WALLETS_STATE.read().unwrap();
        r_state.list.clone()
    }

    /// Reload list of wallets for provided [`ChainTypes`].
    pub fn reload(chain_type: &ChainTypes) {
        let mut w_state = WALLETS_STATE.write().unwrap();
        w_state.list = Self::load_wallets(chain_type);
    }
}
