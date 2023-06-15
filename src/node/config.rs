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

use std::{fs, thread};
use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

use grin_config::{config, ConfigError, ConfigMembers, GlobalConfig};
use grin_config::config::{API_SECRET_FILE_NAME, FOREIGN_API_SECRET_FILE_NAME, SERVER_CONFIG_FILE_NAME};
use grin_core::global::ChainTypes;
use serde::{Deserialize, Serialize};

use crate::Settings;

/// Node config that contains [`GlobalConfig`] to be used by [`grin_servers::Server`].
#[derive(Serialize, Deserialize)]
pub struct NodeConfig {
    pub global_config: GlobalConfig,
    update_needed: AtomicBool,
    updating: AtomicBool
}

impl NodeConfig {
    /// Initialize node config with provided chain type from the disk.
    pub fn init(chain_type: &ChainTypes) -> Self {
        let _ = Self::check_api_secret_files(chain_type, API_SECRET_FILE_NAME);
        let _ = Self::check_api_secret_files(chain_type, FOREIGN_API_SECRET_FILE_NAME);

        let config_path = Settings::get_config_path(SERVER_CONFIG_FILE_NAME, Some(chain_type));

        // Create default config if it doesn't exist or has wrong format.
        if !config_path.exists() || toml::from_str::<ConfigMembers>(
            fs::read_to_string(config_path.clone()).unwrap().as_str()
        ).is_err() {
            let mut default_config = GlobalConfig::for_chain(chain_type);
            default_config.update_paths(&Settings::get_working_path(Some(chain_type)));
            let _ = default_config.write_to_file(config_path.to_str().unwrap());
        }

        let config = GlobalConfig::new(config_path.to_str().unwrap());

        Self {
            global_config: config.unwrap(),
            update_needed: AtomicBool::new(false),
            updating: AtomicBool::new(false)
        }
    }

    /// Write node config on disk.
    pub fn save_config(&self) {
        if self.updating.load(Ordering::Relaxed) {
            self.update_needed.store(true, Ordering::Relaxed);
            return;
        }

        thread::spawn(move || loop {
            let config = Settings::get_node_config();
            config.update_needed.store(false, Ordering::Relaxed);
            config.updating.store(true, Ordering::Relaxed);

            let chain_type = &config.global_config.members.clone().unwrap().server.chain_type;
            let config_path = Settings::get_config_path(SERVER_CONFIG_FILE_NAME, Some(chain_type));

            // Write config to file.
            let conf_out = toml::to_string(&config.global_config.members).unwrap();
            let mut file = File::create(config_path.to_str().unwrap()).unwrap();
            file.write_all(conf_out.as_bytes()).unwrap();

            if !config.update_needed.load(Ordering::Relaxed) {
                config.updating.store(false, Ordering::Relaxed);
                break;
            }
        });
    }

    /// Check that the api secret files exist and are valid.
    fn check_api_secret_files(
        chain_type: &ChainTypes,
        secret_file_name: &str,
    ) -> Result<(), ConfigError> {
        let grin_path = Settings::get_working_path(Some(chain_type));
        let mut api_secret_path = grin_path;
        api_secret_path.push(secret_file_name);
        if !api_secret_path.exists() {
            config::init_api_secret(&api_secret_path)
        } else {
            config::check_api_secret(&api_secret_path)
        }
    }
}