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
use std::collections::BTreeSet;
use std::fs::File;
use std::io::Write;
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::sync::{Arc, mpsc, RwLock};
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU8, Ordering};
use std::thread::Thread;
use std::time::Duration;

use futures::channel::oneshot;
use grin_api::{ApiServer, Router};
use grin_chain::SyncStatus;
use grin_core::global;
use grin_keychain::{ExtKeychain, Keychain};
use grin_util::Mutex;
use grin_util::types::ZeroingString;
use grin_wallet_api::Owner;
use grin_wallet_controller::command::parse_slatepack;
use grin_wallet_controller::controller;
use grin_wallet_controller::controller::ForeignAPIHandlerV2;
use grin_wallet_impls::{DefaultLCProvider, DefaultWalletImpl, HTTPNodeClient};
use grin_wallet_libwallet::{AcctPathMapping, Error, NodeClient, StatusMessage, TxLogEntryType, WalletInst, WalletLCProvider};
use grin_wallet_libwallet::api_impl::owner::{cancel_tx, retrieve_summary_info, retrieve_txs};

use crate::node::{Node, NodeConfig};
use crate::wallet::{ConnectionsConfig, ExternalConnection, WalletConfig};
use crate::wallet::types::{ConnectionMethod, WalletData, WalletInstance};

/// Contains wallet instance, configuration and state, handles wallet commands.
#[derive(Clone)]
pub struct Wallet {
    /// Wallet configuration.
    pub config: WalletConfig,
    /// Wallet instance, initializing on wallet opening and clearing on wallet closing.
    instance: Option<WalletInstance>,
    /// [`WalletInstance`] external connection id applied after opening.
    instance_ext_conn_id: Arc<AtomicI64>,

    /// Wallet sync thread.
    sync_thread: Arc<RwLock<Option<Thread>>>,

    /// Foreign API server.
    foreign_api_server: Arc<RwLock<Option<ApiServer>>>,

    /// Flag to check if wallet reopening is needed.
    reopen: Arc<AtomicBool>,
    /// Flag to check if wallet is open.
    is_open: Arc<AtomicBool>,
    /// Flag to check if wallet is loading.
    closing: Arc<AtomicBool>,
    /// Flag to check if wallet was deleted to remove it from the list.
    deleted: Arc<AtomicBool>,

    /// Error on wallet loading.
    sync_error: Arc<AtomicBool>,
    /// Info loading progress in percents.
    info_sync_progress: Arc<AtomicU8>,
    /// Transactions loading progress in percents.
    txs_sync_progress: Arc<AtomicU8>,

    /// Wallet data.
    data: Arc<RwLock<Option<WalletData>>>,
    /// Attempts amount to update wallet data.
    sync_attempts: Arc<AtomicU8>,

    /// Flag to check if wallet repairing and restoring missing outputs is needed.
    repair_needed: Arc<AtomicBool>,
    /// Wallet repair progress in percents.
    repair_progress: Arc<AtomicU8>,

    /// Identifiers for transactions to cancel.
    cancel_txs: Arc<RwLock<BTreeSet<u32>>>
}

/// Default Foreign API server host.
const DEFAULT_FOREIGN_API_HOST: &str = "127.0.0.1";
/// Default Foreign API server port.
const DEFAULT_FOREIGN_API_PORT: u16 = 3421;

impl Wallet {
    /// Create new [`Wallet`] instance with provided [`WalletConfig`].
    fn new(config: WalletConfig) -> Self {
        Self {
            config,
            instance: None,
            instance_ext_conn_id: Arc::new(AtomicI64::new(0)),
            sync_thread: Arc::from(RwLock::new(None)),
            foreign_api_server: Arc::new(RwLock::new(None)),
            reopen: Arc::new(AtomicBool::new(false)),
            is_open: Arc::from(AtomicBool::new(false)),
            closing: Arc::new(AtomicBool::new(false)),
            deleted: Arc::new(AtomicBool::new(false)),
            sync_error: Arc::from(AtomicBool::new(false)),
            info_sync_progress: Arc::from(AtomicU8::new(0)),
            txs_sync_progress: Arc::from(AtomicU8::new(0)),
            data: Arc::from(RwLock::new(None)),
            sync_attempts: Arc::new(AtomicU8::new(0)),
            repair_needed: Arc::new(AtomicBool::new(false)),
            repair_progress: Arc::new(AtomicU8::new(0)),
            cancel_txs: Arc::new(RwLock::new(BTreeSet::new())),
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

    /// Open the wallet and start [`WalletData`] sync at separate thread.
    pub fn open(&mut self, password: String) -> Result<(), Error> {
        if self.is_open() {
            return Err(Error::GenericError("Already opened".to_string()));
        }

        // Create new wallet instance if sync thread was stopped or instance was not created.
        if self.sync_thread.write().unwrap().is_none() || self.instance.is_none() {
            let new_instance = Self::create_wallet_instance(self.config.clone())?;
            self.instance = Some(new_instance);
            self.instance_ext_conn_id.store(match self.config.ext_conn_id {
                None => 0,
                Some(conn_id) => conn_id
            }, Ordering::Relaxed);
        }

        // Open the wallet.
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
                let label = self.config.account.to_owned();
                wallet_inst.set_parent_key_id_by_name(label.as_str())?;

                // Start new synchronization thread or wake up existing one.
                let mut thread_w = self.sync_thread.write().unwrap();
                if thread_w.is_none() {
                    let thread = start_sync(self.clone());
                    *thread_w = Some(thread);
                } else {
                    println!("unfreeze thread");
                    thread_w.clone().unwrap().unpark();
                }
                self.is_open.store(true, Ordering::Relaxed);
            }
            Err(e) => {
                self.instance = None;
                return Err(e)
            }
        }
        Ok(())
    }

    /// Get external connection id applied to [`WalletInstance`]
    /// after opening if sync is running or take it from config.
    pub fn get_current_ext_conn_id(&self) -> Option<i64> {
        if self.sync_thread.read().unwrap().is_some() {
            let ext_conn_id = self.instance_ext_conn_id.load(Ordering::Relaxed);
            if ext_conn_id == 0 {
                None
            } else {
                Some(ext_conn_id)
            }
        } else {
            self.config.ext_conn_id
        }
    }

    /// Check if wallet was open.
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
        thread::spawn(move || {
            // Stop created API server.
            let api_server_exists = {
                wallet_close.foreign_api_server.read().unwrap().is_some()
            };
            if api_server_exists {
                let mut api_server_w = wallet_close.foreign_api_server.write().unwrap();
                api_server_w.as_mut().unwrap().stop();
                *api_server_w = None;
            }

            // Close the wallet.
            Self::close_wallet(&instance);

            // Mark wallet as not opened.
            wallet_close.closing.store(false, Ordering::Relaxed);
            wallet_close.is_open.store(false, Ordering::Relaxed);

            // Wake up thread to exit.
            wallet_close.refresh();
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
            Ok(())
        })
    }

    /// Set active account from provided label.
    pub fn set_active_account(&mut self, label: &String) -> Result<(), Error> {
        let instance = self.instance.clone().unwrap();
        let mut wallet_lock = instance.lock();
        let lc = wallet_lock.lc_provider()?;
        let wallet_inst = lc.wallet_inst()?;
        wallet_inst.set_parent_key_id_by_name(label.as_str())?;

        // Save account label into config.
        self.config.save_account(label);

        // Clear wallet info.
        let mut w_data = self.data.write().unwrap();
        *w_data = None;

        // Refresh wallet data.
        self.refresh();
        Ok(())
    }

    /// Get list of accounts for the wallet.
    pub fn accounts(&self) -> Vec<AcctPathMapping> {
        let mut api = Owner::new(self.instance.clone().unwrap(), None);
        let mut accounts = vec![];
        let _ = controller::owner_single_use(None, None, Some(&mut api), |api, m| {
            accounts = api.accounts(m)?;
            Ok(())
        });
        accounts
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

    /// Retry synchronization on error.
    pub fn retry_sync(&self) {
        self.set_sync_error(false);
    }

    /// Set an error for wallet on synchronization.
    fn set_sync_error(&self, error: bool) {
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
        let r_data = self.data.read().unwrap();
        r_data.clone()
    }

    /// Wake up wallet thread to refresh wallet info and update statuses.
    fn refresh(&self) {
        let thread_r = self.sync_thread.read().unwrap();
        if let Some(thread) = thread_r.as_ref() {
            thread.unpark();
        }
    }

    /// Receive transaction via Slatepack Message.
    pub fn receive(&self, message: String) -> Result<String, Error> {
        let mut api = Owner::new(self.instance.clone().unwrap(), None);
        match parse_slatepack(&mut api, None, None, Some(message.clone())) {
            Ok((mut slate, _)) => {
                controller::foreign_single_use(api.wallet_inst.clone(), None, |api| {
                    let account = self.config.clone().account;
                    slate = api.receive_tx(&slate, Some(account.as_str()), None)?;
                    Ok(())
                })?;
                let mut response = "".to_string();
                controller::owner_single_use(None, None, Some(&mut api), |api, m| {
                    response = api.create_slatepack_message(m, &slate, Some(0), vec![])?;
                    Ok(())
                })?;

                // Create a directory to which slatepack files will be output.
                let mut slatepack_dir = self.config.get_slatepacks_path();
                let slatepack_file_name = format!("{}.{}.slatepack", slate.id, slate.state);
                slatepack_dir.push(slatepack_file_name);

                // Write Slatepack response into the file.
                let mut output = File::create(slatepack_dir)?;
                output.write_all(response.as_bytes())?;
                output.sync_all()?;

                // Refresh wallet info.
                self.refresh();

                Ok(response)
            }
            Err(_) => {
                Err(Error::GenericError("Parsing error".to_string()))
            }
        }
    }

    pub fn send(&self) {

    }

    /// Cancel transaction.
    pub fn cancel(&mut self, id: u32) {
        // Set cancelling status.
        {
            let mut cancelling_w = self.cancel_txs.write().unwrap();
            cancelling_w.insert(id);
        }

        // Launch tx cancelling at separate thread.
        let wallet_cancel = self.clone();
        let instance = wallet_cancel.instance.clone().unwrap();
        thread::spawn(move || {
            let _ = cancel_tx(instance, None, &None, Some(id), None);
            // Refresh wallet info to update statuses.
            wallet_cancel.refresh();
        });
    }

    /// Check if transaction is cancelling.
    pub fn is_cancelling(&self, id: &u32) -> bool {
        let cancelling_r = self.cancel_txs.read().unwrap();
        cancelling_r.contains(id)
    }

    pub fn finalize(&self) {

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
        self.refresh();
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
            let mut wallet_lock = instance.lock();
            let _ = wallet_lock.lc_provider().unwrap();
            let _ = fs::remove_dir_all(wallet_delete.config.get_data_path());

            // Mark wallet as not opened and deleted.
            wallet_delete.closing.store(false, Ordering::Relaxed);
            wallet_delete.is_open.store(false, Ordering::Relaxed);
            wallet_delete.deleted.store(true, Ordering::Relaxed);

            // Wake up thread to exit.
            wallet_delete.refresh();
        });
    }

    /// Check if wallet was deleted to remove it from list.
    pub fn is_deleted(&self) -> bool {
        self.deleted.load(Ordering::Relaxed)
    }
}

/// Delay in seconds to sync [`WalletData`] (60 seconds as average block time).
const SYNC_DELAY: Duration = Duration::from_millis(60 * 1000);

/// Number of attempts to sync [`WalletData`] before setting an error.
const SYNC_ATTEMPTS: u8 = 10;

/// Launch thread to sync wallet data from node.
fn start_sync(mut wallet: Wallet) -> Thread {
    // Reset progress values.
    wallet.info_sync_progress.store(0, Ordering::Relaxed);
    wallet.txs_sync_progress.store(0, Ordering::Relaxed);
    wallet.repair_progress.store(0, Ordering::Relaxed);

    println!("create new thread");
    thread::spawn(move || loop {
        println!("start new cycle");
        // Stop syncing if wallet was closed.
        if !wallet.is_open() {
            println!("finishing thread at start");
            // Clear thread instance.
            let mut thread_w = wallet.sync_thread.write().unwrap();
            *thread_w = None;

            // Clear wallet info.
            let mut w_data = wallet.data.write().unwrap();
            *w_data = None;
            println!("finish at start complete");
            return;
        }

        // Set an error when required integrated node is not enabled
        // and skip cycle when node sync is not finished.
        if wallet.get_current_ext_conn_id().is_none() {
            wallet.set_sync_error(!Node::is_running() || Node::is_stopping());
            if !Node::is_running() || Node::get_sync_status() != Some(SyncStatus::NoSync) {
                println!("integrated node wait");
                thread::park_timeout(Duration::from_millis(1000));
                continue;
            }
        }

        // Start Foreign API listener if API server was not created.
        let api_server_exists = {
            wallet.foreign_api_server.read().unwrap().is_some()
        };
        if !api_server_exists {
            match start_api_server(&mut wallet) {
                Ok(api_server) => {
                    let mut api_server_w = wallet.foreign_api_server.write().unwrap();
                    *api_server_w = Some(api_server);
                }
                Err(_) => {}
            }
        }

        // Scan outputs if repair is needed or sync data if there is no error.
        if !wallet.sync_error() {
            if wallet.is_repairing() {
                scan_wallet(&wallet)
            } else {
                sync_wallet_data(&wallet);
            }
        }

        // Stop sync if wallet was closed.
        if !wallet.is_open() {
            println!("finishing thread after updating");
            // Clear thread instance.
            let mut thread_w = wallet.sync_thread.write().unwrap();
            *thread_w = None;

            // Clear wallet info.
            let mut w_data = wallet.data.write().unwrap();
            *w_data = None;
            println!("finishing after updating complete");
            return;
        }

        // Repeat after default delay or after 1 second if sync was not success.
        let delay = if wallet.sync_error()
            || wallet.get_sync_attempts() != 0 {
            Duration::from_millis(1000)
        } else {
            SYNC_DELAY
        };
        println!("park for {}", delay.as_millis());
        thread::park_timeout(delay);
    }).thread().clone()
}

/// Start Foreign API server to accept txs via Tor and receive mining rewards from Stratum server.
fn start_api_server(wallet: &mut Wallet) -> Result<ApiServer, Error> {
    // Find free port.
    let free_port = (DEFAULT_FOREIGN_API_PORT..).find(|port| {
        return match TcpListener::bind((DEFAULT_FOREIGN_API_HOST, port.to_owned())) {
            Ok(_) => {
                let node_p2p_port = NodeConfig::get_p2p_port();
                let node_api_port = NodeConfig::get_api_ip_port().1;
                port.to_string() != node_p2p_port && port.to_string() != node_api_port
            },
            Err(_) => false
        }
    }).unwrap();

    // Setup API server address.
    let api_addr = format!("{}:{}", DEFAULT_FOREIGN_API_HOST, free_port);

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
    println!("Starting HTTP Foreign listener API server at {}.", api_addr);
    let socket_addr: SocketAddr = api_addr.parse().unwrap();
    let _ = apis.start(socket_addr, router, None, api_chan)
        .map_err(|_| Error::GenericError("API thread failed to start".to_string()))?;

    println!("HTTP Foreign listener started.");
    Ok(apis)
}

/// Retrieve [`WalletData`] from node.
fn sync_wallet_data(wallet: &Wallet) {
    println!("SYNC start, attempts: {}", wallet.get_sync_attempts());

    let wallet_info = wallet.clone();
    let (info_tx, info_rx) = mpsc::channel::<StatusMessage>();
    // Update info sync progress at separate thread.
    thread::spawn(move || {
        while let Ok(m) = info_rx.recv() {
            println!("SYNC INFO MESSAGE");
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

    // Retrieve wallet info.
    if let Some(instance) = &wallet.instance {
        match retrieve_summary_info(
            instance.clone(),
            None,
            &Some(info_tx),
            true,
            wallet.config.min_confirmations
        ) {
            Ok(info) => {
                // Do not retrieve txs if wallet was closed.
                if !wallet.is_open() {
                    return;
                }
                // Retrieve txs if retrieving info was success.
                if wallet.info_sync_progress() == 100 {
                    let wallet_txs = wallet.clone();
                    let (txs_tx, txs_rx) = mpsc::channel::<StatusMessage>();
                    // Update txs sync progress at separate thread.
                    thread::spawn(move || {
                        while let Ok(m) = txs_rx.recv() {
                            println!("SYNC TXS MESSAGE");
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

                    // Retrieve txs.
                    match retrieve_txs(
                        instance.clone(),
                        None,
                        &Some(txs_tx),
                        true,
                        None,
                        None,
                        None
                    ) {
                        Ok(txs) => {
                            // Do not sync data if wallet was closed.
                            if !wallet.is_open() {
                                return;
                            }
                            // Save data if loading was completed.
                            if wallet.txs_sync_progress() == 100 {
                                // Reset attempts.
                                wallet.reset_sync_attempts();

                                // Setup transactions.
                                let mut txs = txs.1;
                                // Sort txs by creation date.
                                txs.sort_by_key(|tx| -tx.creation_ts.timestamp());
                                // Update txs statuses.
                                for tx in &txs {
                                    if tx.tx_type == TxLogEntryType::TxSentCancelled
                                        || tx.tx_type == TxLogEntryType::TxReceivedCancelled {
                                        // Remove cancelling status.
                                        let mut cancel_w = wallet.cancel_txs.write().unwrap();
                                        cancel_w.remove(&tx.id);
                                    }
                                }

                                // Update wallet data.
                                let mut w_data = wallet.data.write().unwrap();
                                *w_data = Some(WalletData { info: info.1, txs });
                                return;
                            }
                        }
                        Err(e) => {
                            println!("error on retrieve_txs {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                println!("error on retrieve_summary_info {}", e);
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

    println!("SYNC cycle finished, attempts: {}", wallet.get_sync_attempts());

    // Set an error if maximum number of attempts was reached.
    if wallet.get_sync_attempts() >= SYNC_ATTEMPTS {
        wallet.reset_sync_attempts();
        wallet.set_sync_error(true);
    }
}

/// Scan wallet's outputs, repairing and restoring missing outputs if required.
fn scan_wallet(wallet: &Wallet) {
    println!("repair the wallet");
    let (info_tx, info_rx) = mpsc::channel::<StatusMessage>();
    // Update scan progress at separate thread.
    let wallet_scan = wallet.clone();
    thread::spawn(move || {
        while let Ok(m) = info_rx.recv() {
            println!("REPAIR WALLET MESSAGE");
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
            println!("repair was complete");
            // Set sync error if scanning was not complete and wallet is open.
            if wallet.is_open() && wallet.repair_progress.load(Ordering::Relaxed) != 100 {
                wallet.set_sync_error(true);
            } else {
                wallet.repair_needed.store(false, Ordering::Relaxed);
            }
        }
        Err(e) => {
            println!("error on repair {}", e);
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