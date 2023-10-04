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
use serde_derive::{Deserialize, Serialize};
use crate::node::NodeConfig;
use crate::Settings;
use crate::wallet::ConnectionsConfig;

/// Application configuration.
#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    /// Run node server on startup.
    pub auto_start_node: bool,
    /// Chain type for node and wallets.
    pub(crate) chain_type: ChainTypes,

    /// Flag to show wallet list at dual panel wallets mode.
    show_wallets_at_dual_panel: bool,
    /// Flag to show all connections at network panel or integrated node info.
    show_connections_network_panel: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_start_node: false,
            chain_type: ChainTypes::default(),
            show_wallets_at_dual_panel: false,
            show_connections_network_panel: false,
        }
    }
}

impl AppConfig {
    /// Application configuration file name.
    pub const FILE_NAME: &'static str = "app.toml";

    /// Save application configuration to the file.
    pub fn save(&self) {
        Settings::write_to_file(self, Settings::get_config_path(Self::FILE_NAME, None));
    }

    /// Change global [`ChainTypes`] and load new [`NodeConfig`].
    pub fn change_chain_type(chain_type: &ChainTypes) {
        let current_chain_type = Self::chain_type();
        if current_chain_type != *chain_type {
            // Save chain type at app config.
            {
                let mut w_app_config = Settings::app_config_to_update();
                w_app_config.chain_type = *chain_type;
                w_app_config.save();
            }
            // Load node configuration for selected chain type.
            {
                let mut w_node_config = Settings::node_config_to_update();
                let node_config = NodeConfig::for_chain_type(chain_type);
                w_node_config.node = node_config.node;
                w_node_config.peers = node_config.peers;
            }
            // Load connections configuration
            {
                let mut w_node_config = Settings::conn_config_to_update();
                *w_node_config = ConnectionsConfig::for_chain_type(chain_type);
            }
        }
    }

    /// Get current [`ChainTypes`] for node and wallets.
    pub fn chain_type() -> ChainTypes {
        let r_config = Settings::app_config_to_read();
        r_config.chain_type
    }

    /// Check if integrated node is starting with application.
    pub fn autostart_node() -> bool {
        let r_config = Settings::app_config_to_read();
        r_config.auto_start_node
    }

    /// Toggle integrated node autostart.
    pub fn toggle_node_autostart() {
        let autostart = Self::autostart_node();
        let mut w_app_config = Settings::app_config_to_update();
        w_app_config.auto_start_node = !autostart;
        w_app_config.save();
    }

    /// Check if it's needed to show wallet list at dual panel wallets mode.
    pub fn show_wallets_at_dual_panel() -> bool {
        let r_config = Settings::app_config_to_read();
        r_config.show_wallets_at_dual_panel
    }

    /// Toggle flag to show wallet list at dual panel wallets mode.
    pub fn toggle_show_wallets_at_dual_panel() {
        let show = Self::show_wallets_at_dual_panel();
        let mut w_app_config = Settings::app_config_to_update();
        w_app_config.show_wallets_at_dual_panel = !show;
        w_app_config.save();
    }

    /// Check if it's needed to show all connections or integrated node info at network panel.
    pub fn show_connections_network_panel() -> bool {
        let r_config = Settings::app_config_to_read();
        r_config.show_connections_network_panel
    }

    /// Toggle flag to show all connections or integrated node info at network panel.
    pub fn toggle_show_connections_network_panel() {
        let show = Self::show_connections_network_panel();
        let mut w_app_config = Settings::app_config_to_update();
        w_app_config.show_connections_network_panel = !show;
        w_app_config.save();
    }
}