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

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, ToSocketAddrs};
use std::path::PathBuf;
use std::str::FromStr;
use local_ip_address::list_afinet_netifas;
use serde::{Deserialize, Serialize};

use grin_config::{config, ConfigError, ConfigMembers, GlobalConfig};
use grin_config::config::{API_SECRET_FILE_NAME, FOREIGN_API_SECRET_FILE_NAME, SERVER_CONFIG_FILE_NAME};
use grin_core::global::ChainTypes;
use grin_p2p::{PeerAddr, Seeding};
use grin_p2p::msg::PeerAddrs;
use grin_servers::common::types::ChainValidationMode;
use rand::Rng;

use crate::{AppConfig, Settings};
use crate::node::Node;

/// Peers config to save peers DNS names into the file.
#[derive(Serialize, Deserialize, Default)]
pub struct PeersConfig {
    seeds: Vec<String>,
    allowed: Vec<String>,
    denied: Vec<String>,
    preferred: Vec<String>
}

impl PeersConfig {
    /// File name for peers config.
    pub const FILE_NAME: &'static str = "peers.toml";

    /// Save peers config to the file.
    pub fn save(&self) {
        let chain_type = AppConfig::chain_type();
        let config_path = Settings::config_path(Self::FILE_NAME, Some(chain_type.shortname()));
        Settings::write_to_file(self, config_path);
    }

    /// Convert string to [`PeerAddr`] if address is in correct format (`host:port`) and available.
    pub fn peer_to_addr(peer: String) -> Option<PeerAddr> {
        match SocketAddr::from_str(peer.as_str()) {
            // Try to parse IP address first.
            Ok(ip) => Some(PeerAddr(ip)),
            // If that fails it's probably a DNS record.
            Err(_) => {
                if let Ok(mut socket_addr_list) = peer.to_socket_addrs() {
                    if let Some(addr) = socket_addr_list.next() {
                        return Some(PeerAddr(addr));
                    }
                }
                None
            }
        }
    }

    /// Load saved peers to node server [`ConfigMembers`] config.
    pub fn load_to_server_config() {
        let mut w_config = Settings::node_config_to_update();
        // Load seeds.
        for seed in w_config.peers.seeds.clone() {
            if let Some(p) = Self::peer_to_addr(seed.to_string()) {
                let mut seeds = w_config
                    .node
                    .server
                    .p2p_config
                    .seeds
                    .clone()
                    .unwrap_or(PeerAddrs::default());
                seeds.peers.insert(seeds.peers.len(), p);
                w_config.node.server.p2p_config.seeds = Some(seeds);
            }
        }
        // Load allowed peers.
        for peer in w_config.peers.allowed.clone() {
            if let Some(p) = Self::peer_to_addr(peer.clone()) {
                let mut allowed = w_config
                    .node
                    .server
                    .p2p_config
                    .peers_allow
                    .clone()
                    .unwrap_or(PeerAddrs::default());
                allowed.peers.insert(allowed.peers.len(), p);
                w_config.node.server.p2p_config.peers_allow = Some(allowed);
            }
        }
        // Load denied peers.
        for peer in w_config.peers.denied.clone() {
            if let Some(p) = Self::peer_to_addr(peer.clone()) {
                let mut denied = w_config
                    .node
                    .server
                    .p2p_config
                    .peers_deny
                    .clone()
                    .unwrap_or(PeerAddrs::default());
                denied.peers.insert(denied.peers.len(), p);
                w_config.node.server.p2p_config.peers_deny = Some(denied);
            }
        }
        // Load preferred peers.
        for peer in &w_config.peers.preferred.clone() {
            if let Some(p) = Self::peer_to_addr(peer.clone()) {
                let mut preferred = w_config
                    .node
                    .server
                    .p2p_config
                    .peers_preferred
                    .clone()
                    .unwrap_or(PeerAddrs::default());
                preferred.peers.insert(preferred.peers.len(), p);
                w_config.node.server.p2p_config.peers_preferred = Some(preferred);
            }
        }
    }
}

/// Wrapped node config to be used by [`grin_servers::Server`].
#[derive(Serialize, Deserialize)]
pub struct NodeConfig {
    pub(crate) node: ConfigMembers,
    pub(crate) peers: PeersConfig
}

impl NodeConfig {
    /// Initialize config fields from provided [`ChainTypes`].
    pub fn for_chain_type(chain_type: &ChainTypes) -> Self {
        // Check secret files for current chain type.
        let _ = Self::check_api_secret_files(chain_type, API_SECRET_FILE_NAME);
        let _ = Self::check_api_secret_files(chain_type, FOREIGN_API_SECRET_FILE_NAME);

        // Initialize peers config.
        let peers_config = {
            let sub_dir = Some(chain_type.shortname());
            let path = Settings::config_path(PeersConfig::FILE_NAME, sub_dir);
            let config = Settings::read_from_file::<PeersConfig>(path.clone());
            if !path.exists() || config.is_err() {
                Self::save_default_peers_config(chain_type)
            } else {
                config.unwrap()
            }
        };

        // Initialize node config.
        let node_config = {
            let sub_dir = Some(chain_type.shortname());
            let path = Settings::config_path(SERVER_CONFIG_FILE_NAME, sub_dir);
            let config = Settings::read_from_file::<ConfigMembers>(path.clone());
            if !path.exists() || config.is_err() {
                Self::save_default_node_server_config(chain_type)
            } else {
                config.unwrap()
            }
        };

        Self { node: node_config, peers: peers_config }
    }

    /// Save default node config for specified [`ChainTypes`].
    fn save_default_node_server_config(chain_type: &ChainTypes) -> ConfigMembers {
        let sub_dir = Some(chain_type.shortname());
        let path = Settings::config_path(SERVER_CONFIG_FILE_NAME, sub_dir.clone());

        let mut default_config = GlobalConfig::for_chain(chain_type);
        default_config.update_paths(&Settings::base_path(sub_dir));
        let mut config = default_config.members.unwrap();

        // Generate random p2p and api ports.
        Self::setup_default_ports(&mut config);

        // Clear wallet listener url (actually it will be wallet id).
        config.server.stratum_mining_config.clone().unwrap().wallet_listener_url = "".to_string();

        Settings::write_to_file(&config, path);
        config
    }

    /// Generate random p2p and api ports in ranges based on [`ChainTypes`].
    fn setup_default_ports(config: &mut ConfigMembers) {
        let (api, p2p) = match config.server.chain_type {
            ChainTypes::Mainnet => {
                let api = rand::thread_rng().gen_range(30000..33000);
                let p2p = rand::thread_rng().gen_range(33000..37000);
                (api, p2p)
            },
            _ => {
                let api = rand::thread_rng().gen_range(40000..43000);
                let p2p = rand::thread_rng().gen_range(43000..47000);
                (api, p2p)
            }
        };
        let api_addr = config.server.api_http_addr.split_once(":").unwrap().0;
        config.server.api_http_addr = format!("{}:{}", api_addr, api);
        config.server.p2p_config.port = p2p;
    }

    /// Save default peers config for specified [`ChainTypes`].
    fn save_default_peers_config(chain_type: &ChainTypes) -> PeersConfig {
        let sub_dir = Some(chain_type.shortname());
        let path = Settings::config_path(PeersConfig::FILE_NAME, sub_dir);
        let config = PeersConfig::default();
        Settings::write_to_file(&config, path);
        config
    }

    /// Save node config to the file.
    pub fn save(&self) {
        let sub_dir = Some(self.node.server.chain_type.shortname());
        let config_path = Settings::config_path(SERVER_CONFIG_FILE_NAME, sub_dir);
        Settings::write_to_file(&self.node, config_path);
    }

    /// Get server config to use for node server before start.
    pub fn node_server_config() -> ConfigMembers {
        let r_config = Settings::node_config_to_read();
        r_config.node.clone()
    }

    /// Reset node config to default values.
    pub fn reset_to_default() {
        let chain_type = {
            let r_config = Settings::node_config_to_read();
            r_config.node.server.chain_type
        };
        let node_server_config = Self::save_default_node_server_config(&chain_type);
        let peers_config = Self::save_default_peers_config(&chain_type);
        {
            let mut w_config = Settings::node_config_to_update();
            w_config.node = node_server_config;
            w_config.peers = peers_config;
        }
    }

    /// Check that the api secret files exist and are valid.
    fn check_api_secret_files(
        chain_type: &ChainTypes,
        secret_file_name: &str,
    ) -> Result<(), ConfigError> {
        let api_secret_path = Self::get_secret_path(chain_type, secret_file_name);
        if !api_secret_path.exists() {
            config::init_api_secret(&api_secret_path)
        } else {
            config::check_api_secret(&api_secret_path)
        }
    }

    /// Get path for secret file.
    fn get_secret_path(chain_type: &ChainTypes, secret_file_name: &str) -> PathBuf {
        let sub_dir = Some(chain_type.shortname());
        let grin_path = Settings::base_path(sub_dir);
        let mut api_secret_path = grin_path;
        api_secret_path.push(secret_file_name);
        api_secret_path
    }

    /// List of available IP addresses.
    pub fn get_ip_addrs() -> Vec<String> {
        let mut ip_addrs = Vec::new();
        let network_interfaces = list_afinet_netifas();
        if let Ok(network_interfaces) = network_interfaces {
            for (_, ip) in network_interfaces.iter() {
                if ip.is_ipv4() {
                    ip_addrs.push(ip.to_string());
                }
            }
        }
        ip_addrs
    }

    /// Check whether a port is available on the provided host.
    fn is_host_port_available(host: &String, port: &String) -> bool {
        if let Ok(p) = port.parse::<u16>() {
            let ip_addr = Ipv4Addr::from_str(host.as_str()).unwrap();
            let ipv4 = SocketAddrV4::new(ip_addr, p);
            return TcpListener::bind(ipv4).is_ok();
        }
        false
    }

    /// Check whether a port is available across the system at all hosts.
    fn is_port_available(port: &String) -> bool {
        if let Ok(p) = port.parse::<u16>() {
            for ip in Self::get_ip_addrs() {
                let ip_addr = Ipv4Addr::from_str(ip.as_str()).unwrap();
                let ipv4 = SocketAddrV4::new(ip_addr, p);
                if TcpListener::bind(ipv4).is_err() {
                    return false;
                }
            }
        } else {
            return false;
        }
        true
    }

    /// Get stratum server IP address and port.
    pub fn get_stratum_address() -> (String, String) {
        let r_config = Settings::node_config_to_read();
        let saved_stratum_addr = r_config
            .node
            .server
            .stratum_mining_config
            .as_ref()
            .unwrap()
            .stratum_server_addr
            .as_ref()
            .unwrap();
        let (addr, port) = saved_stratum_addr.split_once(":").unwrap();
        (addr.into(), port.into())
    }

    /// Save stratum server IP address and port.
    pub fn save_stratum_address(addr: &String, port: &String) {
        let addr_to_save = format!("{}:{}", addr, port);
        let mut w_config = Settings::node_config_to_update();
        w_config
            .node
            .server
            .stratum_mining_config
            .as_mut()
            .unwrap()
            .stratum_server_addr = Some(addr_to_save);
        w_config.save();
    }

    /// Check if stratum server port is available across the system and config.
    pub fn is_stratum_port_available(ip: &String, port: &String) -> bool {
        if Node::get_stratum_stats().is_running {
            // Check if Stratum server with same address is running.
            let (cur_ip, cur_port) = Self::get_stratum_address();
            let same_running = ip == &cur_ip && port == &cur_port;
            return same_running || Self::is_not_running_stratum_port_available(ip, port);
        }
        Self::is_not_running_stratum_port_available(&ip, &port)
    }

    /// Check if stratum port is available when server is not running.
    fn is_not_running_stratum_port_available(ip: &String, port: &String) -> bool {
        if Self::is_host_port_available(&ip, &port) {
            if &Self::get_p2p_port() != port {
                let (api_ip, api_port) = Self::get_api_ip_port();
                return if &api_ip == ip {
                    &api_port != port
                } else {
                    true
                };
            }
        }
        false
    }

    /// Get stratum mining server wallet address to get rewards.
    pub fn get_stratum_wallet_id() -> Option<i64> {
        let r_config = Settings::node_config_to_read();
        let id = r_config.node.clone().server.stratum_mining_config.unwrap().wallet_listener_url;
        return if id.is_empty() {
            None
        } else {
            if let Ok(id) = id.parse::<i64>() {
                Some(id)
            } else {
                None
            }
        }
    }

    /// Save stratum mining server wallet address to get rewards.
    pub fn save_stratum_wallet_id(id: i64) {
        let mut w_config = Settings::node_config_to_update();
        w_config.node
            .server
            .stratum_mining_config
            .as_mut()
            .unwrap()
            .wallet_listener_url = id.to_string();
        w_config.save();
        println!()
    }

    /// Get the amount of time in seconds to attempt to mine on a particular header.
    pub fn get_stratum_attempt_time() -> String {
        let r_config = Settings::node_config_to_read();
        r_config.node
            .server
            .stratum_mining_config
            .as_ref()
            .unwrap()
            .attempt_time_per_block
            .to_string()
    }

    /// Save stratum attempt time value in seconds.
    pub fn save_stratum_attempt_time(time: u32) {
        let mut w_config = Settings::node_config_to_update();
        w_config.node
            .server
            .stratum_mining_config
            .as_mut()
            .unwrap()
            .attempt_time_per_block = time;
        w_config.save();
    }

    /// Get minimum acceptable share difficulty to request from miners.
    pub fn get_stratum_min_share_diff() -> String {
        let r_config = Settings::node_config_to_read();
        r_config.node
            .server
            .stratum_mining_config
            .as_ref()
            .unwrap()
            .minimum_share_difficulty
            .to_string()
    }

    /// Save minimum acceptable share difficulty.
    pub fn save_stratum_min_share_diff(diff: u64) {
        let mut w_config = Settings::node_config_to_update();
        w_config.node
            .server
            .stratum_mining_config
            .as_mut()
            .unwrap()
            .minimum_share_difficulty = diff;
        w_config.save();
    }

    /// Check if stratum mining server autorun is enabled.
    pub fn is_stratum_autorun_enabled() -> bool {
        let r_config = Settings::node_config_to_read();
        let stratum_config = r_config
            .node
            .server
            .stratum_mining_config
            .as_ref()
            .unwrap();
        if let Some(enable) = stratum_config.enable_stratum_server {
            return enable;
        }
        false
    }

    /// Toggle stratum mining server autorun.
    pub fn toggle_stratum_autorun() {
        let autorun = Self::is_stratum_autorun_enabled();
        let mut w_config = Settings::node_config_to_update();
        w_config.node
            .server
            .stratum_mining_config
            .as_mut()
            .unwrap()
            .enable_stratum_server = Some(!autorun);
        w_config.save();
    }

    /// Get API server address.
    pub fn get_api_address() -> String {
        let r_config = Settings::node_config_to_read();
        r_config.node.server.api_http_addr.clone()
    }

    /// Get API server IP and port.
    pub fn get_api_ip_port() -> (String, String) {
        let saved_addr = Self::get_api_address();
        let (addr, port) = saved_addr.split_once(":").unwrap();
        (addr.into(), port.into())
    }

    /// Save API server IP address and port.
    pub fn save_api_address(addr: &String, port: &String) {
        let addr_to_save = format!("{}:{}", addr, port);
        let mut w_config = Settings::node_config_to_update();
        w_config.node.server.api_http_addr = addr_to_save;
        w_config.save();
    }

    /// Check if api server port is available across the system and config.
    pub fn is_api_port_available(ip: &String, port: &String) -> bool {
        if Node::is_running() {
            // Check if API server with same address is running.
            let same_running = NodeConfig::get_api_address() == format!("{}:{}", ip, port);
            if same_running || Self::is_host_port_available(ip, port) {
                return &Self::get_p2p_port() != port;
            }
            return false;
        } else if Self::is_host_port_available(ip, port) {
            return &Self::get_p2p_port() != port;
        }
        false
    }

    /// Get API secret text.
    pub fn get_api_secret(foreign: bool) -> Option<String> {
        let r_config = Settings::node_config_to_read();
        let api_secret_path = if foreign {
            &r_config
                .node
                .server
                .foreign_api_secret_path
        } else {
            &r_config
                .node
                .server
                .api_secret_path
        }.clone();
        if let Some(secret_path) = api_secret_path {
            if let Ok(file) = File::open(secret_path) {
                let buf_reader = BufReader::new(file);
                let mut lines_iter = buf_reader.lines();
                if let Some(Ok(line)) = lines_iter.next() {
                    return Some(line);
                }
            }
        }
        None
    }

    /// Save API secret text.
    pub fn save_api_secret(api_secret: &String) {
        Self::save_secret(api_secret, API_SECRET_FILE_NAME);
    }

    /// Update Foreign API secret.
    pub fn save_foreign_api_secret(api_secret: &String) {
        Self::save_secret(api_secret, FOREIGN_API_SECRET_FILE_NAME);
    }

    /// Save secret value into specified file.
    fn save_secret(value: &String, file_name: &str) {
        // Remove config value to remove authorization.
        if value.is_empty() {
            let mut w_config = Settings::node_config_to_update();
            match file_name {
                API_SECRET_FILE_NAME => w_config.node.server.api_secret_path = None,
                _ => w_config.node.server.foreign_api_secret_path = None
            }
            w_config.save();
            return;
        }

        let mut secret_enabled = true;
        // Get path for specified secret file.
        let secret_path = {
            let r_config = Settings::node_config_to_read();
            let path = match file_name {
                API_SECRET_FILE_NAME => r_config.node.server.api_secret_path.clone(),
                _ => r_config.node.server.foreign_api_secret_path.clone()
            };
            path.unwrap_or_else(|| {
                secret_enabled = false;
                let chain_type = AppConfig::chain_type();
                let path = Self::get_secret_path(&chain_type, file_name);
                path.to_str().unwrap().to_string()
            })
        };
        // Update secret path at config if authorization was disabled before.
        if !secret_enabled {
            let mut w_config = Settings::node_config_to_update();
            match file_name {
                API_SECRET_FILE_NAME => w_config
                    .node
                    .server
                    .api_secret_path = Some(secret_path.clone()),
                _ => w_config.node.server.foreign_api_secret_path = Some(secret_path.clone())
            };

            w_config.save();
        }
        // Write secret text into file.
        let mut secret_file = File::create(secret_path).unwrap();
        secret_file.write_all(value.as_bytes()).unwrap();
    }

    /// Get Future Time Limit.
    pub fn get_ftl() -> String {
        Settings::node_config_to_read().node.server.future_time_limit.to_string()
    }

    /// Save Future Time Limit.
    pub fn save_ftl(ftl: u64) {
        let mut w_config = Settings::node_config_to_update();
        w_config.node.server.future_time_limit = ftl;
        w_config.save();
    }

    /// Check if full chain validation mode is enabled.
    pub fn is_full_chain_validation() -> bool {
        let mode = Settings::node_config_to_read().node.clone().server.chain_validation_mode;
        mode == ChainValidationMode::EveryBlock
    }

    /// Toggle full chain validation.
    pub fn toggle_full_chain_validation() {
        let validation_enabled = Self::is_full_chain_validation();
        let new_mode = if validation_enabled {
            ChainValidationMode::Disabled
        } else {
            ChainValidationMode::EveryBlock
        };
        let mut w_config = Settings::node_config_to_update();
        w_config.node.server.chain_validation_mode = new_mode;
        w_config.save();
    }

    /// Check if node is running in archive mode.
    pub fn is_archive_mode() -> bool {
        let archive_mode = Settings::node_config_to_read().node.clone().server.archive_mode;
        archive_mode.is_some() && archive_mode.unwrap()
    }

    /// Toggle archive node mode.
    pub fn toggle_archive_mode() {
        let archive_mode = Self::is_archive_mode();
        let mut w_config = Settings::node_config_to_update();
        w_config.node.server.archive_mode = Some(!archive_mode);
        w_config.save();
    }

    /// Get P2P server port.
    pub fn get_p2p_port() -> String {
        Settings::node_config_to_read().node.server.p2p_config.port.to_string()
    }

    /// Check if P2P server port is available across the system and config.
    pub fn is_p2p_port_available(port: &String) -> bool {
        if port.parse::<u16>().is_err() {
            return false;
        }
        let (_, api_port) = Self::get_api_ip_port();
        if Node::is_running() {
            // Check if P2P server with same port is running.
            let same_running = &NodeConfig::get_p2p_port() == port;
            if same_running || Self::is_port_available(port) {
                return &api_port != port;
            }
            return false;
        } else if Self::is_port_available(port) {
            return &api_port != port;
        }
        false
    }

    /// Save P2P server port.
    pub fn save_p2p_port(port: u16) {
        let mut w_config = Settings::node_config_to_update();
        w_config.node.server.p2p_config.port = port;
        w_config.save();
    }

    /// Check if default seed list is used.
    pub fn is_default_seeding_type() -> bool {
        Settings::node_config_to_read().node.server.p2p_config.seeding_type == Seeding::DNSSeed
    }

    /// Toggle seeding type to use default or custom seed list.
    pub fn toggle_seeding_type() {
        let seeding_type = match Self::is_default_seeding_type() {
            true => Seeding::List,
            false => Seeding::DNSSeed
        };
        let mut w_config = Settings::node_config_to_update();
        w_config.node.server.p2p_config.seeding_type = seeding_type;
        w_config.save();
    }

    /// Get custom seed peers.
    pub fn get_custom_seeds() -> Vec<String> {
        Settings::node_config_to_read().peers.seeds.clone()
    }

    /// Save custom seed peer.
    pub fn save_custom_seed(peer: String) {
        let mut w_config = Settings::node_config_to_update();
        let size = w_config.peers.seeds.len();
        w_config.peers.seeds.insert(size, peer);
        w_config.peers.save();
    }

    /// Remove custom seed peer.
    pub fn remove_custom_seed(peer: &String) {
        let mut w_config = Settings::node_config_to_update();
        let mut seeds = w_config.peers.seeds.clone();
        if let Some(index) = seeds.iter().position(|x| x == peer) {
            seeds.remove(index);
        }
        w_config.peers.seeds = seeds;
        w_config.peers.save();
    }

    /// Get denied peer list.
    pub fn get_denied_peers() -> Vec<String> {
        Settings::node_config_to_read().peers.denied.clone()
    }

    /// Save peer to denied list.
    pub fn deny_peer(peer: String) {
        let mut w_config = Settings::node_config_to_update();
        let size = w_config.peers.denied.len();
        w_config.peers.denied.insert(size, peer);
        w_config.peers.save();
    }

    /// Remove denied peer.
    pub fn remove_denied_peer(peer: &String) {
        let mut w_config = Settings::node_config_to_update();
        let mut denied = w_config.peers.denied.clone();
        if let Some(index) = denied.iter().position(|x| x == peer) {
            denied.remove(index);
        }
        w_config.peers.denied = denied;
        w_config.peers.save();
    }

    /// Get allowed peer list.
    pub fn get_allowed_peers() -> Vec<String> {
        Settings::node_config_to_read().peers.allowed.clone()
    }

    /// Save peer to allowed list.
    pub fn allow_peer(peer: String) {
        let mut w_config = Settings::node_config_to_update();
        let size = w_config.peers.allowed.len();
        w_config.peers.allowed.insert(size, peer);
        w_config.peers.save();
    }

    /// Remove allowed peer.
    pub fn remove_allowed_peer(peer: &String) {
        let mut w_config = Settings::node_config_to_update();
        let mut allowed = w_config.peers.allowed.clone();
        if let Some(index) = allowed.iter().position(|x| x == peer) {
            allowed.remove(index);
        }
        w_config.peers.allowed = allowed;
        w_config.peers.save();
    }

    /// Get preferred peer list.
    pub fn get_preferred_peers() -> Vec<String> {
        Settings::node_config_to_read().peers.preferred.clone()
    }

    /// Add peer at preferred list.
    pub fn prefer_peer(peer: String) {
        let mut w_config = Settings::node_config_to_update();
        let size = w_config.peers.preferred.len();
        w_config.peers.preferred.insert(size, peer);
        w_config.peers.save();
    }

    /// Remove preferred peer.
    pub fn remove_preferred_peer(peer: &String) {
        let mut w_config = Settings::node_config_to_update();
        let mut preferred = w_config.peers.preferred.clone();
        if let Some(index) = preferred.iter().position(|x| x == peer) {
            preferred.remove(index);
        }
        w_config.peers.preferred = preferred;
        w_config.peers.save();
    }

    /// How long a banned peer should stay banned in ms.
    pub fn get_p2p_ban_window() -> String {
        Settings::node_config_to_read().node.server.p2p_config.ban_window().to_string()
    }

    /// Save for how long a banned peer should stay banned in ms.
    pub fn save_p2p_ban_window(time: i64) {
        let mut w_config = Settings::node_config_to_update();
        w_config.node.server.p2p_config.ban_window = Some(time);
        w_config.save();
    }

    /// Maximum number of inbound peer connections.
    pub fn get_max_inbound_peers() -> String {
        Settings::node_config_to_read()
            .node.server
            .p2p_config
            .peer_max_inbound_count()
            .to_string()
    }

    /// Save maximum number of inbound peer connections.
    pub fn save_max_inbound_peers(count: u32) {
        let mut w_config = Settings::node_config_to_update();
        w_config.node.server.p2p_config.peer_max_inbound_count = Some(count);
        w_config.save();
    }

    /// Maximum number of outbound peer connections.
    pub fn get_max_outbound_peers() -> String {
        Settings::node_config_to_read()
            .node
            .server
            .p2p_config
            .peer_max_outbound_count()
            .to_string()
    }

    /// Save maximum number of outbound peer connections.
    pub fn save_max_outbound_peers(count: u32) {
        let mut w_config = Settings::node_config_to_update();
        w_config.node.server.p2p_config.peer_max_outbound_count = Some(count);
        // Same value for preferred.
        w_config.node.server.p2p_config.peer_min_preferred_outbound_count = Some(count);
        w_config.save();
    }

    /// Base fee that's accepted into the pool.
    pub fn get_base_fee() -> String {
        Settings::node_config_to_read().node.server.pool_config.accept_fee_base.to_string()
    }

    /// Save base fee that's accepted into the pool.
    pub fn save_base_fee(fee: u64) {
        let mut w_config = Settings::node_config_to_update();
        w_config.node.server.pool_config.accept_fee_base = fee;
        w_config.save();
    }

    /// Reorg cache retention period in minutes.
    pub fn get_reorg_cache_period() -> String {
        Settings::node_config_to_read().node.server.pool_config.reorg_cache_period.to_string()
    }

    /// Save reorg cache retention period in minutes.
    pub fn save_reorg_cache_period(period: u32) {
        let mut w_config = Settings::node_config_to_update();
        w_config.node.server.pool_config.reorg_cache_period = period;
        w_config.save();
    }

    /// Max amount of transactions at pool.
    pub fn get_max_pool_size() -> String {
        Settings::node_config_to_read().node.server.pool_config.max_pool_size.to_string()
    }

    /// Save max amount of transactions at pool.
    pub fn save_max_pool_size(amount: usize) {
        let mut w_config = Settings::node_config_to_update();
        w_config.node.server.pool_config.max_pool_size = amount;
        w_config.save();
    }

    /// Max amount of transactions at stem pool.
    pub fn get_max_stempool_size() -> String {
        Settings::node_config_to_read().node.server.pool_config.max_stempool_size.to_string()
    }

    /// Save max amount of transactions at stem pool.
    pub fn save_max_stempool_size(amount: usize) {
        let mut w_config = Settings::node_config_to_update();
        w_config.node.server.pool_config.max_stempool_size = amount;
        w_config.save();
    }

    /// Max total weight of transactions that can get selected to build a block.
    pub fn get_mineable_max_weight() -> String {
        Settings::node_config_to_read().node.server.pool_config.mineable_max_weight.to_string()
    }

    /// Set max total weight of transactions that can get selected to build a block.
    pub fn save_mineable_max_weight(weight: u64) {
        let mut w_config = Settings::node_config_to_update();
        w_config.node.server.pool_config.mineable_max_weight = weight;
        w_config.save();
    }

    // Dandelion settings

    /// Dandelion epoch duration in seconds.
    pub fn get_dandelion_epoch() -> String {
        Settings::node_config_to_read().node.server.dandelion_config.epoch_secs.to_string()
    }

    /// Save Dandelion epoch duration in seconds.
    pub fn save_dandelion_epoch(secs: u16) {
        let mut w_config = Settings::node_config_to_update();
        w_config.node.server.dandelion_config.epoch_secs = secs;
        w_config.save();
    }

    /// Dandelion embargo timer in seconds.
    /// Fluff and broadcast after embargo expires if tx not seen on network.
    pub fn get_dandelion_embargo() -> String {
        Settings::node_config_to_read().node.server.dandelion_config.embargo_secs.to_string()
    }

    /// Save Dandelion embargo timer in seconds.
    pub fn save_dandelion_embargo(secs: u16) {
        let mut w_config = Settings::node_config_to_update();
        w_config.node.server.dandelion_config.embargo_secs = secs;
        w_config.save();
    }

    /// Dandelion aggregation period in seconds.
    pub fn get_dandelion_aggregation() -> String {
        Settings::node_config_to_read().node.server.dandelion_config.aggregation_secs.to_string()
    }

    /// Save Dandelion aggregation period in seconds.
    pub fn save_dandelion_aggregation(secs: u16) {
        let mut w_config = Settings::node_config_to_update();
        w_config.node.server.dandelion_config.aggregation_secs = secs;
        w_config.save();
    }

    /// Dandelion stem probability (default: stem 90% of the time, fluff 10% of the time).
    pub fn get_stem_probability() -> String {
        Settings::node_config_to_read().node.server.dandelion_config.stem_probability.to_string()
    }

    /// Save Dandelion stem probability.
    pub fn save_stem_probability(percent: u8) {
        let mut w_config = Settings::node_config_to_update();
        w_config.node.server.dandelion_config.stem_probability = percent;
        w_config.save();
    }

    /// Default to always stem our txs as described in Dandelion++ paper.
    pub fn always_stem_our_txs() -> bool {
        Settings::node_config_to_read().node.server.dandelion_config.always_stem_our_txs
    }

    /// Toggle stem of our txs.
    pub fn toggle_always_stem_our_txs() {
        let stem_txs = Self::always_stem_our_txs();
        let mut w_config = Settings::node_config_to_update();
        w_config.node.server.dandelion_config.always_stem_our_txs = !stem_txs;
        w_config.save();
    }
}