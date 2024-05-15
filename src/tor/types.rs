// Copyright 2024 The Grim Developers
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

/// Tor network bridge type with binary path.
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum TorBridge {
    Snowflake(String),
    Obfs4(String)
}

impl TorBridge {
    /// Default Snowflake protocol client binary path.
    pub const DEFAULT_SNOWFLAKE_BIN_PATH: &'static str = "/usr/bin/snowflake-client";
    /// Default Obfs4 protocol proxy client binary path.
    pub const DEFAULT_OBFS4_BIN_PATH: &'static str = "/usr/bin/obfs4proxy";

    /// Get bridge protocol name.
    pub fn protocol_name(&self) -> String {
        match *self {
            TorBridge::Snowflake(_) => "snowflake".to_string(),
            TorBridge::Obfs4(_) => "obfs4".to_string()
        }
    }

    /// Get bridge client binary path.
    pub fn binary_path(&self) -> String {
        match self {
            TorBridge::Snowflake(path) => path.clone(),
            TorBridge::Obfs4(path) => path.clone()
        }
    }
}