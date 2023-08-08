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

use crate::config::AppConfig;
use crate::Settings;
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
        let path = Settings::get_config_path(Self::FILE_NAME, Some(chain_type.shortname()));
        let parsed = Settings::read_from_file::<ConnectionsConfig>(path.clone());
        if !path.exists() || parsed.is_err() {
            let default_config = ConnectionsConfig {
                chain_type: *chain_type,
                external: if chain_type == &ChainTypes::Mainnet {
                    vec![
                        ExternalConnection::default_main()
                    ]
                } else {
                    vec![]
                },
            };
            Settings::write_to_file(&default_config, path);
            default_config
        } else {
            parsed.unwrap()
        }
    }

    /// Save connections configuration to the file.
    pub fn save(&self) {
        let chain_type = AppConfig::chain_type();
        let sub_dir = Some(chain_type.shortname());
        Settings::write_to_file(self, Settings::get_config_path(Self::FILE_NAME, sub_dir));
    }

    /// Get [`ExternalConnection`] list.
    pub fn ext_conn_list() -> Vec<ExternalConnection> {
        let r_config = Settings::conn_config_to_read();
        r_config.external.clone()
    }

    /// Save [`ExternalConnection`] in configuration.
    pub fn add_ext_conn(conn: ExternalConnection) {
        // Do not update default connection.
        if conn.url == ExternalConnection::DEFAULT_MAIN_URL {
            return;
        }
        let mut w_config = Settings::conn_config_to_update();
        let mut exists = false;
        for mut c in w_config.external.iter_mut() {
            // Update connection if config exists.
            if c.id == conn.id {
                c.url = conn.url.clone();
                c.secret = conn.secret.clone();
                exists = true;
                break;
            }
        }
        // Create new connection if URL not exists.
        if !exists {
            w_config.external.push(conn);
        }
        w_config.save();
    }

    /// Get [`ExternalConnection`] by provided identifier.
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
    pub fn update_ext_conn_availability(id: i64, available: bool) {
        let mut w_config = Settings::conn_config_to_update();
        for mut c in w_config.external.iter_mut() {
            if c.id == id {
                c.available = Some(available);
                w_config.save();
                break;
            }
        }
    }

    /// Remove external node connection by provided identifier.
    pub fn remove_ext_conn(id: i64) {
        let mut w_config = Settings::conn_config_to_update();
        let index = w_config.external.iter().position(|c| c.id == id);
        if let Some(i) = index {
            w_config.external.remove(i);
            w_config.save();
        }
    }
}