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
use std::net::IpAddr;
use std::str::FromStr;

use grin_config::{config, ConfigError, ConfigMembers, GlobalConfig};
use grin_config::config::{API_SECRET_FILE_NAME, FOREIGN_API_SECRET_FILE_NAME, SERVER_CONFIG_FILE_NAME};
use grin_core::global::ChainTypes;
use grin_p2p::msg::PeerAddrs;
use grin_p2p::{PeerAddr, Seeding};
use grin_servers::common::types::ChainValidationMode;
use serde::{Deserialize, Serialize};

use crate::Settings;

/// Wrapped node config to be used by [`grin_servers::Server`].
#[derive(Serialize, Deserialize)]
pub struct NodeConfig {
    pub members: ConfigMembers
}

impl NodeConfig {
    /// Initialize integrated node config.
    pub fn init(chain_type: &ChainTypes) -> Self {
        let _ = Self::check_api_secret_files(chain_type, API_SECRET_FILE_NAME);
        let _ = Self::check_api_secret_files(chain_type, FOREIGN_API_SECRET_FILE_NAME);

        let config_members = Self::for_chain_type(chain_type);
        Self {
            members: config_members
        }
    }

    /// Initialize config with provided [`ChainTypes`].
    pub fn for_chain_type(chain_type: &ChainTypes) -> ConfigMembers {
        let path = Settings::get_config_path(SERVER_CONFIG_FILE_NAME, Some(chain_type));
        let parsed = Settings::read_from_file::<ConfigMembers>(path.clone());
        if !path.exists() || parsed.is_err() {
            let mut default_config = GlobalConfig::for_chain(chain_type);
            default_config.update_paths(&Settings::get_working_path(Some(chain_type)));
            let config = default_config.members.unwrap();
            Settings::write_to_file(&config, path);
            config
        } else {
            parsed.unwrap()
        }
    }

    /// Save node config to disk.
    pub fn save(&mut self) {
        let config_path = Settings::get_config_path(
            SERVER_CONFIG_FILE_NAME,
            Some(&self.members.server.chain_type)
        );
        Settings::write_to_file(&self.members, config_path);
    }

    /// Check that the api secret files exist and are valid.
    fn check_api_secret_files(
        chain_type: &ChainTypes,
        secret_file_name: &str,
    ) -> Result<(), ConfigError> {
        let grin_path = Settings::get_working_path(Some(chain_type));
        let mut api_secret_path = grin_path;
        api_secret_path.push(secret_file_name);
        if !api_secret_path.exists() {
            config::init_api_secret(&api_secret_path)
        } else {
            config::check_api_secret(&api_secret_path)
        }
    }

    /// Get stratum server IP address and port.
    pub fn get_stratum_address_port() -> (String, String) {
        let r_config = Settings::node_config_to_read();
        let saved_stratum_addr = r_config
            .members
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
    pub fn save_stratum_address_port(addr: &String, port: &String) {
        let addr_to_save = format!("{}:{}", addr, port);
        let mut w_node_config = Settings::node_config_to_update();
        w_node_config
            .members
            .server
            .stratum_mining_config
            .as_mut()
            .unwrap()
            .stratum_server_addr = Some(addr_to_save);
        w_node_config.save();
    }

    /// Check if stratum mining server autorun is enabled.
    pub fn is_stratum_autorun_enabled() -> bool {
        let stratum_config = Settings::node_config_to_read()
            .members
            .clone()
            .server
            .stratum_mining_config
            .unwrap();
        if let Some(enable) = stratum_config.enable_stratum_server {
            return enable;
        }
        false
    }

    /// Toggle stratum mining server autorun.
    pub fn toggle_stratum_autorun() {
        let autorun = Self::is_stratum_autorun_enabled();
        let mut w_node_config = Settings::node_config_to_update();
        w_node_config.members
            .server
            .stratum_mining_config
            .as_mut()
            .unwrap()
            .enable_stratum_server = Some(!autorun);
        w_node_config.save();
    }

    /// Disable stratum mining server autorun.
    pub fn disable_stratum_autorun() {
        let mut w_node_config = Settings::node_config_to_update();
        w_node_config.members
            .server
            .stratum_mining_config
            .as_mut()
            .unwrap()
            .enable_stratum_server = Some(false);
        w_node_config.save();
    }

    /// Get API server IP address and port.
    pub fn get_api_address_port() -> (String, String) {
        let r_config = Settings::node_config_to_read();
        let saved_api_addr = r_config
            .members
            .server
            .api_http_addr
            .as_str();
        let (addr, port) = saved_api_addr.split_once(":").unwrap();
        (addr.into(), port.into())
    }

    /// Save API server IP address and port.
    pub fn save_api_address_port(addr: &String, port: &String) {
        let addr_to_save = format!("{}:{}", addr, port);
        let mut w_node_config = Settings::node_config_to_update();
        w_node_config.members.server.api_http_addr = addr_to_save;
        w_node_config.save();
    }

    /// Get API secret text.
    pub fn get_api_secret() -> String {
        let r_config = Settings::node_config_to_read();
        let api_secret_path = r_config
            .members
            .server
            .api_secret_path
            .as_ref()
            .unwrap();
        let api_secret_file = File::open(api_secret_path).unwrap();
        let buf_reader = BufReader::new(api_secret_file);
        let mut lines_iter = buf_reader.lines();
        let first_line = lines_iter.next().unwrap();
        first_line.unwrap()
    }

    /// Save API secret text.
    pub fn save_api_secret(api_secret: &String) {
        if api_secret.is_empty() {
            return;
        }
        let r_config = Settings::node_config_to_read();
        let api_secret_path = r_config
            .members
            .server
            .api_secret_path
            .as_ref()
            .unwrap();
        let mut api_secret_file = File::create(api_secret_path).unwrap();
        api_secret_file.write_all(api_secret.as_bytes()).unwrap();
    }

    /// Get Foreign API secret text.
    pub fn get_foreign_api_secret() -> String {
        let r_config = Settings::node_config_to_read();
        let foreign_api_secret_path = r_config
            .members
            .server
            .foreign_api_secret_path
            .as_ref()
            .unwrap();
        let foreign_api_secret_file = File::open(foreign_api_secret_path).unwrap();
        let buf_reader = BufReader::new(foreign_api_secret_file);
        let mut lines_iter = buf_reader.lines();
        let first_line = lines_iter.next().unwrap();
        first_line.unwrap()
    }

    /// Save Foreign API secret text.
    pub fn save_foreign_api_secret(api_secret: &String) {
        if api_secret.is_empty() {
            return;
        }
        let r_config = Settings::node_config_to_read();
        let foreign_api_secret_path = r_config
            .members
            .server
            .foreign_api_secret_path
            .as_ref()
            .unwrap();
        let mut foreign_api_secret_file = File::create(foreign_api_secret_path).unwrap();
        foreign_api_secret_file.write_all(api_secret.as_bytes()).unwrap();
    }

    /// Get Future Time Limit.
    pub fn get_ftl() -> u64 {
        Settings::node_config_to_read().members.server.future_time_limit
    }

    /// Save Future Time Limit.
    pub fn save_ftl(ftl: u64) {
        let mut w_config = Settings::node_config_to_update();
        w_config.members.server.future_time_limit = ftl;
        w_config.save();
    }

    /// Check if full chain validation mode is enabled.
    pub fn is_full_chain_validation() -> bool {
        let mode = Settings::node_config_to_read().members.clone().server.chain_validation_mode;
        mode == ChainValidationMode::EveryBlock
    }

    /// Toggle full chain validation.
    pub fn toggle_chain_validation() {
        let mode = Settings::node_config_to_read().members.clone().server.chain_validation_mode;
        let new_mode = if mode == ChainValidationMode::Disabled {
            ChainValidationMode::Disabled
        } else {
            ChainValidationMode::EveryBlock
        };
        let mut w_node_config = Settings::node_config_to_update();
        w_node_config.members.server.chain_validation_mode = new_mode;
        w_node_config.save();
    }

    /// Check if node is running in archive mode.
    pub fn is_archive_mode() -> bool {
        let archive_mode = Settings::node_config_to_read().members.clone().server.archive_mode;
        archive_mode.is_some() && archive_mode.unwrap()
    }

    /// Toggle archive node mode.
    pub fn toggle_archive_mode() {
        let archive_mode = Self::is_archive_mode();
        let mut w_node_config = Settings::node_config_to_update();
        w_node_config.members.server.archive_mode = Some(!archive_mode);
        w_node_config.save();
    }

    // P2P settings

    /// Get P2P server port.
    pub fn get_p2p_port() -> u16 {
        Settings::node_config_to_read().members.server.p2p_config.port
    }

    /// Get P2P server port.
    pub fn save_p2p_port(port: u16) {
        let mut w_node_config = Settings::node_config_to_update();
        w_node_config.members.server.p2p_config.port = port;
        w_node_config.save();
    }

    /// Get peers seeding type.
    pub fn get_peers_seeding_type() -> Seeding {
        Settings::node_config_to_read().members.server.p2p_config.seeding_type
    }

    /// Get seeds for [`Seeding::List`] type.
    pub fn get_seeds() -> PeerAddrs {
        let r_config = Settings::node_config_to_read();
        r_config.members.server.p2p_config.seeds.clone().unwrap_or(PeerAddrs::default())
    }

    /// Save peers seeding type, with list of peers for [`Seeding::List`] type.
    pub fn save_peers_seeding_type(seeding_type: Seeding, peers: Option<PeerAddrs>) {
        let mut w_node_config = Settings::node_config_to_update();
        w_node_config.members.server.p2p_config.seeding_type = seeding_type;
        if seeding_type == Seeding::List {
            w_node_config.members.server.p2p_config.seeds = peers;
        }
        w_node_config.save();
    }

    /// Get denied peer list.
    pub fn get_denied_peers() -> PeerAddrs {
        let r_config = Settings::node_config_to_read();
        r_config.members.server.p2p_config.peers_deny.clone().unwrap_or(PeerAddrs::default())
    }

    /// Add peer at denied list.
    pub fn deny_peer(peer: String) {
        let ip_addr = IpAddr::from_str(peer.as_str()).unwrap();
        let peer_addr = PeerAddr::from_ip(ip_addr);

        let mut deny_peers = Self::get_denied_peers();
        deny_peers.peers.insert(0, peer_addr);

        let mut w_node_config = Settings::node_config_to_update();
        w_node_config.members.server.p2p_config.peers_deny = Some(deny_peers);
        w_node_config.save();
    }

    /// Save denied peer list.
    pub fn save_denied_peers(peers: Option<PeerAddrs>) {
        let mut w_node_config = Settings::node_config_to_update();
        w_node_config.members.server.p2p_config.peers_deny = peers;
        w_node_config.save();
    }

    /// Get allowed peer list.
    pub fn get_allowed_peers() -> PeerAddrs {
        let r_config = Settings::node_config_to_read();
        r_config.members.server.p2p_config.peers_allow.clone().unwrap_or(PeerAddrs::default())
    }

    /// Save allowed peer list.
    pub fn save_allowed_peers(peers: Option<PeerAddrs>) {
        let mut w_node_config = Settings::node_config_to_update();
        w_node_config.members.server.p2p_config.peers_allow = peers;
        w_node_config.save();
    }

    /// Get preferred peer list.
    pub fn get_preferred_peers() -> PeerAddrs {
        let r_config = Settings::node_config_to_read();
        r_config.members.server.p2p_config.peers_preferred.clone().unwrap_or(PeerAddrs::default())
    }

    /// Add peer at preferred list.
    pub fn prefer_peer(peer: String) {
        let ip_addr = IpAddr::from_str(peer.as_str()).unwrap();
        let peer_addr = PeerAddr::from_ip(ip_addr);

        let mut prefer_peers = Self::get_preferred_peers();
        prefer_peers.peers.insert(0, peer_addr);

        let mut w_node_config = Settings::node_config_to_update();
        w_node_config.members.server.p2p_config.peers_preferred = Some(prefer_peers);
        w_node_config.save();
    }

    /// Save preferred peer list.
    pub fn save_preferred_peers(peers: Option<PeerAddrs>) {
        let mut w_node_config = Settings::node_config_to_update();
        w_node_config.members.server.p2p_config.peers_preferred = peers;
        w_node_config.save();
    }

    /// How long a banned peer should stay banned in ms.
    pub fn get_ban_window() -> i64 {
        Settings::node_config_to_read().members.server.p2p_config.ban_window()
    }

    /// Set how long a banned peer should stay banned in ms.
    pub fn set_ban_window(time: i64) {
        let mut w_node_config = Settings::node_config_to_update();
        w_node_config.members.server.p2p_config.ban_window = Some(time);
        w_node_config.save();
    }

    /// Maximum number of inbound peer connections.
    pub fn get_max_inbound_count() -> u32 {
        Settings::node_config_to_read().members.server.p2p_config.peer_max_inbound_count()
    }

    /// Set maximum number of inbound peer connections.
    pub fn set_max_inbound_count(count: u32) {
        let mut w_node_config = Settings::node_config_to_update();
        w_node_config.members.server.p2p_config.peer_max_inbound_count = Some(count);
        w_node_config.save();
    }

    /// Maximum number of outbound peer connections.
    pub fn get_max_outbound_count() -> u32 {
        Settings::node_config_to_read().members.server.p2p_config.peer_max_outbound_count()
    }

    /// Set maximum number of outbound peer connections.
    pub fn set_max_outbound_count(count: u32) {
        let mut w_node_config = Settings::node_config_to_update();
        w_node_config.members.server.p2p_config.peer_max_outbound_count = Some(count);
        w_node_config.save();
    }

    /// Minimum number of outbound peer connections.
    pub fn get_min_outbound_count() -> u32 {
        Settings::node_config_to_read()
            .members
            .server
            .p2p_config
            .peer_min_preferred_outbound_count()
    }

    /// Set minimum number of outbound peer connections.
    pub fn set_min_outbound_count(count: u32) {
        let mut w_node_config = Settings::node_config_to_update();
        w_node_config.members.server.p2p_config.peer_min_preferred_outbound_count = Some(count);
        w_node_config.save();
    }

    // Pool settings

    /// Base fee that's accepted into the pool.
    pub fn get_base_fee() -> u64 {
        Settings::node_config_to_read().members.server.pool_config.accept_fee_base
    }

    /// Set base fee that's accepted into the pool.
    pub fn set_base_fee(fee: u64) {
        let mut w_node_config = Settings::node_config_to_update();
        w_node_config.members.server.pool_config.accept_fee_base = fee;
        w_node_config.save();
    }

    /// Reorg cache retention period in minute.
    pub fn get_reorg_cache_period() -> u32 {
        Settings::node_config_to_read().members.server.pool_config.reorg_cache_period
    }

    /// Set reorg cache retention period in minute.
    pub fn set_reorg_cache_period(period: u32) {
        let mut w_node_config = Settings::node_config_to_update();
        w_node_config.members.server.pool_config.reorg_cache_period = period;
        w_node_config.save();
    }

    /// Max amount of transactions at pool.
    pub fn get_max_pool_size() -> usize {
        Settings::node_config_to_read().members.server.pool_config.max_pool_size
    }

    /// Set max amount of transactions at pool.
    pub fn set_max_pool_size(amount: usize) {
        let mut w_node_config = Settings::node_config_to_update();
        w_node_config.members.server.pool_config.max_pool_size = amount;
        w_node_config.save();
    }

    /// Max amount of transactions at stem pool.
    pub fn get_max_stempool_size() -> usize {
        Settings::node_config_to_read().members.server.pool_config.max_stempool_size
    }

    /// Set max amount of transactions at stem pool.
    pub fn set_max_stempool_size(amount: usize) {
        let mut w_node_config = Settings::node_config_to_update();
        w_node_config.members.server.pool_config.max_stempool_size = amount;
        w_node_config.save();
    }

    /// Max total weight of transactions that can get selected to build a block.
    pub fn get_mineable_max_weight() -> u64 {
        Settings::node_config_to_read().members.server.pool_config.mineable_max_weight
    }

    /// Set max total weight of transactions that can get selected to build a block.
    pub fn set_mineable_max_weight(weight: u64) {
        let mut w_node_config = Settings::node_config_to_update();
        w_node_config.members.server.pool_config.mineable_max_weight = weight;
        w_node_config.save();
    }

    // Dandelion settings

    /// Dandelion epoch duration in secs.
    pub fn get_epoch() -> u16 {
        Settings::node_config_to_read().members.server.dandelion_config.epoch_secs
    }

    /// Set Dandelion epoch duration in secs.
    pub fn set_epoch(secs: u16) {
        let mut w_node_config = Settings::node_config_to_update();
        w_node_config.members.server.dandelion_config.epoch_secs = secs;
        w_node_config.save();
    }

    /// Dandelion embargo timer in secs.
    /// Fluff and broadcast after embargo expires if tx not seen on network.
    pub fn get_embargo() -> u16 {
        Settings::node_config_to_read().members.server.dandelion_config.embargo_secs
    }

    /// Set Dandelion embargo timer.
    pub fn set_embargo(secs: u16) {
        let mut w_node_config = Settings::node_config_to_update();
        w_node_config.members.server.dandelion_config.embargo_secs = secs;
        w_node_config.save();
    }

    /// Dandelion stem probability (default: stem 90% of the time, fluff 10% of the time).
    pub fn get_stem_probability() -> u8 {
        Settings::node_config_to_read().members.server.dandelion_config.stem_probability
    }

    /// Set Dandelion stem probability.
    pub fn set_stem_probability(percent: u8) {
        let mut w_node_config = Settings::node_config_to_update();
        w_node_config.members.server.dandelion_config.stem_probability = percent;
        w_node_config.save();
    }

    /// Default to always stem our txs as described in Dandelion++ paper.
    pub fn always_stem_our_txs() -> bool {
        Settings::node_config_to_read().members.server.dandelion_config.always_stem_our_txs
    }

    /// Toggle stem of our txs.
    pub fn toggle_always_stem_our_txs() {
        let stem_txs = Self::always_stem_our_txs();
        let mut w_node_config = Settings::node_config_to_update();
        w_node_config.members.server.dandelion_config.always_stem_our_txs = stem_txs;
        w_node_config.save();
    }
}