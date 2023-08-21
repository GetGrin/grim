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
use std::string::ToString;

use grin_core::global::ChainTypes;
use serde_derive::{Deserialize, Serialize};

use crate::{AppConfig, Settings};
use crate::wallet::types::ConnectionMethod;

/// Wallet configuration.
#[derive(Serialize, Deserialize, Clone)]
pub struct WalletConfig {
    /// Current wallet account label.
    pub account: String,
    /// Chain type for current wallet.
    pub chain_type: ChainTypes,
    /// Identifier for a wallet.
    pub id: i64,
    /// Wallet name.
    pub name: String,
    /// External connection identifier.
    pub ext_conn_id: Option<i64>,
    /// Minimal amount of confirmations.
    pub min_confirmations: u64
}

/// Base wallets directory name.
const BASE_DIR_NAME: &'static str = "wallets";
/// Wallet configuration file name.
const CONFIG_FILE_NAME: &'static str = "grim-wallet.toml";
/// Slatepacks directory name.
const SLATEPACKS_DIR_NAME: &'static str = "slatepacks";

/// Default value of minimal amount of confirmations.
const MIN_CONFIRMATIONS_DEFAULT: u64 = 10;

impl WalletConfig {
    /// Default account name value.
    pub const DEFAULT_ACCOUNT_LABEL: &'static str = "default";

    /// Create new wallet config.
    pub fn create(name: String, conn_method: &ConnectionMethod) -> WalletConfig {
        // Setup configuration path.
        let id = chrono::Utc::now().timestamp();
        let chain_type = AppConfig::chain_type();
        let config_path = Self::get_config_file_path(chain_type, id);
        // Write configuration to the file.
        let config = WalletConfig {
            account: Self::DEFAULT_ACCOUNT_LABEL.to_string(),
            chain_type,
            id,
            name,
            ext_conn_id: match conn_method {
                ConnectionMethod::Integrated => None,
                ConnectionMethod::External(id) => Some(*id)
            },
            min_confirmations: MIN_CONFIRMATIONS_DEFAULT
        };
        Settings::write_to_file(&config, config_path);
        config
    }

    /// Load config from provided wallet dir.
    pub fn load(wallet_dir: PathBuf) -> Option<WalletConfig> {
        let mut config_path: PathBuf = wallet_dir.clone();
        config_path.push(CONFIG_FILE_NAME);
        if let Ok(config) = Settings::read_from_file::<WalletConfig>(config_path) {
            return Some(config)
        }
        None
    }

    /// Get wallets base directory path for provided [`ChainTypes`].
    pub fn get_base_path(chain_type: ChainTypes) -> PathBuf {
        let sub_dir = Some(chain_type.shortname());
        let mut wallets_path = Settings::get_base_path(sub_dir);
        wallets_path.push(BASE_DIR_NAME);
        // Create wallets base directory if it doesn't exist.
        if !wallets_path.exists() {
            let _ = fs::create_dir_all(wallets_path.clone());
        }
        wallets_path
    }

    /// Get config file path for provided [`ChainTypes`] and wallet identifier.
    fn get_config_file_path(chain_type: ChainTypes, id: i64) -> PathBuf {
        let mut config_path = Self::get_base_path(chain_type);
        config_path.push(id.to_string());
        // Create if the config path doesn't exist.
        if !config_path.exists() {
            let _ = fs::create_dir_all(config_path.clone());
        }
        config_path.push(CONFIG_FILE_NAME);
        config_path
    }

    /// Get current wallet data path.
    pub fn get_data_path(&self) -> String {
        let chain_type = AppConfig::chain_type();
        let mut config_path = Self::get_base_path(chain_type);
        config_path.push(self.id.to_string());
        config_path.to_str().unwrap().to_string()
    }

    /// Get slatepacks data path for current wallet.
    pub fn get_slatepacks_path(&self) -> PathBuf {
        let mut slatepacks_dir = PathBuf::from(self.get_data_path());
        slatepacks_dir.push(SLATEPACKS_DIR_NAME);
        if !slatepacks_dir.exists() {
            let _ = fs::create_dir_all(slatepacks_dir.clone());
        }
        slatepacks_dir
    }

    /// Save wallet config.
    pub fn save(&self) {
        let config_path = Self::get_config_file_path(self.chain_type, self.id);
        Settings::write_to_file(self, config_path);
    }

    /// Save account label value.
    pub fn save_account(&mut self, label: &String) {
        self.account = label.to_owned();
        self.save();
    }
}