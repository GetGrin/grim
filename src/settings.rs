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

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use grin_config::ConfigError;
use grin_core::global::ChainTypes;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use crate::node::NodeConfig;
use crate::wallet::Wallets;

lazy_static! {
    /// Static settings state to be accessible globally.
    static ref SETTINGS_STATE: Arc<Settings> = Arc::new(Settings::init());
}

/// Application configuration file name.
const APP_CONFIG_FILE_NAME: &'static str = "app.toml";

/// Default external node URL.
const DEFAULT_EXTERNAL_NODE_URL: &'static str = "https://grinnnode.live:3413";

/// Common application settings.
#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    /// Run node server on startup.
    pub auto_start_node: bool,
    /// Chain type for node and wallets.
    chain_type: ChainTypes,
    /// URLs of external nodes for wallets.
    external_nodes_urls: Vec<String>
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_start_node: false,
            chain_type: ChainTypes::default(),
            external_nodes_urls: vec![
                DEFAULT_EXTERNAL_NODE_URL.to_string()
            ],
        }
    }
}

impl AppConfig {
    /// Initialize application config from the file.
    pub fn init() -> Self {
        let path = Settings::get_config_path(APP_CONFIG_FILE_NAME, None);
        let parsed = Settings::read_from_file::<AppConfig>(path.clone());
        if !path.exists() || parsed.is_err() {
            let default_config = AppConfig::default();
            Settings::write_to_file(&default_config, path);
            default_config
        } else {
            parsed.unwrap()
        }
    }

    /// Save app config to file.
    pub fn save(&self) {
        Settings::write_to_file(self, Settings::get_config_path(APP_CONFIG_FILE_NAME, None));
    }

    /// Change chain type and load new [`NodeConfig`].
    pub fn change_chain_type(chain_type: &ChainTypes) {
        let current_chain_type = Self::chain_type();
        if current_chain_type != *chain_type {
            // Save chain type at app config.
            {
                let mut w_app_config = Settings::app_config_to_update();
                w_app_config.chain_type = *chain_type;
                w_app_config.save();
            }
            // Load node config for selected chain type.
            {
                let mut w_node_config = Settings::node_config_to_update();
                let node_config = NodeConfig::for_chain_type(chain_type);
                w_node_config.node = node_config.node;
                w_node_config.peers = node_config.peers;
            }
            // Reload wallets.
            Wallets::reload(chain_type);
        }
    }

    /// Get current [`ChainTypes`] for node and wallets.
    pub fn chain_type() -> ChainTypes {
        let r_config = Settings::app_config_to_read();
        r_config.chain_type
    }

    /// Check if integrated node is starting with application.
    pub fn autostart_node() -> bool {
        let r_config = Settings::app_config_to_read();
        r_config.auto_start_node
    }

    /// Toggle integrated node autostart.
    pub fn toggle_node_autostart() {
        let autostart = Self::autostart_node();
        let mut w_app_config = Settings::app_config_to_update();
        w_app_config.auto_start_node = !autostart;
        w_app_config.save();
    }

    /// Get external nodes URLs.
    pub fn external_nodes_urls() -> Vec<String> {
        let r_config = Settings::app_config_to_read();
        r_config.external_nodes_urls.clone()
    }

    /// Add external node URL.
    pub fn add_external_node_url(address: String) {
        let mut w_config = Settings::app_config_to_update();
        w_config.external_nodes_urls.insert(0, address);
        w_config.save();
    }

}

/// Main application directory name.
const MAIN_DIR_NAME: &'static str = ".grim";

/// Provides access to application, integrated node and wallets configs.
pub struct Settings {
    /// Application config instance.
    app_config: Arc<RwLock<AppConfig>>,
    /// Integrated node config instance.
    node_config: Arc<RwLock<NodeConfig>>,
}

impl Settings {
    /// Initialize settings with app and node configs.
    fn init() -> Self {
        let app_config = AppConfig::init();
        Self {
            node_config: Arc::new(RwLock::new(NodeConfig::for_chain_type(&app_config.chain_type))),
            app_config: Arc::new(RwLock::new(app_config)),
        }
    }

    /// Get node config to read values.
    pub fn node_config_to_read() -> RwLockReadGuard<'static, NodeConfig> {
        SETTINGS_STATE.node_config.read().unwrap()
    }

    /// Get node config to update values.
    pub fn node_config_to_update() -> RwLockWriteGuard<'static, NodeConfig> {
        SETTINGS_STATE.node_config.write().unwrap()
    }

    /// Get app config to read values.
    pub fn app_config_to_read() -> RwLockReadGuard<'static, AppConfig> {
        SETTINGS_STATE.app_config.read().unwrap()
    }

    /// Get app config to update values.
    pub fn app_config_to_update() -> RwLockWriteGuard<'static, AppConfig> {
        SETTINGS_STATE.app_config.write().unwrap()
    }

    /// Get base directory path for config.
    pub fn get_base_path(sub_dir: Option<String>) -> PathBuf {
        // Check if dir exists.
        let mut path = match dirs::home_dir() {
            Some(p) => p,
            None => PathBuf::new(),
        };
        path.push(MAIN_DIR_NAME);
        if sub_dir.is_some() {
            path.push(sub_dir.unwrap());
        }
        // Create if the default path doesn't exist.
        if !path.exists() {
            let _ = fs::create_dir_all(path.clone());
        }
        path
    }

    /// Get config file path from provided name and sub-directory if needed.
    pub fn get_config_path(config_name: &str, sub_dir: Option<String>) -> PathBuf {
        let mut settings_path = Self::get_base_path(sub_dir);
        settings_path.push(config_name);
        settings_path
    }

    /// Read config from the file.
    pub fn read_from_file<T: DeserializeOwned>(config_path: PathBuf) -> Result<T, ConfigError> {
        let file_content = fs::read_to_string(config_path.clone())?;
        let parsed = toml::from_str::<T>(file_content.as_str());
        match parsed {
            Ok(cfg) => Ok(cfg),
            Err(e) => {
                return Err(ConfigError::ParseError(
                    config_path.to_str().unwrap().to_string(),
                    format!("{}", e),
                ));
            }
        }
    }

    /// Write config to the file.
    pub fn write_to_file<T: Serialize>(config: &T, path: PathBuf) {
        let conf_out = toml::to_string(config).unwrap();
        let mut file = File::create(path.to_str().unwrap()).unwrap();
        file.write_all(conf_out.as_bytes()).unwrap();
    }
}