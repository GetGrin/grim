// Copyright 2024 The Grim Developers
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

use egui::CornerRadius;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{fs, thread};

use crate::gui::icons::ARCHIVE_BOX;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::View;
use crate::gui::Colors;

/// Type of button.
pub enum FilePickContentType {
    Button, ItemButton(CornerRadius), Tab
}

/// Button to pick file and parse its data into text.
pub struct FilePickContent {
    /// Content type.
    content_type: FilePickContentType,

    /// Flag to check if button is active.
    active: bool,

    /// Flag to check if file is picking.
    file_picking: Arc<AtomicBool>,

    /// Flag to parse file content after pick.
    parse_file: bool,
    /// Flag to check if file is parsing.
    file_parsing: Arc<AtomicBool>,
    /// File parsing result.
    file_parsing_result: Arc<RwLock<Option<String>>>,
}

impl FilePickContent {
    /// Create new content from provided type.
    pub fn new(content_type: FilePickContentType) -> Self {
        Self {
            content_type,
            active: false,
            file_picking: Arc::new(AtomicBool::new(false)),
            parse_file: true,
            file_parsing: Arc::new(AtomicBool::new(false)),
            file_parsing_result: Arc::new(RwLock::new(None)),
        }
    }

    /// Do not parse file content.
    pub fn no_parse(mut self) -> Self {
        self.parse_file = false;
        self
    }

    /// Enable or disable the button.
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Draw content with provided callback to return path of the file.
    pub fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks, pick: impl FnOnce(String)) {
        if self.file_picking.load(Ordering::Relaxed) {
            View::small_loading_spinner(ui);
            // Check file pick result.
            if let Some(path) = cb.picked_file() {
                self.file_picking.store(false, Ordering::Relaxed);
                if !path.is_empty() {
                    self.on_file_pick(path);
                }
            }
        } else if self.file_parsing.load(Ordering::Relaxed) {
            View::small_loading_spinner(ui);
            // Check file parsing result.
            let has_result = {
                let r_res = self.file_parsing_result.read();
                r_res.is_some()
            };
            if has_result {
                let text = {
                    let r_res = self.file_parsing_result.read();
                    r_res.clone().unwrap()
                };
                // Callback on result.
                pick(text);
                // Clear result.
                let mut w_res = self.file_parsing_result.write();
                *w_res = None;
                self.file_parsing.store(false, Ordering::Relaxed);
            }
        } else {
            // Draw button to pick file.
            match self.content_type {
                FilePickContentType::Button => {
                    let text = format!("{} {}", ARCHIVE_BOX, t!("choose_file"));
                    View::colored_text_button(ui,
                                              text,
                                              Colors::blue(),
                                              Colors::white_or_black(false),
                                              || {
                                                  if let Some(path) = cb.pick_file() {
                                                      if !self.parse_file {
                                                          pick(path);
                                                          return;
                                                      }
                                                      self.on_file_pick(path);
                                                  }
                                              });
                }
                FilePickContentType::ItemButton(r) => {
                    View::item_button(ui, r, ARCHIVE_BOX, Some(Colors::blue()), || {
                        if let Some(path) = cb.pick_file() {
                            if !self.parse_file {
                                pick(path);
                                return;
                            }
                            self.on_file_pick(path);
                        }
                    });
                }
                FilePickContentType::Tab => {
                    let active = match self.active {
                        true => Some(self.file_parsing.load(Ordering::Relaxed) ||
                            self.file_picking.load(Ordering::Relaxed)),
                        false => None
                    };
                    View::tab_button(ui, ARCHIVE_BOX, Some(Colors::blue()), active, |_| {
                        if let Some(path) = cb.pick_file() {
                            if !self.parse_file {
                                pick(path);
                                return;
                            }
                            self.on_file_pick(path);
                        }
                    });
                }
            }
        }
    }

    /// Handle picked file path.
    fn on_file_pick(&self, path: String) {
        // Wait for asynchronous file pick result if path is empty.
        if path.is_empty() {
            self.file_picking.store(true, Ordering::Relaxed);
            return;
        }
        // Do not parse result.
        if !self.parse_file {
            return;
        }
        self.file_parsing.store(true, Ordering::Relaxed);
        let result = self.file_parsing_result.clone();
        thread::spawn(move || {
            if path.ends_with(".gif") {
                //TODO: Detect QR codes on GIF file.
            } else if path.ends_with(".jpeg") || path.ends_with(".jpg") ||
                path.ends_with(".png") {
                //TODO: Detect QR codes on image files.
            } else  {
                // Parse file as plain text.
                let mut w_res = result.write();
                if let Ok(text) = fs::read_to_string(path) {
                    *w_res = Some(text);
                } else {
                    *w_res = Some("".to_string());
                }
            }
        });
    }
}