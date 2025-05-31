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
    #[serde(skip_serializing, skip_deserializing)]
    pub available: Option<bool>,
}

/// Default external node URL for main network.
const DEFAULT_MAIN_URLS: [&'static str; 2] = [
    "https://grincoin.org",
    "https://grinnode.live:3413"
];

/// Default external node URL for main network.
const DEFAULT_TEST_URLS: [&'static str; 1] = [
    "https://testnet.grincoin.org"
];

impl ExternalConnection {
    /// Get default connections for provided chain type.
    pub fn default(chain_type: &ChainTypes) -> Vec<ExternalConnection> {
        let urls = match chain_type {
            ChainTypes::Mainnet => DEFAULT_MAIN_URLS.to_vec(),
            _ => DEFAULT_TEST_URLS.to_vec()
        };
        urls.iter().enumerate().map(|(index, url)| {
            ExternalConnection {
                id: index as i64,
                url: url.to_string(),
                secret: None,
                available: None,
            }
        }).collect::<Vec<ExternalConnection>>()
    }

    /// Create new external connection.
    pub fn new(url: String, secret: Option<String>) -> Self {
        let id = chrono::Utc::now().timestamp();
        Self {
            id,
            url,
            secret,
            available: None,
        }
    }

    /// Check external connection availability.
    pub fn check(id: Option<i64>, ui_ctx: &egui::Context) {
        let conn_list = ConnectionsConfig::ext_conn_list();
        for conn in conn_list {
            if let Some(id) = id {
                if id == conn.id {
                    check_ext_conn(&conn, ui_ctx);
                }
            } else {
                check_ext_conn(&conn, ui_ctx);
            }
        }
    }
}

/// Check connection availability.
fn check_ext_conn(conn: &ExternalConnection, ui_ctx: &egui::Context) {
    let conn = conn.clone();
    let ui_ctx = ui_ctx.clone();
    ConnectionsConfig::update_ext_conn_status(conn.id, None);
    std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let url = url::Url::parse(conn.url.as_str()).unwrap();
                if let Ok(_) = url.socket_addrs(|| None) {
                    let addr = format!("{}v2/foreign", url.to_string());
                    // Setup http client.
                    let client = hyper_tor::Client::builder()
                        .build::<_, hyper_tor::Body>(hyper_tls::HttpsConnector::new());
                    let mut req_setup = hyper_tor::Request::builder()
                        .method(hyper_tor::Method::POST)
                        .uri(addr.clone());
                    // Setup secret key auth.
                    if let Some(key) = conn.secret {
                        let basic_auth = format!(
                            "Basic {}",
                            to_base64(&format!("grin:{}", key))
                        );
                        req_setup = req_setup
                            .header(hyper_tor::header::AUTHORIZATION, basic_auth.clone());
                    }
                    let req = req_setup.body(hyper_tor::Body::from(
                        r#"{"id":1,"jsonrpc":"2.0","method":"get_version","params":{} }"#)
                    ).unwrap();
                    // Send request.
                    match client.request(req).await {
                        Ok(res) => {
                            let status = res.status().as_u16();
                            // Available on 200 HTTP status code.
                            ConnectionsConfig::update_ext_conn_status(conn.id, Some(status == 200));
                        }
                        Err(_) => ConnectionsConfig::update_ext_conn_status(conn.id, Some(false))
                    }
                } else {
                    ConnectionsConfig::update_ext_conn_status(conn.id, Some(false));
                }
                // Repaint ui on change.
                ui_ctx.request_repaint();
            });
    });
}