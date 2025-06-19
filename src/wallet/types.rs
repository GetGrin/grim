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

use std::sync::Arc;

use grin_keychain::ExtKeychain;
use grin_util::Mutex;
use grin_wallet_impls::{DefaultLCProvider, HTTPNodeClient};
use grin_wallet_libwallet::{Error, Slate, SlateState, SlatepackAddress, TxLogEntry, TxLogEntryType, WalletInfo, WalletInst};
use grin_wallet_util::OnionV3Address;
use serde_derive::{Deserialize, Serialize};

/// Mnemonic phrase word.
#[derive(Clone)]
pub struct PhraseWord {
    /// Word text.
    pub text: String,
    /// Flag to check if word is valid.
    pub valid: bool,
}

/// Mnemonic phrase setup mode.
#[derive(PartialEq, Clone)]
pub enum PhraseMode {
    /// Generate new mnemonic phrase.
    Generate,
    /// Import existing mnemonic phrase.
    Import
}

/// Mnemonic phrase size based on entropy.
#[derive(PartialEq, Clone)]
pub enum PhraseSize { Words12, Words15, Words18, Words21, Words24 }

impl PhraseSize {
    pub const VALUES: [PhraseSize; 5] = [
        PhraseSize::Words12,
        PhraseSize::Words15,
        PhraseSize::Words18,
        PhraseSize::Words21,
        PhraseSize::Words24
    ];

    /// Get entropy value.
    pub fn value(&self) -> usize {
        match *self {
            PhraseSize::Words12 => 12,
            PhraseSize::Words15 => 15,
            PhraseSize::Words18 => 18,
            PhraseSize::Words21 => 21,
            PhraseSize::Words24 => 24
        }
    }

    /// Get entropy size for current phrase size.
    pub fn entropy_size(&self) -> usize {
        match *self {
            PhraseSize::Words12 => 16,
            PhraseSize::Words15 => 20,
            PhraseSize::Words18 => 24,
            PhraseSize::Words21 => 28,
            PhraseSize::Words24 => 32
        }
    }

    /// Get phrase type for entropy size.
    pub fn type_for_value(count: usize) -> Option<PhraseSize> {
        if Self::is_correct_count(count) {
            match count {
                12 => {
                    Some(PhraseSize::Words12)
                }
                15 => {
                    Some(PhraseSize::Words15)
                }
                18 => {
                    Some(PhraseSize::Words18)
                }
                21 => {
                    Some(PhraseSize::Words21)
                }
                24 => {
                    Some(PhraseSize::Words24)
                }
                _ => {
                    None
                }
            }
        } else {
            None
        }
    }

    /// Check if correct entropy size was provided.
    pub fn is_correct_count(count: usize) -> bool {
        count == 12 || count == 15 || count == 18 || count == 21 || count == 24
    }
}

/// Wallet connection method.
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum ConnectionMethod {
    /// Integrated node.
    Integrated,
    /// External node, contains connection identifier and URL.
    External(i64, String)
}

/// Wallet instance type.
pub type WalletInstance = Arc<
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

/// Wallet account data.
#[derive(Clone)]
pub struct WalletAccount {
    /// Spendable balance amount.
    pub spendable_amount: u64,
    /// Account label.
    pub label: String,
    /// Account BIP32 derivation path.
    pub path: String
}

/// Wallet balance and transactions data.
#[derive(Clone)]
pub struct WalletData {
    /// Balance data for current account.
    pub info: WalletInfo,
    /// Transactions data.
    pub txs: Option<Vec<WalletTransaction>>
}

impl WalletData {
    /// Update transaction action status.
    pub fn on_tx_action(&mut self, id: String, action: Option<WalletTransactionAction>) {
        if self.txs.is_none() {
            return;
        }
        for tx in self.txs.as_mut().unwrap() {
            if let Some(slate_id) = tx.data.tx_slate_id {
                if slate_id.to_string() == id {
                    tx.action = action;
                    tx.action_error = None;
                    break;
                }
            }
        }
    }

    /// Update transaction action error status.
    pub fn on_tx_error(&mut self, id: String, err: Option<Error>) {
        if self.txs.is_none() {
            return;
        }
        for tx in self.txs.as_mut().unwrap() {
            if let Some(slate_id) = tx.data.tx_slate_id {
                if slate_id.to_string() == id {
                    tx.action_error = err;
                    break;
                }
            }
        }
    }

    /// Get transaction by slate identifier.
    pub fn tx_by_slate_id(&self, id: String) -> Option<WalletTransaction> {
        if self.txs.is_none() {
            return None;
        }
        for tx in self.txs.as_ref().unwrap() {
            if let Some(slate_id) = tx.data.tx_slate_id {
                if slate_id.to_string() == id {
                    return Some(tx.clone());
                }
            }
        }
        None
    }
}

/// Wallet transaction action.
#[derive(Clone, PartialEq)]
pub enum WalletTransactionAction {
    Cancelling, Finalizing, Posting, SendingTor
}

/// Wallet transaction data.
#[derive(Clone)]
pub struct WalletTransaction {
    /// Information from database.
    pub data: TxLogEntry,
    /// State of transaction Slate.
    pub state: SlateState,
    /// Calculated transaction amount between debited and credited amount.
    pub amount: u64,
    /// Block height where tx was included.
    pub height: Option<u64>,

    /// Action on transaction.
    pub action: Option<WalletTransactionAction>,
    /// Action result error.
    pub action_error: Option<Error>
}

impl WalletTransaction {
    /// Check if transactions can be finalized after receiving response.
    pub fn can_finalize(&self) -> bool {
        !self.cancelling() && !self.data.confirmed &&
            (!self.sending_tor() || self.action_error.is_some()) &&
            (self.data.tx_type == TxLogEntryType::TxSent ||
                self.data.tx_type == TxLogEntryType::TxReceived) &&
            (self.state == SlateState::Invoice1 || self.state == SlateState::Standard1)
    }

    /// Check if transaction was finalized.
    pub fn finalized(&self) -> bool {
        (self.data.tx_type == TxLogEntryType::TxSent ||
            self.data.tx_type == TxLogEntryType::TxReceived) &&
        self.state == SlateState::Invoice3 || self.state == SlateState::Standard3
    }

    /// Check if transaction is sending over Tor.
    pub fn sending_tor(&self) -> bool {
        if let Some(a) = self.action.as_ref() {
            return a == &WalletTransactionAction::SendingTor;
        }
        false
    }

    /// Check if transaction is cancelling.
    pub fn cancelling(&self) -> bool {
        if let Some(a) = self.action.as_ref() {
            return a == &WalletTransactionAction::Cancelling;
        }
        false
    }

    /// Check if transaction is posting.
    pub fn posting(&self) -> bool {
        if let Some(a) = self.action.as_ref() {
            return a == &WalletTransactionAction::Posting;
        }
        false
    }

    /// Check if transaction can be cancelled.
    pub fn can_cancel(&self) -> bool {
        !self.cancelling() && !self.data.confirmed &&
            (!self.sending_tor() || self.action_error.is_some()) &&
            self.data.tx_type != TxLogEntryType::TxReceivedCancelled &&
            self.data.tx_type != TxLogEntryType::TxSentCancelled
    }

    /// Check if transaction is finalizing.
    pub fn finalizing(&self) -> bool {
        if let Some(a) = self.action.as_ref() {
            return a == &WalletTransactionAction::Finalizing;
        }
        false
    }

    /// Get receiver address if payment proof was created.
    pub fn receiver(&self) -> Option<SlatepackAddress> {
        if let Some(proof) = &self.data.payment_proof {
            let onion_addr = OnionV3Address::from_bytes(proof.receiver_address.to_bytes());
            if let Ok(addr) = SlatepackAddress::try_from(onion_addr) {
                return Some(addr);
            }
        }
        None
    }
}

/// Task for the wallet.
#[derive(Clone)]
pub enum WalletTask {
    /// Open Slatepack message parsing result and making an action.
    OpenMessage(String),
    /// Create request to send.
    /// * amount
    /// * receiver
    Send(u64, Option<SlatepackAddress>),
    /// Resend request over Tor.
    /// * local tx id
    /// * receiver
    SendTor(u32, SlatepackAddress),
    /// Invoice creation.
    /// * amount
    Receive(u64),
    /// Transaction finalization.
    /// * tx
    /// * local tx id
    Finalize(Option<Slate>, u32),
    /// Post transaction to blockchain.
    /// * tx
    /// * local tx id
    Post(Option<Slate>, u32),
    /// Cancel transaction.
    /// * tx
    Cancel(WalletTransaction),
}