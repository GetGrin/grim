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

use std::{cmp, thread};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use grin_core::global;
use grin_core::global::ChainTypes;
use grin_keychain::{ExtKeychain, Identifier, Keychain};
use grin_util::types::ZeroingString;
use grin_wallet_api::{Foreign, ForeignCheckMiddlewareFn, Owner};
use grin_wallet_impls::{DefaultLCProvider, DefaultWalletImpl, HTTPNodeClient};
use grin_wallet_libwallet::{Error, NodeClient, NodeVersionInfo, OutputStatus, scan, Slate, slate_versions, SlatepackArmor, Slatepacker, SlatepackerArgs, TxLogEntry, wallet_lock, WalletBackend, WalletInfo, WalletInst, WalletLCProvider};
use log::debug;
use parking_lot::Mutex;
use uuid::Uuid;

use crate::AppConfig;
use crate::node::NodeConfig;
use crate::wallet::{ConnectionsConfig, WalletConfig};
use crate::wallet::selection::lock_tx_context;
use crate::wallet::tx::{add_inputs_to_slate, new_tx_slate};
use crate::wallet::updater::{cancel_tx, refresh_output_state, retrieve_txs};

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
    /// Initialize wallets from base directory for provided [`ChainType`].
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

    /// Reinitialize wallets for provided [`ChainTypes`].
    pub fn reinit(&mut self, chain_type: ChainTypes) {
        self.list = Self::init(chain_type);
    }

    /// Add created [`Wallet`] to the list.
    pub fn add(&mut self, wallet: Wallet) {
        self.selected_id = Some(wallet.config.id);
        self.list.insert(0, wallet);
    }

    /// Select wallet with provided identifier.
    pub fn select(&mut self, id: Option<i64>) {
        self.selected_id = id;
    }

    /// Check if wallet is selected for provided identifier.
    pub fn is_selected(&self, id: i64) -> bool {
        return Some(id) == self.selected_id;
    }

    /// Check if selected wallet is open.
    pub fn is_selected_open(&self) -> bool {
        for w in &self.list {
            if Some(w.config.id) == self.selected_id {
                return w.is_open()
            }
        }
        false
    }

    /// Open selected wallet.
    pub fn open_selected(&mut self, password: String) -> Result<(), Error> {
        for w in self.list.iter_mut() {
            if Some(w.config.id) == self.selected_id {
                return w.open(password);
            }
        }
        Err(Error::GenericError("Wallet is not selected".to_string()))
    }

    /// Load the wallet by scanning available outputs at separate thread.
    pub fn load(w: &mut Wallet) {
        if !w.is_open() {
            return;
        }
        let mut wallet = w.clone();
        thread::spawn(move || {
            // Get pmmr range output indexes.
            match wallet.pmmr_range() {
                Ok((mut lowest_index, highest_index)) => {
                    println!("pmmr_range {} {}", lowest_index, highest_index);
                    let mut from_index = lowest_index;
                    loop {
                        // Scan outputs for last retrieved index.
                        println!("scan_outputs {} {}", from_index, highest_index);
                        match wallet.scan_outputs(from_index, highest_index) {
                            Ok(last_index) => {
                                println!("last_index {}", last_index);
                                if lowest_index == 0 {
                                    lowest_index = last_index;
                                }
                                if last_index == highest_index {
                                    wallet.loading_progress = 100;
                                    break;
                                } else {
                                    from_index = last_index;
                                }

                                // Update loading progress.
                                let range = highest_index - lowest_index;
                                let progress = last_index - lowest_index;
                                wallet.loading_progress = cmp::min(
                                    (progress / range) as u8 * 100,
                                    99
                                );
                                println!("progress {}", wallet.loading_progress);
                            }
                            Err(e) => {
                                wallet.loading_error = Some(e);
                                break;
                            }
                        }
                    }
                    wallet.is_loaded.store(true, Ordering::Relaxed);
                }
                Err(e) => {
                    wallet.loading_error = Some(e);
                }
            }
        });
    }
}

/// Contains wallet instance and config.
#[derive(Clone)]
pub struct Wallet {
    /// Wallet instance.
    instance: WalletInstance,

    /// Wallet configuration.
    pub(crate) config: WalletConfig,

    /// Flag to check if wallet is open.
    is_open: Arc<AtomicBool>,

    /// Flag to check if wallet is loaded and ready to use.
    is_loaded: Arc<AtomicBool>,
    /// Error on wallet loading.
    loading_error: Option<Error>,
    /// Loading progress in percents
    loading_progress: u8,
}

/// Wallet instance type.
type WalletInstance = Arc<
    Mutex<
        Box<
            dyn WalletInst<
                'static,
                DefaultLCProvider<'static, HTTPNodeClient, ExtKeychain>,
                HTTPNodeClient,
                ExtKeychain,
            >,
        >,
    >,
>;

impl Wallet {
    /// Create wallet from provided instance and config.
    fn new(instance: WalletInstance, config: WalletConfig) -> Self {
        Self {
            instance,
            config,
            is_loaded: Arc::new(AtomicBool::new(false)),
            is_open: Arc::new(AtomicBool::new(false)),
            loading_error: None,
            loading_progress: 0,
        }
    }

    /// Create new wallet.
    pub fn create(
        name: String,
        password: String,
        mnemonic: String,
        external_node_url: Option<String>
    ) -> Result<Wallet, Error> {
        let config = WalletConfig::create(name, external_node_url);
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

    /// Initialize wallet from provided data path.
    fn init(data_path: PathBuf) -> Option<Wallet> {
        let wallet_config = WalletConfig::load(data_path.clone());
        if let Some(config) = wallet_config {
            if let Ok(instance) = Self::create_wallet_instance(config.clone()) {
                return Some(Wallet::new(instance, config));
            }
        }
        None
    }

    /// Reinitialize wallet instance to apply new config e.g. on change connection settings.
    pub fn reinit(&mut self) -> Result<(), Error> {
        self.close()?;
        self.instance = Self::create_wallet_instance(self.config.clone())?;
        Ok(())
    }

    /// Create wallet instance from provided config.
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
        let (node_api_url, node_secret) = if let Some(url) = &config.external_node_url {
            (url.to_owned(), ConnectionsConfig::get_external_connection_secret(url.to_owned()))
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

    /// Instantiate wallet from provided node client and config.
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

    /// Open wallet instance.
    pub fn open(&self, password: String) -> Result<(), Error> {
        let mut wallet_lock = self.instance.lock();
        let lc = wallet_lock.lc_provider()?;
        lc.close_wallet(None)?;
        lc.open_wallet(None, ZeroingString::from(password), false, false)?;
        self.is_open.store(true, Ordering::Relaxed);
        Ok(())
    }

    /// Check if wallet is open.
    pub fn is_open(&self) -> bool {
        self.is_open.load(Ordering::Relaxed)
    }

    /// Close wallet.
    pub fn close(&mut self) -> Result<(), Error> {
        if self.is_open() {
            let mut wallet_lock = self.instance.lock();
            let lc = wallet_lock.lc_provider()?;
            lc.close_wallet(None)?;
            self.is_open.store(false, Ordering::Relaxed);
        }
        Ok(())
    }

    /// Scan wallet outputs to check/repair the wallet.
    fn scan_outputs(
        &self,
        last_retrieved_index: u64,
        highest_index: u64
    ) -> Result<u64, Error> {
        let wallet = self.instance.clone();
        let info = scan(
            wallet.clone(),
            None,
            false,
            last_retrieved_index,
            highest_index,
            &None,
        )?;
        let result = info.last_pmmr_index;

        let parent_key_id = {
            wallet_lock!(wallet.clone(), w);
            w.parent_key_id().clone()
        };
        {
            wallet_lock!(wallet, w);
            let mut batch = w.batch(None)?;
            batch.save_last_confirmed_height(&parent_key_id, info.height)?;
            batch.commit()?;
        };
        Ok(result)
    }

    /// Get pmmr indices representing the outputs for the wallet.
    fn pmmr_range(&self) -> Result<(u64, u64), Error> {
        wallet_lock!(self.instance.clone(), w);
        let pmmr_range = w.w2n_client().height_range_to_pmmr_indices(0, None)?;
        Ok(pmmr_range)
    }

    /// Create transaction.
    pub fn tx_create(
        &self,
        amount: u64,
        minimum_confirmations: u64,
        selection_strategy_is_use_all: bool,
    ) -> Result<(Vec<TxLogEntry>, String), Error> {
        let parent_key_id = {
            wallet_lock!(self.instance, w);
            w.parent_key_id().clone()
        };

        let slate = {
            wallet_lock!(self.instance, w);
            let mut slate = new_tx_slate(&mut **w, amount, false, 2, false, None)?;
            let height = w.w2n_client().get_chain_tip()?.0;

            let context = add_inputs_to_slate(
                &mut **w,
                None,
                &mut slate,
                height,
                minimum_confirmations,
                500,
                1,
                selection_strategy_is_use_all,
                &parent_key_id,
                true,
                false,
                false,
            )?;

            {
                let mut batch = w.batch(None)?;
                batch.save_private_context(slate.id.as_bytes(), &context)?;
                batch.commit()?;
            }

            lock_tx_context(&mut **w, None, &slate, height, &context, None)?;
            slate.compact()?;
            slate
        };

        let packer = Slatepacker::new(SlatepackerArgs {
            sender: None, // sender
            recipients: vec![],
            dec_key: None,
        });
        let slatepack = packer.create_slatepack(&slate)?;
        let api = Owner::new(self.instance.clone(), None);
        let txs = api.retrieve_txs(None, false, None, Some(slate.id), None)?;
        let result = (
            txs.1,
            SlatepackArmor::encode(&slatepack)?,
        );
        Ok(result)
    }

    /// Callback to check slate compatibility at current node.
    fn check_middleware(
        name: ForeignCheckMiddlewareFn,
        node_version_info: Option<NodeVersionInfo>,
        slate: Option<&Slate>,
    ) -> Result<(), Error> {
        match name {
            ForeignCheckMiddlewareFn::BuildCoinbase => Ok(()),
            _ => {
                let mut bhv = 3;
                if let Some(n) = node_version_info {
                    bhv = n.block_header_version;
                }
                if let Some(s) = slate {
                    if bhv > 4
                        && s.version_info.block_header_version
                        < slate_versions::GRIN_BLOCK_HEADER_VERSION
                    {
                        Err(Error::Compatibility(
                            "Incoming Slate is not compatible with this wallet. \
						 Please upgrade the node or use a different one."
                                .into(),
                        ))?;
                    }
                }
                Ok(())
            }
        }
    }

    /// Receive transaction.
    pub fn tx_receive(
        &self,
        account: &str,
        slate_armored: &str
    ) -> Result<(Vec<TxLogEntry>, String), Error> {
        let foreign_api =
            Foreign::new(self.instance.clone(), None, Some(Self::check_middleware), false);
        let owner_api = Owner::new(self.instance.clone(), None);

        let mut slate =
            owner_api.slate_from_slatepack_message(None, slate_armored.to_owned(), vec![0])?;
        let slatepack =
            owner_api.decode_slatepack_message(None, slate_armored.to_owned(), vec![0])?;

        let _ret_address = slatepack.sender;

        slate = foreign_api.receive_tx(&slate, Some(&account), None)?;
        let txs = owner_api.retrieve_txs(None, false, None, Some(slate.id), None)?;
        let packer = Slatepacker::new(SlatepackerArgs {
            sender: None, // sender
            recipients: vec![],
            dec_key: None,
        });
        let slatepack = packer.create_slatepack(&slate)?;
        let result = (
            txs.1,
            SlatepackArmor::encode(&slatepack)?,
        );
        Ok(result)
    }

    /// Cancel transaction.
    pub fn tx_cancel(&self, id: u32) -> Result<String, Error> {
        wallet_lock!(self.instance, w);
        let parent_key_id = w.parent_key_id();
        cancel_tx(&mut **w, None, &parent_key_id, Some(id), None)?;
        Ok("".to_owned())
    }

    /// Get transaction info.
    pub fn get_tx(&self, tx_slate_id: &str) -> Result<(bool, Vec<TxLogEntry>), Error> {
        let api = Owner::new(self.instance.clone(), None);
        let uuid = Uuid::parse_str(tx_slate_id).unwrap();
        let txs = api.retrieve_txs(None, true, None, Some(uuid), None)?;
        Ok(txs)
    }

    /// Finalize transaction.
    pub fn tx_finalize(&self, slate_armored: &str) -> Result<(bool, Vec<TxLogEntry>), Error> {
        let owner_api = Owner::new(self.instance.clone(), None);
        let mut slate =
            owner_api.slate_from_slatepack_message(None, slate_armored.to_owned(), vec![0])?;
        let slatepack =
            owner_api.decode_slatepack_message(None, slate_armored.to_owned(), vec![0])?;

        let _ret_address = slatepack.sender;

        slate = owner_api.finalize_tx(None, &slate)?;
        let txs = owner_api.retrieve_txs(None, false, None, Some(slate.id), None)?;
        Ok(txs)
    }

    /// Post transaction to node for broadcasting.
    pub fn tx_post(&self, tx_slate_id: &str) -> Result<(), Error> {
        let api = Owner::new(self.instance.clone(), None);
        let tx_uuid = Uuid::parse_str(tx_slate_id).unwrap();
        let (_, txs) = api.retrieve_txs(None, true, None, Some(tx_uuid.clone()), None)?;
        if txs[0].confirmed {
            return Err(Error::GenericError(format!(
                "Transaction with id {} is already confirmed. Not posting.",
                tx_slate_id
            )));
        }
        let stored_tx = api.get_stored_tx(None, None, Some(&tx_uuid))?;
        match stored_tx {
            Some(stored_tx) => {
                api.post_tx(None, &stored_tx, true)?;
                Ok(())
            }
            None => Err(Error::GenericError(format!(
                "Transaction with id {} does not have transaction data. Not posting.",
                tx_slate_id
            ))),
        }
    }

    /// Get transactions and base wallet info.
    pub fn get_txs_info(
        &self,
        minimum_confirmations: u64
    ) -> Result<(bool, Vec<TxLogEntry>, WalletInfo), Error> {
        let refreshed = Self::update_state(self.instance.clone()).unwrap_or(false);
        let wallet_info = {
            wallet_lock!(self.instance, w);
            let parent_key_id = w.parent_key_id();
            Self::get_info(&mut **w, &parent_key_id, minimum_confirmations)?
        };
        let api = Owner::new(self.instance.clone(), None);

        let txs = api.retrieve_txs(None, false, None, None, None)?;
        Ok((refreshed, txs.1, wallet_info))
    }

    /// Update wallet instance state.
    fn update_state<'a, L, C, K>(
        wallet_inst: Arc<Mutex<Box<dyn WalletInst<'a, L, C, K>>>>,
    ) -> Result<bool, Error>
        where
            L: WalletLCProvider<'a, C, K>,
            C: NodeClient + 'a,
            K: Keychain + 'a,
    {
        let parent_key_id = {
            wallet_lock!(wallet_inst, w);
            w.parent_key_id().clone()
        };
        let mut client = {
            wallet_lock!(wallet_inst, w);
            w.w2n_client().clone()
        };
        let tip = client.get_chain_tip()?;

        // Step1: Update outputs and transactions purely based on UTXO state.
        {
            wallet_lock!(wallet_inst, w);
            if !match refresh_output_state(&mut **w, None, tip.0, &parent_key_id, true) {
                Ok(_) => true,
                Err(_) => false,
            } {
                // We are unable to contact the node.
                return Ok(false);
            }
        }

        let mut txs = {
            wallet_lock!(wallet_inst, w);
            retrieve_txs(&mut **w, None, None, None, Some(&parent_key_id), true)?
        };

        for tx in txs.iter_mut() {
            // Step 2: Cancel any transactions with an expired TTL.
            if let Some(e) = tx.ttl_cutoff_height {
                if tip.0 >= e {
                    wallet_lock!(wallet_inst, w);
                    let parent_key_id = w.parent_key_id();
                    cancel_tx(&mut **w, None, &parent_key_id, Some(tx.id), None)?;
                    continue;
                }
            }
            // Step 3: Update outstanding transactions with no change outputs by kernel.
            if tx.confirmed {
                continue;
            }
            if tx.amount_debited != 0 && tx.amount_credited != 0 {
                continue;
            }
            if let Some(e) = tx.kernel_excess {
                let res = client.get_kernel(&e, tx.kernel_lookup_min_height, Some(tip.0));
                let kernel = match res {
                    Ok(k) => k,
                    Err(_) => return Ok(false),
                };
                if let Some(k) = kernel {
                    debug!("Kernel Retrieved: {:?}", k);
                    wallet_lock!(wallet_inst, w);
                    let mut batch = w.batch(None)?;
                    tx.confirmed = true;
                    tx.update_confirmation_ts();
                    batch.save_tx_log_entry(tx.clone(), &parent_key_id)?;
                    batch.commit()?;
                }
            }
        }

        return Ok(true);
    }

    /// Get summary info about the wallet.
    fn get_info<'a, T: ?Sized, C, K>(
        wallet: &mut T,
        parent_key_id: &Identifier,
        minimum_confirmations: u64,
    ) -> Result<WalletInfo, Error>
        where
            T: WalletBackend<'a, C, K>,
            C: NodeClient + 'a,
            K: Keychain + 'a,
    {
        let current_height = wallet.last_confirmed_height()?;
        let outputs = wallet
            .iter()
            .filter(|out| out.root_key_id == *parent_key_id);

        let mut unspent_total = 0;
        let mut immature_total = 0;
        let mut awaiting_finalization_total = 0;
        let mut unconfirmed_total = 0;
        let mut locked_total = 0;
        let mut reverted_total = 0;

        for out in outputs {
            match out.status {
                OutputStatus::Unspent => {
                    if out.is_coinbase && out.lock_height > current_height {
                        immature_total += out.value;
                    } else if out.num_confirmations(current_height) < minimum_confirmations {
                        // Treat anything less than minimum confirmations as "unconfirmed".
                        unconfirmed_total += out.value;
                    } else {
                        unspent_total += out.value;
                    }
                }
                OutputStatus::Unconfirmed => {
                    // We ignore unconfirmed coinbase outputs completely.
                    if !out.is_coinbase {
                        if minimum_confirmations == 0 {
                            unconfirmed_total += out.value;
                        } else {
                            awaiting_finalization_total += out.value;
                        }
                    }
                }
                OutputStatus::Locked => {
                    locked_total += out.value;
                }
                OutputStatus::Reverted => reverted_total += out.value,
                OutputStatus::Spent => {}
            }
        }

        Ok(WalletInfo {
            last_confirmed_height: current_height,
            minimum_confirmations,
            total: unspent_total + unconfirmed_total + immature_total,
            amount_awaiting_finalization: awaiting_finalization_total,
            amount_awaiting_confirmation: unconfirmed_total,
            amount_immature: immature_total,
            amount_locked: locked_total,
            amount_currently_spendable: unspent_total,
            amount_reverted: reverted_total,
        })
    }
}