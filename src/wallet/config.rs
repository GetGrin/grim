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
use grin_wallet_libwallet::{Slate};
use rand::Rng;
use serde_derive::{Deserialize, Serialize};

use crate::{AppConfig, Settings};
use crate::wallet::ConnectionsConfig;
use crate::wallet::types::{ConnectionMethod, WalletTransaction};

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
    pub min_confirmations: u64,
    /// Flag to use Dandelion to broadcast transactions.
    pub use_dandelion: Option<bool>,
    /// Flag to enable Tor listener on start.
    pub enable_tor_listener: Option<bool>,
    /// Wallet API port.
    pub api_port: Option<u16>,
    /// Delay in blocks before another transaction broadcasting attempt.
    pub tx_broadcast_timeout: Option<u64>,
}

/// Base wallets directory name.
const BASE_DIR_NAME: &'static str = "wallets";
/// Base wallets directory name.
const DB_DIR_NAME: &'static str = "db";
/// Wallet data directory name.
const DATA_DIR_NAME: &'static str = "wallet_data";
/// Wallet configuration file name.
const CONFIG_FILE_NAME: &'static str = "grim-wallet.toml";
/// Slatepacks directory name.
const SLATEPACKS_DIR_NAME: &'static str = "slatepacks";
/// Seed file name.
const SEED_FILE: &str = "wallet.seed";

/// Default value of minimal amount of confirmations.
const MIN_CONFIRMATIONS_DEFAULT: u64 = 10;

impl WalletConfig {
    /// Default account name value.
    pub const DEFAULT_ACCOUNT_LABEL: &'static str = "default";

    /// Default value of timeout for broadcasting transaction in blocks.
    pub const BROADCASTING_TIMEOUT_DEFAULT: u64 = 10;

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
                ConnectionMethod::External(id, _) => Some(*id)
            },
            min_confirmations: MIN_CONFIRMATIONS_DEFAULT,
            use_dandelion: Some(true),
            enable_tor_listener: Some(false),
            api_port: Some(rand::rng().random_range(10000..30000)),
            tx_broadcast_timeout: Some(Self::BROADCASTING_TIMEOUT_DEFAULT),
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

    /// Get wallet name by provided identifier.
    pub fn read_name_by_id(id: i64) -> Option<String> {
        let mut wallet_dir = WalletConfig::get_base_path(AppConfig::chain_type());
        wallet_dir.push(id.to_string());
        if let Some(cfg) = Self::load(wallet_dir) {
            return Some(cfg.name);
        }
        None
    }

    /// Get wallet API port by provided identifier.
    pub fn read_api_port_by_id(id: i64) -> Option<u16> {
        let mut wallet_dir = WalletConfig::get_base_path(AppConfig::chain_type());
        wallet_dir.push(id.to_string());
        if let Some(cfg) = Self::load(wallet_dir) {
            return cfg.api_port;
        }
        None
    }

    /// Get wallet connection method.
    pub fn connection(&self) -> ConnectionMethod {
        if let Some(ext_conn_id) = self.ext_conn_id {
            if let Some(conn) = ConnectionsConfig::ext_conn(ext_conn_id) {
                return ConnectionMethod::External(conn.id, conn.url);
            }
        }
        ConnectionMethod::Integrated
    }

    /// Save wallet config.
    pub fn save(&self) {
        let config_path = Self::get_config_file_path(self.chain_type, self.id);
        Settings::write_to_file(self, config_path);
    }

    /// Get wallets base directory path for provided [`ChainTypes`].
    pub fn get_base_path(chain_type: ChainTypes) -> PathBuf {
        let sub_dir = Some(chain_type.shortname());
        let mut wallets_path = Settings::base_path(sub_dir);
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

    /// Get current wallet path.
    pub fn get_wallet_path(&self) -> String {
        let chain_type = AppConfig::chain_type();
        let mut data_path = Self::get_base_path(chain_type);
        data_path.push(self.id.to_string());
        data_path.to_str().unwrap().to_string()
    }

    /// Get wallet data path.
    pub fn get_data_path(&self) -> String {
        let mut data_path = PathBuf::from(self.get_wallet_path());
        data_path.push(DATA_DIR_NAME);
        data_path.to_str().unwrap().to_string()
    }

    /// Get wallet seed path.
    pub fn seed_path(&self) -> String {
        let mut path = PathBuf::from(self.get_data_path());
        path.push(SEED_FILE);
        path.to_str().unwrap().to_string()
    }

    /// Get wallet database data path.
    pub fn get_db_path(&self) -> String {
        let mut path = PathBuf::from(self.get_data_path());
        path.push(DB_DIR_NAME);
        path.to_str().unwrap().to_string()
    }

    /// Get Slatepack file path for transaction.
    pub fn get_tx_slate_path(&self, tx: &WalletTransaction) -> PathBuf {
        let mut path = PathBuf::from(self.get_wallet_path());
        path.push(SLATEPACKS_DIR_NAME);
        if !path.exists() {
            let _ = fs::create_dir_all(path.clone());
        }
        let file = format!("{}.{}.slatepack", tx.data.tx_slate_id.unwrap(), tx.state);
        path.push(file);
        path
    }

    /// Get Slatepack file path for Slate.
    pub fn get_slate_path(&self, slate: &Slate) -> PathBuf {
        let mut path = PathBuf::from(self.get_wallet_path());
        path.push(SLATEPACKS_DIR_NAME);
        if !path.exists() {
            let _ = fs::create_dir_all(path.clone());
        }
        let file = format!("{}.{}.slatepack", slate.id, slate.state);
        path.push(file);
        path
    }

    /// Get path to extra db storage.
    pub fn get_extra_db_path(&self) -> String {
        let mut path = PathBuf::from(self.get_db_path());
        path.push("extra");
        if !path.exists() {
            let _ = fs::create_dir_all(path.clone());
        }
        path.to_str().unwrap().to_string()
    }
}