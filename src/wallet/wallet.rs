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

use std::ffi::OsString;
use std::path::PathBuf;
use crate::node::NodeConfig;

use crate::wallet::WalletConfig;

/// Wallet loaded from config.
#[derive(Clone)]
pub struct Wallet {
    /// Identifier for a wallet, name of wallet directory.
    id: OsString,
    /// Base path for wallet data.
    pub(crate) path: String,
    /// Loaded file config.
    pub(crate) config: WalletConfig,
}

impl Wallet {
    /// Create new wallet from provided name.
    pub fn create(name: String ) {

    }

    /// Load wallet from provided data path.
    pub fn load(data_path: PathBuf) -> Option<Wallet> {
        if !data_path.is_dir() {
            return None;
        }
        let wallet_config = WalletConfig::load(data_path.clone());
        if let Some(config) = wallet_config {
            // Set id as wallet directory name.
            let id = data_path.file_name().unwrap().to_os_string();
            let path = data_path.to_str().unwrap().to_string();
            return Some(Self { id, path, config });
        }
        None
    }

    /// Get wallet node connection URL.
    pub fn get_connection_url(&self) -> String {
        match self.config.get_external_node_url() {
            None => {
                format!("http://{}", NodeConfig::get_api_address())
            }
            Some(url) => url.to_string()
        }
    }
}