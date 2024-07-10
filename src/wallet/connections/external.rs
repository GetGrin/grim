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

use grin_util::to_base64;
use serde_derive::{Deserialize, Serialize};

use crate::wallet::ConnectionsConfig;

/// External connection for the wallet.
#[derive(Serialize, Deserialize, Clone, PartialEq)]
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
    fn check_conn_availability(&self) {
        let conn = self.clone();
        ConnectionsConfig::update_ext_conn_status(conn.id, None);
        std::thread::spawn(move || {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    let url = url::Url::parse(conn.url.as_str()).unwrap();
                    if let Ok(_) = url.socket_addrs(|| None) {
                        let addr = format!("{}v2/foreign", url.to_string());
                        // Setup http client.
                        let client = hyper::Client::builder()
                            .build::<_, hyper::Body>(hyper_tls::HttpsConnector::new());
                        let mut req_setup = hyper::Request::builder()
                            .method(hyper::Method::POST)
                            .uri(addr.clone());
                        // Setup secret key auth.
                        if let Some(key) = conn.secret {
                            let basic_auth = format!(
                                "Basic {}",
                                to_base64(&format!("grin:{}", key))
                            );
                            req_setup = req_setup
                                .header(hyper::header::AUTHORIZATION, basic_auth.clone());
                        }
                        let req = req_setup.body(hyper::Body::from(
                            r#"{"id":1,"jsonrpc":"2.0","method":"get_version","params":{} }"#)
                        ).unwrap();
                        // Send request.
                        match client.request(req).await {
                            Ok(res) => {
                                let status = res.status().as_u16();
                                // Available on 200 HTTP status code.
                                if status == 200 {
                                    ConnectionsConfig::update_ext_conn_status(conn.id, Some(true));
                                } else {
                                    ConnectionsConfig::update_ext_conn_status(conn.id, Some(false));
                                }
                            }
                            Err(_) => {
                                ConnectionsConfig::update_ext_conn_status(conn.id, Some(false));
                            }
                        }
                    } else {
                        ConnectionsConfig::update_ext_conn_status(conn.id, Some(false));
                    }
                });
        });
    }

    /// Check external connections availability.
    pub fn check_ext_conn_availability(id: Option<i64>) {
        let conn_list = ConnectionsConfig::ext_conn_list();
        for conn in conn_list {
            if let Some(id) = id {
                if id == conn.id {
                    conn.check_conn_availability();
                }
            } else {
                conn.check_conn_availability();
            }
        }
    }
}