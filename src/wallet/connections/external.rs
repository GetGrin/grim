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

use std::time::Duration;

use serde_derive::{Deserialize, Serialize};

use crate::AppConfig;
use crate::wallet::ConnectionsConfig;

/// External connection for the wallet.
#[derive(Serialize, Deserialize, Clone)]
pub struct ExternalConnection {
    /// Connection identifier.
    pub id: i64,
    /// Node URL.
    pub url: String,
    /// Optional API secret key.
    pub secret: Option<String>,

    /// Flag to check if server is available.
    #[serde(skip_serializing)]
    pub available: Option<bool>
}

impl ExternalConnection {
    /// Default external node URL for main network.
    pub const DEFAULT_MAIN_URL: &'static str = "https://grinnode.live:3413";

    /// External connections availability check delay.
    const AV_CHECK_DELAY: Duration = Duration::from_millis(60 * 1000);

    /// Create default external connection.
    pub fn default_main() -> Self {
        Self { id: 1, url: Self::DEFAULT_MAIN_URL.to_string(), secret: None, available: None }
    }

    /// Create new external connection.
    pub fn new(url: String, secret: Option<String>) -> Self {
        let id = chrono::Utc::now().timestamp();
        Self { id, url, secret, available: None }
    }

    /// Check connection availability.
    pub fn check_conn_availability(&self) {
        // Check every connection at separate thread.
        let conn = self.clone();
        std::thread::spawn(move || {
            let url = url::Url::parse(conn.url.as_str()).unwrap();
            if let Ok(addr) = url.socket_addrs(|| None) {
                match std::net::TcpStream::connect_timeout(&addr[0], Self::AV_CHECK_DELAY) {
                    Ok(_) => {
                        ConnectionsConfig::update_ext_conn_availability(conn.id, true);
                    }
                    Err(_) => {
                        ConnectionsConfig::update_ext_conn_availability(conn.id, false);
                    }
                }
            } else {
                ConnectionsConfig::update_ext_conn_availability(conn.id, false);
            }
        });
    }

    /// Start external connections availability check at another thread.
    pub fn start_ext_conn_availability_check() {
        std::thread::spawn(move || {
            let chain_type = AppConfig::chain_type();
            loop {
                // Check external connections URLs availability.
                let conn_list = ConnectionsConfig::ext_conn_list();
                for conn in conn_list {
                    // Check every connection at separate thread.
                    conn.check_conn_availability();
                }

                // Stop checking if connections are not showing or network type was changed.
                if !AppConfig::show_connections_network_panel()
                    || chain_type != AppConfig::chain_type() {
                    break;
                }
            }
        });
    }
}