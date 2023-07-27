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
use crate::wallet::WalletList;

/// Wallet configuration.
#[derive(Serialize, Deserialize, Clone)]
pub struct WalletConfig {
    /// Chain type for current wallet.
    chain_type: ChainTypes,
    /// Identifier for a wallet.
    id: i64,
    /// Readable wallet name.
    name: String,
    /// External node connection URL.
    external_node_url: Option<String>,
}

/// Wallet configuration file name.
const CONFIG_FILE_NAME: &'static str = "grim-wallet.toml";

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

    /// Get config file path for provided [`ChainTypes`] and wallet identifier.
    fn get_config_file_path(chain_type: &ChainTypes, id: i64) -> PathBuf {
        let mut config_path = WalletList::get_base_path(chain_type);
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
        let mut config_path = WalletList::get_base_path(&chain_type);
        config_path.push(self.id.to_string());
        config_path.to_str().unwrap().to_string()
    }

    /// Save wallet config.
    fn save(&self) {
        let config_path = Self::get_config_file_path(&self.chain_type, self.id);
        Settings::write_to_file(self, config_path);
    }

    /// Get readable wallet name.
    pub fn get_name(&self) -> &String {
        &self.name
    }

    /// Set readable wallet name.
    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.save();
    }

    /// Get external node connection URL.
    pub fn get_external_node_url(&self) -> &Option<String> {
        &self.external_node_url
    }

    /// Set external node connection URL.
    pub fn set_external_node_url(&mut self, url: Option<String>) {
        self.external_node_url = url;
        self.save();
    }
}