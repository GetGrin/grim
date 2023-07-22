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

use grin_keychain::mnemonic::from_entropy;
use rand::{Rng, thread_rng};

/// Wallet creation step.
#[derive(PartialEq)]
pub enum Step {
    /// Mnemonic phrase input.
    EnterMnemonic,
    /// Mnemonic phrase confirmation for [`Mnemonic`].
    ConfirmMnemonic,
    /// Wallet connection setup.
    SetupConnection
}

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

/// Mnemonic phrase container.
pub struct Mnemonic {
    /// Phrase setup mode.
    pub(crate) mode: PhraseMode,
    /// Size of phrase based on words count.
    pub(crate) size: PhraseSize,
    /// Words for phrase.
    pub(crate) words: Vec<String>
}

impl Default for Mnemonic {
    fn default() -> Self {
        let size = PhraseSize::Words24;
        let mode = PhraseMode::Generate;
        let words = Self::generate_words(&mode, &size);
        Self { mode, size, words }
    }
}

impl Mnemonic {
    /// Change mnemonic phrase setup [`PhraseMode`].
    pub fn set_mode(&mut self, mode: PhraseMode) {
        self.mode = mode;
        self.words = Self::generate_words(&self.mode, &self.size);
    }

    /// Change mnemonic phrase words [`PhraseSize`].
    pub fn set_size(&mut self, size: PhraseSize) {
        self.size = size;
        self.words = Self::generate_words(&self.mode, &self.size);
    }

    /// Setup words based on provided [`PhraseMode`] and [`PhraseSize`].
    fn generate_words(mode: &PhraseMode, size: &PhraseSize) -> Vec<String> {
        match mode {
            PhraseMode::Generate => {
                let mut rng = thread_rng();
                let mut entropy: Vec<u8> = Vec::with_capacity(size.entropy_size());
                for _ in 0..size.entropy_size() {
                    entropy.push(rng.gen());
                }
                from_entropy(&entropy).unwrap()
                    .split(" ")
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
            },
            PhraseMode::Import => {
                let mut words = Vec::with_capacity(size.value());
                for _ in 0..size.value() {
                    words.push("".to_string())
                }
                words
            }
        }
    }
}