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

use crate::wallet::types::{PhraseMode, PhraseSize, PhraseWord};

/// Mnemonic phrase container.
pub struct Mnemonic {
    /// Phrase setup mode.
    mode: PhraseMode,
    /// Size of phrase based on words count.
    size: PhraseSize,
    /// Generated words.
    words: Vec<PhraseWord>,
    /// Words to confirm the phrase.
    confirmation: Vec<PhraseWord>,
    /// Flag to check if entered phrase if valid.
    valid: bool,
}

impl Default for Mnemonic {
    fn default() -> Self {
        let size = PhraseSize::Words24;
        let mode = PhraseMode::Generate;
        let words = Self::generate_words(&mode, &size);
        let confirmation = Self::empty_words(&size);
        Self { mode, size, words, confirmation, valid: true }
    }
}

impl Mnemonic {
    /// Generate words based on provided [`PhraseMode`].
    pub fn set_mode(&mut self, mode: PhraseMode) {
        self.mode = mode;
        self.words = Self::generate_words(&self.mode, &self.size);
        self.confirmation = Self::empty_words(&self.size);
        self.valid = true;
    }

    /// Get current phrase mode.
    pub fn mode(&self) -> PhraseMode {
        self.mode.clone()
    }

    /// Generate words based on provided [`PhraseSize`].
    pub fn set_size(&mut self, size: PhraseSize) {
        self.size = size;
        self.words = Self::generate_words(&self.mode, &self.size);
        self.confirmation = Self::empty_words(&self.size);
        self.valid = true;
    }

    /// Get current phrase size.
    pub fn size(&self) -> PhraseSize {
        self.size.clone()
    }

    /// Get words based on current [`PhraseMode`].
    pub fn words(&self, edit: bool) -> Vec<PhraseWord> {
        match self.mode {
            PhraseMode::Generate => {
                if edit {
                    &self.confirmation
                } else {
                    &self.words
                }
            }
            PhraseMode::Import => &self.words
        }.clone()
    }

    /// Check if current phrase is valid.
    pub fn valid(&self) -> bool {
        self.valid
    }

    /// Get phrase from words.
    pub fn get_phrase(&self) -> String {
        self.words.iter()
            .enumerate()
            .map(|(i, x)| if i == 0 { "" } else { " " }.to_owned() + &x.text)
            .collect::<String>()
    }

    /// Generate [`PhraseWord`] list based on provided [`PhraseMode`] and [`PhraseSize`].
    fn generate_words(mode: &PhraseMode, size: &PhraseSize) -> Vec<PhraseWord> {
        match mode {
            PhraseMode::Generate => {
                let mut rng = thread_rng();
                let mut entropy: Vec<u8> = Vec::with_capacity(size.entropy_size());
                for _ in 0..size.entropy_size() {
                    entropy.push(rng.gen());
                }
                from_entropy(&entropy).unwrap()
                    .split(" ")
                    .map(|s| {
                        let text = s.to_string();
                        PhraseWord {
                            text,
                            valid: true,
                        }
                    })
                    .collect::<Vec<PhraseWord>>()
            },
            PhraseMode::Import => {
                Self::empty_words(size)
            }
        }
    }

    /// Generate empty list of words based on provided [`PhraseSize`].
    fn empty_words(size: &PhraseSize) -> Vec<PhraseWord> {
        let mut words = Vec::with_capacity(size.value());
        for _ in 0..size.value() {
            words.push(PhraseWord {
                text: "".to_string(),
                valid: true,
            });
        }
        words
    }

    /// Insert word into provided index and return validation result.
    pub fn insert(&mut self, index: usize, word: &String) -> bool {
        // Check if word is valid.
        let found = search(word).is_ok();
        if !found {
            return false;
        }
        let is_confirmation = self.mode == PhraseMode::Generate;
        if is_confirmation {
            let w = self.words.get(index).unwrap();
            if word != &w.text {
                return false;
            }
        }

        // Save valid word at list.
        let words = if is_confirmation {
            &mut self.confirmation
        } else {
            &mut self.words
        };
        words.remove(index);
        words.insert(index, PhraseWord { text: word.to_owned(), valid: true });

        // Validate phrase when all words are entered.
        let mut has_empty = false;
        let _: Vec<_> = words.iter().map(|w| {
            if w.text.is_empty() {
                has_empty = true;
            }
        }).collect();
        if !has_empty {
            self.valid = to_entropy(self.get_phrase().as_str()).is_ok();
        }
        true
    }

    /// Get word from provided index.
    pub fn get(&self, index: usize) -> Option<PhraseWord> {
        let words = match self.mode {
            PhraseMode::Generate => &self.confirmation,
            PhraseMode::Import => &self.words
        };
        let word = words.get(index);
        if let Some(w) = word {
            return Some(PhraseWord {
                text: w.text.clone(),
                valid: w.valid
            });
        }
        None
    }

    /// Setup phrase from provided text if possible.
    pub fn import(&mut self, text: &ZeroingString) {
        let words_split = text.trim().split(" ");
        let count = words_split.clone().count();
        if let Some(size) = PhraseSize::type_for_value(count) {
            // Setup phrase size.
            let confirm = self.mode == PhraseMode::Generate;
            if !confirm {
                self.words = Self::empty_words(&size);
                self.size = size;
            } else if self.size != size {
                return;
            }

            // Setup word list.
            let mut words = vec![];
            words_split.for_each(|w| {
                let mut text = w.to_string();
                text.retain(|c| c.is_alphabetic());
                let valid = search(&text).is_ok();
                words.push(PhraseWord { text, valid });
            });
            let mut has_invalid = false;
            for (i, w) in words.iter().enumerate() {
                if !self.insert(i, &w.text) {
                    has_invalid = true;
                }
            }
            self.valid = !has_invalid;
        }
    }

    /// Check if phrase has invalid or empty words.
    pub fn has_empty_or_invalid(&self) -> bool {
        let words = match self.mode {
            PhraseMode::Generate => &self.confirmation,
            PhraseMode::Import => &self.words
        };
        let mut has_empty = false;
        let mut has_invalid = false;
        let _: Vec<_> = words.iter().map(|w| {
            if w.text.is_empty() {
                has_empty = true;
            }
            if !w.valid {
                has_invalid = true;
            }
        }).collect();
        has_empty || has_invalid
    }
}