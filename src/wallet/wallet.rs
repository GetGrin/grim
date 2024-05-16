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

use std::{fs, thread};
use std::fs::File;
use std::io::Write;
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::sync::{Arc, mpsc};
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU8, Ordering};
use std::thread::Thread;
use std::time::Duration;
use futures::channel::oneshot;
use serde_json::{json, Value};
use tor_rtcompat::tokio::TokioNativeTlsRuntime;
use rand::Rng;

use grin_api::{ApiServer, Router};
use grin_chain::SyncStatus;
use grin_core::global;
use grin_core::global::ChainTypes;
use grin_keychain::{ExtKeychain, Identifier, Keychain};
use grin_util::{Mutex, ToHex};
use grin_util::secp::SecretKey;
use grin_util::types::ZeroingString;
use grin_wallet_api::Owner;
use grin_wallet_controller::controller;
use grin_wallet_controller::controller::ForeignAPIHandlerV2;
use grin_wallet_impls::{DefaultLCProvider, DefaultWalletImpl, HTTPNodeClient};
use grin_wallet_libwallet::{address, Error, InitTxArgs, IssueInvoiceTxArgs, NodeClient, RetrieveTxQueryArgs, RetrieveTxQuerySortField, RetrieveTxQuerySortOrder, Slate, SlatepackAddress, SlateState, SlateVersion, StatusMessage, TxLogEntry, TxLogEntryType, VersionedSlate, WalletInst, WalletLCProvider};
use grin_wallet_libwallet::api_impl::owner::{cancel_tx, retrieve_summary_info, retrieve_txs};
use grin_wallet_util::OnionV3Address;

use crate::AppConfig;
use crate::node::{Node, NodeConfig};
use crate::tor::Tor;
use crate::wallet::{ConnectionsConfig, ExternalConnection, WalletConfig};
use crate::wallet::types::{ConnectionMethod, WalletAccount, WalletData, WalletInstance, WalletTransaction};

/// Contains wallet instance, configuration and state, handles wallet commands.
#[derive(Clone)]
pub struct Wallet {
    /// Wallet configuration.
    config: Arc<RwLock<WalletConfig>>,
    /// Wallet instance, initializing on wallet opening and clearing on wallet closing.
    instance: Option<WalletInstance>,
    /// [`WalletInstance`] external connection id applied after opening.
    instance_ext_conn_id: Arc<AtomicI64>,

    /// Wallet Slatepack address to receive txs at transport.
    slatepack_address: Arc<RwLock<Option<String>>>,

    /// Wallet sync thread.
    sync_thread: Arc<RwLock<Option<Thread>>>,

    /// Running wallet foreign API server and port.
    foreign_api_server: Arc<RwLock<Option<(ApiServer, u16)>>>,

    /// Flag to check if wallet reopening is needed.
    reopen: Arc<AtomicBool>,
    /// Flag to check if wallet is open.
    is_open: Arc<AtomicBool>,
    /// Flag to check if wallet is closing.
    closing: Arc<AtomicBool>,

    /// Flag to check if wallet was deleted to remove it from the list.
    deleted: Arc<AtomicBool>,

    /// Error on wallet loading.
    sync_error: Arc<AtomicBool>,
    /// Info loading progress in percents.
    info_sync_progress: Arc<AtomicU8>,
    /// Transactions loading progress in percents.
    txs_sync_progress: Arc<AtomicU8>,

    /// Wallet accounts.
    accounts: Arc<RwLock<Vec<WalletAccount>>>,

    /// Wallet info to show at ui.
    data: Arc<RwLock<Option<WalletData>>>,
    /// Attempts amount to update wallet data.
    sync_attempts: Arc<AtomicU8>,

    /// Flag to check if wallet repairing and restoring missing outputs is needed.
    repair_needed: Arc<AtomicBool>,
    /// Wallet repair progress in percents.
    repair_progress: Arc<AtomicU8>
}

impl Wallet {
    /// Create new [`Wallet`] instance with provided [`WalletConfig`].
    fn new(config: WalletConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            instance: None,
            instance_ext_conn_id: Arc::new(AtomicI64::new(0)),
            slatepack_address: Arc::new(RwLock::new(None)),
            sync_thread: Arc::from(RwLock::new(None)),
            foreign_api_server: Arc::new(RwLock::new(None)),
            reopen: Arc::new(AtomicBool::new(false)),
            is_open: Arc::from(AtomicBool::new(false)),
            closing: Arc::new(AtomicBool::new(false)),
            deleted: Arc::new(AtomicBool::new(false)),
            sync_error: Arc::from(AtomicBool::new(false)),
            info_sync_progress: Arc::from(AtomicU8::new(0)),
            txs_sync_progress: Arc::from(AtomicU8::new(0)),
            accounts: Arc::new(RwLock::new(vec![])),
            data: Arc::new(RwLock::new(None)),
            sync_attempts: Arc::new(AtomicU8::new(0)),
            repair_needed: Arc::new(AtomicBool::new(false)),
            repair_progress: Arc::new(AtomicU8::new(0))
        }
    }

    /// Create new wallet.
    pub fn create(
        name: String,
        password: String,
        mnemonic: String,
        conn_method: &ConnectionMethod
    ) -> Result<Wallet, Error> {
        let config = WalletConfig::create(name, conn_method);
        let w = Wallet::new(config.clone());
        {
            let instance = Self::create_wallet_instance(config)?;
            let mut w_lock = instance.lock();
            let p = w_lock.lc_provider()?;
            p.create_wallet(None,
                            Some(ZeroingString::from(mnemonic.clone())),
                            mnemonic.len(),
                            ZeroingString::from(password),
                            false,
            )?;
        }
        Ok(w)
    }

    /// Initialize [`Wallet`] from provided data path.
    pub fn init(data_path: PathBuf) -> Option<Wallet> {
        let wallet_config = WalletConfig::load(data_path.clone());
        if let Some(config) = wallet_config {
            return Some(Wallet::new(config));
        }
        None
    }

    /// Create [`WalletInstance`] from provided [`WalletConfig`].
    fn create_wallet_instance(config: WalletConfig) -> Result<WalletInstance, Error> {
        // Assume global chain type has already been initialized.
        let chain_type = config.chain_type;
        if !global::GLOBAL_CHAIN_TYPE.is_init() {
            global::init_global_chain_type(chain_type);
        } else {
            global::set_global_chain_type(chain_type);
            global::set_local_chain_type(chain_type);
        }

        // Setup node client.
        let (node_api_url, node_secret) = if let Some(id) = config.ext_conn_id {
            if let Some(conn) = ConnectionsConfig::ext_conn(id) {
                (conn.url, conn.secret)
            } else {
                (ExternalConnection::DEFAULT_MAIN_URL.to_string(), None)
            }
        } else {
            let api_url = format!("http://{}", NodeConfig::get_api_address());
            let api_secret = NodeConfig::get_foreign_api_secret();
            (api_url, api_secret)
        };
        let node_client = HTTPNodeClient::new(&node_api_url, node_secret)?;

        // Create wallet instance.
        let wallet = Self::inst_wallet::<
            DefaultLCProvider<HTTPNodeClient, ExtKeychain>,
            HTTPNodeClient,
            ExtKeychain,
        >(config, node_client)?;
        Ok(wallet)
    }

    /// Instantiate [`WalletInstance`] from provided node client and [`WalletConfig`].
    fn inst_wallet<L, C, K>(
        config: WalletConfig,
        node_client: C,
    ) -> Result<Arc<Mutex<Box<dyn WalletInst<'static, L, C, K>>>>, Error>
        where
            DefaultWalletImpl<'static, C>: WalletInst<'static, L, C, K>,
            L: WalletLCProvider<'static, C, K>,
            C: NodeClient + 'static,
            K: Keychain + 'static,
    {
        let mut wallet = Box::new(DefaultWalletImpl::<'static, C>::new(node_client).unwrap())
            as Box<dyn WalletInst<'static, L, C, K>>;
        let lc = wallet.lc_provider()?;
        lc.set_top_level_directory(config.get_data_path().as_str())?;
        Ok(Arc::new(Mutex::new(wallet)))
    }

    /// Get parent key identifier for current account.
    pub fn get_parent_key_id(&self) -> Result<Identifier, Error> {
        let instance = self.instance.clone().unwrap();
        let mut w_lock = instance.lock();
        let lc = w_lock.lc_provider()?;
        let w_inst = lc.wallet_inst()?;
        Ok(w_inst.parent_key_id())
    }

    /// Get wallet [`SecretKey`] for transports.
    pub fn secret_key(&self) -> Result<SecretKey, Error> {
        let instance = self.instance.clone().unwrap();
        let mut w_lock = instance.lock();
        let lc = w_lock.lc_provider()?;
        let w_inst = lc.wallet_inst()?;
        let k = w_inst.keychain((&None).as_ref())?;
        let parent_key_id = w_inst.parent_key_id();
        let sec_key = address::address_from_derivation_path(&k, &parent_key_id, 0)
            .map_err(|e| Error::TorConfig(format!("{:?}", e)))?;
        Ok(sec_key)
    }

    /// Get unique opened wallet identifier, including current account.
    pub fn identifier(&self) -> String {
        let config = self.get_config();
        format!("wallet_{}_{}", config.id, config.account.to_hex())
    }

    /// Get Slatepack address to receive txs at transport.
    pub fn slatepack_address(&self) -> Option<String> {
        let r_address = self.slatepack_address.read();
        if r_address.is_some() {
            let addr = r_address.clone();
            return addr
        }
        None
    }

    /// Get wallet config.
    pub fn get_config(&self) -> WalletConfig {
        self.config.read().clone()
    }

    /// Change wallet name.
    pub fn change_name(&self, name: String) {
        let mut w_config = self.config.write();
        w_config.name = name;
        w_config.save();
    }

    /// Check if start of Tor listener on wallet opening is needed.
    pub fn auto_start_tor_listener(&self) -> bool {
        let r_config = self.config.read();
        r_config.enable_tor_listener.unwrap_or(true)
    }

    /// Update start of Tor listener on wallet opening.
    pub fn update_auto_start_tor_listener(&self, start: bool) {
        let mut w_config = self.config.write();
        w_config.enable_tor_listener = Some(start);
        w_config.save();
    }

    /// Check if Dandelion usage is needed to post transactions.
    pub fn can_use_dandelion(&self) -> bool {
        let r_config = self.config.read();
        r_config.use_dandelion.unwrap_or(true)
    }

    /// Update usage of Dandelion to post transactions.
    pub fn update_use_dandelion(&self, use_dandelion: bool) {
        let mut w_config = self.config.write();
        w_config.use_dandelion = Some(use_dandelion);
        w_config.save();
    }

    /// Update minimal amount of confirmations.
    pub fn update_min_confirmations(&self, min_confirmations: u64) {
        let mut w_config = self.config.write();
        w_config.min_confirmations = min_confirmations;
        w_config.save();
    }

    /// Update external connection identifier.
    pub fn update_ext_conn_id(&self, id: Option<i64>) {
        let mut w_config = self.config.write();
        w_config.ext_conn_id = id;
        w_config.save();
    }

    /// Open the wallet and start [`WalletData`] sync at separate thread.
    pub fn open(&mut self, password: String) -> Result<(), Error> {
        if self.is_open() {
            return Err(Error::GenericError("Already opened".to_string()));
        }

        // Create new wallet instance if sync thread was stopped or instance was not created.
        if self.sync_thread.read().is_none() || self.instance.is_none() {
            let config = self.get_config();
            let new_instance = Self::create_wallet_instance(config.clone())?;
            self.instance = Some(new_instance);
            self.instance_ext_conn_id.store(match config.ext_conn_id {
                None => 0,
                Some(conn_id) => conn_id
            }, Ordering::Relaxed);
        }

        // Open the wallet.
        {
            let instance = self.instance.clone().unwrap();
            let mut wallet_lock = instance.lock();
            let lc = wallet_lock.lc_provider()?;
            match lc.open_wallet(None, ZeroingString::from(password), false, false) {
                Ok(_) => {
                    // Reset an error on opening.
                    self.set_sync_error(false);
                    self.reset_sync_attempts();

                    // Set current account.
                    let wallet_inst = lc.wallet_inst()?;
                    let label = self.get_config().account.to_owned();
                    wallet_inst.set_parent_key_id_by_name(label.as_str())?;

                    // Start new synchronization thread or wake up existing one.
                    let mut thread_w = self.sync_thread.write();
                    if thread_w.is_none() {
                        let thread = start_sync(self.clone());
                        *thread_w = Some(thread);
                    } else {
                        thread_w.clone().unwrap().unpark();
                    }
                    self.is_open.store(true, Ordering::Relaxed);
                }
                Err(e) => {
                    self.instance = None;
                    return Err(e)
                }
            }
        }

        // Set slatepack address.
        let mut api = Owner::new(self.instance.clone().unwrap(), None);
        controller::owner_single_use(None, None, Some(&mut api), |api, m| {
            let mut w_address = self.slatepack_address.write();
            *w_address = Some(api.get_slatepack_address(m, 0)?.to_string());
            Ok(())
        })?;

        Ok(())
    }

    /// Get external connection id applied to [`WalletInstance`]
    /// after wallet opening if sync is running or get it from config.
    pub fn get_current_ext_conn_id(&self) -> Option<i64> {
        if self.sync_thread.read().is_some() {
            let ext_conn_id = self.instance_ext_conn_id.load(Ordering::Relaxed);
            if ext_conn_id == 0 {
                None
            } else {
                Some(ext_conn_id)
            }
        } else {
            self.get_config().ext_conn_id
        }
    }

    /// Check if wallet is open.
    pub fn is_open(&self) -> bool {
        self.is_open.load(Ordering::Relaxed)
    }

    /// Check if wallet is closing.
    pub fn is_closing(&self) -> bool {
        self.closing.load(Ordering::Relaxed)
    }

    /// Close the wallet.
    pub fn close(&self) {
        if !self.is_open() || self.instance.is_none() {
            return;
        }
        self.closing.store(true, Ordering::Relaxed);

        // Close wallet at separate thread.
        let wallet_close = self.clone();
        let instance = wallet_close.instance.clone().unwrap();
        let service_id = wallet_close.identifier();
        thread::spawn(move || {
            // Stop running API server.
            let api_server_exists = {
                wallet_close.foreign_api_server.read().is_some()
            };
            if api_server_exists {
                let mut w_api_server = wallet_close.foreign_api_server.write();
                w_api_server.as_mut().unwrap().0.stop();
                *w_api_server = None;
            }

            // Stop running Tor service.
            Tor::stop_service(&service_id);

            // Close the wallet.
            Self::close_wallet(&instance);

            // Mark wallet as not opened.
            wallet_close.closing.store(false, Ordering::Relaxed);
            wallet_close.is_open.store(false, Ordering::Relaxed);

            // Wake up thread to exit.
            wallet_close.sync();
        });
    }

    /// Close wallet for provided [`WalletInstance`].
    fn close_wallet(instance: &WalletInstance) {
        let mut wallet_lock = instance.lock();
        let lc = wallet_lock.lc_provider().unwrap();
        let _ = lc.close_wallet(None);
    }

    /// Create account into wallet.
    pub fn create_account(&self, label: &String) -> Result<(), Error> {
        let mut api = Owner::new(self.instance.clone().unwrap(), None);
        controller::owner_single_use(None, None, Some(&mut api), |api, m| {
            api.create_account_path(m, label)?;

            // Update account list at separate thread.
            if let Some(data) = self.get_data() {
                let last_height = data.info.last_confirmed_height;
                let wallet = self.clone();
                thread::spawn(move || {
                    update_accounts(&wallet, last_height, None);
                });
            }

            // Sync wallet data.
            self.sync();
            Ok(())
        })
    }

    /// Set active account from provided label.
    pub fn set_active_account(&mut self, label: &String) -> Result<(), Error> {
        let mut api = Owner::new(self.instance.clone().unwrap(), None);
        controller::owner_single_use(None, None, Some(&mut api), |api, m| {
            api.set_active_account(m, label)?;
            // Set Slatepack address.
            let mut w_address = self.slatepack_address.write();
            *w_address = Some(api.get_slatepack_address(m, 0)?.to_string());
            Ok(())
        })?;

        // Stop service from previous account.
        let cur_service_id = self.identifier();
        Tor::stop_service(&cur_service_id);

        // Save account label into config.
        let mut w_config = self.config.write();
        w_config.account = label.to_owned();
        w_config.save();

        // Clear wallet info.
        let mut w_data = self.data.write();
        *w_data = None;

        // Reset progress values.
        self.info_sync_progress.store(0, Ordering::Relaxed);
        self.txs_sync_progress.store(0, Ordering::Relaxed);

        // Sync wallet data.
        self.sync();
        Ok(())
    }

    /// Get list of accounts for the wallet.
    pub fn accounts(&self) -> Vec<WalletAccount> {
        self.accounts.read().clone()
    }

    /// Set wallet reopen status.
    pub fn set_reopen(&self, reopen: bool) {
        self.reopen.store(reopen, Ordering::Relaxed);
    }

    /// Check if wallet reopen is needed.
    pub fn reopen_needed(&self) -> bool {
        self.reopen.load(Ordering::Relaxed)
    }

    /// Get wallet transactions synchronization progress.
    pub fn txs_sync_progress(&self) -> u8 {
        self.txs_sync_progress.load(Ordering::Relaxed)
    }

    /// Get wallet info synchronization progress.
    pub fn info_sync_progress(&self) -> u8 {
        self.info_sync_progress.load(Ordering::Relaxed)
    }

    /// Check if wallet had an error on synchronization.
    pub fn sync_error(&self) -> bool {
        self.sync_error.load(Ordering::Relaxed)
    }

    /// Set an error for wallet on synchronization.
    pub fn set_sync_error(&self, error: bool) {
        self.sync_error.store(error, Ordering::Relaxed);
    }

    /// Get current wallet synchronization attempts before setting an error.
    fn get_sync_attempts(&self) -> u8 {
        self.sync_attempts.load(Ordering::Relaxed)
    }

    /// Increment wallet synchronization attempts before setting an error.
    fn increment_sync_attempts(&self) {
        let mut attempts = self.get_sync_attempts();
        attempts += 1;
        self.sync_attempts.store(attempts, Ordering::Relaxed);
    }

    /// Reset wallet synchronization attempts.
    fn reset_sync_attempts(&self) {
        self.sync_attempts.store(0, Ordering::Relaxed);
    }

    /// Get wallet data.
    pub fn get_data(&self) -> Option<WalletData> {
        let r_data = self.data.read();
        r_data.clone()
    }

    /// Wake up wallet thread to sync wallet data and update statuses.
    pub fn sync(&self) {
        let thread_r = self.sync_thread.read();
        if let Some(thread) = thread_r.as_ref() {
            thread.unpark();
        }
    }

    /// Get running Foreign API server port.
    pub fn foreign_api_port(&self) -> Option<u16> {
        let r_api = self.foreign_api_server.read();
        if r_api.is_some() {
            let api = r_api.as_ref().unwrap();
            return Some(api.1);
        }
        None
    }

    /// Parse Slatepack message into [`Slate`].
    pub fn parse_slatepack(&self, message: &String) -> Result<Slate, Error> {
        let api = Owner::new(self.instance.clone().unwrap(), None);
        api.slate_from_slatepack_message(None, message.clone(), vec![])
    }

    /// Create Slatepack message from provided slate.
    fn create_slatepack_message(&self, slate: &Slate) -> Result<String, Error> {
        let mut message = "".to_string();
        let mut api = Owner::new(self.instance.clone().unwrap(), None);
        controller::owner_single_use(None, None, Some(&mut api), |api, m| {
            message = api.create_slatepack_message(m, &slate, Some(0), vec![])?;
            Ok(())
        })?;

        // Write Slatepack message to file.
        let slatepack_dir = self.get_config().get_slatepack_path(&slate);
        let mut output = File::create(slatepack_dir)?;
        output.write_all(message.as_bytes())?;
        output.sync_all()?;
        Ok(message)
    }

    /// Read slatepack from file.
    pub fn read_slatepack(&self, slate: &Slate) -> Option<String> {
        let slatepack_path = self.get_config().get_slatepack_path(slate);
        match fs::read_to_string(slatepack_path) {
            Ok(s) => Some(s),
            Err(_) => None
        }
    }

    /// Get last stored [`Slate`] for transaction.
    pub fn read_slate_by_tx(&self, tx: &WalletTransaction) -> Option<(Slate, String)> {
        let mut slate = None;
        if let Some(slate_id) = tx.data.tx_slate_id {
            // Get slate state based on tx state and status.
            let state = if tx.posting {
                if tx.data.tx_type == TxLogEntryType::TxSent {
                    Some(SlateState::Standard3)
                } else {
                    Some(SlateState::Invoice3)
                }
            } else if !tx.data.confirmed && (tx.data.tx_type == TxLogEntryType::TxSent ||
                tx.data.tx_type == TxLogEntryType::TxReceived) {
                if tx.can_finalize {
                    if tx.data.tx_type == TxLogEntryType::TxSent {
                        Some(SlateState::Standard1)
                    } else {
                        Some(SlateState::Invoice1)
                    }
                } else {
                    if tx.data.tx_type == TxLogEntryType::TxReceived {
                        Some(SlateState::Standard2)
                    } else {
                        Some(SlateState::Invoice2)
                    }
                }
            } else {
                None
            };
            // Get slate from state by reading Slatepack message file.
            if let Some(st) = state {
                let mut s = Slate::blank(0, false);
                s.id = slate_id;
                s.state = st;
                if let Some(m) = self.read_slatepack(&s) {
                    if let Ok(s) = self.parse_slatepack(&m) {
                        slate = Some((s, m));
                    }
                }
            }
        }
        slate
    }

    /// Get transaction for [`Slate`] id.
    pub fn tx_by_slate(&self, slate: &Slate) -> Option<WalletTransaction> {
        if let Some(data) = self.get_data() {
            let txs = data.txs.clone().iter().map(|tx| tx.clone()).filter(|tx| {
                tx.data.tx_slate_id == Some(slate.id)
            }).collect::<Vec<WalletTransaction>>();
            return if let Some(tx) = txs.get(0) {
                Some(tx.clone())
            } else {
                None
            }
        }
        None
    }

    /// Initialize a transaction to send amount, return request for funds receiver.
    pub fn send(&self, amount: u64) -> Result<(Slate, String), Error> {
        let config = self.get_config();
        let args = InitTxArgs {
            src_acct_name: Some(config.account),
            amount,
            minimum_confirmations: config.min_confirmations,
            num_change_outputs: 1,
            selection_strategy_is_use_all: false,
            ..Default::default()
        };
        let api = Owner::new(self.instance.clone().unwrap(), None);
        let slate = api.init_send_tx(None, args)?;

        // Lock outputs to for this transaction.
        api.tx_lock_outputs(None, &slate)?;

        // Create Slatepack message response.
        let message_resp = self.create_slatepack_message(&slate)?;

        // Sync wallet info.
        self.sync();

        Ok((slate, message_resp))
    }

    /// Send amount to provided address with Tor transport.
    pub async fn send_tor(&mut self, amount: u64, addr: &SlatepackAddress) -> Option<Slate> {
        // Initialize transaction.
        let send_res = self.send(amount);

        if send_res.is_err() {
            return None;
        }
        let slate = send_res.unwrap().0;

        // Function to cancel initialized tx in case of error.
        let cancel_tx = || {
            let instance = self.instance.clone().unwrap();
            let id = slate.clone().id;
            cancel_tx(instance, None, &None, None, Some(id.clone())).unwrap();
            // Setup posting flag, and ability to finalize.
            {
                let mut w_data = self.data.write();
                let mut data = w_data.clone().unwrap();
                let txs = data.txs.iter_mut().map(|tx| {
                    if tx.data.tx_slate_id == Some(id) {
                        tx.cancelling = false;
                        tx.posting = false;
                        tx.can_finalize = false;
                        tx.data.tx_type = if tx.data.tx_type == TxLogEntryType::TxReceived {
                            TxLogEntryType::TxReceivedCancelled
                        } else {
                            TxLogEntryType::TxSentCancelled
                        };
                    }
                    tx.clone()
                }).collect::<Vec<WalletTransaction>>();
                data.txs = txs;
                *w_data = Some(data);
            }
            // Refresh wallet info to update statuses.
            self.sync();
        };

        // Initialize parameters.
        let tor_addr = OnionV3Address::try_from(addr).unwrap().to_http_str();
        let url = format!("{}/v2/foreign", tor_addr);
        let slate_send = VersionedSlate::into_version(slate.clone(), SlateVersion::V4).unwrap();
        let body = json!({
				"jsonrpc": "2.0",
				"method": "receive_tx",
				"id": 1,
				"params": [
							slate_send,
							null,
							null
						]
			}).to_string();

        // Send request to receiver.
        let req_res = Tor::post(body, url).await;
        if req_res.is_none() {
            cancel_tx();
            return None;
        }

        // Parse response and finalize transaction.
        let res: Value = serde_json::from_str(&req_res.unwrap()).unwrap();
        if res["error"] != json!(null) {
            let report = format!(
                "Posting transaction slate: Error: {}, Message: {}",
                res["error"]["code"], res["error"]["message"]
            );
            cancel_tx();
            return None;
        }

        // Slatepack message json value.
        let slate_value = res["result"]["Ok"].clone();

        let mut ret_slate = None;
        match Slate::deserialize_upgrade(&serde_json::to_string(&slate_value).unwrap()) {
            Ok(s) => {
                let mut api = Owner::new(self.instance.clone().unwrap(), None);
                controller::owner_single_use(None, None, Some(&mut api), |api, m| {
                    // Finalize transaction.
                    return if let Ok(slate) = api.finalize_tx(m, &s) {
                        ret_slate = Some(slate.clone());
                        // Save Slatepack message to file.
                        let _ = self.create_slatepack_message(&slate).unwrap_or("".to_string());
                        // Post transaction to blockchain.
                        let result = self.post(&slate, self.can_use_dandelion());
                        match result {
                            Ok(_) => {
                                Ok(())
                            }
                            Err(e) => {
                                Err(e)
                            }
                        }
                    } else {
                        Err(Error::GenericError("TX finalization error".to_string()))
                    };
                }).unwrap();
            }
            Err(_) => {}
        };

        if ret_slate.is_none() {
            cancel_tx();
        }
        ret_slate
    }

    /// Initialize an invoice transaction to receive amount, return request for funds sender.
    pub fn issue_invoice(&self, amount: u64) -> Result<(Slate, String), Error> {
        let args = IssueInvoiceTxArgs {
            dest_acct_name: None,
            amount,
            target_slate_version: None,
        };
        let api = Owner::new(self.instance.clone().unwrap(), None);
        let slate = api.issue_invoice_tx(None, args)?;

        // Create Slatepack message response.
        let response = self.create_slatepack_message(&slate.clone())?;

        // Sync wallet info.
        self.sync();

        Ok((slate, response))
    }

    /// Handle message from the invoice issuer to send founds, return response for funds receiver.
    pub fn pay(&self, message: &String) -> Result<String, Error> {
        let slate = self.parse_slatepack(message)?;
        let config = self.get_config();
        let args = InitTxArgs {
            src_acct_name: None,
            amount: slate.amount,
            minimum_confirmations: config.min_confirmations,
            selection_strategy_is_use_all: false,
            ..Default::default()
        };
        let api = Owner::new(self.instance.clone().unwrap(), None);
        let slate = api.process_invoice_tx(None, &slate, args)?;
        api.tx_lock_outputs(None, &slate)?;

        // Create Slatepack message response.
        let response = self.create_slatepack_message(&slate)?;

        // Sync wallet info.
        self.sync();

        Ok(response)
    }

    /// Handle message to receive funds, return response to sender.
    pub fn receive(&self, message: &String) -> Result<String, Error> {
        let mut slate = self.parse_slatepack(message)?;
        let api = Owner::new(self.instance.clone().unwrap(), None);
        controller::foreign_single_use(api.wallet_inst.clone(), None, |api| {
            slate = api.receive_tx(&slate, Some(self.get_config().account.as_str()), None)?;
            Ok(())
        })?;
        // Create Slatepack message response.
        let response = self.create_slatepack_message(&slate)?;

        // Sync wallet info.
        self.sync();

        Ok(response)
    }

    /// Finalize transaction from provided message as sender or invoice issuer with Dandelion.
    pub fn finalize(&self, message: &String, dandelion: bool) -> Result<Slate, Error> {
        let mut slate = self.parse_slatepack(message)?;
        let api = Owner::new(self.instance.clone().unwrap(), None);
        slate = api.finalize_tx(None, &slate)?;
        // Save Slatepack message to file.
        let _ = self.create_slatepack_message(&slate)?;
        // Post transaction to blockchain.
        let _ = self.post(&slate, dandelion);
        // Sync wallet info.
        self.sync();
        Ok(slate)
    }

    /// Post transaction to blockchain.
    pub fn post(&self, slate: &Slate, dandelion: bool) -> Result<(), Error> {
        // Post transaction to blockchain.
        let api = Owner::new(self.instance.clone().unwrap(), None);
        api.post_tx(None, slate, dandelion)?;
        // Setup transaction repost height, posting flag and ability to finalize.
        let mut slate = slate.clone();
        if slate.state == SlateState::Invoice2 {
            slate.state = SlateState::Invoice3
        } else if slate.state == SlateState::Standard2 {
            slate.state = SlateState::Standard3
        };
        if let Some(tx) = self.tx_by_slate(&slate) {
            let mut w_data = self.data.write();
            let mut data = w_data.clone().unwrap();
            for t in &mut data.txs {
                if t.data.id == tx.data.id {
                    t.repost_height = Some(data.info.last_confirmed_height);
                    t.posting = true;
                    t.can_finalize = false;
                }
            }
            *w_data = Some(data);
        }
        // Sync wallet info.
        self.sync();
        Ok(())
    }

    /// Cancel transaction.
    pub fn cancel(&mut self, id: u32) {
        // Setup cancelling status.
        {
            let mut w_data = self.data.write();
            let mut data = w_data.clone().unwrap();
            let txs = data.txs.iter_mut().map(|tx| {
                if tx.data.id == id {
                    tx.cancelling = true;
                    tx.can_finalize = false;
                }
                tx.clone()
            }).collect::<Vec<WalletTransaction>>();
            data.txs = txs;
            *w_data = Some(data);
        }

        let wallet = self.clone();
        thread::spawn(move || {
            let instance = wallet.instance.clone().unwrap();
            cancel_tx(instance, None, &None, Some(id), None).unwrap();
            // Setup posting flag, and ability to finalize.
            let mut w_data = wallet.data.write();
            let mut data = w_data.clone().unwrap();
            let txs = data.txs.iter_mut().map(|tx| {
                if tx.data.id == id {
                    tx.cancelling = false;
                    tx.posting = false;
                    tx.can_finalize = false;
                    tx.data.tx_type = if tx.data.tx_type == TxLogEntryType::TxReceived {
                        TxLogEntryType::TxReceivedCancelled
                    } else {
                        TxLogEntryType::TxSentCancelled
                    };
                }
                tx.clone()
            }).collect::<Vec<WalletTransaction>>();
            data.txs = txs;
            *w_data = Some(data);
            // Refresh wallet info to update statuses.
            wallet.sync();
        });
    }

    /// Change wallet password.
    pub fn change_password(&self, old: String, new: String) -> Result<(), Error> {
        let instance = self.instance.clone().unwrap();
        let mut wallet_lock = instance.lock();
        let lc = wallet_lock.lc_provider()?;
        lc.change_password(None, ZeroingString::from(old), ZeroingString::from(new))
    }

    /// Initiate wallet repair by scanning its outputs.
    pub fn repair(&self) {
        self.repair_needed.store(true, Ordering::Relaxed);
        self.sync();
    }

    /// Check if wallet is repairing.
    pub fn is_repairing(&self) -> bool {
        self.repair_needed.load(Ordering::Relaxed)
    }

    /// Get wallet repairing progress.
    pub fn repairing_progress(&self) -> u8 {
        self.repair_progress.load(Ordering::Relaxed)
    }

    /// Get recovery phrase.
    pub fn get_recovery(&self, password: String) -> Result<ZeroingString, Error> {
        let instance = self.instance.clone().unwrap();
        let mut wallet_lock = instance.lock();
        let lc = wallet_lock.lc_provider().unwrap();
        lc.get_mnemonic(None, ZeroingString::from(password))
    }

    /// Close the wallet, delete its files and mark it as deleted.
    pub fn delete_wallet(&self) {
        if !self.is_open() || self.instance.is_none() {
            return;
        }
        self.closing.store(true, Ordering::Relaxed);

        // Delete wallet at separate thread.
        let wallet_delete = self.clone();
        let instance = wallet_delete.instance.clone().unwrap();
        thread::spawn(move || {
            // Close the wallet.
            Self::close_wallet(&instance);

            // Remove wallet files.
            let _ = fs::remove_dir_all(wallet_delete.get_config().get_data_path());

            // Mark wallet as not opened and deleted.
            wallet_delete.closing.store(false, Ordering::Relaxed);
            wallet_delete.is_open.store(false, Ordering::Relaxed);
            wallet_delete.deleted.store(true, Ordering::Relaxed);

            // Start sync to exit.
            wallet_delete.sync();
        });
    }

    /// Check if wallet was deleted to remove it from list.
    pub fn is_deleted(&self) -> bool {
        self.deleted.load(Ordering::Relaxed)
    }
}

/// Delay in seconds to sync [`WalletData`] (60 seconds as average block time).
const SYNC_DELAY: Duration = Duration::from_millis(60 * 1000);

/// Delay in seconds for sync thread to wait before start of new attempt.
const ATTEMPT_DELAY: Duration = Duration::from_millis(3 * 1000);

/// Number of attempts to sync [`WalletData`] before setting an error.
const SYNC_ATTEMPTS: u8 = 10;

/// Launch thread to sync wallet data from node.
fn start_sync(mut wallet: Wallet) -> Thread {
    // Reset progress values.
    wallet.info_sync_progress.store(0, Ordering::Relaxed);
    wallet.txs_sync_progress.store(0, Ordering::Relaxed);
    wallet.repair_progress.store(0, Ordering::Relaxed);

    thread::spawn(move || loop {
        // Close wallet on chain type change.
        if wallet.get_config().chain_type != AppConfig::chain_type() {
            wallet.close();
        }

        // Stop syncing if wallet was closed.
        if !wallet.is_open() {
            // Clear thread instance.
            let mut thread_w = wallet.sync_thread.write();
            *thread_w = None;

            // Clear wallet info.
            let mut w_data = wallet.data.write();
            *w_data = None;
            return;
        }

        // Check integrated node state.
        if wallet.get_current_ext_conn_id().is_none() {
            let not_enabled = !Node::is_running() || Node::is_stopping();
            if not_enabled {
                // Reset loading progress.
                wallet.info_sync_progress.store(0, Ordering::Relaxed);
                wallet.txs_sync_progress.store(0, Ordering::Relaxed);
            }
            // Set an error when required integrated node is not enabled.
            wallet.set_sync_error(not_enabled);
            // Skip cycle when node sync is not finished.
            if !Node::is_running() || Node::get_sync_status() != Some(SyncStatus::NoSync) {
                thread::park_timeout(ATTEMPT_DELAY);
                continue;
            }
        }

        // Start Foreign API listener if API server is not running.
        let mut api_server_running = {
            wallet.foreign_api_server.read().is_some()
        };
        if !api_server_running && wallet.is_open() {
            match start_api_server(&mut wallet) {
                Ok(api_server) => {
                    let mut api_server_w = wallet.foreign_api_server.write();
                    *api_server_w = Some(api_server);
                    api_server_running = true;
                }
                Err(_) => {}
            }
        }

        // Start Tor service if API server is running and wallet is open.
        if wallet.auto_start_tor_listener() && wallet.is_open() && api_server_running &&
            !Tor::is_service_running(&wallet.identifier()) {
            let r_foreign_api = wallet.foreign_api_server.read();
            let api = r_foreign_api.as_ref().unwrap();
            if let Ok(sec_key) = wallet.secret_key() {
                Tor::start_service(api.1, sec_key, &wallet.identifier());
            }
        }

        // Scan outputs if repair is needed or sync data if there is no error.
        if !wallet.sync_error() {
            if wallet.is_repairing() {
                repair_wallet(&wallet)
            } else {
                sync_wallet_data(&wallet);
            }
        }

        // Stop sync if wallet was closed.
        if !wallet.is_open() {
            // Clear thread instance.
            let mut thread_w = wallet.sync_thread.write();
            *thread_w = None;

            // Clear wallet info.
            let mut w_data = wallet.data.write();
            *w_data = None;
            return;
        }

        // Repeat after default or attempt delay if synchronization was not successful.
        let failed_sync = wallet.sync_error() || wallet.get_sync_attempts() != 0;
        let delay = if failed_sync {
            ATTEMPT_DELAY
        } else {
            SYNC_DELAY
        };
        thread::park_timeout(delay);
    }).thread().clone()
}

/// Retrieve [`WalletData`] from node.
fn sync_wallet_data(wallet: &Wallet) {
    let wallet_info = wallet.clone();
    let (info_tx, info_rx) = mpsc::channel::<StatusMessage>();
    // Update info sync progress at separate thread.
    thread::spawn(move || {
        while let Ok(m) = info_rx.recv() {
            match m {
                StatusMessage::UpdatingOutputs(_) => {}
                StatusMessage::UpdatingTransactions(_) => {}
                StatusMessage::FullScanWarn(_) => {}
                StatusMessage::Scanning(_, progress) => {
                    wallet_info.info_sync_progress.store(progress, Ordering::Relaxed);
                }
                StatusMessage::ScanningComplete(_) => {
                    wallet_info.info_sync_progress.store(100, Ordering::Relaxed);
                }
                StatusMessage::UpdateWarning(_) => {}
            }
        }
    });

    let config = wallet.get_config();

    // Retrieve wallet info.
    if let Some(instance) = &wallet.instance {
        if let Ok(info) = retrieve_summary_info(
            instance.clone(),
            None,
            &Some(info_tx),
            true,
            config.min_confirmations
        ) {
            // Do not retrieve txs if wallet was closed.
            if !wallet.is_open() {
                return;
            }

            if wallet.info_sync_progress() == 100 {
                // Retrieve accounts data.
                let last_height = info.1.last_confirmed_height;
                let spendable = if wallet.get_data().is_none() {
                    None
                } else {
                    Some(info.1.amount_currently_spendable)
                };
                update_accounts(wallet, last_height, spendable);

                // Update txs sync progress at separate thread.
                let wallet_txs = wallet.clone();
                let (txs_tx, txs_rx) = mpsc::channel::<StatusMessage>();
                thread::spawn(move || {
                    while let Ok(m) = txs_rx.recv() {
                        match m {
                            StatusMessage::UpdatingOutputs(_) => {}
                            StatusMessage::UpdatingTransactions(_) => {}
                            StatusMessage::FullScanWarn(_) => {}
                            StatusMessage::Scanning(_, progress) => {
                                wallet_txs.txs_sync_progress.store(progress, Ordering::Relaxed);
                            }
                            StatusMessage::ScanningComplete(_) => {
                                wallet_txs.txs_sync_progress.store(100, Ordering::Relaxed);
                            }
                            StatusMessage::UpdateWarning(_) => {}
                        }
                    }
                });

                let txs_args = RetrieveTxQueryArgs {
                    exclude_cancelled: Some(false),
                    sort_field: Some(RetrieveTxQuerySortField::CreationTimestamp),
                    sort_order: Some(RetrieveTxQuerySortOrder::Desc),
                    ..Default::default()
                };
                if let Ok(txs) = retrieve_txs(instance.clone(),
                                              None,
                                              &Some(txs_tx),
                                              true,
                                              None,
                                              None,
                                              Some(txs_args)) {
                    // Do not sync data if wallet was closed.
                    if !wallet.is_open() {
                        return;
                    }
                    // Save data if loading was completed.
                    if wallet.txs_sync_progress() == 100 {
                        // Reset attempts.
                        wallet.reset_sync_attempts();

                        // Filter transactions for current account.
                        let filter_txs = txs.1.iter().map(|v| v.clone()).filter(|tx| {
                            match wallet.get_parent_key_id() {
                                Ok(key) => {
                                    tx.parent_key_id == key
                                }
                                Err(_) => {
                                    true
                                }
                            }
                        }).collect::<Vec<TxLogEntry>>();

                        // Create wallet txs.
                        let mut new_txs: Vec<WalletTransaction> = vec![];
                        for tx in &filter_txs {
                            // Setup transaction amount.
                            let amount = if tx.amount_debited > tx.amount_credited {
                                tx.amount_debited - tx.amount_credited
                            } else {
                                tx.amount_credited - tx.amount_debited
                            };

                            let unconfirmed_sent_or_received = tx.tx_slate_id.is_some() &&
                                !tx.confirmed && (tx.tx_type == TxLogEntryType::TxSent ||
                                tx.tx_type == TxLogEntryType::TxReceived);

                            // Setup transaction posting status based on slate state.
                            let posting = if unconfirmed_sent_or_received {
                                // Create slate to check existing file.
                                let is_invoice = tx.tx_type == TxLogEntryType::TxReceived;
                                let mut slate = Slate::blank(0, is_invoice);
                                slate.id = tx.tx_slate_id.unwrap();
                                slate.state = match is_invoice {
                                    true => SlateState::Invoice3,
                                    _ => SlateState::Standard3
                                };

                                // Setup posting status if we have other tx with same slate id.
                                let mut same_tx_posting = false;
                                for t in &mut new_txs {
                                    if t.data.tx_slate_id == tx.tx_slate_id &&
                                        tx.tx_type != t.data.tx_type {
                                        same_tx_posting = t.posting ||
                                            wallet.read_slatepack(&slate).is_some();
                                        if same_tx_posting && !t.posting {
                                            t.posting = true;
                                        }
                                        break;
                                    }
                                }
                                same_tx_posting || wallet.read_slatepack(&slate).is_some()
                            } else {
                                false
                            };

                            // Setup flag for ability to finalize transaction.
                            let can_finalize = if !posting && unconfirmed_sent_or_received {
                                // Create slate to check existing file.
                                let mut slate = Slate::blank(1, false);
                                slate.id = tx.tx_slate_id.unwrap();
                                slate.state = match tx.tx_type {
                                    TxLogEntryType::TxReceived => SlateState::Invoice1,
                                    _ => SlateState::Standard1
                                };
                                wallet.read_slatepack(&slate).is_some()
                            } else {
                                false
                            };

                            // Setup reposting height.
                            let mut repost_height = None;
                            if posting {
                                if let Some(mut data) = wallet.get_data() {
                                    for t in data.txs {
                                        if t.data.id == tx.id {
                                            repost_height = t.repost_height;
                                            break;
                                        }
                                    }
                                }
                            }

                            // Add transaction to list.
                            new_txs.push(WalletTransaction {
                                data: tx.clone(),
                                amount,
                                cancelling: false,
                                posting,
                                can_finalize,
                                repost_height,
                            })
                        }

                        // Update wallet data.
                        let mut w_data = wallet.data.write();
                        *w_data = Some(WalletData { info: info.1, txs: new_txs });
                        return;
                    }
                }
            }
        }
    }

    // Reset progress.
    wallet.info_sync_progress.store(0, Ordering::Relaxed);
    wallet.txs_sync_progress.store(0, Ordering::Relaxed);

    // Exit if wallet was closed.
    if !wallet.is_open() {
        return;
    }

    // Set an error if data was not loaded after opening or increment attempts count.
    if wallet.get_data().is_none() {
        wallet.set_sync_error(true);
    } else {
        wallet.increment_sync_attempts();
    }

    // Set an error if maximum number of attempts was reached.
    if wallet.get_sync_attempts() >= SYNC_ATTEMPTS {
        wallet.reset_sync_attempts();
        wallet.set_sync_error(true);
    }
}

/// Start Foreign API server to receive txs over transport and mining rewards.
fn start_api_server(wallet: &mut Wallet) -> Result<(ApiServer, u16), Error> {
    let host = "127.0.0.1";
    // Find free port.
    let port = if wallet.get_config().chain_type == ChainTypes::Mainnet {
        rand::thread_rng().gen_range(37000..40000)
    } else {
        rand::thread_rng().gen_range(47000..50000)
    };
    let free_port = (port..).find(|port| {
        return match TcpListener::bind((host, port.to_owned())) {
            Ok(_) => {
                let node_p2p_port = NodeConfig::get_p2p_port();
                let node_api_port = NodeConfig::get_api_ip_port().1;
                port.to_string() != node_p2p_port && port.to_string() != node_api_port
            },
            Err(_) => false
        }
    }).unwrap();

    // Setup API server address.
    let api_addr = format!("{}:{}", host, free_port);

    // Start Foreign API server thread.
    let instance = wallet.instance.clone().unwrap();
    let api_handler_v2 = ForeignAPIHandlerV2::new(instance,
                                                  Arc::new(Mutex::new(None)),
                                                  false,
                                                  Mutex::new(None));
    let mut router = Router::new();
    router
        .add_route("/v2/foreign", Arc::new(api_handler_v2))
        .map_err(|_| Error::GenericError("Router failed to add route".to_string()))?;

    let api_chan: &'static mut (oneshot::Sender<()>, oneshot::Receiver<()>) =
        Box::leak(Box::new(oneshot::channel::<()>()));

    let mut apis = ApiServer::new();
    let socket_addr: SocketAddr = api_addr.parse().unwrap();
    let _ = apis.start(socket_addr, router, None, api_chan)
        .map_err(|_| Error::GenericError("API thread failed to start".to_string()))?;
    Ok((apis, free_port))
}

/// Update wallet accounts data.
fn update_accounts(wallet: &Wallet, current_height: u64, current_spendable: Option<u64>) {
    // Update only current account if list is not empty.
    if current_spendable.is_some() {
        let mut accounts = wallet.accounts.read().clone();
        for mut a in accounts.iter_mut() {
            if a.label == wallet.get_config().account {
                a.spendable_amount = current_spendable.unwrap();
            }
        }
        // Save accounts data.
        let mut w_data = wallet.accounts.write();
        *w_data = accounts;
    } else {
        let mut api = Owner::new(wallet.instance.clone().unwrap(), None);
        let _ = controller::owner_single_use(None, None, Some(&mut api), |api, m| {
            let mut accounts = vec![];
            for a in api.accounts(m)? {
                api.set_active_account(m, a.label.as_str())?;
                // Calculate wallet account balance.
                let outputs = api.retrieve_outputs(m, true, false, None)?;
                let min_confirmations = wallet.get_config().min_confirmations;
                let mut spendable_amount = 0;
                for out_mapping in outputs.1 {
                    let out = out_mapping.output;
                    if out.status == grin_wallet_libwallet::OutputStatus::Unspent {
                        if !out.is_coinbase || out.lock_height <= current_height
                            || out.num_confirmations(current_height) >= min_confirmations {
                            spendable_amount += out.value;
                        }
                    }
                }

                // Add account to list.
                accounts.push(WalletAccount {
                    spendable_amount,
                    label: a.label,
                    path: a.path.to_bip_32_string(),
                });
            }
            // Sort in reverse.
            accounts.reverse();

            // Save accounts data.
            let mut w_data = wallet.accounts.write();
            *w_data = accounts;

            // Set current active account from config.
            api.set_active_account(m, wallet.get_config().account.as_str())?;

            Ok(())
        });
    }
}

/// Scan wallet's outputs, repairing and restoring missing outputs if required.
fn repair_wallet(wallet: &Wallet) {
    let (info_tx, info_rx) = mpsc::channel::<StatusMessage>();
    // Update scan progress at separate thread.
    let wallet_scan = wallet.clone();
    thread::spawn(move || {
        while let Ok(m) = info_rx.recv() {
            match m {
                StatusMessage::UpdatingOutputs(_) => {}
                StatusMessage::UpdatingTransactions(_) => {}
                StatusMessage::FullScanWarn(_) => {}
                StatusMessage::Scanning(_, progress) => {
                    wallet_scan.repair_progress.store(progress, Ordering::Relaxed);
                }
                StatusMessage::ScanningComplete(_) => {
                    wallet_scan.repair_progress.store(100, Ordering::Relaxed);
                }
                StatusMessage::UpdateWarning(_) => {}
            }
        }
    });

    // Start wallet scanning.
    let api = Owner::new(wallet.instance.clone().unwrap(), Some(info_tx));
    match api.scan(None, Some(1), false) {
        Ok(()) => {
            // Set sync error if scanning was not complete and wallet is open.
            if wallet.is_open() && wallet.repair_progress.load(Ordering::Relaxed) != 100 {
                wallet.set_sync_error(true);
            } else {
                wallet.repair_needed.store(false, Ordering::Relaxed);
            }
        }
        Err(_) => {
            // Set sync error if wallet is open.
            if wallet.is_open() {
                wallet.set_sync_error(true);
            } else {
                wallet.repair_needed.store(false, Ordering::Relaxed);
            }
        }
    }

    // Reset repair progress.
    wallet.repair_progress.store(0, Ordering::Relaxed);
}