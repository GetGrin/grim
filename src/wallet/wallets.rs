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

use std::path::PathBuf;
use std::sync::{Arc, mpsc, RwLock};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::thread;
use std::thread::Thread;
use std::time::Duration;

use grin_chain::SyncStatus;
use grin_core::global;
use grin_keychain::{ExtKeychain, Keychain};
use grin_util::secp::SecretKey;
use grin_util::types::ZeroingString;
use grin_wallet_impls::{DefaultLCProvider, DefaultWalletImpl, HTTPNodeClient};
use grin_wallet_libwallet::{Error, NodeClient, StatusMessage, WalletInst, WalletLCProvider};
use grin_wallet_libwallet::api_impl::owner::{retrieve_summary_info, retrieve_txs};
use parking_lot::Mutex;

use crate::node::{Node, NodeConfig};
use crate::wallet::{ConnectionsConfig, ExternalConnection, WalletConfig};
use crate::wallet::types::{ConnectionMethod, WalletData, WalletInstance};

/// Contains wallet instance, configuration and state, handles wallet commands.
#[derive(Clone)]
pub struct Wallet {
    /// Wallet instance, initializing on wallet opening and clearing on wallet closing.
    instance: Option<WalletInstance>,
    /// Wallet configuration.
    pub config: WalletConfig,

    /// Wallet updating thread.
    thread: Arc<RwLock<Option<Thread>>>,

    /// Flag to check if wallet reopening is needed.
    reopen: Arc<AtomicBool>,
    /// Flag to check if wallet is open.
    is_open: Arc<AtomicBool>,
    /// Flag to check if wallet is loading.
    closing: Arc<AtomicBool>,

    /// Error on wallet loading.
    load_error: Arc<AtomicBool>,
    /// Info loading progress in percents
    info_load_progress: Arc<AtomicU8>,
    /// Transactions loading progress in percents
    txs_load_progress: Arc<AtomicU8>,

    /// Wallet data.
    data: Arc<RwLock<Option<WalletData>>>,
    /// Attempts amount to update wallet data.
    data_update_attempts: Arc<AtomicU8>
}

impl Wallet {
    /// Create new [`Wallet`] instance with provided [`WalletConfig`].
    fn new(config: WalletConfig) -> Self {
        Self {
            instance: None,
            config,
            thread: Arc::from(RwLock::new(None)),
            reopen: Arc::new(AtomicBool::new(false)),
            is_open: Arc::from(AtomicBool::new(false)),
            closing: Arc::new(AtomicBool::new(false)),
            load_error: Arc::from(AtomicBool::new(false)),
            info_load_progress: Arc::from(AtomicU8::new(0)),
            txs_load_progress: Arc::from(AtomicU8::new(0)),
            data: Arc::from(RwLock::new(None)),
            data_update_attempts: Arc::new(AtomicU8::new(0))
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

    /// Open the wallet and start update the data at separate thread.
    pub fn open(&mut self, password: String) -> Result<(), Error> {
        if self.is_open() {
            return Err(Error::GenericError("Already opened".to_string()));
        }

        // Create new wallet instance.
        let instance = Self::create_wallet_instance(self.config.clone())?;
        self.instance = Some(instance.clone());

        // Open the wallet.
        let mut wallet_lock = instance.lock();
        let lc = wallet_lock.lc_provider()?;
        match lc.open_wallet(None, ZeroingString::from(password), false, false) {
            Ok(keychain) => {
                // Start data updating if thread was not launched.
                let mut thread_w = self.thread.write().unwrap();
                if thread_w.is_none() {
                    println!("create new thread");
                    let thread = start_wallet(self.clone(), keychain.clone());
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

    /// Set wallet reopen status.
    pub fn set_reopen(&self, reopen: bool) {
        self.reopen.store(reopen, Ordering::Relaxed);
    }

    /// Check if wallet reopen is needed.
    pub fn reopen_needed(&self) -> bool {
        self.reopen.load(Ordering::Relaxed)
    }

    /// Get wallet transactions loading progress.
    pub fn txs_load_progress(&self) -> u8 {
        self.txs_load_progress.load(Ordering::Relaxed)
    }

    /// Get wallet info loading progress.
    pub fn info_load_progress(&self) -> u8 {
        self.info_load_progress.load(Ordering::Relaxed)
    }

    /// Check if wallet had an error on loading.
    pub fn load_error(&self) -> bool {
        self.load_error.load(Ordering::Relaxed)
    }

    /// Set an error for wallet on loading.
    pub fn set_load_error(&self, error: bool) {
        self.load_error.store(error, Ordering::Relaxed);
    }

    /// Get wallet data update attempts.
    fn get_data_update_attempts(&self) -> u8 {
        self.data_update_attempts.load(Ordering::Relaxed)
    }

    /// Increment wallet data update attempts.
    fn increment_data_update_attempts(&self) {
        let mut attempts = self.get_data_update_attempts();
        attempts += 1;
        self.data_update_attempts.store(attempts, Ordering::Relaxed);
    }

    /// Reset wallet data update attempts.
    fn reset_data_update_attempts(&self) {
        self.data_update_attempts.store(0, Ordering::Relaxed);
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
        let mut wallet_close = self.clone();
        let instance = wallet_close.instance.clone().unwrap();
        thread::spawn(move || {
            // Close the wallet.
            let mut wallet_lock = instance.lock();
            let lc = wallet_lock.lc_provider().unwrap();
            let _ = lc.close_wallet(None);
            wallet_close.instance = None;

            // Clear wallet info.
            let mut w_data = wallet_close.data.write().unwrap();
            *w_data = None;

            // Reset wallet loading values.
            wallet_close.info_load_progress.store(0, Ordering::Relaxed);
            wallet_close.txs_load_progress.store(0, Ordering::Relaxed);
            wallet_close.set_load_error(false);
            wallet_close.reset_data_update_attempts();

            // Mark wallet as not opened.
            wallet_close.closing.store(false, Ordering::Relaxed);
            wallet_close.is_open.store(false, Ordering::Relaxed);

            // Wake up wallet thread.
            let thread_r = wallet_close.thread.read().unwrap();
            if let Some(thread) = thread_r.as_ref() {
                thread.unpark();
            }
        });
    }

    /// Get wallet data.
    pub fn get_data(&self) -> Option<WalletData> {
        let r_data = self.data.read().unwrap();
        r_data.clone()
    }
}

/// Delay in seconds to update wallet data every minute as average block time.
const DATA_UPDATE_DELAY: Duration = Duration::from_millis(60 * 1000);

/// Number of attempts to update data after wallet opening before setting an error.
const DATA_UPDATE_ATTEMPTS: u8 = 10;

/// Launch thread to update wallet data.
fn start_wallet(wallet: Wallet, keychain: Option<SecretKey>) -> Thread {
    let wallet_update = wallet.clone();
    thread::spawn(move || loop {
        println!("start new cycle");
        // Stop updating if wallet was closed.
        if !wallet_update.is_open() {
            println!("finishing thread at start");
            let mut thread_w = wallet_update.thread.write().unwrap();
            *thread_w = None;
            println!("finish at start complete");
            return;
        }

        // Set an error when required integrated node is not enabled and
        // skip next cycle of update when node sync is not finished.
        if wallet_update.config.ext_conn_id.is_none() {
            wallet_update.set_load_error(!Node::is_running() || Node::is_stopping());
            if !Node::is_running() || Node::get_sync_status() != Some(SyncStatus::NoSync) {
                println!("integrated node wait");
                thread::park_timeout(Duration::from_millis(1000));
                continue;
            }
        }

        // Update wallet data if there is no error.
        if !wallet_update.load_error() {
            update_wallet_data(&wallet_update, keychain.clone());
        }

        // Stop updating if wallet was closed.
        if !wallet_update.is_open() {
            println!("finishing thread after updating");
            let mut thread_w = wallet_update.thread.write().unwrap();
            *thread_w = None;
            println!("finishing after updating complete");
            return;
        }

        // Repeat after default delay or after 1 second if update was not success.
        let delay = if wallet_update.load_error()
            || wallet_update.get_data_update_attempts() != 0 {
            Duration::from_millis(1000)
        } else {
            DATA_UPDATE_DELAY
        };
        println!("park for {}", delay.as_millis());
        thread::park_timeout(delay);
    }).thread().clone()
}

/// Handle [`WalletCommand::UpdateData`] command to update [`WalletData`].
fn update_wallet_data(wallet: &Wallet, keychain: Option<SecretKey>) {
    println!("UPDATE start, attempts: {}", wallet.get_data_update_attempts());

    let wallet_scan = wallet.clone();
    let (info_tx, info_rx) = mpsc::channel::<StatusMessage>();
    // Update info loading progress at separate thread.
    thread::spawn(move || {
        while let Ok(m) = info_rx.recv() {
            println!("UPDATE INFO MESSAGE");
            match m {
                StatusMessage::UpdatingOutputs(_) => {}
                StatusMessage::UpdatingTransactions(_) => {}
                StatusMessage::FullScanWarn(_) => {}
                StatusMessage::Scanning(_, progress) => {
                    wallet_scan.info_load_progress.store(progress, Ordering::Relaxed);
                }
                StatusMessage::ScanningComplete(_) => {
                    wallet_scan.info_load_progress.store(100, Ordering::Relaxed);
                }
                StatusMessage::UpdateWarning(_) => {}
            }
        }
    });

    // Retrieve wallet info.
    if let Some(instance) = &wallet.instance {
        match retrieve_summary_info(
            instance.clone(),
            keychain.as_ref(),
            &Some(info_tx),
            true,
            wallet.config.min_confirmations
        ) {
            Ok(info) => {
                // Do not retrieve txs if wallet was closed.
                if !wallet.is_open() {
                    println!("UPDATE stop at retrieve_summary_info");
                    return;
                }

                // Add attempt if scanning was not complete
                // or set an error on initial request.
                if wallet.info_load_progress() != 100 {
                    println!("UPDATE retrieve_summary_info was not completed");
                    if wallet.get_data().is_none() {
                        wallet.set_load_error(true);
                    } else {
                        wallet.increment_data_update_attempts();
                    }
                } else {
                    println!("UPDATE before retrieve_txs");

                    let wallet_txs = wallet.clone();
                    let (txs_tx, txs_rx) = mpsc::channel::<StatusMessage>();
                    // Update txs loading progress at separate thread.
                    thread::spawn(move || {
                        while let Ok(m) = txs_rx.recv() {
                            println!("UPDATE TXS MESSAGE");
                            match m {
                                StatusMessage::UpdatingOutputs(_) => {}
                                StatusMessage::UpdatingTransactions(_) => {}
                                StatusMessage::FullScanWarn(_) => {}
                                StatusMessage::Scanning(_, progress) => {
                                    wallet_txs.txs_load_progress.store(progress, Ordering::Relaxed);
                                }
                                StatusMessage::ScanningComplete(_) => {
                                    wallet_txs.txs_load_progress.store(100, Ordering::Relaxed);
                                }
                                StatusMessage::UpdateWarning(_) => {}
                            }
                        }
                    });

                    // Retrieve txs.
                    match retrieve_txs(
                        instance.clone(),
                        keychain.as_ref(),
                        &Some(txs_tx),
                        true,
                        None,
                        None,
                        None
                    ) {
                        Ok(txs) => {
                            // Do not update data if wallet was closed.
                            if !wallet.is_open() {
                                return;
                            }

                            // Add attempt if retrieving was not complete
                            // or set an error on initial request.
                            if wallet.txs_load_progress() != 100 {
                                if wallet.get_data().is_none() {
                                    wallet.set_load_error(true);
                                } else {
                                    wallet.increment_data_update_attempts();
                                }
                            } else {
                                // Set wallet data.
                                let mut w_data = wallet.data.write().unwrap();
                                *w_data = Some(WalletData { info: info.1, txs: txs.1 });

                                // Reset attempts.
                                wallet.reset_data_update_attempts();
                            }
                        }
                        Err(e) => {
                            println!("error on retrieve_txs {}", e);
                            // Increment attempts value in case of error.
                            wallet.increment_data_update_attempts();
                        }
                    }
                }
            }
            Err(e) => {
                println!("error on retrieve_summary_info {}", e);
                // Increment attempts value in case of error.
                wallet.increment_data_update_attempts();
            }
        }
    }

    // Reset progress values.
    wallet.info_load_progress.store(0, Ordering::Relaxed);
    wallet.txs_load_progress.store(0, Ordering::Relaxed);

    println!("UPDATE finish, attempts: {}", wallet.get_data_update_attempts());

    // Set an error if maximum number of attempts was reached.
    if wallet.get_data_update_attempts() >= DATA_UPDATE_ATTEMPTS {
        wallet.reset_data_update_attempts();
        wallet.set_load_error(true);
    }
}