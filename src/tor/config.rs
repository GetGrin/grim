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

use std::path::PathBuf;
use serde_derive::{Deserialize, Serialize};

use crate::Settings;
use crate::tor::{TorBridge, TorProxy};

const TOR_CONFIG_VERSION: i32 = 1;

/// Tor configuration.
#[derive(Serialize, Deserialize, Clone)]
pub struct TorConfig {
    /// Proxy for tor connections.
    proxy: Option<TorProxy>,
    /// SOCKS5 proxy type.
    proxy_socks5: TorProxy,
    /// HTTP proxy type.
    proxy_http: TorProxy,

    /// Selected bridge type.
    bridge: Option<TorBridge>,
    /// Webtunnel bridge type.
    webtunnel: TorBridge,
    /// Obfs4 bridge type.
    obfs4: TorBridge,
    /// Snowflake bridge type.
    snowflake: TorBridge,

    /// Config version.
    ver: Option<i32>
}

impl Default for TorConfig {
    fn default() -> Self {
        let webtunnel = Self::default_webtunnel_bridge();
        Self {
            proxy: None,
            proxy_socks5: TorProxy::HTTP(TorProxy::DEFAULT_SOCKS5_URL.to_string()),
            proxy_http: TorProxy::HTTP(TorProxy::DEFAULT_HTTP_URL.to_string()),
            bridge: Some(webtunnel.clone()),
            webtunnel,
            obfs4: TorBridge::Obfs4(
                TorBridge::DEFAULT_OBFS4_BIN_PATH.to_string(),
                TorBridge::DEFAULT_OBFS4_CONN_LINE.to_string()
            ),
            snowflake: TorBridge::Snowflake(
                TorBridge::DEFAULT_SNOWFLAKE_BIN_PATH.to_string(),
                TorBridge::DEFAULT_SNOWFLAKE_CONN_LINE.to_string()
            ),
            ver: Some(TOR_CONFIG_VERSION),
        }
    }
}

impl TorConfig {
    /// Tor configuration file name.
    pub const FILE_NAME: &'static str = "tor.toml";

    /// Directory for Tor data files.
    const DIR_NAME: &'static str = "tor";

    /// Subdirectory name for Tor state.
    const STATE_SUB_DIR: &'static str = "state";
    /// Subdirectory name for Tor cache.
    const CACHE_SUB_DIR: &'static str = "cache";
    /// Subdirectory name for Tor keystore.
    const KEYSTORE_DIR: &'static str = "keystore";

    /// Webtunnel binary name.
    pub const WEBTUNNEL_BIN: &'static str = "webtunnel";
    /// Webtunnel Android binary name.
    pub const WEBTUNNEL_ANDROID_BIN: &'static str = "libwebtunnel.so";

    /// Save application configuration to the file.
    pub fn save(&self) {
        Settings::write_to_file(self, Settings::config_path(Self::FILE_NAME, None));
    }

    /// Get base Tor directory path.
    fn base_path() -> PathBuf {
        Settings::base_path(Some(Self::DIR_NAME.to_string()))
    }

    /// Get path from subdirectory name.
    fn sub_dir_path(name: &str) -> String {
        let mut base = Self::base_path();
        base.push(name);
        base.to_str().unwrap().to_string()
    }

    /// Get Tor state directory path.
    pub fn state_path() -> String {
        Self::sub_dir_path(Self::STATE_SUB_DIR)
    }

    /// Get Tor cache directory path.
    pub fn cache_path() -> String {
        Self::sub_dir_path(Self::CACHE_SUB_DIR)
    }

    /// Get Tor keystore directory path.
    pub fn keystore_path() -> String {
        let mut base = PathBuf::from(Self::state_path());
        base.push(Self::KEYSTORE_DIR);
        base.to_str().unwrap().to_string()
    }

    /// Get default webtunnel bridge.
    pub fn default_webtunnel_bridge() -> TorBridge {
        TorBridge::Webtunnel(
            if egui::os::OperatingSystem::from_target_os() == egui::os::OperatingSystem::Android {
                "".to_string()
            } else {
                TorConfig::webtunnel_path()
            },
            TorBridge::DEFAULT_WEBTUNNEL_CONN_LINE.to_string()
        )
    }

    /// Webtunnel binary path.
    pub fn webtunnel_path() -> String {
        let os = egui::os::OperatingSystem::from_target_os();
        if os == egui::os::OperatingSystem::Android {
            let base = std::env::var("NATIVE_LIBS_DIR").unwrap_or_default();
            format!("{}/{}", base, Self::WEBTUNNEL_ANDROID_BIN)
        } else {
            let mut base = Self::base_path();
            base.push(Self::WEBTUNNEL_BIN);
            base.to_str().unwrap().to_string()
        }
    }

    /// Save Tor bridge.
    pub fn save_bridge(bridge: Option<TorBridge>) {
        let mut w_tor_config = Settings::tor_config_to_update();
        w_tor_config.bridge = bridge.clone();
        if bridge.is_some() {
            let bridge = bridge.unwrap();
            match &bridge {
                TorBridge::Webtunnel(_, _) => {
                    w_tor_config.webtunnel = bridge
                }
                TorBridge::Obfs4(_, _) => {
                    w_tor_config.obfs4 = bridge
                }
                TorBridge::Snowflake(_, _) => {
                    w_tor_config.snowflake = bridge
                }
           }
        }
        w_tor_config.save();
    }

    /// Get current Tor bridge if enabled.
    pub fn get_bridge() -> Option<TorBridge> {
        let r_config = Settings::tor_config_to_read();
        r_config.bridge.clone()
    }

    /// Get saved Webtunnel bridge.
    pub fn get_webtunnel() -> TorBridge {
        let r_config = Settings::tor_config_to_read();
        r_config.webtunnel.clone()
    }

    /// Get saved Obfs4 bridge.
    pub fn get_obfs4() -> TorBridge {
        let r_config = Settings::tor_config_to_read();
        r_config.obfs4.clone()
    }

    /// Get saved Snowflake bridge.
    pub fn get_snowflake() -> TorBridge {
        let r_config = Settings::tor_config_to_read();
        r_config.snowflake.clone()
    }

    /// Save proxy for Tor connections.
    pub fn save_proxy(proxy: Option<TorProxy>) {
        let mut w_config = Settings::tor_config_to_update();
        w_config.proxy = proxy.clone();
        if let Some(p) = proxy {
            match p {
                TorProxy::SOCKS5(_) => {
                    w_config.proxy_socks5 = p
                }
                TorProxy::HTTP(_) => {
                    w_config.proxy_http = p
                }
            }
        }
        w_config.save();
    }

    /// Get used proxy for Tor connections.
    pub fn get_proxy() -> Option<TorProxy> {
        let r_config = Settings::tor_config_to_read();
        r_config.proxy.clone()
    }

    /// Get saved SOCKS5 proxy.
    pub fn get_socks5_proxy() -> TorProxy {
        let r_config = Settings::tor_config_to_read();
        r_config.proxy_socks5.clone()
    }

    /// Get saved HTTP proxy.
    pub fn get_http_proxy() -> TorProxy {
        let r_config = Settings::tor_config_to_read();
        r_config.proxy_http.clone()
    }

    /// Check config version to migrate if needed.
    pub fn migrate(&mut self) {
        match self.ver {
            None => {
                // Migrate to 1st version.
                self.bridge = Some(TorConfig::default_webtunnel_bridge());
                self.ver = Some(1);
            }
            Some(_) => {}
        }
        self.save();
    }
}