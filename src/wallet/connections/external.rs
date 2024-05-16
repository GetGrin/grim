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

use serde_derive::{Deserialize, Serialize};
use tor_rtcompat::BlockOn;
use tor_rtcompat::tokio::TokioNativeTlsRuntime;

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
            let runtime = TokioNativeTlsRuntime::create().unwrap();
            runtime.block_on(async {
                let url = url::Url::parse(conn.url.as_str()).unwrap();
                if let Ok(_) = url.socket_addrs(|| None) {
                    let client = hyper::Client::builder()
                        .build::<_, hyper::Body>(hyper_tls::HttpsConnector::new());
                    let req = hyper::Request::builder()
                        .method(hyper::Method::GET)
                        .uri(format!("{}/v2/owner", url.to_string()))
                        .body(hyper::Body::from(
                            r#"{"id":1,"jsonrpc":"2.0","method":"get_status","params":{} }"#)
                        )
                        .unwrap();
                    match client.request(req).await {
                        Ok(res) => {
                            let status = res.status().as_u16();
                            // Available on 200 and 401 status code.
                            if status == 200 || status == 401 {
                                ConnectionsConfig::update_ext_conn_availability(conn.id, true);
                            } else {
                                ConnectionsConfig::update_ext_conn_availability(conn.id, false);
                            }
                        }
                        Err(e) => {
                            ConnectionsConfig::update_ext_conn_availability(conn.id, false);
                        }
                    }
                } else {
                    ConnectionsConfig::update_ext_conn_availability(conn.id, false);
                }
            });

        });
    }

    /// Check external connections availability at another thread.
    pub fn start_ext_conn_availability_check() {
        let conn_list = ConnectionsConfig::ext_conn_list();
        for conn in conn_list {
            // Check every connection at separate thread.
            conn.check_conn_availability();
        }
    }
}