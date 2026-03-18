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

use egui::os;
use serde_derive::{Deserialize, Serialize};
use crate::tor::TorConfig;

/// Tor connection proxy type.
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum TorProxy {
    /// SOCKS5 proxy URL.
    SOCKS5(String),
    /// HTTP proxy URL.
    HTTP(String)
}

impl TorProxy {
    /// Default SOCKS5 proxy URL.
    pub const DEFAULT_SOCKS5_URL: &'static str = "socks5://127.0.0.1:9050";
    /// Default HTTP proxy URL.
    pub const DEFAULT_HTTP_URL: &'static str = "http://127.0.0.1:9050";

    /// Get proxy URL.
    pub fn url(&self) -> String {
        match self {
            TorProxy::SOCKS5(url) => url.into(),
            TorProxy::HTTP(url) => url.into()
        }
    }
}

/// Tor network bridge type.
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum TorBridge {
    /// Obfs4 bridge with binary path and connection line.
    Webtunnel(String, String),
    /// Obfs4 bridge with binary path and connection line.
    Obfs4(String, String),
    /// Snowflake bridge with binary path and connection line.
    Snowflake(String, String)
}

impl TorBridge {
    /// Default Obfs4 protocol proxy client binary path.
    pub const DEFAULT_OBFS4_BIN_PATH: &'static str = "/usr/bin/obfs4proxy";
    /// Default Snowflake protocol client binary path.
    pub const DEFAULT_SNOWFLAKE_BIN_PATH: &'static str = "/usr/bin/snowflake-client";

    /// Default webtunnel protocol connection line.
    pub const DEFAULT_WEBTUNNEL_CONN_LINE: &'static str = "webtunnel [2001:db8:beb:5884:ffcc:bfe3:2858:b06b]:443 1E242C749707B4A68A269F0D31311CE36CDFEC28 url=https://wt.gri.mw/74Fm0lKUWWMMjZpKf6iSC0UH";
    pub const ADDITIONAL_WEBTUNNEL_CONN_LINE_1: &'static str = "webtunnel [2001:db8:f1c4:ca39:40a2:2e3f:f66b:2308]:443 93557BF013203581B6B7C3BF016425F1758F7CD6 url=https://diffusesystems.net/UvVD4kzlcS8HLlpxDdRWXidiDTDt0EiZ ver=0.0.3";
    pub const ADDITIONAL_WEBTUNNEL_CONN_LINE_2: &'static str = "webtunnel [2001:db8:eedb:cae7:a345:4f72:f9cc:5de0]:443 B3C81E7A0CA474270DAA4A2C8633E1CA8935C37D url=https://wordpress.far-east-investment.ru/sORes7268CEUSRD7hAWvJU5A ver=0.0.3";
    pub const ADDITIONAL_WEBTUNNEL_CONN_LINE_3: &'static str = "webtunnel [2001:db8:945c:e0b9:7e4c:c974:ff00:d4c5]:443 91937F3EFB3BE5169788AC7C8BF07460B7E306DB url=https://kabel.entreri.de/YXbp1dNrJeOF8giAFFYWxvmf ver=0.0.3";
    pub const ADDITIONAL_WEBTUNNEL_CONN_LINE_4: &'static str = "webtunnel [2001:db8:4767:7aa2:df21:1b2b:d7f9:caee]:443 CD193CF0D0C29551928C01FCB28D1200D9F27CFA url=https://occurrence.pics/68SzSlQCRgnfSo32eLyjC1V3 ver=0.0.3";
    pub const ADDITIONAL_WEBTUNNEL_CONN_LINE_5: &'static str = "webtunnel [2001:db8:4c9a:ffe8:70f8:d5af:a8f1:cf56]:443 DA1ECF055635C1A6ED7F5B5F36296A5E3015CE57 url=https://1axfa6xb.xoomlia.com/6qtxxkjw/ ver=0.0.3";
    pub const ADDITIONAL_WEBTUNNEL_CONN_LINE_6: &'static str = "webtunnel [2001:db8:a12b:ff8:8a1a:a05b:5f21:2ccc]:443 F2A9C5AEE0A420EB9D55F9497B3C0FA243A2A770 url=https://bridge.lovecloud.me/wss-wc3p0euqrlne98t9 ver=0.0.3";
    pub const ADDITIONAL_WEBTUNNEL_CONN_LINE_7: &'static str = "webtunnel [2001:db8:8ed6:e6c9:5fc9:9f20:a373:2374]:443 1636A2EFFBAA4B162F5FF461A1663EB55C41AE11 url=https://hanoi.delivery/roQFPLtlspWT6yIKeXD6lEci ver=0.0.3";

    pub const DEFAULT_WEBTUNNEL_CONN_LINES: [&'static str; 8] = [
        TorBridge::DEFAULT_WEBTUNNEL_CONN_LINE,
        TorBridge::ADDITIONAL_WEBTUNNEL_CONN_LINE_1,
        TorBridge::ADDITIONAL_WEBTUNNEL_CONN_LINE_2,
        TorBridge::ADDITIONAL_WEBTUNNEL_CONN_LINE_3,
        TorBridge::ADDITIONAL_WEBTUNNEL_CONN_LINE_4,
        TorBridge::ADDITIONAL_WEBTUNNEL_CONN_LINE_5,
        TorBridge::ADDITIONAL_WEBTUNNEL_CONN_LINE_6,
        TorBridge::ADDITIONAL_WEBTUNNEL_CONN_LINE_7,
    ];

    /// Default Obfs4 protocol connection line.
    pub const DEFAULT_OBFS4_CONN_LINE: &'static str = "obfs4 45.76.43.226:3479 7AAFDC594147E72635DD64DB47A8CD8781F463F6 cert=bJ720bjXkmFGGAD77BsCMopkDzQ/cXDj0QntOmsBYw7Fqohq7Y7yZMV7FlECQNB1tyq1AA iat-mode=0";
    /// Default Snowflake protocol connection line.
    pub const DEFAULT_SNOWFLAKE_CONN_LINE: &'static str = "snowflake 192.0.2.4:80 8838024498816A039FCBBAB14E6F40A0843051FA fingerprint=8838024498816A039FCBBAB14E6F40A0843051FA url=https://1098762253.rsc.cdn77.org/ fronts=www.cdn77.com,www.phpmyadmin.net ice=stun:stun.l.google.com:19302,stun:stun.antisip.com:3478,stun:stun.bluesip.net:3478,stun:stun.dus.net:3478,stun:stun.epygi.com:3478,stun:stun.sonetel.net:3478,stun:stun.uls.co.za:3478,stun:stun.voipgate.com:3478,stun:stun.voys.nl:3478 utls-imitate=hellorandomizedalpn";

    /// Get bridge protocol name.
    pub fn protocol_name(&self) -> String {
        match *self {
            TorBridge::Webtunnel(_, _) => "webtunnel".to_string(),
            TorBridge::Obfs4(_, _) => "obfs4".to_string(),
            TorBridge::Snowflake(_, _) => "snowflake".to_string(),
        }
    }

    /// Get bridge client binary path.
    pub fn binary_path(&self) -> String {
        let is_android = os::OperatingSystem::from_target_os() == os::OperatingSystem::Android;
        match self {
            TorBridge::Webtunnel(path, _) => if is_android {
                TorConfig::webtunnel_path()
            } else {
                path.clone()
            },
            TorBridge::Obfs4(path, _) => path.clone(),
            TorBridge::Snowflake(path, _) => path.clone()
        }
    }

    /// Get bridge client connection line.
    pub fn connection_line(&self) -> String {
        match self {
            TorBridge::Webtunnel(_, line) => line.clone(),
            TorBridge::Obfs4(_, line) => line.clone(),
            TorBridge::Snowflake(_, line) => line.clone()
        }
    }

    /// Save binary path to provided bridge.
    pub fn save_bridge_bin_path(bridge: &TorBridge, path: String) {
        match bridge {
            TorBridge::Webtunnel(_, line) => {
                TorConfig::save_bridge(Some(TorBridge::Webtunnel(path, line.into())));
            }
            TorBridge::Obfs4(_, line) => {
                TorConfig::save_bridge(Some(TorBridge::Obfs4(path, line.into())));
            }
            TorBridge::Snowflake(_, line) => {
                TorConfig::save_bridge(Some(TorBridge::Snowflake(path, line.into())));
            }
        }
    }

    /// Save connection line to provided bridge.
    pub fn save_bridge_conn_line(bridge: &TorBridge, line: String) {
        match bridge {
            TorBridge::Webtunnel(path, _) => {
                TorConfig::save_bridge(Some(TorBridge::Webtunnel(path.into(), line)));
            }
            TorBridge::Obfs4(path, _) => {
                TorConfig::save_bridge(
                    Some(TorBridge::Obfs4(path.into(), line))
                );
            }
            TorBridge::Snowflake(path, _) => {
                TorConfig::save_bridge(
                    Some(TorBridge::Snowflake(path.into(), line))
                );
            }
        }
    }
}