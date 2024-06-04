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
use grin_wallet_libwallet::{TxLogEntry, TxLogEntryType, WalletInfo, WalletInst};

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
#[derive(PartialEq)]
pub enum ConnectionMethod {
    /// Integrated node.
    Integrated,
    /// External node, contains connection identifier.
    External(i64)
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

/// Wallet transaction data.
#[derive(Clone)]
pub struct WalletTransaction {
    /// Transaction information.
    pub data: TxLogEntry,
    /// Calculated transaction amount between debited and credited amount.
    pub amount: u64,
    /// Flag to check if transaction is cancelling.
    pub cancelling: bool,
    /// Flag to check if transaction is posting after finalization.
    pub posting: bool,
    /// Flag to check if transaction can be finalized based on Slatepack message state.
    pub can_finalize: bool,
    /// Block height when tx was confirmed.
    pub conf_height: Option<u64>,
    /// Block height when tx was reposted.
    pub repost_height: Option<u64>,
    /// Flag to check if tx was received after sync from node.
    pub from_node: bool,
}

impl WalletTransaction {
    /// Check if transaction can be cancelled.
    pub fn can_cancel(&self) -> bool {
        self.from_node && !self.cancelling && !self.posting && !self.data.confirmed &&
            self.data.tx_type != TxLogEntryType::TxReceivedCancelled
            && self.data.tx_type != TxLogEntryType::TxSentCancelled
    }

    /// Check if transaction can be reposted.
    pub fn can_repost(&self, data: &WalletData) -> bool {
        let last_height = data.info.last_confirmed_height;
        let min_conf = data.info.minimum_confirmations;
        self.from_node && self.posting && self.repost_height.is_some() &&
            last_height - self.repost_height.unwrap() > min_conf
    }
}