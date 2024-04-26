// Copyright 2024 The Grim Developers
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

use serde_derive::{Deserialize, Serialize};
use crate::Settings;

/// Tor SOCKS proxy server configuration.
#[derive(Serialize, Deserialize, Clone)]
pub struct TorServerConfig {
    socks_port: u16
}

/// Default SOCKS port value.
const DEFAULT_SOCKS_PORT: u16 = 9060;

impl Default for TorServerConfig {
    fn default() -> Self {
        Self {
            socks_port: DEFAULT_SOCKS_PORT,
        }
    }
}

impl TorServerConfig {
    /// Tor configuration file name.
    pub const FILE_NAME: &'static str = "tor.toml";

    /// Directory for config and Tor related files.
    const DIR_NAME: &'static str = "tor";

    /// Subdirectory name for Tor state.
    const STATE_SUB_DIR: &'static str = "state";
    /// Subdirectory name for Tor cache.
    const CACHE_SUB_DIR: &'static str = "cache";
    /// Subdirectory name for Tor keystore.
    const KEYSTORE_DIR: &'static str = "keystore";

    /// Save application configuration to the file.
    pub fn save(&self) {
        Settings::write_to_file(self, Settings::get_config_path(Self::FILE_NAME,
                                                                Some(Self::DIR_NAME.to_string())));
    }

    /// Get subdirectory path from dir name.
    fn sub_dir_path(name: &str) -> String {
        let mut base = Settings::get_base_path(Some(Self::DIR_NAME.to_string()));
        base.push(name);
        base.to_str().unwrap().to_string()
    }

    /// Get Tor state directory path.
    pub fn state_path() -> String {
        Self::sub_dir_path(Self::STATE_SUB_DIR)
    }

    /// Get Tor cache directory path.
    pub fn cache_path() -> String {
        Self::sub_dir_path(Self::CACHE_SUB_DIR)
    }

    /// Get Tor keystore directory path.
    pub fn keystore_path() -> String {
        Self::sub_dir_path(Self::KEYSTORE_DIR)
    }

    /// Get SOCKS port value.
    pub fn socks_port() -> u16 {
        let r_config = Settings::tor_config_to_read();
        r_config.socks_port
    }

    /// Save SOCKS port value.
    pub fn save_socks_port(port: u16) {
        let mut w_config = Settings::tor_config_to_update();
        w_config.socks_port = port;
        w_config.save();
    }
}