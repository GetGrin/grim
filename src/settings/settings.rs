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
use lazy_static::lazy_static;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::node::NodeConfig;
use crate::settings::AppConfig;
use crate::wallet::ConnectionsConfig;

lazy_static! {
    /// Static settings state to be accessible globally.
    static ref SETTINGS_STATE: Arc<Settings> = Arc::new(Settings::init());
}

/// Main application directory name.
const MAIN_DIR_NAME: &'static str = ".grim";

/// Contains initialized configurations.
pub struct Settings {
    /// Application configuration.
    app_config: Arc<RwLock<AppConfig>>,
    /// Integrated node configuration.
    node_config: Arc<RwLock<NodeConfig>>,
    /// Wallet connections configuration.
    conn_config: Arc<RwLock<ConnectionsConfig>>,
}

impl Settings {
    /// Initialize settings with app and node configs.
    fn init() -> Self {
        let app_config_path = Settings::get_config_path(AppConfig::FILE_NAME, None);
        let app_config = Self::init_config::<AppConfig>(app_config_path);
        let chain_type = &app_config.chain_type;
        Self {
            node_config: Arc::new(RwLock::new(NodeConfig::for_chain_type(chain_type))),
            conn_config: Arc::new(RwLock::new(ConnectionsConfig::for_chain_type(chain_type))),
            app_config: Arc::new(RwLock::new(app_config)),
        }
    }

    /// Initialize configuration from provided file path or set [`Default`] if file not exists.
    pub fn init_config<T: Default + Serialize + DeserializeOwned>(path: PathBuf) -> T {
        let parsed = Self::read_from_file::<T>(path.clone());
        if !path.exists() || parsed.is_err() {
            let default_config = T::default();
            Settings::write_to_file(&default_config, path);
            default_config
        } else {
            parsed.unwrap()
        }
    }

    /// Get node configuration to read values.
    pub fn node_config_to_read() -> RwLockReadGuard<'static, NodeConfig> {
        SETTINGS_STATE.node_config.read().unwrap()
    }

    /// Get node configuration to update values.
    pub fn node_config_to_update() -> RwLockWriteGuard<'static, NodeConfig> {
        SETTINGS_STATE.node_config.write().unwrap()
    }

    /// Get app configuration to read values.
    pub fn app_config_to_read() -> RwLockReadGuard<'static, AppConfig> {
        SETTINGS_STATE.app_config.read().unwrap()
    }

    /// Get app configuration to update values.
    pub fn app_config_to_update() -> RwLockWriteGuard<'static, AppConfig> {
        SETTINGS_STATE.app_config.write().unwrap()
    }

    /// Get connections configuration to read values.
    pub fn conn_config_to_read() -> RwLockReadGuard<'static, ConnectionsConfig> {
        SETTINGS_STATE.conn_config.read().unwrap()
    }

    /// Get connections configuration to update values.
    pub fn conn_config_to_update() -> RwLockWriteGuard<'static, ConnectionsConfig> {
        SETTINGS_STATE.conn_config.write().unwrap()
    }

    /// Get base directory path for configuration.
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

    /// Get configuration file path from provided name and sub-directory if needed.
    pub fn get_config_path(config_name: &str, sub_dir: Option<String>) -> PathBuf {
        let mut settings_path = Self::get_base_path(sub_dir);
        settings_path.push(config_name);
        settings_path
    }

    /// Read configuration from the file.
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

    /// Write configuration to the file.
    pub fn write_to_file<T: Serialize>(config: &T, path: PathBuf) {
        let conf_out = toml::to_string(config).unwrap();
        let mut file = File::create(path.to_str().unwrap()).unwrap();
        file.write_all(conf_out.as_bytes()).unwrap();
    }
}