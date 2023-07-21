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
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::wallets::creation::StepControl;

/// Mnemonic phrase setup mode.
#[derive(PartialEq)]
pub enum PhraseMode {
    /// Generate new mnemonic phrase.
    Generate,
    /// Import existing mnemonic phrase.
    Import
}

/// Mnemonic phrase type based on words count.
pub enum PhraseType { Words12, Words15, Words18, Words21, Words24 }

impl PhraseType {
    pub fn value(&self) -> usize {
        match *self {
            PhraseType::Words12 => 12,
            PhraseType::Words15 => 15,
            PhraseType::Words18 => 18,
            PhraseType::Words21 => 21,
            PhraseType::Words24 => 24
        }
    }
}

/// Mnemonic phrase container.
pub struct Mnemonic {
    /// Phrase setup mode.
    pub(crate) mode: PhraseMode,
    /// Type of phrase based on words count.
    size: PhraseType,
    /// Words for phrase.
    words: Vec<String>
}

impl Default for Mnemonic {
    fn default() -> Self {
        let size = PhraseType::Words12;
        let size_value = size.value();
        Self { mode: PhraseMode::Generate, size, words: Vec::with_capacity(size_value) }
    }
}

impl Mnemonic {
    /// Change mnemonic phrase setup [`PhraseMode`].
    fn set_mode(&mut self, mode: PhraseMode) {
        self.mode = mode;
        self.setup_words();
    }

    /// Change mnemonic phrase words [`PhraseType`].
    fn set_size(&mut self, size: PhraseType) {
        self.size = size;
        self.setup_words();
    }

    /// Setup words based on current [`PhraseMode`] and [`PhraseType`].
    fn setup_words(&mut self) {
        self.words = match self.mode {
            PhraseMode::Generate => {
                let mut rng = thread_rng();
                let mut entropy: Vec<u8> = Vec::with_capacity(self.size.value());
                for _ in 0..self.size.value() {
                    entropy.push(rng.gen());
                }
                from_entropy(&entropy).unwrap()
                    .split(" ")
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
            },
            PhraseMode::Import => Vec::with_capacity(self.size.value())
        };
    }
}

/// Mnemonic phrase setup content.
pub struct MnemonicSetup {
    /// Current mnemonic phrase.
    mnemonic: Mnemonic,
    /// Word value for [`Modal`].
    word_edit: String,
}

impl Default for MnemonicSetup {
    fn default() -> Self {
        Self {
            mnemonic: Mnemonic::default(),
            word_edit: "".to_string(),
        }
    }
}

impl MnemonicSetup {
    pub fn ui(&self, ui: &mut egui::Ui, step: &dyn StepControl, cb: &dyn PlatformCallbacks) {

    }

    pub fn get_mnemonic_mode(&self) -> &PhraseMode {
        &self.mnemonic.mode
    }

    /// Reset mnemonic phrase to default values.
    pub fn reset(&mut self) {
        self.mnemonic = Mnemonic::default();
    }
}