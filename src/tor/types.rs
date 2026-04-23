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
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
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
    pub const ADDITIONAL_WEBTUNNEL_CONN_LINE_1: &'static str = "webtunnel [2001:db8:289b:84cd:4be3:77f1:1cdd:9cb1]:443 D71C8E9C2180D2F35DEBF4A39BFCA6972F076D1C sni-imitation=yandex.ru,google.com,dzen.ru,vk.com,mail.ru,ozon.ru,ya.ru,www.wildberries.ru,rutube.ru,www.avito.ru,ok.ru,vkvideo.ru url=https://streaming.the-forgotten-tales.com/gz9X1VBgl0r1Xfx3dHdNl5Tl";
    pub const ADDITIONAL_WEBTUNNEL_CONN_LINE_2: &'static str = "webtunnel [2001:db8:dee9:5852:b4dc:7e14:21bd:c99b]:443 8ADF1761FA735FDD763781BB94A16EAB64A1CF6C url=https://app01.oneclickhost.eu/WJSgXJRlNnMStkuLZygVJ7lo";
    pub const ADDITIONAL_WEBTUNNEL_CONN_LINE_3: &'static str = "webtunnel [2001:db8:eedb:cae7:a345:4f72:f9cc:5de0]:443 B3C81E7A0CA474270DAA4A2C8633E1CA8935C37D url=https://wordpress.far-east-investment.ru/sORes7268CEUSRD7hAWvJU5A";
    pub const ADDITIONAL_WEBTUNNEL_CONN_LINE_4: &'static str = "webtunnel [2001:db8:945c:e0b9:7e4c:c974:ff00:d4c5]:443 91937F3EFB3BE5169788AC7C8BF07460B7E306DB url=https://kabel.entreri.de/YXbp1dNrJeOF8giAFFYWxvmf";
    pub const ADDITIONAL_WEBTUNNEL_CONN_LINE_5: &'static str = "webtunnel [2001:db8:7d4:9e13:8c7a:7e3:1f62:d790]:443 B7E362F9079D0C908F204581EB019034023BB224 url=https://balades-et-gouts.fr/xt70R9oyJt3B1xj89UCWPdLt";
    pub const ADDITIONAL_WEBTUNNEL_CONN_LINE_6: &'static str = "webtunnel [2001:db8:8c88:1b17:d7ae:cb68:f28e:e31c]:443 C115DAC2FE991CA25DDD43D7D4D398FEA9AA4C01 url=https://foglab.net/t9crLwo4LzFDWHdwcGf9gFrk";
    pub const ADDITIONAL_WEBTUNNEL_CONN_LINE_7: &'static str = "webtunnel [2001:db8:8ed6:e6c9:5fc9:9f20:a373:2374]:443 1636A2EFFBAA4B162F5FF461A1663EB55C41AE11 url=https://hanoi.delivery/roQFPLtlspWT6yIKeXD6lEci";

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
    pub const DEFAULT_OBFS4_CONN_LINE: &'static str = "obfs4 51.83.248.35:25981 D08B4760D128C1A65506577E063D9D26C2A71815 cert=UJWUh+sIDdOKja/byBM2+qP9AFNl86hkGRFJ/lM1GWKP79eCu3PT4WTXI2gdXYULbQ0EMg iat-mode=0";
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

    /// Get bridge client binary name.
    pub fn binary_name(&self) -> String {
        let path = self.binary_path();
        path.split(std::path::MAIN_SEPARATOR_STR).last().unwrap().to_string()
    }

    /// Get bridge client connection line.
    pub fn connection_line(&self) -> String {
        match self {
            TorBridge::Webtunnel(_, line) => line.clone(),
            TorBridge::Obfs4(_, line) => line.clone(),
            TorBridge::Snowflake(_, line) => line.clone()
        }
    }

    /// Update bridge connection line.
    pub fn update_conn_line(&mut self, l: String) {
        *self = match TorConfig::get_bridge().unwrap() {
            TorBridge::Webtunnel(bin, _) => TorBridge::Webtunnel(bin, l.clone()),
            TorBridge::Obfs4(bin, _) => TorBridge::Obfs4(bin, l.clone()),
            TorBridge::Snowflake(bin, _) => TorBridge::Snowflake(bin, l.clone()),
        };
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