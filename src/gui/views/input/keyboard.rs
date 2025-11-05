// Copyright 2025 The Grim Developers
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

use std::string::ToString;
use egui::{Align, Align2, Button, Color32, CursorIcon, Layout, Margin, Rect, Response, RichText, Sense, Shadow, Vec2, Widget};
use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::gui::icons::{ARROW_FAT_UP, BACKSPACE, GLOBE_SIMPLE, KEY_RETURN};
use crate::gui::views::{KeyboardEvent, KeyboardLayout, KeyboardState, View};
use crate::gui::Colors;
use crate::AppConfig;

lazy_static! {
    /// Keyboard window state.
    static ref WINDOW_STATE: Arc<RwLock<KeyboardState >> = Arc::new(
        RwLock::new(KeyboardState::default())
    );
}

/// Software keyboard content.
pub struct KeyboardContent {
    /// Keyboard content state.
    state: KeyboardState,
}

impl Default for KeyboardContent {
    fn default() -> Self {
        Self {
            state: KeyboardState::default(),
        }
    }
}

impl KeyboardContent {
    /// Maximum keyboard content width.
    const MAX_WIDTH: f32 = 600.0;
    /// Maximum numbers layout width.
    const MAX_WIDTH_NUMBERS: f32 = 400.0;

    /// Keyboard window id.
    pub const WINDOW_ID: &'static str = "soft_keyboard_window";

    /// Draw keyboard content as separate [`Window`].
    pub fn window_ui(&mut self, numeric: bool, ctx: &egui::Context) {
        let width = ctx.content_rect().width();
        let layer_id = egui::Window::new(Self::WINDOW_ID)
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .min_width(width)
            .default_width(width)
            .anchor(Align2::CENTER_BOTTOM, Vec2::new(0.0, 0.0))
            .frame(egui::Frame {
                shadow: Shadow {
                    offset: Default::default(),
                    blur: 30.0 as u8,
                    spread: 3.0 as u8,
                    color: Color32::from_black_alpha(32),
                },
                inner_margin: Margin {
                    left: View::get_left_inset() as i8,
                    right: View::get_right_inset() as i8,
                    top: 1.0 as i8,
                    bottom: View::get_bottom_inset() as i8,
                },
                fill: Colors::fill(),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.set_min_width(width);
                // Setup state.
                {
                    let r_state = WINDOW_STATE.read();
                    self.state = (*r_state).clone();
                }
                // Calculate content width.
                let side_insets = View::get_left_inset() + View::get_right_inset();
                let available_width = width - side_insets;
                let w = f32::min(available_width, if numeric {
                    Self::MAX_WIDTH_NUMBERS
                } else {
                    Self::MAX_WIDTH
                });
                // Draw content.
                View::max_width_ui(ui, w, |ui| {
                    self.ui(numeric, ui);
                });
                // Save state.
                let mut w_state = WINDOW_STATE.write();
                *w_state = self.state.clone();
            }).unwrap().response.layer_id;

        // Always show keyboard above others windows.
        ctx.move_to_top(layer_id);
    }

    /// Draw keyboard content.
    pub fn ui(&mut self, numeric: bool, ui: &mut egui::Ui) {
        // Setup layout.
        if numeric {
            self.state.layout = Arc::new(KeyboardLayout::NUMBERS);
        } else if *self.state.layout == KeyboardLayout::NUMBERS {
            self.state.layout = Arc::new(KeyboardLayout::TEXT);
        }

        // Setup spacing between buttons.
        ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 0.0);
        // Setup vertical padding inside buttons.
        ui.style_mut().spacing.button_padding = egui::vec2(0.0, if numeric {
            12.0
        } else {
            10.0
        });

        // Draw input buttons.
        let button_rect = match *self.state.layout {
            KeyboardLayout::TEXT => self.text_ui(ui),
            KeyboardLayout::SYMBOLS => self.symbols_ui(ui),
            KeyboardLayout::NUMBERS => self.numbers_ui(ui),
        };

        // Draw bottom keyboard buttons.
        let bottom_size = {
            let mut r = button_rect.clone();
            r.set_width(ui.available_width());
            r.size()
        };
        let button_width = ui.available_width() / match *self.state.layout {
            KeyboardLayout::TEXT => 11.0,
            KeyboardLayout::SYMBOLS => 10.0,
            KeyboardLayout::NUMBERS => 4.0,
        };
        ui.allocate_ui_with_layout(bottom_size, Layout::right_to_left(Align::Center), |ui| {
            match *self.state.layout {
                KeyboardLayout::TEXT => {
                    // Enter key input.
                    ui.horizontal_centered(|ui| {
                        ui.set_max_width(button_width * 2.0);
                        self.custom_button_ui(KEY_RETURN.to_string(),
                                               Colors::white_or_black(false),
                                               Some(Colors::green()),
                                               ui,
                                               |_, c| {
                                                   c.state.last_event =
                                                       Arc::new(Some(KeyboardEvent::ENTER));
                                               });
                    });
                    // Custom input key.
                    ui.horizontal_centered(|ui| {
                        ui.set_max_width(button_width);
                        self.input_button_ui("m3", true, ui);
                    });
                    // Space key input.
                    ui.horizontal_centered(|ui| {
                        ui.set_max_width(button_width * 5.0);
                        self.custom_button_ui(" ".to_string(),
                                              Colors::inactive_text(),
                                              None,
                                              ui,
                                              |l, c| {
                                                  c.state.last_event =
                                                      Arc::new(Some(KeyboardEvent::TEXT(l)));
                                              });
                    });
                    // Switch to english and back.
                    ui.horizontal_centered(|ui| {
                        ui.set_max_width(button_width);
                        self.custom_button_ui(GLOBE_SIMPLE.to_string(),
                                               Colors::text_button(),
                                               Some(Colors::fill_lite()),
                                               ui,
                                               |_, _| {
                                                   AppConfig::toggle_english_keyboard()
                                               });
                    });
                    // Switch to symbols layout.
                    self.custom_button_ui("!@ツ".to_string(),
                                           Colors::text_button(),
                                           Some(Colors::fill_lite()),
                                           ui,
                                           |_, c| {
                                               c.state.layout = Arc::new(KeyboardLayout::SYMBOLS);
                                           });
                }
                KeyboardLayout::SYMBOLS => {
                    // Enter key input.
                    ui.horizontal_centered(|ui| {
                        ui.set_max_width(button_width * 2.0);
                        self.custom_button_ui(KEY_RETURN.to_string(),
                                               Colors::white_or_black(false),
                                               Some(Colors::green()),
                                               ui,
                                               |_, c| {
                                                   c.state.last_event =
                                                       Arc::new(Some(KeyboardEvent::ENTER));
                                               });
                    });
                    // Custom input key.
                    ui.horizontal_centered(|ui| {
                        ui.set_max_width(button_width);
                        self.input_button_ui("ツ", false, ui);
                    });
                    // Space key input.
                    ui.horizontal_centered(|ui| {
                        ui.set_max_width(button_width * 4.0);
                        self.custom_button_ui(" ".to_string(),
                                              Colors::inactive_text(),
                                              None,
                                              ui,
                                              |l, c| {
                                                  c.state.last_event =
                                                      Arc::new(Some(KeyboardEvent::TEXT(l)));
                                              });
                    });
                    // Switch to text layout.
                    let label = {
                        let q = t!("keyboard.q", locale = Self::input_locale().as_str());
                        let w = t!("keyboard.w", locale = Self::input_locale().as_str());
                        let e = t!("keyboard.e", locale = Self::input_locale().as_str());
                        format!("{}{}{}", q, w, e).to_uppercase()
                    };
                    self.custom_button_ui(label,
                                          Colors::text_button(),
                                          Some(Colors::fill_lite()),
                                          ui,
                                          |_, c| {
                                              c.state.layout = Arc::new(KeyboardLayout::TEXT);
                                          });
                }
                KeyboardLayout::NUMBERS => {
                    ui.horizontal_centered(|ui| {
                        ui.set_max_width(button_width * 2.0);
                        self.custom_button_ui(KEY_RETURN.to_string(),
                                              Colors::white_or_black(false),
                                              Some(Colors::green()),
                                              ui,
                                              |_, c| {
                                                  c.state.last_event =
                                                      Arc::new(Some(KeyboardEvent::ENTER));
                                              });
                    });
                    ui.horizontal_centered(|ui| {
                        ui.set_max_width(button_width);
                        self.input_button_ui("0", true, ui);
                    });
                    ui.horizontal_centered(|ui| {
                        ui.set_max_width(button_width);
                        self.input_button_ui(".", false, ui);
                    });
                }
            }
        });
    }

    /// Draw numbers content returning button [`Rect`].
    fn numbers_ui(&mut self, ui: &mut egui::Ui) -> Rect {
        let mut button_rect = ui.available_rect_before_wrap();
        let tl_0: Vec<&str> = vec!["1", "2", "3", "+"];
        ui.columns(tl_0.len(), |columns| {
            for (index, s) in tl_0.iter().enumerate() {
                let last = index == tl_0.len() - 1;
                button_rect = self.input_button_ui(s, !last, &mut columns[index]);
            }
        });

        let tl_1: Vec<&str> = vec!["4", "5", "6", ","];
        ui.columns(tl_1.len(), |columns| {
            for (index, s) in tl_1.iter().enumerate() {
                let last = index == tl_1.len() - 1;
                self.input_button_ui(s, !last, &mut columns[index]);
            }
        });

        let tl_2: Vec<&str> = vec!["7", "8", "9", BACKSPACE];
        ui.columns(tl_2.len(), |columns| {
            for (index, s) in tl_2.iter().enumerate() {
                if index == tl_2.len() - 1 {
                    self.custom_button_ui(BACKSPACE.to_string(),
                                           Colors::red(),
                                           Some(Colors::fill_lite()),
                                           &mut columns[index],
                                           |_, c| {
                                               c.state.last_event =
                                                   Arc::new(Some(KeyboardEvent::CLEAR));
                                           });
                } else {
                    self.input_button_ui(s, true, &mut columns[index]);
                }
            }
        });

        button_rect
    }

    /// Draw text content returning button [`Rect`].
    fn text_ui(&mut self, ui: &mut egui::Ui) -> Rect {
        let mut button_rect = ui.available_rect_before_wrap();
        let tl_0: Vec<&str> = vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "01"];
        ui.columns(tl_0.len(), |columns| {
            for (index, s) in tl_0.iter().enumerate() {
                button_rect = self.input_button_ui(s, true, &mut columns[index]);
            }
        });

        let tl_1: Vec<&str> = vec!["q", "w", "e", "r", "t", "y", "u", "i", "o", "p", "p1"];
        ui.columns(tl_1.len(), |columns| {
            for (index, s) in tl_1.iter().enumerate() {
                self.input_button_ui(s, true, &mut columns[index]);
            }
        });

        let tl_2: Vec<&str> = vec!["a", "s", "d", "f", "g", "h", "j", "k", "l", "l1", "l2"];
        ui.columns(tl_2.len(), |columns| {
            for (index, s) in tl_2.iter().enumerate() {
                self.input_button_ui(s, true, &mut columns[index]);
            }
        });

        let tl_3: Vec<&str> =
            vec![ARROW_FAT_UP, "z", "x", "c", "v", "b", "n", "m", "m1", "m2", BACKSPACE];
        ui.columns(tl_3.len(), |columns| {
            for (index, s) in tl_3.iter().enumerate() {
                if index == 0 {
                    let shift = self.state.shift.load(Ordering::Relaxed);
                    let color = if shift {
                        Colors::yellow_dark()
                    } else {
                        Colors::inactive_text()
                    };
                    self.custom_button_ui(ARROW_FAT_UP.to_string(),
                                           color,
                                           Some(Colors::fill_lite()),
                                           &mut columns[index],
                                          |_, c| {
                                              c.state.shift.store(!shift, Ordering::Relaxed);
                        });
                } else if index == tl_3.len() - 1 {
                    self.custom_button_ui(BACKSPACE.to_string(),
                                           Colors::red(),
                                           Some(Colors::fill_lite()),
                                           &mut columns[index],
                                           |_, c| {
                                               c.state.last_event =
                                                   Arc::new(Some(KeyboardEvent::CLEAR));
                                           });
                } else {
                    self.input_button_ui(s, true, &mut columns[index]);
                }
            }
        });

        button_rect
    }

    /// Draw symbols content returning button [`Rect`].
    fn symbols_ui(&mut self, ui: &mut egui::Ui) -> Rect {
        let mut button_rect = ui.available_rect_before_wrap();
        let tl_0: Vec<&str> = vec!["[", "]", "{", "}", "#", "%", "^", "*", "+", "="];
        ui.columns(tl_0.len(), |columns| {
            for (index, s) in tl_0.iter().enumerate() {
                button_rect = self.input_button_ui(s, false, &mut columns[index]);
            }
        });

        let tl_1: Vec<&str> = vec!["_", "\\", "|", "~", "<", ">", "№", "√", "π", "•"];
        ui.columns(tl_1.len(), |columns| {
            for (index, s) in tl_1.iter().enumerate() {
                self.input_button_ui(s, false, &mut columns[index]);
            }
        });

        let tl_2: Vec<&str> = vec!["-", "/", ":", ";", "(", ")", "`", "&", "@", "\""];
        ui.columns(tl_2.len(), |columns| {
            for (index, s) in tl_2.iter().enumerate() {
                self.input_button_ui(s, false, &mut columns[index]);
            }
        });

        let tl_3: Vec<&str> = vec![".", ",", "?", "!", "€", "£", "¥", "$", "¢", BACKSPACE];
        ui.columns(tl_3.len(), |columns| {
            for (index, s) in tl_3.iter().enumerate() {
                if index == tl_3.len() - 1 {
                    self.custom_button_ui(BACKSPACE.to_string(),
                                           Colors::red(),
                                           Some(Colors::fill_lite()),
                                           &mut columns[index],
                                           |_, c| {
                                               c.state.last_event =
                                                   Arc::new(Some(KeyboardEvent::CLEAR));
                                           });
                } else {
                    self.input_button_ui(s, false, &mut columns[index]);
                }
            }
        });

        button_rect
    }

    /// Draw custom keyboard button.
    fn custom_button_ui(&mut self,
                        s: String,
                        color: Color32,
                        bg: Option<Color32>,
                        ui: &mut egui::Ui,
                        cb: impl FnOnce(String, &mut KeyboardContent)) -> Response {
        ui.vertical_centered_justified(|ui| {
            // Disable expansion on click/hover.
            ui.style_mut().visuals.widgets.hovered.expansion = 0.0;
            ui.style_mut().visuals.widgets.active.expansion = 0.0;
            // Setup fill colors.
            ui.visuals_mut().widgets.inactive.weak_bg_fill = Colors::white_or_black(false);
            ui.visuals_mut().widgets.hovered.weak_bg_fill = Colors::fill_lite();
            ui.visuals_mut().widgets.active.weak_bg_fill = Colors::fill();
            // Setup stroke colors.
            ui.visuals_mut().widgets.inactive.bg_stroke = View::item_stroke();
            ui.visuals_mut().widgets.hovered.bg_stroke = View::item_stroke();
            ui.visuals_mut().widgets.active.bg_stroke = View::hover_stroke();

            let shift = self.state.shift.load(Ordering::Relaxed);
            let label = if shift {
                s.to_uppercase()
            } else {
                s.to_string()
            };
            let mut button = Button::new(RichText::new(label.clone()).size(18.0).color(color))
                .corner_radius(egui::CornerRadius::ZERO);
            if let Some(bg) = bg {
                button = button.fill(bg);
            }
            // Setup long press/touch.
            let long_press = s == BACKSPACE;
            if long_press {
                button = button.sense(Sense::click_and_drag());
            }
            // Draw button.
            let resp = button.ui(ui).on_hover_cursor(CursorIcon::PointingHand);
            if resp.clicked() || resp.long_touched() || resp.dragged() {
                cb(label, self);
            }
        }).response
    }

    /// Draw input button.
    fn input_button_ui(&mut self, s: &str, translate: bool, ui: &mut egui::Ui) -> Rect {
        let value = if translate {
            t!(format!("keyboard.{}", s).as_str(), locale = Self::input_locale().as_str())
        } else {
            s.to_string()
        };
        let rect = self.custom_button_ui(value, Colors::text_button(), None, ui, |l, c| {
            c.state.last_event = Arc::new(Some(KeyboardEvent::TEXT(l)));
            c.state.shift.store(false, Ordering::Relaxed);
        }).rect;
        rect
    }

    /// Get input locale.
    fn input_locale() -> String {
        let english = AppConfig::english_keyboard();
        if english {
            "en".to_string()
        } else {
            AppConfig::locale().unwrap_or("en".to_string())
        }
    }

    /// Check last keyboard input event.
    pub fn consume_event() -> Option<KeyboardEvent> {
        let empty = {
            let r_state = WINDOW_STATE.read();
            r_state.last_event.is_none()
        };
        if !empty {
            let mut w_state = WINDOW_STATE.write();
            let event = w_state.last_event.as_ref().clone().unwrap();
            w_state.last_event = Arc::new(None);
            return Some(event);
        }
        None
    }

    /// Emulate stop of Shift key press.
    pub fn unshift() {
        let r_state = WINDOW_STATE.read();
        r_state.shift.store(false, Ordering::Relaxed);
    }

    /// Reset keyboard window state.
    pub fn reset_window_state() {
        let mut w_state = WINDOW_STATE.write();
        w_state.layout = Arc::new(KeyboardLayout::TEXT);
        // *w_state = KeyboardState::default();
    }
}