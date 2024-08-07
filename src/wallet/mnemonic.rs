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

use grin_keychain::mnemonic::{from_entropy, search, to_entropy};
use grin_util::ZeroingString;
use rand::{Rng, thread_rng};

use crate::wallet::types::{PhraseMode, PhraseSize};

/// Mnemonic phrase container.
pub struct Mnemonic {
    /// Phrase setup mode.
    pub(crate) mode: PhraseMode,
    /// Size of phrase based on words count.
    pub(crate) size: PhraseSize,
    /// Generated words.
    pub(crate) words: Vec<String>,
    /// Words to confirm the phrase.
    pub(crate) confirm_words: Vec<String>
}

impl Default for Mnemonic {
    fn default() -> Self {
        let size = PhraseSize::Words24;
        let mode = PhraseMode::Generate;
        let words = Self::generate_words(&mode, &size);
        let confirm_words = Self::empty_words(&size);
        Self { mode, size, words, confirm_words }
    }
}

impl Mnemonic {
    /// Change mnemonic phrase setup [`PhraseMode`].
    pub fn set_mode(&mut self, mode: PhraseMode) {
        self.mode = mode;
        self.words = Self::generate_words(&self.mode, &self.size);
        self.confirm_words = Self::empty_words(&self.size);
    }

    /// Change mnemonic phrase words [`PhraseSize`].
    pub fn set_size(&mut self, size: PhraseSize) {
        self.size = size;
        self.words = Self::generate_words(&self.mode, &self.size);
        self.confirm_words = Self::empty_words(&self.size);
    }

    /// Check if provided word is in BIP39 format.
    pub fn is_valid_word(&self, word: &String) -> bool {
        search(word).is_ok()
    }

    /// Check if current phrase is valid.
    pub fn is_valid_phrase(&self) -> bool {
        to_entropy(self.get_phrase().as_str()).is_ok()
    }

    /// Get phrase from words.
    pub fn get_phrase(&self) -> String {
        self.words.iter().map(|x| x.to_string() + " ").collect::<String>()
    }

    /// Generate list of words based on provided [`PhraseMode`] and [`PhraseSize`].
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
                    .map(|s| String::from(s))
                    .collect::<Vec<String>>()
            },
            PhraseMode::Import => {
                Self::empty_words(size)
            }
        }
    }

    /// Generate empty list of words based on provided [`PhraseSize`].
    fn empty_words(size: &PhraseSize) -> Vec<String> {
        let mut words = Vec::with_capacity(size.value());
        for _ in 0..size.value() {
            words.push(String::from(""))
        }
        words
    }

    /// Set words from provided text if possible.
    pub fn import_text(&mut self, text: &ZeroingString, confirmation: bool) {
        let words_split = text.trim().split(" ");
        let count = words_split.clone().count();
        if let Some(size) = PhraseSize::type_for_value(count) {
            if !confirmation {
                self.size = size;
            } else if self.size != size {
                return;
            }
            let mut words = vec![];
            words_split.for_each(|word| {
                if confirmation && !self.is_valid_word(&word.to_string()) {
                    words = vec![];
                    return;
                }
                words.push(word.to_string())
            });
            if confirmation {
                if !words.is_empty() {
                    self.confirm_words = words;
                }
            } else {
                self.words = words;
            }
        }
    }
}