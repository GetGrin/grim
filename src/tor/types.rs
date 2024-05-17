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

/// Tor network bridge type.
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum TorBridge {
    /// Obfs4 bridge with connection line and binary path.
    Obfs4(String, String),
    /// Snowflake bridge with connection line and binary path.
    Snowflake(String, String)
}

impl TorBridge {
    /// Default Obfs4 protocol proxy client binary path.
    pub const DEFAULT_OBFS4_BIN_PATH: &'static str = "/usr/bin/obfs4proxy";
    /// Default Snowflake protocol client binary path.
    pub const DEFAULT_SNOWFLAKE_BIN_PATH: &'static str = "/usr/bin/snowflake-client";

    /// Default Obfs4 protocol connection line.
    pub const DEFAULT_OBFS4_CONN_LINE: &'static str = "obfs4 45.76.43.226:3479 7AAFDC594147E72635DD64DB47A8CD8781F463F6 cert=bJ720bjXkmFGGAD77BsCMopkDzQ/cXDj0QntOmsBYw7Fqohq7Y7yZMV7FlECQNB1tyq1AA iat-mode=0";
    /// Default Snowflake protocol connection line.
    pub const DEFAULT_SNOWFLAKE_CONN_LINE: &'static str = "snowflake 192.0.2.4:80 8838024498816A039FCBBAB14E6F40A0843051FA fingerprint=8838024498816A039FCBBAB14E6F40A0843051FA url=https://1098762253.rsc.cdn77.org/ fronts=www.cdn77.com,www.phpmyadmin.net ice=stun:stun.l.google.com:19302,stun:stun.antisip.com:3478,stun:stun.bluesip.net:3478,stun:stun.dus.net:3478,stun:stun.epygi.com:3478,stun:stun.sonetel.net:3478,stun:stun.uls.co.za:3478,stun:stun.voipgate.com:3478,stun:stun.voys.nl:3478 utls-imitate=hellorandomizedalpn";

    /// Get bridge protocol name.
    pub fn protocol_name(&self) -> String {
        match *self {
            TorBridge::Obfs4(_, _) => "obfs4".to_string(),
            TorBridge::Snowflake(_, _) => "snowflake".to_string()
        }
    }

    /// Get bridge client binary path.
    pub fn binary_path(&self) -> String {
        match self {
            TorBridge::Obfs4(path, _) => path.clone(),
            TorBridge::Snowflake(path, _) => path.clone()
        }
    }

    /// Get bridge client connection line.
    pub fn connection_line(&self) -> String {
        match self {
            TorBridge::Obfs4(_, line) => line.clone(),
            TorBridge::Snowflake(_, line) => line.clone()
        }
    }
}