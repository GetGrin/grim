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

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use grin_core::global;
use grin_core::global::ChainTypes;
use grin_keychain::{ExtKeychain, Identifier, Keychain};
use grin_util::types::ZeroingString;
use grin_wallet_api::{Foreign, ForeignCheckMiddlewareFn, Owner};
use grin_wallet_impls::{DefaultLCProvider, DefaultWalletImpl, HTTPNodeClient};
use grin_wallet_libwallet::{Error, NodeClient, NodeVersionInfo, OutputStatus, Slate, slate_versions, SlatepackArmor, Slatepacker, SlatepackerArgs, TxLogEntry, wallet_lock, WalletBackend, WalletInfo, WalletInst, WalletLCProvider};
use grin_wallet_libwallet::Error::GenericError;
use lazy_static::lazy_static;
use log::debug;
use parking_lot::Mutex;
use uuid::Uuid;

use crate::{AppConfig, Settings};
use crate::node::NodeConfig;
use crate::wallet::selection::lock_tx_context;
use crate::wallet::tx::{add_inputs_to_slate, new_tx_slate};
use crate::wallet::updater::{cancel_tx, refresh_output_state, retrieve_txs};
use crate::wallet::WalletConfig;

lazy_static! {
    /// Global wallets state.
    static ref WALLETS_STATE: Arc<RwLock<Wallets>> = Arc::new(RwLock::new(Wallets::init()));
}

/// Manages [`Wallet`] list and state.
pub struct Wallets {
    /// List of wallets.
    list: Vec<Wallet>,
    /// Selected [`Wallet`] identifier.
    selected_id: Option<i64>,
    /// Identifiers of opened wallets.
    opened_ids: BTreeSet<i64>
}

impl Wallets {
    /// Base wallets directory name.
    pub const BASE_DIR_NAME: &'static str = "wallets";

    /// Initialize manager by loading list of wallets into state.
    fn init() -> Self {
        Self {
            list: Self::load_wallets(&AppConfig::chain_type()),
            selected_id: None,
            opened_ids: BTreeSet::default()
        }
    }

    /// Create new wallet and add it to state.
    pub fn create_wallet(
        name: String,
        password: String,
        mnemonic: String,
        external_node_url: Option<String>
    )-> Result<(), Error> {
        let wallet = Wallet::create(name, password, mnemonic, external_node_url)?;
        let mut w_state = WALLETS_STATE.write().unwrap();
        w_state.list.push(wallet);
        Ok(())
    }

    /// Load wallets for provided [`ChainType`].
    fn load_wallets(chain_type: &ChainTypes) -> Vec<Wallet> {
        let mut wallets = Vec::new();
        let wallets_dir = Self::get_base_path(chain_type);
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

    /// Get list of wallets.
    pub fn list() -> Vec<Wallet> {
        let r_state = WALLETS_STATE.read().unwrap();
        r_state.list.clone()
    }

    /// Select [`Wallet`] with provided identifier.
    pub fn select(id: Option<i64>) {
        let mut w_state = WALLETS_STATE.write().unwrap();
        w_state.selected_id = id;
    }

    /// Get selected [`Wallet`] identifier.
    pub fn selected_id() -> Option<i64> {
        let r_state = WALLETS_STATE.read().unwrap();
        r_state.selected_id
    }

    /// Open [`Wallet`] with provided identifier and password.
    pub fn open(id: i64, password: String) -> Result<(), Error> {
        let list = Self::list();
        let mut w_state = WALLETS_STATE.write().unwrap();
        for mut w in list {
            if w.config.id == id {
                w.open(password)?;
                break;
            }
        }
        w_state.opened_ids.insert(id);
        Ok(())
    }

    /// Close [`Wallet`] with provided identifier.
    pub fn close(id: i64) -> Result<(), Error> {
        let list = Self::list();
        let mut w_state = WALLETS_STATE.write().unwrap();
        for mut w in list {
            if w.config.id == id {
                w.close()?;
                break;
            }
        }
        w_state.opened_ids.remove(&id);
        Ok(())
    }

    /// Check if [`Wallet`] with provided identifier was open.
    pub fn is_open(id: i64) -> bool {
        let r_state = WALLETS_STATE.read().unwrap();
        r_state.opened_ids.contains(&id)
    }

    /// Get wallets base directory path for provided [`ChainTypes`].
    pub fn get_base_path(chain_type: &ChainTypes) -> PathBuf {
        let mut wallets_path = Settings::get_base_path(Some(chain_type.shortname()));
        wallets_path.push(Self::BASE_DIR_NAME);
        // Create wallets base directory if it doesn't exist.
        if !wallets_path.exists() {
            let _ = fs::create_dir_all(wallets_path.clone());
        }
        wallets_path
    }

    /// Reload list of wallets for provided [`ChainTypes`].
    pub fn reload(chain_type: &ChainTypes) {
        let wallets = Self::load_wallets(chain_type);
        let mut w_state = WALLETS_STATE.write().unwrap();
        w_state.selected_id = None;
        w_state.opened_ids = BTreeSet::default();
        w_state.list = wallets;
    }
}

/// Wallet instance and config wrapper.
#[derive(Clone)]
pub struct Wallet {
    /// Wallet instance.
    instance: WalletInstance,

    /// Wallet data path.
    path: String,
    /// Wallet configuration.
    pub(crate) config: WalletConfig,
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
    /// Create new wallet, make it open and selected.
    fn create(
        name: String,
        password: String,
        mnemonic: String,
        external_node_url: Option<String>
    ) -> Result<Wallet, Error> {
        let config = WalletConfig::create(name, external_node_url);
        let wallet = Self::create_wallet_instance(config.clone())?;
        let w = Wallet {
            instance: wallet,
            path: config.get_data_path(),
            config,
        };

        {
            let mut w_lock = w.instance.lock();
            let p = w_lock.lc_provider()?;

            // Create wallet.
            p.create_wallet(None,
                            Some(ZeroingString::from(mnemonic.clone())),
                            mnemonic.len(),
                            ZeroingString::from(password.clone()),
                            false,
            )?;

            // Open wallet.
            p.open_wallet(None, ZeroingString::from(password), false, false)?;
        }

        Ok(w)
    }

    /// Initialize wallet from provided data path.
    fn init(data_path: PathBuf) -> Option<Wallet> {
        let wallet_config = WalletConfig::load(data_path.clone());
        if let Some(config) = wallet_config {
            let path = data_path.to_str().unwrap().to_string();
            if let Ok(instance) = Self::create_wallet_instance(config.clone()) {
                return Some(Self { instance, path, config });
            }
        }
        None
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
            (url.to_owned(), None)
        } else {
            (NodeConfig::get_api_address(), NodeConfig::get_api_secret())
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

    /// Open wallet.
    fn open(&mut self, password: String) -> Result<(), Error> {
        let mut wallet_lock = self.instance.lock();
        let lc = wallet_lock.lc_provider()?;
        lc.open_wallet(None, ZeroingString::from(password), false, false)?;
        Ok(())
    }

    /// Close wallet.
    fn close(&mut self) -> Result<(), Error> {
        let mut wallet_lock = self.instance.lock();
        let lc = wallet_lock.lc_provider()?;
        lc.close_wallet(None)?;
        Ok(())
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
            return Err(Error::from(GenericError(format!(
                "Transaction with id {} is already confirmed. Not posting.",
                tx_slate_id
            ))));
        }
        let stored_tx = api.get_stored_tx(None, None, Some(&tx_uuid))?;
        match stored_tx {
            Some(stored_tx) => {
                api.post_tx(None, &stored_tx, true)?;
                Ok(())
            }
            None => Err(Error::from(GenericError(format!(
                "Transaction with id {} does not have transaction data. Not posting.",
                tx_slate_id
            )))),
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