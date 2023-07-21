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

use egui::{RichText, ScrollArea};

use crate::gui::Colors;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Root, View};
use crate::gui::views::wallets::creation::types::{Mnemonic, PhraseMode, PhraseSize};

/// Mnemonic phrase setup content.
pub struct MnemonicSetup {
    /// Current mnemonic phrase.
    pub(crate) mnemonic: Mnemonic,
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
    pub fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        ScrollArea::vertical()
            .id_source("mnemonic_words_list")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(10.0);

                // Show mode and type setup.
                self.mode_type_ui(ui);

                ui.add_space(12.0);
                View::horizontal_line(ui, Colors::ITEM_STROKE);
                ui.add_space(12.0);

                // Show words setup.
                self.words_ui(ui);
            });
    }

    /// Draw mode and size setup.
    fn mode_type_ui(&mut self, ui: &mut egui::Ui) {
        // Show mode setup.
        let mut mode = self.mnemonic.mode.clone();
        ui.columns(2, |columns| {
            columns[0].vertical_centered(|ui| {
                let create_mode = PhraseMode::Generate;
                let create_text = t!("wallets.create");
                View::radio_value(ui, &mut mode, create_mode, create_text);
            });
            columns[1].vertical_centered(|ui| {
                let import_mode = PhraseMode::Import;
                let import_text = t!("wallets.recover");
                View::radio_value(ui, &mut mode, import_mode, import_text);
            });
        });
        if mode != self.mnemonic.mode {
            self.mnemonic.set_mode(mode)
        }

        ui.add_space(10.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("wallets.words_count"))
                .size(16.0)
                .color(Colors::GRAY)
            );
        });
        ui.add_space(6.0);

        // Show mnemonic phrase size setup.
        let mut size = self.mnemonic.size.clone();
        ui.columns(5, |columns| {
            columns[0].vertical_centered(|ui| {
                let words12 = PhraseSize::Words12;
                let text = words12.value().to_string();
                View::radio_value(ui, &mut size, words12, text);
            });
            columns[1].vertical_centered(|ui| {
                let words15 = PhraseSize::Words15;
                let text = words15.value().to_string();
                View::radio_value(ui, &mut size, words15, text);
            });
            columns[2].vertical_centered(|ui| {
                let words18 = PhraseSize::Words18;
                let text = words18.value().to_string();
                View::radio_value(ui, &mut size, words18, text);
            });
            columns[3].vertical_centered(|ui| {
                let words21 = PhraseSize::Words21;
                let text = words21.value().to_string();
                View::radio_value(ui, &mut size, words21, text);
            });
            columns[4].vertical_centered(|ui| {
                let words24 = PhraseSize::Words24;
                let text = words24.value().to_string();
                View::radio_value(ui, &mut size, words24, text);
            });
        });
        if size != self.mnemonic.size {
            self.mnemonic.set_size(size);
        }
    }

    /// Draw words setup based on selected [`PhraseMode`].
    fn words_ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            // Show word list based on setup mode.
            match self.mnemonic.mode {
                PhraseMode::Generate => self.word_list_generate_ui(ui),
                PhraseMode::Import => self.word_list_import_ui(ui)
            }
        });
    }

    /// Draw word list for [`PhraseMode::Generate`] mode.
    fn word_list_generate_ui(&mut self, ui: &mut egui::Ui) {
        // Calculate rows count based on available ui width.
        const PADDING: f32 = 24.0;
        let w = ui.available_width();
        let min_panel_w = Root::SIDE_PANEL_MIN_WIDTH;
        let double_min_panel_w = (min_panel_w * 2.0) - PADDING;
        let cols = if w >= (min_panel_w * 1.5) - PADDING && w < double_min_panel_w {
            3
        } else if w >= double_min_panel_w {
            4
        } else {
            2
        };

        // Show words amount.
        let mut word_number = 0;
        let _ = self.mnemonic.words.chunks(cols).map(|chunk| {
            let size = chunk.len();
            word_number += 1;
            if size > 1 {
                ui.columns(cols, |columns| {
                    columns[0].horizontal(|ui| {
                        ui.add_space(PADDING);
                        Self::generated_word_ui(ui, word_number, chunk, 0);
                    });
                    columns[1].horizontal(|ui| {
                        ui.add_space(PADDING);
                        word_number += 1;
                        Self::generated_word_ui(ui, word_number, chunk, 1);
                    });
                    if size > 2 {
                        columns[2].horizontal(|ui| {
                            ui.add_space(PADDING);
                            word_number += 1;
                            Self::generated_word_ui(ui, word_number, chunk, 2);
                        });
                    }
                    if size > 3 {
                        columns[3].horizontal(|ui| {
                            ui.add_space(PADDING);
                            word_number += 1;
                            Self::generated_word_ui(ui, word_number, chunk, 3);
                        });
                    }
                });
            } else {
                ui.columns(cols, |columns| {
                    columns[0].horizontal(|ui| {
                        ui.add_space(PADDING);
                        Self::generated_word_ui(ui, word_number, chunk, 0);
                    });
                });
                ui.add_space(12.0);
            }
            ui.add_space(8.0);
        }).collect::<Vec<_>>();
    }

    /// Draw generated word at given index from provided chunk.
    fn generated_word_ui(ui: &mut egui::Ui,
                                  word_number: usize,
                                  chunk: &[String],
                                  index: usize) {
        let word = chunk.get(index).unwrap();
        let text = format!("#{} {}", word_number, word);
        ui.label(RichText::new(text).size(16.0).color(Colors::BLACK));
    }


    /// Draw word list for [`PhraseMode::Import`] mode.
    fn word_list_import_ui(&mut self, ui: &mut egui::Ui) {

    }

    /// Reset mnemonic phrase to default values.
    pub fn reset(&mut self) {
        self.mnemonic = Mnemonic::default();
    }
}