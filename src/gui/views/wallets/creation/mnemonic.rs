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

use egui::{Id, RichText, TextStyle, Widget};

use crate::gui::Colors;
use crate::gui::icons::PENCIL;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, Root, View};
use crate::gui::views::types::{ModalContainer, ModalPosition};
use crate::wallet::Mnemonic;
use crate::wallet::types::{PhraseMode, PhraseSize};

/// Mnemonic phrase setup content.
pub struct MnemonicSetup {
    /// Current mnemonic phrase.
    pub(crate) mnemonic: Mnemonic,

    /// Flag to check if entered phrase was valid.
    pub(crate) valid_phrase: bool,

    /// Current word number to edit at [`Modal`].
    word_num_edit: usize,
    /// Entered word value for [`Modal`].
    word_edit: String,
    /// Flag to check if entered word is valid.
    valid_word_edit: bool,

    /// [`Modal`] identifiers allowed at this ui container.
    modal_ids: Vec<&'static str>
}

/// Identifier for word input [`Modal`].
pub const WORD_INPUT_MODAL: &'static str = "word_input_modal";

impl Default for MnemonicSetup {
    fn default() -> Self {
        Self {
            mnemonic: Mnemonic::default(),
            valid_phrase: true,
            word_num_edit: 0,
            word_edit: String::from(""),
            valid_word_edit: true,
            modal_ids: vec![
                WORD_INPUT_MODAL
            ]
        }
    }
}

impl ModalContainer for MnemonicSetup {
    fn modal_ids(&self) -> &Vec<&'static str> {
        &self.modal_ids
    }

    fn modal_ui(&mut self,
                ui: &mut egui::Ui,
                _: &mut eframe::Frame,
                modal: &Modal,
                cb: &dyn PlatformCallbacks) {
        match modal.id {
            WORD_INPUT_MODAL => self.word_modal_ui(ui, modal, cb),
            _ => {}
        }
    }
}

impl MnemonicSetup {
    /// Draw content for phrase input step.
    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, cb: &dyn PlatformCallbacks) {
        // Draw modal content for current ui container.
        self.current_modal_ui(ui, frame, cb);

        ui.add_space(10.0);

        // Show mode and type setup.
        self.mode_type_ui(ui);

        ui.add_space(12.0);
        View::horizontal_line(ui, Colors::ITEM_STROKE);
        ui.add_space(6.0);

        // Show words setup.
        self.word_list_ui(ui, self.mnemonic.mode == PhraseMode::Import, cb);
    }

    /// Draw content for phrase confirmation step.
    pub fn confirm_ui(&mut self,
                      ui: &mut egui::Ui,
                      frame: &mut eframe::Frame,
                      cb: &dyn PlatformCallbacks) {
        // Draw modal content for current ui container.
        self.current_modal_ui(ui, frame, cb);

        ui.add_space(4.0);
        ui.vertical_centered(|ui| {
            let text = format!("{}:", t!("wallets.recovery_phrase"));
            ui.label(RichText::new(text).size(16.0).color(Colors::GRAY));
        });
        ui.add_space(4.0);
        self.word_list_ui(ui, true, cb);
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
            for (index, word) in PhraseSize::VALUES.iter().enumerate() {
                columns[index].vertical_centered(|ui| {
                    let text = word.value().to_string();
                    View::radio_value(ui, &mut size, word.clone(), text);
                });
            }
        });
        if size != self.mnemonic.size {
            self.mnemonic.set_size(size);
        }
    }

    /// Draw list of words for mnemonic phrase.
    fn word_list_ui(&mut self, ui: &mut egui::Ui, edit_words: bool, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.scope(|ui| {
            // Setup spacing between columns.
            ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 6.0);

            // Select list of words based on current mode and edit flag.
            let words = match self.mnemonic.mode {
                PhraseMode::Generate => {
                    if edit_words {
                        self.mnemonic.confirm_words.clone()
                    } else {
                        self.mnemonic.words.clone()
                    }
                }
                PhraseMode::Import => self.mnemonic.words.clone()
            };

            let mut word_number = 0;
            let cols = list_columns_count(ui);
            let _ = words.chunks(cols).map(|chunk| {
                let size = chunk.len();
                word_number += 1;
                if size > 1 {
                    ui.columns(cols, |columns| {
                        columns[0].horizontal(|ui| {
                            let word = chunk.get(0).unwrap();
                            self.word_item_ui(ui, word_number, word, edit_words, cb);
                        });
                        columns[1].horizontal(|ui| {
                            word_number += 1;
                            let word = chunk.get(1).unwrap();
                            self.word_item_ui(ui, word_number, word, edit_words, cb);
                        });
                        if size > 2 {
                            columns[2].horizontal(|ui| {
                                word_number += 1;
                                let word = chunk.get(2).unwrap();
                                self.word_item_ui(ui, word_number, word, edit_words, cb);
                            });
                        }
                        if size > 3 {
                            columns[3].horizontal(|ui| {
                                word_number += 1;
                                let word = chunk.get(3).unwrap();
                                self.word_item_ui(ui, word_number, word, edit_words, cb);
                            });
                        }
                    });
                } else {
                    ui.columns(cols, |columns| {
                        columns[0].horizontal(|ui| {
                            let word = chunk.get(0).unwrap();
                            self.word_item_ui(ui, word_number, word, edit_words, cb);
                        });
                    });
                }
            }).collect::<Vec<_>>();
        });
        ui.add_space(6.0);
    }

    /// Draw word list item for current mode.
    fn word_item_ui(&mut self,
                    ui: &mut egui::Ui,
                    num: usize,
                    word: &String,
                    edit: bool,
                    cb: &dyn PlatformCallbacks) {
        if edit {
            ui.add_space(6.0);
            View::button(ui, PENCIL.to_string(), Colors::BUTTON, || {
                // Setup modal values.
                self.word_num_edit = num;
                self.word_edit = word.clone();
                self.valid_word_edit = true;
                // Show word edit modal.
                Modal::new(WORD_INPUT_MODAL)
                    .position(ModalPosition::CenterTop)
                    .title(t!("wallets.recovery_phrase"))
                    .show();
                cb.show_keyboard();
            });
            ui.label(RichText::new(format!("#{} {}", num, word))
                .size(17.0)
                .color(Colors::BLACK));
        } else {
            ui.add_space(12.0);
            let text = format!("#{} {}", num, word);
            ui.label(RichText::new(text).size(17.0).color(Colors::BLACK));
        }
    }

    /// Reset mnemonic phrase to default values.
    pub fn reset(&mut self) {
        self.mnemonic = Mnemonic::default();
    }

    /// Draw word input [`Modal`] content.
    fn word_modal_ui(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("wallets.enter_word", "number" => self.word_num_edit))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw word value text edit.
            let text_edit_resp = egui::TextEdit::singleline(&mut self.word_edit)
                .id(Id::from(modal.id).with(self.word_num_edit))
                .font(TextStyle::Heading)
                .desired_width(ui.available_width())
                .cursor_at_end(true)
                .ui(ui);
            text_edit_resp.request_focus();
            if text_edit_resp.clicked() {
                cb.show_keyboard();
            }

            // Show error when specified word is not valid.
            if !self.valid_word_edit {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("wallets.not_valid_word"))
                    .size(17.0)
                    .color(Colors::RED));
            }
            ui.add_space(12.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::WHITE, || {
                        // Close modal.
                        cb.hide_keyboard();
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    // Callback to save the word.
                    let mut save = || {
                        // Check if word is valid.
                        let word_index = self.word_num_edit - 1;
                        if !self.mnemonic.is_valid_word(&self.word_edit, word_index) {
                            self.valid_word_edit = false;
                            return;
                        }
                        self.valid_word_edit = true;

                        // Select list where to save word.
                        let words = match self.mnemonic.mode {
                            PhraseMode::Generate => &mut self.mnemonic.confirm_words,
                            PhraseMode::Import => &mut self.mnemonic.words
                        };

                        // Save word at list.
                        words.remove(word_index);
                        words.insert(word_index, self.word_edit.clone());

                        // Close modal or go to next word to edit.
                        let close_modal = words.len() == self.word_num_edit
                            || !words.get(self.word_num_edit).unwrap().is_empty();
                        if close_modal {
                            // Check if entered phrase was valid when all words were entered.
                            if !self.mnemonic.words.contains(&String::from("")) {
                                self.valid_phrase = self.mnemonic.is_valid_phrase();
                            }
                            cb.hide_keyboard();
                            modal.close();
                        } else {
                            self.word_num_edit += 1;
                            self.word_edit = String::from("");
                        }
                    };
                    // Call save on Enter key press.
                    View::on_enter_key(ui, || {
                        (save)();
                    });
                    // Show save button.
                    View::button(ui, t!("continue"), Colors::WHITE, save);
                });
            });
            ui.add_space(6.0);
        });
    }
}

/// Calculate word list columns count based on available ui width.
fn list_columns_count(ui: &mut egui::Ui) -> usize {
    let w = ui.available_width();
    let min_panel_w = Root::SIDE_PANEL_WIDTH - 12.0;
    let double_min_panel_w = min_panel_w * 2.0;
    if w >= min_panel_w * 1.5 && w < double_min_panel_w {
        3
    } else if w >= double_min_panel_w {
        4
    } else {
        2
    }
}