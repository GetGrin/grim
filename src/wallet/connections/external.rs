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

/// External node connection for the wallet.
#[derive(Serialize, Deserialize, Clone)]
pub struct ExternalConnection {
    /// Node URL.
    pub url: String,
    /// Optional API secret key.
    pub secret: Option<String>
}

impl ExternalConnection {
    /// Default external node URL.
    pub const DEFAULT_EXTERNAL_NODE_URL: &'static str = "https://grinnnode.live:3413";

    pub fn new(url: String, secret: Option<String>) -> Self {
        Self { url, secret }
    }
}

impl Default for ExternalConnection {
    fn default() -> Self {
        Self { url: Self::DEFAULT_EXTERNAL_NODE_URL.to_string(), secret: None }
    }
}