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
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU8, Ordering};
use std::thread;
use std::time::Duration;

use grin_chain::SyncStatus;
use grin_core::global;
use grin_core::global::ChainTypes;
use grin_keychain::{ExtKeychain, Keychain};
use grin_util::secp::SecretKey;
use grin_util::types::ZeroingString;
use grin_wallet_impls::{DefaultLCProvider, DefaultWalletImpl, HTTPNodeClient};
use grin_wallet_libwallet::{Error, NodeClient, StatusMessage, WalletBackend, WalletInfo, WalletInst, WalletLCProvider};
use grin_wallet_libwallet::api_impl::owner::retrieve_summary_info;
use parking_lot::Mutex;

use crate::AppConfig;
use crate::node::{Node, NodeConfig};
use crate::wallet::{ConnectionsConfig, ExternalConnection, WalletConfig};
use crate::wallet::types::{ConnectionMethod, WalletInstance};

/// [`Wallet`] list wrapper.
pub struct Wallets {
    /// List of wallets.
    pub(crate) list: Vec<Wallet>,
    /// Selected [`Wallet`] identifier.
    selected_id: Option<i64>,
}

impl Default for Wallets {
    fn default() -> Self {
        Self {
            list: Self::init(AppConfig::chain_type()),
            selected_id: None
        }
    }
}

impl Wallets {
    /// Initialize [`Wallet`] list from base directory for provided [`ChainType`].
    fn init(chain_type: ChainTypes) -> Vec<Wallet> {
        let mut wallets = Vec::new();
        let wallets_dir = WalletConfig::get_base_path(chain_type);
        // Load wallets from base directory.
        for dir in wallets_dir.read_dir().unwrap() {
            let wallet_dir = dir.unwrap().path();
            if wallet_dir.is_dir() {
                let wallet = Wallet::init(wallet_dir);
                if let Some(w) = wallet {
                    wallets.push(w);
                }
            }
        }
        wallets
    }

    /// Reinitialize [`Wallet`] list for provided [`ChainTypes`].
    pub fn reinit(&mut self, chain_type: ChainTypes) {
        for w in self.list.iter_mut() {
            let _ = w.close();
        }
        self.list = Self::init(chain_type);
    }

    /// Add created [`Wallet`] to the list.
    pub fn add(&mut self, wallet: Wallet) {
        self.selected_id = Some(wallet.config.id);
        self.list.insert(0, wallet);
    }

    /// Select [`Wallet`] with provided identifier.
    pub fn select(&mut self, id: Option<i64>) {
        self.selected_id = id;
    }

    /// Check if [`Wallet`] is selected for provided identifier.
    pub fn is_selected(&self, id: i64) -> bool {
        return Some(id) == self.selected_id;
    }

    /// Check if selected [`Wallet`] is open.
    pub fn is_selected_open(&self) -> bool {
        for w in &self.list {
            if Some(w.config.id) == self.selected_id {
                return w.is_open()
            }
        }
        false
    }

    /// Open selected [`Wallet`].
    pub fn open_selected(&mut self, password: String) -> Result<(), Error> {
        for w in self.list.iter_mut() {
            if Some(w.config.id) == self.selected_id {
                return w.open(password.clone());
            }
        }
        Err(Error::GenericError("Wallet is not selected".to_string()))
    }
}

/// Contains wallet instance and handles wallet commands.
#[derive(Clone)]
pub struct Wallet {
    /// Wallet instance.
    instance: WalletInstance,
    /// Wallet configuration.
    pub config: WalletConfig,

    /// Identifier for launched wallet thread.
    thread_id: Arc<AtomicI64>,

    /// Flag to check if wallet is open.
    is_open: Arc<AtomicBool>,

    /// Error on wallet loading.
    loading_error: Arc<AtomicBool>,
    /// Loading progress in percents
    loading_progress: Arc<AtomicU8>,

    /// Wallet balance information.
    info: Arc<RwLock<Option<WalletInfo>>>
}

impl Wallet {
    /// Instantiate [`Wallet`] from provided [`WalletInstance`] and [`WalletConfig`].
    fn new(instance: WalletInstance, config: WalletConfig) -> Self {
        Self {
            instance,
            config,
            thread_id: Arc::new(AtomicI64::new(0)),
            is_open: Arc::from(AtomicBool::new(false)),
            loading_error: Arc::from(AtomicBool::new(false)),
            loading_progress: Arc::new(AtomicU8::new(0)),
            info: Arc::new(RwLock::new(None)),
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
        let instance = Self::create_wallet_instance(config.clone())?;
        let w = Wallet::new(instance, config);
        {
            let mut w_lock = w.instance.lock();
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
    fn init(data_path: PathBuf) -> Option<Wallet> {
        let wallet_config = WalletConfig::load(data_path.clone());
        if let Some(config) = wallet_config {
            if let Ok(instance) = Self::create_wallet_instance(config.clone()) {
                return Some(Wallet::new(instance, config));
            }
        }
        None
    }

    /// Reinitialize [`WalletInstance`] to apply new [`WalletConfig`].
    pub fn reinit(&mut self) -> Result<(), Error> {
        self.close()?;
        self.instance = Self::create_wallet_instance(self.config.clone())?;
        Ok(())
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

    /// Open the wallet and start commands handling and updating info at separate thread.
    pub fn open(&self, password: String) -> Result<(), Error> {
        let mut wallet_lock = self.instance.lock();
        let lc = wallet_lock.lc_provider()?;
        lc.close_wallet(None)?;

        // Open the wallet.
        match lc.open_wallet(None, ZeroingString::from(password), false, false) {
            Ok(keychain) => {
                self.is_open.store(true, Ordering::Relaxed);
                // Start info updating and commands handling.
                start_wallet(self.clone(), keychain);
            }
            Err(e) => return Err(e)
        }
        Ok(())
    }

    /// Get wallet thread identifier.
    fn get_thread_id(&self) -> i64 {
        self.thread_id.load(Ordering::Relaxed)
    }

    /// Refresh wallet thread identifier.
    fn new_thread_id(&self) -> i64 {
        let id = chrono::Utc::now().timestamp();
        self.thread_id.store(id, Ordering::Relaxed);
        id
    }

    /// Get wallet loading progress after opening.
    pub fn loading_progress(&self) -> u8 {
        self.loading_progress.load(Ordering::Relaxed)
    }

    /// Check if wallet had an error on loading.
    pub fn loading_error(&self) -> bool {
        self.loading_error.load(Ordering::Relaxed)
    }

    /// Set an error for wallet on loading.
    pub fn set_loading_error(&self, error: bool) {
        self.loading_error.store(error, Ordering::Relaxed);
    }

    /// Check if wallet was open.
    pub fn is_open(&self) -> bool {
        self.is_open.load(Ordering::Relaxed)
    }

    /// Close the wallet.
    pub fn close(&mut self) -> Result<(), Error> {
        if self.is_open() {
            let mut wallet_lock = self.instance.lock();
            let lc = wallet_lock.lc_provider()?;
            lc.close_wallet(None)?;
            // Clear wallet info.
            let mut w_info = self.info.write().unwrap();
            *w_info = None;
            // Mark wallet as not opened.
            self.is_open.store(false, Ordering::Relaxed);
        }
        Ok(())
    }

    /// Get wallet balance info.
    pub fn get_info(&self) -> Option<WalletInfo> {
        let r_info = self.info.read().unwrap();
        r_info.clone()
    }
}

/// Delay in seconds to update wallet info.
const INFO_UPDATE_DELAY: Duration = Duration::from_millis(20 * 1000);

/// Launch thread for commands handling and info updating after [`INFO_UPDATE_DELAY`].
fn start_wallet(mut wallet: Wallet, keychain_mask: Option<SecretKey>) {
    // Setup initial loading values.
    wallet.loading_progress.store(0, Ordering::Relaxed);
    wallet.set_loading_error(false);

    // Launch loop at separate thread to update wallet info.
    let thread_id = wallet.new_thread_id();
    thread::spawn(move || loop {
        // Stop updating if wallet was closed or thread changed.
        if !wallet.is_open() || thread_id != wallet.get_thread_id() {
            break;
        }

        // Setup error when required integrated node is not enabled or skip when it's not ready.
        if wallet.config.ext_conn_id.is_none() {
            if !Node::is_running() {
                wallet.set_loading_error(true);
            } else if Node::get_sync_status() != Some(SyncStatus::NoSync) {
                wallet.set_loading_error(false);
                thread::sleep(Duration::from_millis(1000));
                continue;
            }
        }

        // Update wallet info if it was no loading error after several attempts.
        if !wallet.loading_error() {
            let wallet_scan = wallet.clone();
            let (tx, rx) = mpsc::channel::<StatusMessage>();
            // Update loading progress at separate thread.
            thread::spawn(move || {
                while let Ok(m) = rx.recv() {
                    // Stop updating if wallet was closed or maximum.
                    if !wallet_scan.is_open() || wallet_scan.get_thread_id() != thread_id {
                        return;
                    }
                    match m {
                        StatusMessage::UpdatingOutputs(_) => {}
                        StatusMessage::UpdatingTransactions(_) => {}
                        StatusMessage::FullScanWarn(_) => {}
                        StatusMessage::Scanning(_, progress) => {
                            wallet_scan.loading_progress.store(progress, Ordering::Relaxed);
                        }
                        StatusMessage::ScanningComplete(_) => {
                            wallet_scan.loading_progress.store(100, Ordering::Relaxed);
                        }
                        StatusMessage::UpdateWarning(_) => {}
                    }
                }
            });

            // Retrieve wallet info.
            match retrieve_summary_info(
                wallet.instance.clone(),
                keychain_mask.as_ref(),
                &Some(tx),
                true,
                wallet.config.min_confirmations
            ) {
                Ok(info) => {
                    if wallet.loading_progress() != 100 {
                        wallet.set_loading_error(true);
                    } else {
                        let mut w_info = wallet.info.write().unwrap();
                        *w_info = Some(info.1);
                    }
                }
                Err(_) => {
                    wallet.set_loading_error(true);
                    wallet.loading_progress.store(0, Ordering::Relaxed);
                }
            }
        }

        // Stop updating if wallet was closed.
        if !wallet.is_open() || wallet.get_thread_id() != thread_id {
            break;
        }

        // Repeat after default delay or after 1 second if update was not complete.
        let delay = if wallet.loading_progress() == 100 && !wallet.loading_error() {
            INFO_UPDATE_DELAY
        } else {
            Duration::from_millis(1000)
        };
        thread::sleep(delay);
    });
}