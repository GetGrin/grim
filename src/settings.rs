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

lazy_static! {
    /// Static settings state to be accessible globally.
    static ref SETTINGS_STATE: Arc<Settings> = Arc::new(Settings::init());
}

const APP_CONFIG_FILE_NAME: &'static str = "app.toml";

/// Application settings config.
#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    /// Run node server on startup.
    pub auto_start_node: bool,
    /// Chain type for node server.
    pub node_chain_type: ChainTypes
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_start_node: false,
            node_chain_type: ChainTypes::default(),
        }
    }
}

impl AppConfig {
    /// Initialize application config from the disk.
    pub fn init() -> Self {
        let config_path = Settings::get_config_path(APP_CONFIG_FILE_NAME, None);
        let parsed = Settings::read_from_file::<AppConfig>(config_path.clone());
        if !config_path.exists() || parsed.is_err() {
            let default_config = AppConfig::default();
            Settings::write_to_file(&default_config, config_path);
            default_config
        } else {
            parsed.unwrap()
        }
    }

    pub fn save(&self) {
        Settings::write_to_file(self, Settings::get_config_path(APP_CONFIG_FILE_NAME, None));
    }
}

pub struct Settings {
    app_config: Arc<RwLock<AppConfig>>,
    node_config: Arc<RwLock<NodeConfig>>
}

impl Settings {
    /// Initialize settings with app and node configs from the disk.
    fn init() -> Self {
        let app_config = AppConfig::init();
        let chain_type = app_config.node_chain_type;
        Self {
            app_config: Arc::new(RwLock::new(app_config)),
            node_config: Arc::new(RwLock::new(NodeConfig::init(&chain_type)))
        }
    }

    pub fn get_node_config() -> RwLockReadGuard<'static, NodeConfig> {
        SETTINGS_STATE.node_config.read().unwrap()
    }

    pub fn get_app_config() -> RwLockReadGuard<'static, AppConfig> {
        SETTINGS_STATE.app_config.read().unwrap()
    }

    pub fn get_app_config_to_update() -> RwLockWriteGuard<'static, AppConfig> {
        SETTINGS_STATE.app_config.write().unwrap()
    }

    /// Get working directory path for application.
    pub fn get_working_path(chain_type: Option<&ChainTypes>) -> PathBuf {
        // Check if dir exists
        let mut path = match dirs::home_dir() {
            Some(p) => p,
            None => PathBuf::new(),
        };
        path.push(".grim");
        if chain_type.is_some() {
            path.push(chain_type.unwrap().shortname());
        }
        // Create if the default path doesn't exist
        if !path.exists() {
            let _ = fs::create_dir_all(path.clone());
        }
        path
    }

    /// Get config file path from provided name and [`ChainTypes`] if needed.
    pub fn get_config_path(config_name: &str, chain_type: Option<&ChainTypes>) -> PathBuf {
        let main_path = Self::get_working_path(chain_type);
        let mut settings_path = main_path.clone();
        settings_path.push(config_name);
        settings_path
    }

    /// Read config from file
    pub fn read_from_file<T: DeserializeOwned>(config_path: PathBuf) -> Result<T, ConfigError> {
        let file_content = fs::read_to_string(config_path.clone())?;
        let parsed = toml::from_str::<T>(file_content.as_str());
        match parsed {
            Ok(cfg) => { Ok(cfg) }
            Err(e) => {
                return Err(ConfigError::ParseError(
                    config_path.to_str().unwrap().to_string(),
                    format!("{}", e),
                ));
            }
        }
    }

    /// Write config to a file
    pub fn write_to_file<T: Serialize>(config: &T, path: PathBuf) {
        let conf_out = toml::to_string(config).unwrap();
        let mut file = File::create(path.to_str().unwrap()).unwrap();
        file.write_all(conf_out.as_bytes()).unwrap();
    }
}