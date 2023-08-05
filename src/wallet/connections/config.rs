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

use std::sync::{Arc, RwLock};

use lazy_static::lazy_static;
use serde_derive::{Deserialize, Serialize};

use crate::Settings;
use crate::wallet::ExternalConnection;

lazy_static! {
    /// Static connections state to be accessible globally.
    static ref CONNECTIONS_STATE: Arc<RwLock<ConnectionsConfig>> = Arc::new(
        RwLock::new(
            Settings::init_config(Settings::get_config_path(CONFIG_FILE_NAME, None))
        )
    );
}

/// Wallet connections configuration.
#[derive(Serialize, Deserialize, Clone)]
pub struct ConnectionsConfig {
    /// URLs of external connections for wallets.
    external: Vec<ExternalConnection>
}

impl Default for ConnectionsConfig {
    fn default() -> Self {
        Self {
            external: vec![
                ExternalConnection::default()
            ],
        }
    }
}

/// Wallet configuration file name.
const CONFIG_FILE_NAME: &'static str = "connections.toml";

impl ConnectionsConfig {
    /// Save connections config to file.
    pub fn save(&self) {
        Settings::write_to_file(self, Settings::get_config_path(CONFIG_FILE_NAME, None));
    }

    /// Get external connections for the wallet.
    pub fn external_connections() -> Vec<ExternalConnection> {
        let r_config = CONNECTIONS_STATE.read().unwrap();
        r_config.external.clone()
    }

    /// Save external connection for the wallet in app config.
    pub fn add_external_connection(conn: ExternalConnection) {
        // Do not update default connection.
        if conn.url == ExternalConnection::DEFAULT_EXTERNAL_NODE_URL {
            return;
        }
        let mut w_config = CONNECTIONS_STATE.write().unwrap();
        let mut exists = false;
        for mut c in w_config.external.iter_mut() {
            // Update connection if URL exists.
            if c.url == conn.url {
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

    /// Save external connection for the wallet in app config.
    pub fn update_external_connection(conn: ExternalConnection, updated: ExternalConnection) {
        // Do not update default connection.
        if conn.url == ExternalConnection::DEFAULT_EXTERNAL_NODE_URL {
            return;
        }
        let mut w_config = CONNECTIONS_STATE.write().unwrap();
        for mut c in w_config.external.iter_mut() {
            // Update connection if URL exists.
            if c.url == conn.url {
                c.url = updated.url.clone();
                c.secret = updated.secret.clone();
                break;
            }
        }
        w_config.save();
    }

    /// Get external node connection secret from provided URL.
    pub fn get_external_connection_secret(url: String) -> Option<String> {
        let r_config = CONNECTIONS_STATE.read().unwrap();
        for c in &r_config.external {
            if c.url == url {
                return c.secret.clone();
            }
        }
        None
    }

    /// Remove external node connection.
    pub fn remove_external_connection(conn: &ExternalConnection) {
        let mut w_config = CONNECTIONS_STATE.write().unwrap();
        let index = w_config.external.iter().position(|c| c.url == conn.url);
        if let Some(i) = index {
            w_config.external.remove(i);
            w_config.save();
        }
    }
}