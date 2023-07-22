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
use crate::gui::icons::PENCIL;
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
    pub fn enter_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        ScrollArea::vertical()
            .id_source("mnemonic_words_list")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(10.0);

                // Show mode and type setup.
                self.mode_type_ui(ui);

                ui.add_space(12.0);
                View::horizontal_line(ui, Colors::ITEM_STROKE);
                ui.add_space(6.0);

                // Show words setup.
                self.word_list_ui(ui);
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

    /// Calculate word list columns count based on available ui width.
    fn calc_columns_count(ui: &mut egui::Ui) -> usize {
        let w = ui.available_width();
        let min_panel_w = Root::SIDE_PANEL_MIN_WIDTH - 12.0;
        let double_min_panel_w = min_panel_w * 2.0;
        if w >= min_panel_w * 1.5 && w < double_min_panel_w {
            3
        } else if w >= double_min_panel_w {
            4
        } else {
            2
        }
    }

    /// Draw word list for mnemonic phrase.
    fn word_list_ui(&self, ui: &mut egui::Ui) {
        ui.scope(|ui| {
            // Setup spacing between columns.
            ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 6.0);

            if self.mnemonic.mode == PhraseMode::Generate {
                ui.add_space(6.0)
            }

            let mut word_number = 0;
            let cols = Self::calc_columns_count(ui);
            let _ = self.mnemonic.words.chunks(cols).map(|chunk| {
                let size = chunk.len();
                word_number += 1;
                if size > 1 {
                    ui.columns(cols, |columns| {
                        columns[0].horizontal(|ui| {
                            self.word_item_ui(ui, word_number, chunk, 0);
                        });
                        columns[1].horizontal(|ui| {
                            word_number += 1;
                            self.word_item_ui(ui, word_number, chunk, 1);
                        });
                        if size > 2 {
                            columns[2].horizontal(|ui| {
                                word_number += 1;
                                self.word_item_ui(ui, word_number, chunk, 2);
                            });
                        }
                        if size > 3 {
                            columns[3].horizontal(|ui| {
                                word_number += 1;
                                self.word_item_ui(ui, word_number, chunk, 3);
                            });
                        }
                    });
                } else {
                    ui.columns(cols, |columns| {
                        columns[0].horizontal(|ui| {
                            self.word_item_ui(ui, word_number, chunk, 0);
                        });
                    });
                    ui.add_space(12.0);
                }
            }).collect::<Vec<_>>();
        });
    }

    /// Draw word item at given index from provided chunk.
    fn word_item_ui(&self,
                    ui: &mut egui::Ui,
                    word_number: usize,
                    chunk: &[String],
                    index: usize) {
        match self.mnemonic.mode {
            PhraseMode::Generate => {
                ui.add_space(12.0);
                let word = chunk.get(index).unwrap();
                let text = format!("#{} {}", word_number, word);
                ui.label(RichText::new(text).size(17.0).color(Colors::BLACK));
            }
            PhraseMode::Import => {
                let mut size = ui.available_size();
                size.x = 90.0;
                ui.allocate_ui(size, |ui| {
                    View::button(ui, PENCIL.to_string(), Colors::BUTTON, || {
                        //TODO: open modal
                    });
                });
                ui.label(RichText::new(format!("#{}", word_number))
                    .size(17.0)
                    .color(Colors::BLACK));
            }
        }
    }

    /// Reset mnemonic phrase to default values.
    pub fn reset(&mut self) {
        self.mnemonic = Mnemonic::default();
    }
}