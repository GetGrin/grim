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
use grin_wallet_impls::{DefaultLCProvider, HTTPNodeClient};
use grin_wallet_libwallet::WalletInst;
use parking_lot::Mutex;

/// Mnemonic phrase setup mode.
#[derive(PartialEq, Clone)]
pub enum PhraseMode {
    /// Generate new mnemonic phrase.
    Generate,
    /// Import existing mnemonic phrase.
    Import
}

/// Mnemonic phrase size based on words count.
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

    /// Gen words count number.
    pub fn value(&self) -> usize {
        match *self {
            PhraseSize::Words12 => 12,
            PhraseSize::Words15 => 15,
            PhraseSize::Words18 => 18,
            PhraseSize::Words21 => 21,
            PhraseSize::Words24 => 24
        }
    }

    /// Gen entropy size for current phrase size.
    pub fn entropy_size(&self) -> usize {
        match *self {
            PhraseSize::Words12 => 16,
            PhraseSize::Words15 => 20,
            PhraseSize::Words18 => 24,
            PhraseSize::Words21 => 28,
            PhraseSize::Words24 => 32
        }
    }
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

/// Wallet node connection method type.
#[derive(PartialEq)]
pub enum ConnectionMethod {
    /// Integrated node.
    Integrated,
    /// External node, contains connection identifier.
    External(i64)
}