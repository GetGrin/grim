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

use grin_core::global::ChainTypes;
use serde_derive::{Deserialize, Serialize};

use crate::{AppConfig, Settings};
use crate::wallet::ExternalConnection;

/// Wallet connections configuration.
#[derive(Serialize, Deserialize, Clone)]
pub struct ConnectionsConfig {
    /// Network type for connections.
    chain_type: ChainTypes,
    /// URLs of external connections for wallets.
    external: Vec<ExternalConnection>
}

impl ConnectionsConfig {
    /// Wallet connections configuration file name.
    pub const FILE_NAME: &'static str = "connections.toml";

    /// Initialize configuration for provided [`ChainTypes`].
    pub fn for_chain_type(chain_type: &ChainTypes) -> Self {
        let path = Settings::config_path(Self::FILE_NAME, Some(chain_type.shortname()));
        let parsed = Settings::read_from_file::<ConnectionsConfig>(path.clone());
        if !path.exists() || parsed.is_err() {
            let default_config = ConnectionsConfig {
                chain_type: *chain_type,
                external: ExternalConnection::default(chain_type),
            };
            Settings::write_to_file(&default_config, path);
            default_config
        } else {
            parsed.unwrap()
        }
    }

    /// Save connections configuration.
    pub fn save(&mut self) {
        let config = self.clone();
        let sub_dir = Some(AppConfig::chain_type().shortname());
        Settings::write_to_file(&config, Settings::config_path(Self::FILE_NAME, sub_dir));
    }

    /// Get [`ExternalConnection`] list.
    pub fn ext_conn_list() -> Vec<ExternalConnection> {
        let r_config = Settings::conn_config_to_read();
        r_config.external.clone()
    }

    /// Save [`ExternalConnection`] in configuration.
    pub fn add_ext_conn(conn: ExternalConnection) {
        let mut w_config = Settings::conn_config_to_update();
        if let Some(pos) = w_config.external.iter().position(|c| {
            c.id == conn.id
        }) {
            w_config.external.remove(pos);
            w_config.external.insert(pos, conn);
        } else {
            w_config.external.push(conn);
        }
        w_config.save();
    }

    /// Get external node connection with provided identifier.
    pub fn ext_conn(id: i64) -> Option<ExternalConnection> {
        let r_config = Settings::conn_config_to_read();
        for c in &r_config.external {
            if c.id == id {
                return Some(c.clone());
            }
        }
        None
    }

    /// Set [`ExternalConnection`] availability flag.
    pub fn update_ext_conn_status(id: i64, available: Option<bool>) {
        let mut w_config = Settings::conn_config_to_update();
        for c in w_config.external.iter_mut() {
            if c.id == id {
                c.available = available;
                w_config.save();
                break;
            }
        }
    }

    /// Remove [`ExternalConnection`] with provided identifier.
    pub fn remove_ext_conn(id: i64) {
        let mut w_config = Settings::conn_config_to_update();
        w_config.external = w_config.external.iter().filter(|c| c.id != id).cloned().collect();
        w_config.save();
    }
}