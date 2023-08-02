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

use grin_core::global::ChainTypes;
use serde_derive::{Deserialize, Serialize};

use crate::{AppConfig, Settings};

/// Wallet configuration.
#[derive(Serialize, Deserialize, Clone)]
pub struct WalletConfig {
    /// Chain type for current wallet.
    pub(crate) chain_type: ChainTypes,
    /// Identifier for a wallet.
    pub(crate) id: i64,
    /// Human-readable wallet name for ui.
    pub(crate) name: String,
    /// External node connection URL.
    pub(crate) external_node_url: Option<String>,
}

/// Wallet configuration file name.
const CONFIG_FILE_NAME: &'static str = "grim-wallet.toml";
/// Base wallets directory name.
pub const BASE_DIR_NAME: &'static str = "wallets";

impl WalletConfig {
    /// Create wallet config.
    pub fn create(name: String, external_node_url: Option<String>) -> WalletConfig {
        let id = chrono::Utc::now().timestamp();
        let chain_type = AppConfig::chain_type();
        let config_path = Self::get_config_file_path(&chain_type, id);

        let config = WalletConfig { chain_type, id, name, external_node_url };
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
    pub fn get_base_path(chain_type: &ChainTypes) -> PathBuf {
        let mut wallets_path = Settings::get_base_path(Some(chain_type.shortname()));
        wallets_path.push(BASE_DIR_NAME);
        // Create wallets base directory if it doesn't exist.
        if !wallets_path.exists() {
            let _ = fs::create_dir_all(wallets_path.clone());
        }
        wallets_path
    }

    /// Get config file path for provided [`ChainTypes`] and wallet identifier.
    fn get_config_file_path(chain_type: &ChainTypes, id: i64) -> PathBuf {
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
        let mut config_path = Self::get_base_path(&chain_type);
        config_path.push(self.id.to_string());
        config_path.to_str().unwrap().to_string()
    }

    /// Save wallet config.
    fn save(&self) {
        let config_path = Self::get_config_file_path(&self.chain_type, self.id);
        Settings::write_to_file(self, config_path);
    }

    /// Set external node connection URL.
    pub fn save_external_node_url(&mut self, url: Option<String>) {
        self.external_node_url = url;
        self.save();
    }
}