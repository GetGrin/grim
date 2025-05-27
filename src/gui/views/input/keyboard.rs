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

use eframe::emath::Align;
use eframe::epaint::{Margin, Shadow};
use egui::{Align2, Button, Color32, CursorIcon, Layout, Rect, Response, RichText, Vec2, Widget};
use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::gui::icons::{ARROW_FAT_UP, BACKSPACE, GLOBE_SIMPLE, KEY_RETURN};
use crate::gui::views::{KeyboardEvent, KeyboardLayout, View};
use crate::gui::Colors;
use crate::AppConfig;

lazy_static! {
    /// Last input event.
    static ref LAST_EVENT: Arc<RwLock<Option<KeyboardEvent >>> = Arc::new(RwLock::new(None));
    /// Flag to show keyboard [`egui::Window`].
    static ref SHOW_WINDOW: AtomicBool = AtomicBool::new(false);
    /// Flag to enable text shifting.
    static ref UPPERCASE: AtomicBool = AtomicBool::new(false);
    /// Flag to show numeric layout.
    static ref NUMERIC: AtomicBool = AtomicBool::new(false);
}

/// Software keyboard content.
pub struct KeyboardContent {
    /// Keyboard layout.
    layout: KeyboardLayout
}

impl Default for KeyboardContent {
    fn default() -> Self {
        Self {
            layout: KeyboardLayout::TEXT,
        }
    }
}

impl KeyboardContent {
    /// Maximum keyboard content width.
    const MAX_WIDTH: f32 = 600.0;
    /// Maximum numbers layout width.
    const MAX_WIDTH_NUMBERS: f32 = 400.0;

    /// Draw keyboard content as separate [`Window`].
    pub fn window_ui(&mut self, ctx: &egui::Context) {
        if !KeyboardContent::window_showing() {
            self.layout = KeyboardLayout::TEXT;
            return;
        }
        let width = ctx.screen_rect().width();
        let layer_id = egui::Window::new("soft_keyboard")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .min_width(width)
            .default_width(width)
            .anchor(Align2::CENTER_BOTTOM, Vec2::new(0.0, 0.0))
            .frame(egui::Frame {
                shadow: Shadow {
                    offset: Default::default(),
                    blur: 30.0,
                    spread: 3.0,
                    color: Color32::from_black_alpha(32),
                },
                inner_margin:  Margin {
                    left: View::get_left_inset(),
                    right: View::get_right_inset(),
                    top: 1.0,
                    bottom: View::get_bottom_inset(),
                },
                fill: Colors::fill(),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.set_min_width(width);
                // Calculate content width.
                let side_insets = View::get_left_inset() + View::get_right_inset();
                let available_width = width - side_insets;
                let w = f32::min(available_width, if self.layout == KeyboardLayout::NUMBERS {
                    Self::MAX_WIDTH_NUMBERS
                } else {
                    Self::MAX_WIDTH
                });
                // Draw content.
                View::max_width_ui(ui, w, |ui| {
                    self.ui(ui);
                });
            }).unwrap().response.layer_id;

        // Always show keyboard above others windows.
        ctx.move_to_top(layer_id);
    }

    /// Draw keyboard content.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // Setup spacing between buttons.
        ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 0.0);
        // Setup vertical padding inside buttons.
        ui.style_mut().spacing.button_padding = egui::vec2(0.0, 8.0);

        // Set numbers layout type if passed on opening.
        if NUMERIC.load(Ordering::Relaxed) {
            self.layout = KeyboardLayout::NUMBERS;
        } else if self.layout == KeyboardLayout::NUMBERS {
            self.layout = KeyboardLayout::TEXT;
        }

        let button_rect = match self.layout {
            KeyboardLayout::TEXT => Self::text_ui(ui),
            KeyboardLayout::SYMBOLS => Self::symbols_ui(ui),
            KeyboardLayout::NUMBERS => Self::numbers_ui(ui),
        };
        ui.add_space(2.0);

        // Draw bottom keyboard buttons.
        let bottom_size = {
            let mut r = button_rect.clone();
            r.set_width(ui.available_width());
            r.size()
        };
        let button_width = ui.available_width() / match self.layout {
            KeyboardLayout::TEXT => 11.0,
            KeyboardLayout::SYMBOLS => 10.0,
            KeyboardLayout::NUMBERS => 4.0,
        };
        ui.allocate_ui_with_layout(bottom_size, Layout::right_to_left(Align::Center), |ui| {
            if self.layout == KeyboardLayout::NUMBERS {
                ui.horizontal_centered(|ui| {
                    ui.set_max_width(button_width * 2.0 + 1.0);
                    Self::custom_button_ui(KEY_RETURN.to_string(),
                                           Colors::white_or_black(false),
                                           Some(Colors::green()),
                                           ui,
                                           |_| {
                                               Self::input_event(KeyboardEvent::ENTER);
                                           });
                });
                ui.horizontal_centered(|ui| {
                    ui.set_max_width(button_width);
                    Self::input_button_ui("0", true, ui);
                });
                ui.horizontal_centered(|ui| {
                    ui.set_max_width(button_width);
                    Self::input_button_ui(".", false, ui);
                });
            } else {
                // Enter key input.
                ui.horizontal_centered(|ui| {
                    ui.set_max_width(button_width * 2.0 + 1.0);
                    Self::custom_button_ui(KEY_RETURN.to_string(),
                                           Colors::white_or_black(false),
                                           Some(Colors::green()),
                                           ui,
                                           |_| {
                                               Self::input_event(KeyboardEvent::ENTER);
                                           });
                });
                // Backspace key input.
                ui.horizontal_centered(|ui| {
                    ui.set_max_width(button_width * 1.0);
                    Self::custom_button_ui(BACKSPACE.to_string(),
                                           Colors::red(),
                                           Some(Colors::fill_lite()),
                                           ui,
                                           |_| {
                                               Self::input_event(KeyboardEvent::CLEAR);
                                           });
                });
                // Space key input.
                ui.horizontal_centered(|ui| {
                    ui.set_max_width(button_width * 4.0);
                    Self::custom_button_ui(" ".to_string(), Colors::inactive_text(), None, ui, |l| {
                        Self::input_event(KeyboardEvent::TEXT(l));
                    });
                });
                if self.layout == KeyboardLayout::TEXT {
                    // Switch to english and back.
                    ui.horizontal_centered(|ui| {
                        ui.set_max_width(button_width);
                        Self::custom_button_ui(GLOBE_SIMPLE.to_string(),
                                               Colors::text_button(),
                                               Some(Colors::fill_lite()),
                                               ui,
                                               |_| {
                                                   AppConfig::toggle_english_keyboard()
                                               });
                    });
                    // Shift key input.
                    ui.horizontal_centered(|ui| {
                        ui.set_max_width(button_width);
                        let uppercase = UPPERCASE.load(Ordering::Relaxed);
                        let color = if uppercase {
                            Colors::yellow_dark()
                        } else {
                            Colors::inactive_text()
                        };
                        Self::custom_button_ui(ARROW_FAT_UP.to_string(),
                                               color,
                                               Some(Colors::fill_lite()),
                                               ui, |_| {
                                UPPERCASE.store(!uppercase, Ordering::Relaxed);
                            });
                    });
                }
                // Switch to symbols and back.
                ui.horizontal_centered(|ui| {
                    let label = if self.layout == KeyboardLayout::SYMBOLS {
                        let q = t!("keyboard.q", locale = Self::locale().as_str());
                        let w = t!("keyboard.w", locale = Self::locale().as_str());
                        let e = t!("keyboard.e", locale = Self::locale().as_str());
                        format!("{}{}{}", q, w, e).to_uppercase()
                    } else {
                        "!@ツ".to_string()
                    };
                    let mut mode = self.layout.clone();
                    Self::custom_button_ui(label,
                                           Colors::text(false),
                                           Some(Colors::fill_lite()),
                                           ui,
                                           |_| {
                                               if self.layout == KeyboardLayout::SYMBOLS {
                                                   mode = KeyboardLayout::TEXT;
                                               } else {
                                                   mode = KeyboardLayout::SYMBOLS;
                                               }
                                           });
                    self.layout = mode;
                });
            }
        });
    }

    /// Draw numbers content returning button [`Rect`].
    fn numbers_ui(ui: &mut egui::Ui) -> Rect {
        let mut button_rect = ui.available_rect_before_wrap();
        let tl_0: Vec<&str> = vec!["1", "2", "3", "-"];
        ui.columns(tl_0.len(), |columns| {
            for (index, s) in tl_0.iter().enumerate() {
                let last = index == tl_0.len() - 1;
                button_rect = Self::input_button_ui(s, !last, &mut columns[index]);
            }
        });
        ui.add_space(2.0);

        let tl_1: Vec<&str> = vec!["4", "5", "6", "+"];
        ui.columns(tl_1.len(), |columns| {
            for (index, s) in tl_1.iter().enumerate() {
                let last = index == tl_1.len() - 1;
                Self::input_button_ui(s, !last, &mut columns[index]);
            }
        });
        ui.add_space(2.0);

        let tl_2: Vec<&str> = vec!["7", "8", "9", BACKSPACE];
        ui.columns(tl_2.len(), |columns| {
            for (index, s) in tl_2.iter().enumerate() {
                if index == tl_2.len() - 1 {
                    Self::custom_button_ui(BACKSPACE.to_string(),
                                           Colors::red(),
                                           Some(Colors::fill_lite()),
                                           &mut columns[index],
                                           |_| {
                                               Self::input_event(KeyboardEvent::CLEAR);
                                           });
                } else {
                    Self::input_button_ui(s, true, &mut columns[index]);
                }
            }
        });

        button_rect
    }

    /// Draw text content returning button [`Rect`].
    fn text_ui(ui: &mut egui::Ui) -> Rect {
        let mut button_rect = ui.available_rect_before_wrap();
        let tl_0: Vec<&str> = vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "01"];
        ui.columns(tl_0.len(), |columns| {
            for (index, s) in tl_0.iter().enumerate() {
                button_rect = Self::input_button_ui(s, true, &mut columns[index]);
            }
        });
        ui.add_space(2.0);

        let tl_1: Vec<&str> = vec!["q", "w", "e", "r", "t", "y", "u", "i", "o", "p", "p1"];
        ui.columns(tl_1.len(), |columns| {
            for (index, s) in tl_1.iter().enumerate() {
                Self::input_button_ui(s, true, &mut columns[index]);
            }
        });
        ui.add_space(2.0);

        let tl_2: Vec<&str> = vec!["a", "s", "d", "f", "g", "h", "j", "k", "l", "l1", "l2"];
        ui.columns(tl_2.len(), |columns| {
            for (index, s) in tl_2.iter().enumerate() {
                Self::input_button_ui(s, true, &mut columns[index]);
            }
        });
        ui.add_space(2.0);

        let tl_3: Vec<&str> = vec!["z1", "z", "x", "c", "v", "b", "n", "m", "m1", "m2", "m3"];
        ui.columns(tl_3.len(), |columns| {
            for (index, s) in tl_3.iter().enumerate() {
                Self::input_button_ui(s, true, &mut columns[index]);
            }
        });

        button_rect
    }

    /// Draw symbols content returning button [`Rect`].
    fn symbols_ui(ui: &mut egui::Ui) -> Rect {
        let mut button_rect = ui.available_rect_before_wrap();
        let tl_0: Vec<&str> = vec!["[", "]", "{", "}", "#", "%", "^", "*", "+", "="];
        ui.columns(tl_0.len(), |columns| {
            for (index, s) in tl_0.iter().enumerate() {
                button_rect = Self::input_button_ui(s, false, &mut columns[index]);
            }
        });
        ui.add_space(2.0);

        let tl_1: Vec<&str> = vec!["_", "\\", "|", "~", "<", ">", "№", "√", "π", "•"];
        ui.columns(tl_1.len(), |columns| {
            for (index, s) in tl_1.iter().enumerate() {
                Self::input_button_ui(s, false, &mut columns[index]);
            }
        });
        ui.add_space(2.0);

        let tl_2: Vec<&str> = vec!["-", "/", ":", ";", "(", ")", "`", "&", "@", "\""];
        ui.columns(tl_2.len(), |columns| {
            for (index, s) in tl_2.iter().enumerate() {
                Self::input_button_ui(s, false, &mut columns[index]);
            }
        });
        ui.add_space(2.0);

        let tl_3: Vec<&str> = vec![".", ",", "?", "!", "€", "£", "¥", "$", "¢", "ツ"];
        ui.columns(tl_3.len(), |columns| {
            for (index, s) in tl_3.iter().enumerate() {
                Self::input_button_ui(s, false, &mut columns[index]);
            }
        });

        button_rect
    }

    /// Draw custom keyboard button.
    fn custom_button_ui(s: String,
                        color: Color32,
                        bg: Option<Color32>,
                        ui: &mut egui::Ui,
                        mut cb: impl FnMut(String)) -> Response {
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

            let label = if UPPERCASE.load(Ordering::Relaxed) {
                s.to_uppercase()
            } else {
                s.to_string()
            };
            let mut button = Button::new(RichText::new(label.clone()).size(17.0).color(color))
                .rounding(egui::Rounding::ZERO);
            if let Some(bg) = bg {
                button = button.fill(bg);
            }
            let resp = button.ui(ui).on_hover_cursor(CursorIcon::PointingHand);
            if resp.clicked() {
                (cb)(label);
            }
        }).response
    }

    /// Draw input button.
    fn input_button_ui(s: &str, translate: bool, ui: &mut egui::Ui) -> Rect {
        let value = if translate {
            t!(format!("keyboard.{}", s).as_str(), locale = Self::locale().as_str())
        } else {
            s.to_string()
        };
        let rect = Self::custom_button_ui(value, Colors::text_button(), None, ui, |l| {
            Self::input_event(KeyboardEvent::TEXT(l));
            UPPERCASE.store(false, Ordering::Relaxed);
        }).rect;
        rect
    }

    /// Get input locale.
    fn locale() -> String {
        let english = AppConfig::english_keyboard();
        if english {
            "en".to_string()
        } else {
            AppConfig::locale().unwrap_or("en".to_string())
        }
    }

    /// Save keyboard event to consume later.
    fn input_event(event: KeyboardEvent) {
        let mut w_input = LAST_EVENT.write();
        *w_input = Some(event);
    }

    /// Check last keyboard input event.
    pub fn consume_event() -> Option<KeyboardEvent> {
        let empty = {
            let r_input = LAST_EVENT.read();
            r_input.is_none()
        };
        if !empty {
            let mut w_input = LAST_EVENT.write();
            let res = w_input.clone();
            *w_input = None;
            return res;
        }
        None
    }

    /// Check if keyboard is showing.
    pub fn window_showing() -> bool {
        SHOW_WINDOW.load(Ordering::Relaxed)
    }

    /// Show keyboard.
    pub fn show_window(numeric: bool) {
        NUMERIC.store(numeric, Ordering::Relaxed);
        SHOW_WINDOW.store(true, Ordering::Relaxed);
    }

    /// Emulate Shift key pressing.
    pub fn shift() {
        UPPERCASE.store(true, Ordering::Relaxed);
    }

    /// Emulate Shift key pressing.
    pub fn unshift() {
        UPPERCASE.store(false, Ordering::Relaxed);
    }

    /// Hide keyboard.
    pub fn hide() {
        SHOW_WINDOW.store(false, Ordering::Relaxed);
    }
}