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

use futures::channel::oneshot;
use grin_api::{ApiServer, Router};
use grin_chain::SyncStatus;
use grin_keychain::{ExtKeychain, Identifier, Keychain};
use grin_util::secp::SecretKey;
use grin_util::types::ZeroingString;
use grin_util::{Mutex, ToHex};
use grin_wallet_api::Owner;
use grin_wallet_controller::command::parse_slatepack;
use grin_wallet_controller::controller;
use grin_wallet_controller::controller::ForeignAPIHandlerV2;
use grin_wallet_impls::{DefaultLCProvider, DefaultWalletImpl, HTTPNodeClient, LMDBBackend};
use grin_wallet_libwallet::api_impl::owner::{cancel_tx, retrieve_summary_info, retrieve_txs};
use grin_wallet_libwallet::{address, Error, InitTxArgs, IssueInvoiceTxArgs, NodeClient, RetrieveTxQueryArgs, RetrieveTxQuerySortField, RetrieveTxQuerySortOrder, Slate, SlateState, SlateVersion, SlatepackAddress, StatusMessage, TxLogEntry, TxLogEntryType, VersionedSlate, WalletBackend, WalletInfo, WalletInitStatus, WalletInst, WalletLCProvider};
use grin_wallet_util::OnionV3Address;
use parking_lot::RwLock;
use rand::Rng;
use serde_json::{json, Value};
use std::fs::File;
use std::io::Write;
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc};
use std::thread::Thread;
use std::time::Duration;
use std::{fs, thread};

use crate::node::{Node, NodeConfig};
use crate::tor::Tor;
use crate::wallet::seed::WalletSeed;
use crate::wallet::store::TxHeightStore;
use crate::wallet::types::{ConnectionMethod, PhraseMode, WalletAccount, WalletData, WalletInstance, WalletTask, WalletTransaction, WalletTransactionAction};
use crate::wallet::{ConnectionsConfig, Mnemonic, WalletConfig};
use crate::AppConfig;

/// Contains wallet instance, configuration and state, handles wallet commands.
#[derive(Clone)]
pub struct Wallet {
    /// Wallet configuration.
    config: Arc<RwLock<WalletConfig>>,
    /// Wallet instance, initializing on wallet opening and clearing on wallet closing.
    instance: Arc<RwLock<Option<WalletInstance>>>,
    /// Connection of current wallet instance.
    connection: Arc<RwLock<ConnectionMethod>>,

    /// Wallet Slatepack address to receive txs at transport.
    slatepack_address: Arc<RwLock<Option<String>>>,

    /// Wallet accounts.
    accounts: Arc<RwLock<Vec<WalletAccount>>>,

    /// Wallet sync thread.
    sync_thread: Arc<RwLock<Option<Thread>>>,
    /// Flag to check if wallet is syncing.
    syncing: Arc<AtomicBool>,
    /// Info loading progress in percents.
    info_sync_progress: Arc<AtomicU8>,
    /// Error on wallet loading.
    sync_error: Arc<AtomicBool>,
    /// Attempts amount to update wallet data.
    sync_attempts: Arc<AtomicU8>,

    /// Wallet data.
    data: Arc<RwLock<Option<WalletData>>>,

    /// Flag to check if wallet reopening is needed.
    reopen: Arc<AtomicBool>,
    /// Flag to check if wallet is open.
    is_open: Arc<AtomicBool>,
    /// Flag to check if wallet is closing.
    closing: Arc<AtomicBool>,
    /// Flag to check if wallet was deleted to remove it from the list.
    deleted: Arc<AtomicBool>,

    /// Running wallet foreign API server and port.
    foreign_api_server: Arc<RwLock<Option<(ApiServer, u16)>>>,

    /// Flag to check if wallet repairing and restoring missing outputs is needed.
    repair_needed: Arc<AtomicBool>,
    /// Wallet repair progress in percents.
    repair_progress: Arc<AtomicU8>,

    /// Flag to check if Slatepack message file is opening.
    message_opening: Arc<AtomicBool>,

    /// Flag to check if sending request is creating.
    send_creating: Arc<AtomicBool>,
    /// Flag to check if invoice is creating.
    invoice_creating: Arc<AtomicBool>,

    /// Tasks sender.
    tasks_sender: Arc<RwLock<Option<Sender<WalletTask>>>>,
    /// Transaction identifier after successful task completion.
    /// To be replaced with https://github.com/lucasmerlin/hello_egui/tree/main/crates/egui_inbox.
    task_result_slate_id: Arc<RwLock<Option<String>>>,
}

impl Wallet {
    /// Create new [`Wallet`] instance with provided [`WalletConfig`].
    fn new(config: WalletConfig) -> Self {
        let connection = config.connection();
        Self {
            config: Arc::new(RwLock::new(config)),
            instance: Arc::new(RwLock::new(None)),
            connection: Arc::new(RwLock::new(connection)),
            slatepack_address: Arc::new(RwLock::new(None)),
            sync_thread: Arc::from(RwLock::new(None)),
            foreign_api_server: Arc::new(RwLock::new(None)),
            reopen: Arc::new(AtomicBool::new(false)),
            is_open: Arc::from(AtomicBool::new(false)),
            closing: Arc::new(AtomicBool::new(false)),
            deleted: Arc::new(AtomicBool::new(false)),
            sync_error: Arc::from(AtomicBool::new(false)),
            info_sync_progress: Arc::from(AtomicU8::new(0)),
            accounts: Arc::new(RwLock::new(vec![])),
            data: Arc::new(RwLock::new(None)),
            sync_attempts: Arc::new(AtomicU8::new(0)),
            syncing: Arc::new(AtomicBool::new(false)),
            repair_needed: Arc::new(AtomicBool::new(false)),
            repair_progress: Arc::new(AtomicU8::new(0)),
            message_opening: Arc::new(AtomicBool::from(false)),
            send_creating: Arc::new(AtomicBool::new(false)),
            invoice_creating: Arc::new(AtomicBool::new(false)),
            tasks_sender: Arc::new(RwLock::new(None)),
            task_result_slate_id: Arc::new(RwLock::new(None)),
        }
    }

    /// Create new wallet.
    pub fn create(
        name: &String,
        password: &ZeroingString,
        mnemonic: &Mnemonic,
        conn_method: &ConnectionMethod
    ) -> Result<Wallet, Error> {
        let config = WalletConfig::create(name.clone(), conn_method);
        let w = Wallet::new(config.clone());
        {
            // create directory if it doesn't exist
            fs::create_dir_all(config.get_data_path())
                .map_err(|_| Error::IO("Directory creation error".to_string()))?;
            // Create seed file.
            let _ = WalletSeed::init_file(config.seed_path().as_str(),
                                          ZeroingString::from(mnemonic.get_phrase()),
                                          password.clone())
                .map_err(|_| Error::IO("Seed file creation error".to_string()))?;
            let node_client = Self::create_node_client(&config)?;
            let mut wallet: LMDBBackend<'static, HTTPNodeClient, ExtKeychain> =
                match LMDBBackend::new(config.get_data_path().as_str(), node_client) {
                    Err(_) => {
                        return Err(Error::Lifecycle("DB creation error".to_string()).into());
                    }
                    Ok(d) => d,
                };
            // Save init status of this wallet, to determine whether it needs a full UTXO scan
            let mut batch = wallet.batch_no_mask()?;
            match mnemonic.mode() {
                PhraseMode::Generate => batch.save_init_status(WalletInitStatus::InitNoScanning)?,
                PhraseMode::Import => batch.save_init_status(WalletInitStatus::InitNeedsScanning)?,
            }
            batch.commit()?;
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

    /// Create [`HTTPNodeClient`] from provided config.
    fn create_node_client(config: &WalletConfig) -> Result<HTTPNodeClient, Error> {
        let integrated = || {
            let api_url = format!("http://{}", NodeConfig::get_api_address());
            let api_secret = NodeConfig::get_api_secret(true);
            (api_url, api_secret)
        };
        let (node_api_url, node_secret) = if let Some(id) = config.ext_conn_id {
            if let Some(conn) = ConnectionsConfig::ext_conn(id) {
                (conn.url, conn.secret)
            } else {
                integrated()
            }
        } else {
            integrated()
        };
        let client = if AppConfig::use_proxy() {
            let socks = AppConfig::use_socks_proxy();
            let url = if socks {
                AppConfig::socks_proxy_url()
            } else {
                AppConfig::http_proxy_url()
            }.unwrap_or("".to_string());
            let res = url.replace("http://", "").replace("socks5://", "").parse();
            if let Ok(addr) = res {
                let scheme = if socks {
                    "socks5://"
                } else {
                    "http://"
                };
                HTTPNodeClient::new_proxy(&node_api_url, node_secret, Some((addr, scheme)))?
            } else {
                HTTPNodeClient::new_proxy(&node_api_url, node_secret, None)?
            }
        } else {
            HTTPNodeClient::new_proxy(&node_api_url, node_secret, None)?
        };
        Ok(client)
    }

    /// Create [`WalletInstance`] from provided [`WalletConfig`].
    fn create_wallet_instance(config: &mut WalletConfig) -> Result<WalletInstance, Error> {
        // Setup node client.
        let node_client = Self::create_node_client(config)?;

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
        config: &mut WalletConfig,
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
        lc.set_top_level_directory(config.get_wallet_path().as_str())?;
        Ok(Arc::new(Mutex::new(wallet)))
    }

    /// Open the wallet and start [`WalletData`] sync at separate thread.
    pub fn open(&self, password: ZeroingString) -> Result<(), Error> {
        if self.is_open() {
            return Err(Error::GenericError("Already opened".to_string()));
        }

        // Create new wallet instance if sync thread was stopped or instance was not created.
        let has_instance = {
            let r_inst = self.instance.as_ref().read();
            r_inst.is_some()
        };
        if self.sync_thread.read().is_none() || !has_instance {
            let mut config = self.get_config();
            // Setup current connection.
            {
                let mut w_conn = self.connection.write();
                *w_conn = config.connection();
            }
            let new_instance = Self::create_wallet_instance(&mut config)?;
            let mut w_inst = self.instance.write();
            *w_inst = Some(new_instance);
        }

        // Open the wallet.
        {
            let instance = {
                let r_inst = self.instance.as_ref().read();
                r_inst.clone().unwrap()
            };
            let mut wallet_lock = instance.lock();
            let lc = wallet_lock.lc_provider()?;
            match lc.open_wallet(None, password, false, false) {
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
                    if !self.syncing() {
                        let mut w_inst = self.instance.write();
                        *w_inst = None;
                    }
                    return Err(e)
                }
            }
        }

        // Set slatepack address.
        let r_inst = self.instance.as_ref().read();
        let instance = r_inst.clone().unwrap();
        let mut api = Owner::new(instance, None);
        controller::owner_single_use(None, None, Some(&mut api), |api, m| {
            let mut w_address = self.slatepack_address.write();
            *w_address = Some(api.get_slatepack_address(m, 0)?.to_string());
            Ok(())
        })?;

        Ok(())
    }

    /// Get parent key identifier for current account.
    fn get_parent_key_id(&self) -> Result<Identifier, Error> {
        let r_inst = self.instance.as_ref().read();
        let instance = r_inst.clone().unwrap();
        let mut w_lock = instance.lock();
        let lc = w_lock.lc_provider()?;
        let w_inst = lc.wallet_inst()?;
        Ok(w_inst.parent_key_id())
    }

    /// Get wallet [`SecretKey`] for transports.
    pub fn get_secret_key(&self) -> Result<SecretKey, Error> {
        let r_inst = self.instance.as_ref().read();
        let instance = r_inst.clone().unwrap();
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

    /// Get transaction broadcasting delay in blocks.
    pub fn broadcasting_delay(&self) -> u64 {
        let r_config = self.config.read();
        r_config.tx_broadcast_timeout.unwrap_or(WalletConfig::BROADCASTING_TIMEOUT_DEFAULT)
    }

    /// Update transaction broadcasting delay in blocks.
    pub fn update_broadcasting_delay(&self, delay: u64) {
        let mut w_config = self.config.write();
        w_config.tx_broadcast_timeout = Some(delay);
        w_config.save();
    }

    /// Update external connection identifier.
    pub fn update_connection(&self, conn: &ConnectionMethod) {
        let mut w_config = self.config.write();
        w_config.ext_conn_id = match conn {
            ConnectionMethod::Integrated => None,
            ConnectionMethod::External(id, _) => Some(id.clone())
        };
        w_config.save();
    }

    /// Get external connection URL applied to [`WalletInstance`]
    /// after wallet opening if sync is running or get it from config.
    pub fn get_current_connection(&self) -> ConnectionMethod {
        if self.sync_thread.read().is_some() {
            let r_conn = self.connection.read();
            r_conn.clone()
        } else {
            let config = self.get_config();
            config.connection()
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
        let has_instance = {
            let r_inst = self.instance.read();
            r_inst.is_some()
        };
        if !self.is_open() || !has_instance {
            return;
        }
        self.closing.store(true, Ordering::Relaxed);

        // Close wallet at separate thread.
        let wallet_close = self.clone();
        let service_id = wallet_close.identifier();
        let conn = wallet_close.connection.clone();
        thread::spawn(move || {
            // Wait common operations to finish.
            while wallet_close.message_opening() || wallet_close.send_creating() ||
                wallet_close.invoice_creating() {
                thread::sleep(Duration::from_millis(300));
            }
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
            let r_inst = wallet_close.instance.as_ref().read();
            let instance = r_inst.clone().unwrap();
            Self::close_wallet(&instance);
            wallet_close.closing.store(false, Ordering::Relaxed);
            wallet_close.is_open.store(false, Ordering::Relaxed);
            // Setup current connection.
            {
                let mut w_conn = conn.write();
                *w_conn = wallet_close.get_config().connection();
            }
            // Start sync to exit from thread.
            wallet_close.sync();
        });
    }

    /// Close wallet for provided [`WalletInstance`].
    fn close_wallet(instance: &WalletInstance) {
        let mut wallet_lock = instance.lock();
        let lc = wallet_lock.lc_provider().unwrap();
        let _ = lc.close_wallet(None);
    }

    /// Set wallet reopen status.
    pub fn set_reopen(&self, reopen: bool) {
        self.reopen.store(reopen, Ordering::Relaxed);
    }

    /// Check if wallet reopen is needed.
    pub fn reopen_needed(&self) -> bool {
        self.reopen.load(Ordering::Relaxed)
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

    /// Send a task to the wallet.
    pub fn task(&self, task: WalletTask) {
        let r_tasks = self.tasks_sender.read();
        if r_tasks.is_some() {
            let _ = r_tasks.as_ref().unwrap().send(task);
        }
    }

    /// Create account into wallet.
    pub fn create_account(&self, label: &String) -> Result<(), Error> {
        let r_inst = self.instance.as_ref().read();
        let instance = r_inst.clone().unwrap();
        let mut api = Owner::new(instance, None);
        controller::owner_single_use(None, None, Some(&mut api), |api, m| {
            let id = api.create_account_path(m, label)?;
            if self.get_data().is_none() {
                return Err(Error::GenericError("No wallet data".to_string()));
            }
            let current_height = self.get_data().unwrap().info.last_confirmed_height;
            if let Some(spendable_amount) = self.account_balance(current_height, api, m) {
                let mut w_data = self.accounts.write();
                w_data.push(WalletAccount {
                    spendable_amount,
                    label: label.clone(),
                    path: id.to_bip_32_string(),
                });
                w_data.sort_by_key(|w| w.label != label.clone());
            }
            Ok(())
        })
    }

    /// Set active account from provided label.
    pub fn set_active_account(&self, label: &String) -> Result<(), Error> {
        let r_inst = self.instance.as_ref().read();
        let instance = r_inst.clone().unwrap();
        let mut api = Owner::new(instance, None);
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

        // Sync wallet data.
        self.sync();
        Ok(())
    }

    /// Calculate current account balance.
    fn account_balance(
        &self,
        current_height: u64,
        o: &mut Owner<DefaultLCProvider<HTTPNodeClient, ExtKeychain>, HTTPNodeClient, ExtKeychain>,
        m: Option<&SecretKey>)
        -> Option<u64> {
        if let Ok(outputs) = o.retrieve_outputs(m, false, false, None) {
            let mut spendable = 0;
            let min_confirmations = self.get_config().min_confirmations;
            for out_mapping in outputs.1 {
                let out = out_mapping.output;
                if out.status == grin_wallet_libwallet::OutputStatus::Unspent {
                    if !out.is_coinbase || out.lock_height <= current_height
                        || out.num_confirmations(current_height) >= min_confirmations {
                        spendable += out.value;
                    }
                }
            }
            return Some(spendable);
        }
        None
    }

    /// Get list of accounts for the wallet.
    pub fn accounts(&self) -> Vec<WalletAccount> {
        self.accounts.read().clone()
    }

    /// Get wallet data.
    pub fn get_data(&self) -> Option<WalletData> {
        let r_data = self.data.read();
        r_data.clone()
    }

    /// Sync wallet data from node at sync thread or locally synchronously.
    pub fn sync(&self) {
        let thread_r = self.sync_thread.read();
        if let Some(thread) = thread_r.as_ref() {
            thread.unpark();
        }
    }

    /// Check if wallet is syncing.
    pub fn syncing(&self) -> bool {
        self.syncing.load(Ordering::Relaxed)
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

    /// Open Slatepack message with the wallet.
    fn open_message(&self, message: &String) {
        if !self.is_open() || message.is_empty() {
            return;
        }
        let w = self.clone();
        let load = self.message_opening.clone();
        let msg = message.clone();
        thread::spawn(move || {
            load.store(true, Ordering::Relaxed);
            if let Ok(s) = w.parse_slatepack(&msg) {
                // Check if message with same id and state already exists.
                let exists = w.slatepack_exists(&s);
                if exists {
                    w.on_tx_result(&s);
                    load.store(false, Ordering::Relaxed);
                    return;
                }
                // Create response or finalize.
                match s.state {
                    SlateState::Standard1 | SlateState::Invoice1 => {
                        if s.state != SlateState::Standard1 {
                            if let Ok(s) = w.pay(&s) {
                                sync_wallet_data(&w, false);
                                w.on_tx_result(&s);
                            }
                        } else {
                            if let Ok(s) = w.receive(&s) {
                                sync_wallet_data(&w, false);
                                w.on_tx_result(&s);
                            }
                        }
                    }
                    SlateState::Standard2 | SlateState::Invoice2 => {
                        match w.finalize(&s) {
                            Ok(s) => {
                                match w.post(&s) {
                                    Ok(_) => {
                                        sync_wallet_data(&w, false);
                                    }
                                    Err(e) => {
                                        w.on_tx_error(s.id.to_string(), Some(e));
                                    }
                                }
                            }
                            Err(e) => {
                                w.on_tx_error(s.id.to_string(), Some(e));
                            }
                        }
                    }
                    _ => {}
                };
            }
            load.store(false, Ordering::Relaxed);
        });
    }

    /// Check if Slatepack message is opening.
    pub fn message_opening(&self) -> bool {
        self.message_opening.load(Ordering::Relaxed)
    }

    /// Parse Slatepack message into [`Slate`].
    fn parse_slatepack(&self, text: &String) -> Result<Slate, grin_wallet_controller::Error> {
        let r_inst = self.instance.as_ref().read();
        let instance = r_inst.clone().unwrap();
        let mut api = Owner::new(instance, None);
        match parse_slatepack(&mut api, None, None, Some(text.clone())) {
            Ok(s) => Ok(s.0),
            Err(e) => Err(e)
        }
    }

    /// Create Slatepack message from provided slate.
    fn create_slatepack_message(&self, slate: &Slate) -> Result<String, Error> {
        let mut message = "".to_string();
        let r_inst = self.instance.as_ref().read();
        let instance = r_inst.clone().unwrap();
        let mut api = Owner::new(instance, None);
        controller::owner_single_use(None, None, Some(&mut api), |api, m| {
            message = api.create_slatepack_message(m, &slate, Some(0), vec![])?;
            Ok(())
        })?;

        // Write Slatepack message to file.
        let slatepack_dir = self.get_config().get_slate_path(&slate);
        let mut output = File::create(slatepack_dir)?;
        output.write_all(message.as_bytes())?;
        output.sync_all()?;
        Ok(message)
    }

    /// Check if Slatepack file exists.
    pub fn slatepack_exists(&self, slate: &Slate) -> bool {
        let slatepack_path = self.get_config().get_slate_path(slate);
        fs::exists(slatepack_path).unwrap()
    }

    /// Read Slatepack file content.
    pub fn read_slatepack(&self, tx: &WalletTransaction) -> Option<String> {
        let slatepack_path = self.get_config().get_tx_slate_path(tx);
        if let Ok(m) = fs::read_to_string(slatepack_path) {
            return Some(m);
        }
        None
    }

    /// Initialize a transaction to send amount, return request for funds receiver.
    fn send(&self, a: u64, r: Option<SlatepackAddress>) -> Result<Slate, Error> {
        let config = self.get_config();
        let args = InitTxArgs {
            payment_proof_recipient_address: r,
            src_acct_name: Some(config.account),
            amount: a,
            minimum_confirmations: config.min_confirmations,
            num_change_outputs: 1,
            selection_strategy_is_use_all: false,
            ..Default::default()
        };
        let r_inst = self.instance.as_ref().read();
        let instance = r_inst.clone().unwrap();
        let api = Owner::new(instance, None);
        let slate = api.init_send_tx(None, args)?;

        // Lock outputs to for this transaction.
        api.tx_lock_outputs(None, &slate)?;

        // Create Slatepack message response.
        let _ = self.create_slatepack_message(&slate)?;

        Ok(slate)
    }

    /// Send slate to Tor address.
    async fn send_tor(&self, slate: &Slate, addr: &SlatepackAddress) -> Result<Slate, Error> {
        self.on_tx_action(slate.id.to_string(), Some(WalletTransactionAction::SendingTor));

        let tor_addr = OnionV3Address::try_from(addr).unwrap().to_http_str();
        let url = format!("{}/v2/foreign", tor_addr);
        let slate_send = VersionedSlate::into_version(slate.clone(), SlateVersion::V4)?;
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
            return Err(Error::GenericError("Tor request error".to_string()));
        }

        // Parse response.
        let res: Value = serde_json::from_str(&req_res.unwrap()).unwrap();
        if res["error"] != json!(null) {
            return Err(Error::GenericError("Response error".to_string()));
        }
        let slate_value = res["result"]["Ok"].clone();
        let res = Slate::deserialize_upgrade(&serde_json::to_string(&slate_value).unwrap());

        // Clear tx action.
        self.on_tx_action(slate.id.to_string(), None);

        res
    }

    /// Check if request to send funds is creating.
    pub fn send_creating(&self) -> bool {
        self.send_creating.load(Ordering::Relaxed)
    }

    /// Initialize an invoice transaction to receive amount, return request for funds sender.
    fn issue_invoice(&self, amount: u64) -> Result<Slate, Error> {
        let args = IssueInvoiceTxArgs {
            dest_acct_name: None,
            amount,
            target_slate_version: None,
        };
        let r_inst = self.instance.as_ref().read();
        let instance = r_inst.clone().unwrap();
        let api = Owner::new(instance, None);
        let slate = api.issue_invoice_tx(None, args)?;

        // Create Slatepack message response.
        let _ = self.create_slatepack_message(&slate)?;

        Ok(slate)
    }

    /// Check if request to receive funds is creating.
    pub fn invoice_creating(&self) -> bool {
        self.invoice_creating.load(Ordering::Relaxed)
    }

    /// Handle message from the invoice issuer to send founds, return response for funds receiver.
    fn pay(&self, slate: &Slate) -> Result<Slate, Error> {
        let config = self.get_config();
        let args = InitTxArgs {
            src_acct_name: None,
            amount: slate.amount,
            minimum_confirmations: config.min_confirmations,
            selection_strategy_is_use_all: false,
            ..Default::default()
        };
        let r_inst = self.instance.as_ref().read();
        let instance = r_inst.clone().unwrap();
        let api = Owner::new(instance, None);
        let slate = api.process_invoice_tx(None, &slate, args)?;
        api.tx_lock_outputs(None, &slate)?;

        // Create Slatepack message response.
        let _ = self.create_slatepack_message(&slate)?;

        Ok(slate)
    }

    /// Create response to sender to receive funds.
    fn receive(&self, slate: &Slate) -> Result<Slate, Error> {
        let r_inst = self.instance.as_ref().read();
        let instance = r_inst.clone().unwrap();
        let api = Owner::new(instance, None);
        let mut slate = slate.clone();
        controller::foreign_single_use(api.wallet_inst.clone(), None, |api| {
            slate = api.receive_tx(&slate, Some(self.get_config().account.as_str()), None)?;
            Ok(())
        })?;

        // Create Slatepack message response.
        let _ = self.create_slatepack_message(&slate)?;

        Ok(slate)
    }

    /// Finalize transaction from provided message as sender or invoice issuer.
    fn finalize(&self, slate: &Slate) -> Result<Slate, Error> {
        self.on_tx_action(slate.id.to_string(), Some(WalletTransactionAction::Finalizing));

        let r_inst = self.instance.as_ref().read();
        let instance = r_inst.clone().unwrap();
        let api = Owner::new(instance, None);
        let slate = api.finalize_tx(None, slate)?;

        // Save Slatepack message to file.
        let _ = self.create_slatepack_message(&slate)?;

        // Clear tx action.
        self.on_tx_action(slate.id.to_string(), None);

        Ok(slate)
    }

    /// Post transaction to blockchain.
    fn post(&self, slate: &Slate) -> Result<(), Error> {
        self.on_tx_action(slate.id.to_string(), Some(WalletTransactionAction::Posting));

        let r_inst = self.instance.as_ref().read();
        let instance = r_inst.clone().unwrap();
        let api = Owner::new(instance, None);
        api.post_tx(None, slate, self.can_use_dandelion())?;

        // Clear tx action.
        self.on_tx_action(slate.id.to_string(), None);

        Ok(())
    }

    /// Cancel transaction.
    fn cancel(&self, tx: &WalletTransaction) -> Result<(), Error> {
        let id = tx.data.tx_slate_id.unwrap().to_string();
        self.on_tx_action(id.clone(), Some(WalletTransactionAction::Cancelling));

        let r_inst = self.instance.as_ref().read();
        let instance = r_inst.clone().unwrap();
        cancel_tx(instance, None, &None, Some(tx.data.id), None)?;

        // Clear tx action.
        self.on_tx_action(id, None);

        Ok(())
    }

    /// Update transaction action status.
    fn on_tx_action(&self, id: String, action: Option<WalletTransactionAction>) {
        let mut w_data = self.data.write();
        w_data.as_mut().unwrap().on_tx_action(id, action);
    }

    /// Update transaction action error status.
    fn on_tx_error(&self, id: String, err: Option<Error>) {
        let mut w_data = self.data.write();
        w_data.as_mut().unwrap().on_tx_error(id, err);
    }

    /// Save identifier of successful transaction after tasks completion to consume later.
    fn on_tx_result(&self, slate: &Slate) {
        let mut w_res = self.task_result_slate_id.write();
        *w_res = Some(slate.id.to_string());
    }

    /// Consume identifier of successful transaction task.
    pub fn consume_tx_task_result(&self) -> Option<String> {
        let res = {
            let r_res = self.task_result_slate_id.read();
            r_res.clone()
        };
        // Clear result.
        if res.is_some() {
            let mut w_res = self.task_result_slate_id.write();
            *w_res = None;
        }
        res
    }

    /// Get possible transaction confirmation height, .
    fn tx_height(&self, tx: &WalletTransaction) -> Result<Option<u64>, Error> {
        let mut tx_height = None;
        if tx.data.confirmed && tx.data.kernel_excess.is_some() {
            let r_inst = self.instance.as_ref().read();
            let instance = r_inst.clone().unwrap();
            let mut w_lock = instance.lock();
            let w = w_lock.lc_provider()?.wallet_inst()?;
            if let Ok(res) = w.w2n_client().get_kernel(
                tx.data.kernel_excess.as_ref().unwrap(),
                tx.data.kernel_lookup_min_height,
                None
            ) {
                tx_height = Some(match res {
                    None => 0,
                    Some((_, h, _)) => h
                });
            }
        } else if tx.broadcasting() {
            tx_height = match self.get_data() {
                None => None,
                Some(data) => Some(data.info.last_confirmed_height)
            };
        }
        Ok(tx_height)
    }

    /// Get transaction from database.
    fn get_tx(&self, tx_id: u32) -> Option<Slate> {
        let r_inst = self.instance.as_ref().read();
        let instance = r_inst.clone().unwrap();
        let api = Owner::new(instance, None);
        if let Ok(s) = api.get_stored_tx(None, Some(tx_id), None) {
            return s;
        }
        None
    }

    /// Change wallet password.
    pub fn change_password(&self, old: String, new: String) -> Result<(), Error> {
        let r_inst = self.instance.as_ref().read();
        let instance = r_inst.clone().unwrap();
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

    /// Deleting wallet database files.
    pub fn delete_db(&self, reopen: bool) {
        let wallet_delete = self.clone();
        // Close wallet if open.
        if self.is_open() {
            self.close();
        }
        thread::spawn(move || {
            // Wait wallet to be closed.
            if wallet_delete.is_open() {
                thread::sleep(Duration::from_millis(300));
            }
            // Remove wallet db files.
            let _ = fs::remove_dir_all(wallet_delete.get_config().get_db_path());
            // Start sync to close thread.
            wallet_delete.sync();
            wallet_delete.repair();
            // Mark wallet to reopen.
            wallet_delete.set_reopen(reopen);
        });
    }

    /// Get recovery phrase.
    pub fn get_recovery(&self, password: String) -> Result<ZeroingString, Error> {
        let r_inst = self.instance.as_ref().read();
        let instance = r_inst.clone().unwrap();
        let mut wallet_lock = instance.lock();
        let lc = wallet_lock.lc_provider().unwrap();
        lc.get_mnemonic(None, ZeroingString::from(password))
    }

    /// Close the wallet, delete its files and mark it as deleted.
    pub fn delete_wallet(&self) {
        if self.is_open() {
            self.close();
        }
        // Mark wallet as deleted.
        let wallet_delete = self.clone();
        wallet_delete.deleted.store(true, Ordering::Relaxed);

        thread::spawn(move || {
            // Wait wallet to be closed.
            if wallet_delete.is_open() {
                thread::sleep(Duration::from_millis(100));
            }
            // Remove wallet files.
            let _ = fs::remove_dir_all(wallet_delete.get_config().get_wallet_path());
            // Mark wallet as deleted.
            wallet_delete.deleted.store(true, Ordering::Relaxed);
            // Start sync to close thread.
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
fn start_sync(wallet: Wallet) -> Thread {
    // Start tasks thread.
    let (tx, rx) = mpsc::channel();
    {
        let mut w_tasks = wallet.tasks_sender.write();
        *w_tasks = Some(tx);
    }
    let wallet_thread = wallet.clone();
    thread::spawn(move ||  loop {
        let wallet_task = wallet_thread.clone();
        if let Ok(task) = rx.recv() {
            thread::spawn(move || {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(async {
                        handle_task(&wallet_task, task).await;
                    })});
        }
        if wallet_thread.is_closing() || !wallet_thread.is_open() {
            break;
        }
    });

    // Reset progress values.
    wallet.info_sync_progress.store(0, Ordering::Relaxed);
    wallet.repair_progress.store(0, Ordering::Relaxed);

    // To call on sync thread stop.
    let on_thread_stop = |wallet: Wallet| {
        // Clear thread instance.
        let mut thread_w = wallet.sync_thread.write();
        *thread_w = None;

        // Clear wallet info.
        let mut w_data = wallet.data.write();
        *w_data = None;

        // Clear syncing status.
        wallet.syncing.store(false, Ordering::Relaxed);
    };

    thread::spawn(move || loop {
        // Set syncing status.
        wallet.syncing.store(true, Ordering::Relaxed);

        // Close wallet on chain type change.
        if wallet.get_config().chain_type != AppConfig::chain_type() {
            wallet.close();
        }

        // Stop syncing if wallet was closed.
        if !wallet.is_open() || wallet.is_closing() {
            on_thread_stop(wallet);
            return;
        }

        // Check integrated node state.
        if wallet.get_current_connection() == ConnectionMethod::Integrated {
            let not_enabled = !Node::is_running() || Node::is_stopping();
            if not_enabled {
                // Reset loading progress.
                wallet.info_sync_progress.store(0, Ordering::Relaxed);
            }
            // Set an error when required integrated node is not enabled.
            wallet.set_sync_error(not_enabled);
            // Skip cycle when node sync is not finished.
            if !Node::is_running() || Node::get_sync_status() != Some(SyncStatus::NoSync) {
                thread::park_timeout(ATTEMPT_DELAY);
                continue;
            }
        }

        // Scan outputs if repair is needed or sync data if there is no error.
        if !wallet.sync_error() {
            if wallet.is_repairing() {
                repair_wallet(&wallet);
                // Stop sync if wallet was closed.
                if !wallet.is_open() || wallet.is_closing() {
                    on_thread_stop(wallet);
                    return;
                }
            }
            // Retrieve data from local database if current data is empty.
            if wallet.get_data().is_none() {
                sync_wallet_data(&wallet, false);
            }

            if wallet.is_open() && !wallet.is_closing() {
                // Start Foreign API listener if not running.
                let mut api_server_running = {
                    wallet.foreign_api_server.read().is_some()
                };
                if !api_server_running {
                    match start_api_server(&wallet) {
                        Ok(api_server) => {
                            let mut api_server_w = wallet.foreign_api_server.write();
                            *api_server_w = Some(api_server);
                            api_server_running = true;
                        }
                        Err(_) => {}
                    }
                }

                // Start unfailed Tor service if API server is running.
                let service_id = wallet.identifier();
                if wallet.auto_start_tor_listener() && api_server_running &&
                    !Tor::is_service_failed(&service_id) {
                    let r_foreign_api = wallet.foreign_api_server.read();
                    let api = r_foreign_api.as_ref().unwrap();
                    if let Ok(sec_key) = wallet.get_secret_key() {
                        Tor::start_service(api.1, sec_key, &wallet.identifier());
                    }
                }
            }

            // Sync wallet from node.
            sync_wallet_data(&wallet, true);
        }

        // Stop sync if wallet was closed.
        if !wallet.is_open() || wallet.is_closing() {
            on_thread_stop(wallet);
            return;
        }

        // Setup flag to check if sync was failed.
        let failed_sync = wallet.sync_error() || wallet.get_sync_attempts() != 0;

        // Clear syncing status.
        if !failed_sync {
            wallet.syncing.store(false, Ordering::Relaxed);
        }

        // Repeat after default or attempt delay if synchronization was not successful.
        let delay = if failed_sync {
            ATTEMPT_DELAY
        } else {
            SYNC_DELAY
        };
        thread::park_timeout(delay);
    }).thread().clone()
}

/// Handle wallet task.
async fn handle_task(w: &Wallet, t: WalletTask) {
    let send_tor = async |s: &Slate, r: &SlatepackAddress| {
        match w.send_tor(&s, r).await {
            Ok(s) => {
                match w.finalize(&s) {
                    Ok(s) => {
                        match w.post(&s) {
                            Ok(_) => {
                                sync_wallet_data(&w, false);
                                w.on_tx_result(&s);
                            }
                            Err(e) => {
                                w.on_tx_error(s.id.to_string(), Some(e));
                            }
                        }
                    }
                    Err(e) => {
                        w.on_tx_error(s.id.to_string(), Some(e));
                    }
                }
            }
            Err(e) => {
                w.on_tx_error(s.id.to_string(), Some(e));
                w.on_tx_result(&s);
            }
        }
    };
    match &t {
        WalletTask::OpenMessage(m) => {
            w.open_message(m);
        }
        WalletTask::Send(a, r) => {
            w.send_creating.store(true, Ordering::Relaxed);
            if let Ok(s) = w.send(*a, r.clone()) {
                sync_wallet_data(&w, false);
                if let Some(r) = r {
                    send_tor(&s, r).await;
                } else {
                    w.on_tx_result(&s);
                }
            }
            w.send_creating.store(false, Ordering::Relaxed);
        }
        WalletTask::SendTor(id, r) => {
            if let Some(s) = w.get_tx(*id) {
               send_tor(&s, r).await;
            }
        }
        WalletTask::Receive(a) => {
            w.invoice_creating.store(true, Ordering::Relaxed);
            if let Ok(s) = w.issue_invoice(*a) {
                sync_wallet_data(&w, false);
                w.on_tx_result(&s);
            }
            w.invoice_creating.store(false, Ordering::Relaxed);
        },
        WalletTask::Finalize(s, id) => {
            let slate = match s {
                None => &w.get_tx(*id).unwrap(),
                Some(s) => s
            };
            match w.finalize(slate) {
                Ok(s) => {
                    match w.post(&s) {
                        Ok(_) => {
                            sync_wallet_data(&w, false);
                        }
                        Err(e) => {
                            w.on_tx_error(slate.id.to_string(), Some(e));
                        }
                    }
                }
                Err(e) => {
                    w.on_tx_error(slate.id.to_string(), Some(e));
                }
            }
        }
        WalletTask::Post(s, id) => {
            let slate = match s {
                None => &w.get_tx(*id).unwrap(),
                Some(s) => s
            };
            match w.post(slate) {
                Ok(_) => {
                    sync_wallet_data(&w, false);
                }
                Err(e) => {
                    w.on_tx_error(slate.id.to_string(), Some(e));
                }
            }

        }
        WalletTask::Cancel(tx) => {
            match w.cancel(tx) {
                Ok(_) => {
                    sync_wallet_data(&w, false);
                }
                Err(e) => {
                    let id = tx.data.tx_slate_id.unwrap().to_string();
                    w.on_tx_error(id, Some(e));
                }
            }
        }
    };
}

/// Refresh [`WalletData`] from local base or node.
fn sync_wallet_data(wallet: &Wallet, from_node: bool) {
    // Update info sync progress at separate thread.
    let wallet_info = wallet.clone();
    let (info_tx, info_rx) = mpsc::channel::<StatusMessage>();
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
    let r_inst = wallet.instance.as_ref().read();
    if r_inst.is_some() {
        let instance = r_inst.clone().unwrap();
        if let Ok((_, info)) = retrieve_summary_info(
            instance.clone(),
            None,
            &Some(info_tx),
            from_node,
            config.min_confirmations
        ) {
            // Do not retrieve txs if wallet was closed or its first sync.
            if !wallet.is_open() || wallet.is_closing() ||
                (!from_node && info.last_confirmed_height == 0) {
                return;
            }

            // Setup accounts data.
            let last_height = info.last_confirmed_height;
            let spendable = if wallet.get_data().is_none() {
                None
            } else {
                Some(info.amount_currently_spendable)
            };
            update_accounts(wallet, last_height, spendable);

            if wallet.info_sync_progress() == 100 || !from_node {
                // Update wallet info.
                {
                    let mut w_data = wallet.data.write();
                    let txs = if w_data.is_some() {
                        w_data.clone().unwrap().txs
                    } else {
                        None
                    };
                    *w_data = Some(WalletData { info: info.clone(), txs });
                }

                // Update wallet transactions.
                if update_txs(wallet, instance.clone(), info).is_ok() {
                    wallet.reset_sync_attempts();
                    return;
                }
            }
        }
    }

    // Reset progress.
    wallet.info_sync_progress.store(0, Ordering::Relaxed);

    // Exit if wallet was closed or closing.
    if !wallet.is_open() || wallet.is_closing() {
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

/// Update wallet transactions.
fn update_txs(wallet: &Wallet, instance: WalletInstance, info: WalletInfo)
    -> Result<(), Error> {
    // Retrieve txs from local database.
    let txs_args = RetrieveTxQueryArgs {
        exclude_cancelled: Some(false),
        sort_field: Some(RetrieveTxQuerySortField::CreationTimestamp),
        sort_order: Some(RetrieveTxQuerySortOrder::Desc),
        ..Default::default()
    };
    let txs = retrieve_txs(instance, None, &None, false, None, None, Some(txs_args.clone()))?;

    // Exit if wallet was closed.
    if !wallet.is_open() || wallet.is_closing() {
        return Err(Error::GenericError("Wallet is not open".to_string()));
    }

    // Filter transactions for current account.
    let account_txs = txs.1.iter().map(|v| v.clone()).filter(|tx| {
        match wallet.get_parent_key_id() {
            Ok(key) => {
                tx.parent_key_id == key && tx.tx_slate_id.is_some()
            }
            Err(_) => {
                true
            }
        }
    }).collect::<Vec<TxLogEntry>>();

    let tx_height_store = TxHeightStore::new(wallet.get_config().get_extra_db_path());
    let data = wallet.get_data().unwrap();
    let data_txs = data.txs.unwrap_or(vec![]);
    let mut new_txs: Vec<WalletTransaction> = vec![];
    for tx in &account_txs {
        let mut height: Option<u64> = None;
        let mut broadcasting_height: Option<u64> = None;
        let mut action: Option<WalletTransactionAction> = None;
        let mut action_error: Option<Error> = None;
        for t in &data_txs {
            if t.data.id == tx.id {
                action = t.action.clone();
                action_error = t.action_error.clone();
                height = t.height;
                broadcasting_height = t.broadcasting_height;
                break;
            }
        }
        let mut new = WalletTransaction::new(tx.clone(),
                                             wallet,
                                             height,
                                             broadcasting_height,
                                             action,
                                             action_error);
        // Update Slate state for unconfirmed.
        let unconfirmed = !tx.confirmed && (tx.tx_type == TxLogEntryType::TxSent ||
            tx.tx_type == TxLogEntryType::TxReceived);
        if unconfirmed {
            new.update_slate_state(wallet);
        }

        // Setup initial tx heights.
        if height.is_none() && tx.confirmed {
            height = if let Some(height) = tx_height_store.read_tx_height(tx.id) {
                Some(height)
            } else {
                tx_height_store.delete_broadcasting_height(tx.id);
                let h = wallet.tx_height(&new)?;
                if let Some(h) = h {
                    tx_height_store.write_tx_height(tx.id, h);
                }
                h
            };
            new.height = height;
        } else if broadcasting_height.is_none() && new.broadcasting() {
            broadcasting_height = if let Some(h) = tx_height_store.read_broadcasting_height(tx.id) {
                Some(h)
            } else {
                let h = data.info.last_confirmed_height;
                tx_height_store.write_broadcasting_height(tx.id, h);
                Some(h)
            };
            new.broadcasting_height = Some(broadcasting_height.unwrap_or(0));
        }

        new_txs.push(new);
    }
    // Update wallet txs.
    let mut w_data = wallet.data.write();
    *w_data = Some(WalletData { info, txs: Some(new_txs) });
    Ok(())
}

/// Start Foreign API server to receive txs over transport and mining rewards.
fn start_api_server(wallet: &Wallet) -> Result<(ApiServer, u16), Error> {
    let host = "127.0.0.1";
    let port = wallet.get_config().api_port.unwrap_or(rand::rng().random_range(10000..30000));
    let free_port = (port..).find(|port| {
        return match TcpListener::bind((host, port.to_owned())) {
            Ok(_) => {
                let node_p2p_port = NodeConfig::get_p2p_port();
                let node_api_port = NodeConfig::get_api_ip_port().1;
                let free = port.to_string() != node_p2p_port && port.to_string() != node_api_port;
                if free {
                    let mut config = wallet.config.write();
                    config.api_port = Some(*port);
                    config.save();
                }
                free
            },
            Err(_) => false
        }
    }).unwrap();

    // Setup API server address.
    let api_addr = format!("{}:{}", host, free_port);

    // Start Foreign API server thread.
    let r_inst = wallet.instance.as_ref().read();
    let instance = r_inst.clone().unwrap();
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
fn update_accounts(wallet: &Wallet, height: u64, spendable: Option<u64>) {
    let current_account = wallet.get_config().account;
    if let Some(amount) = spendable {
        let mut accounts = wallet.accounts.read().clone();
        for a in accounts.iter_mut() {
            if a.label == current_account {
                a.spendable_amount = amount;
            }
        }
        // Save accounts data.
        let mut w_data = wallet.accounts.write();
        *w_data = accounts;
    } else {
        let r_inst = wallet.instance.as_ref().read();
        let instance = r_inst.clone().unwrap();
        let mut api = Owner::new(instance, None);
        let _ = controller::owner_single_use(None, None, Some(&mut api), |api, m| {
            let mut accounts = vec![];
            for a in api.accounts(m)? {
                api.set_active_account(m, a.label.as_str())?;
                // Calculate account balance.
                if let Some(spendable_amount) = wallet.account_balance(height, api, m) {
                    accounts.push(WalletAccount {
                        spendable_amount,
                        label: a.label,
                        path: a.path.to_bip_32_string(),
                    });
                }
            }
            accounts.sort_by_key(|w| w.label != current_account);

            // Save accounts data.
            let mut w_data = wallet.accounts.write();
            *w_data = accounts;

            // Set current active account from config.
            api.set_active_account(m, current_account.as_str())?;

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
    let r_inst = wallet.instance.as_ref().read();
    let instance = r_inst.clone().unwrap();
    let api = Owner::new(instance, Some(info_tx));
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