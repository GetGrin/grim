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

use std::path::PathBuf;
use serde_derive::{Deserialize, Serialize};
use crate::Settings;

/// Tor configuration.
#[derive(Serialize, Deserialize, Clone)]
pub struct TorConfig {
    // Flag to check if Tor bridges usage is needed.
    pub(crate) use_bridges: Option<bool>
}

impl Default for TorConfig {
    fn default() -> Self {
        Self {
            use_bridges: Some(false)
        }
    }
}

impl TorConfig {
    /// Tor configuration file name.
    pub const FILE_NAME: &'static str = "tor.toml";

    /// Directory for Tor data files.
    const DIR_NAME: &'static str = "tor";

    /// Subdirectory name for Tor state.
    const STATE_SUB_DIR: &'static str = "state";
    /// Subdirectory name for Tor cache.
    const CACHE_SUB_DIR: &'static str = "cache";
    /// Subdirectory name for Tor keystore.
    const KEYSTORE_DIR: &'static str = "keystore";

    /// Save application configuration to the file.
    pub fn save(&self) {
        Settings::write_to_file(self, Settings::get_config_path(Self::FILE_NAME, None));
    }

    /// Get path from subdirectory name.
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
        let mut base = PathBuf::from(Self::state_path());
        base.push(Self::KEYSTORE_DIR);
        base.to_str().unwrap().to_string()
    }
}