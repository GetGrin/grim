// Copyright 2025 The Grim Developers
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

use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::{Request, Response};
use hyper_proxy2::{Intercept, Proxy, ProxyConnector};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::{Client, Error};
use hyper_util::rt::TokioExecutor;

use crate::AppConfig;

/// Handles http requests.
pub struct HttpClient {
}

impl HttpClient {
    /// Send request.
    pub async fn send(req: Request<Full<Bytes>>) -> Result<Response<Incoming>, Error> {
        if AppConfig::use_proxy() {
            if let Some(url) = AppConfig::socks_proxy_url() {
                Self::send_socks_proxy(url, req).await
            } else {
                Self::send_http_proxy(AppConfig::http_proxy_url().unwrap(), req).await
            }
        } else {
            let client = Client::builder(TokioExecutor::new())
                .build::<_, Full<Bytes>>(HttpsConnector::new());
            client.request(req).await
        }
    }

    /// Create socks proxy client.
    pub async fn send_socks_proxy(proxy_url: String, req: Request<Full<Bytes>>)
        -> Result<Response<Incoming>, Error> {
        let connector = HttpsConnector::new();
        let uri = proxy_url.parse().unwrap();
        let proxy = hyper_socks2::SocksConnector {
            proxy_addr: uri,
            auth: None,
            connector,
        }.with_tls().unwrap();
        let client = Client::builder(TokioExecutor::new())
            .build::<_, Full<Bytes>>(proxy);
        client.request(req).await
    }

    /// Create http proxy client.
    pub async fn send_http_proxy(proxy_url: String, req: Request<Full<Bytes>>)
        -> Result<Response<Incoming>, Error> {
        let uri = proxy_url.parse().unwrap();
        let proxy = Proxy::new(Intercept::All, uri);
        let connector = HttpsConnector::new();
        let proxy_connector = ProxyConnector::from_proxy(connector, proxy).unwrap();
        let client = Client::builder(TokioExecutor::new())
            .build::<_, Full<Bytes>>(proxy_connector);
        client.request(req).await
    }
}