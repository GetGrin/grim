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

use grin_config::{config, ConfigError, ConfigMembers, GlobalConfig};
use grin_config::config::{API_SECRET_FILE_NAME, FOREIGN_API_SECRET_FILE_NAME, SERVER_CONFIG_FILE_NAME};
use grin_core::global::ChainTypes;
use serde::{Deserialize, Serialize};

use crate::Settings;

/// Wrapped node config to be used by [`grin_servers::Server`].
#[derive(Serialize, Deserialize)]
pub struct NodeConfig {
    pub members: ConfigMembers
}

impl NodeConfig {
    /// Initialize integrated node config.
    pub fn init(chain_type: ChainTypes) -> Self {
        let _ = Self::check_api_secret_files(chain_type, API_SECRET_FILE_NAME);
        let _ = Self::check_api_secret_files(chain_type, FOREIGN_API_SECRET_FILE_NAME);

        let config_members = Self::for_chain_type(chain_type);
        Self {
            members: config_members
        }
    }

    /// Initialize config with provided [`ChainTypes`].
    pub fn for_chain_type(chain_type: ChainTypes) -> ConfigMembers {
        let path = Settings::get_config_path(SERVER_CONFIG_FILE_NAME, Some(chain_type));
        let parsed = Settings::read_from_file::<ConfigMembers>(path.clone());
        if !path.exists() || parsed.is_err() {
            let mut default_config = GlobalConfig::for_chain(&chain_type);
            default_config.update_paths(&Settings::get_working_path(Some(chain_type)));
            let config = default_config.members.unwrap();
            Settings::write_to_file(&config, path);
            config
        } else {
            parsed.unwrap()
        }
    }

    /// Save node config to disk.
    pub fn save(&mut self) {
        let chain_type = self.members.server.chain_type;
        let config_path = Settings::get_config_path(SERVER_CONFIG_FILE_NAME, Some(chain_type));
        Settings::write_to_file(&self.members, config_path);
    }

    /// Check that the api secret files exist and are valid.
    fn check_api_secret_files(
        chain_type: ChainTypes,
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