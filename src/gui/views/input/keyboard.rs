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

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use eframe::emath::Align;
use eframe::epaint::{Margin, Shadow};
use egui::{Align2, Button, Color32, CursorIcon, Layout, Rect, Response, RichText, Stroke, Vec2, Widget};
use parking_lot::RwLock;
use lazy_static::lazy_static;
use crate::AppConfig;
use crate::gui::Colors;
use crate::gui::icons::{ARROW_FAT_UP, BACKSPACE, KEY_RETURN, TRANSLATE};
use crate::gui::views::{KeyboardEvent, KeyboardLayout, View};

lazy_static! {
    /// Last input event.
    static ref LAST_EVENT: Arc<RwLock<Option<KeyboardEvent >>> = Arc::new(RwLock::new(None));
    /// Flag to show keyboard.
    static ref SHOW_KEYBOARD: AtomicBool = AtomicBool::new(false);
    /// Flag to enable text shifting.
    static ref UPPERCASE: AtomicBool = AtomicBool::new(false);
    /// Flag to show numeric layout.
    static ref NUMERIC: AtomicBool = AtomicBool::new(false);
}

/// Software keyboard content.
pub struct KeyboardContent {
    /// Keyboard layout.
    mode: KeyboardLayout
}

impl Default for KeyboardContent {
    fn default() -> Self {
        Self {
            mode: KeyboardLayout::TEXT,
        }
    }
}

impl KeyboardContent {
    /// Maximum keyboard content width.
    const MAX_WIDTH: f32 = 600.0;
    /// Buttons content margin.
    const MARGIN: f32 = 5.0;

    /// Draw keyboard content as separate [`Window`].
    pub fn window_ui(&mut self, ctx: &egui::Context) {
        if !KeyboardContent::showing() {
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
                    left: View::get_left_inset() + Self::MARGIN,
                    right: View::get_right_inset() + Self::MARGIN,
                    top: Self::MARGIN,
                    bottom: View::get_bottom_inset() + Self::MARGIN,
                },
                fill: Colors::fill(),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.set_min_width(width);
                // Calculate content width.
                let side_insets = View::get_left_inset() + View::get_right_inset();
                let available_width = width - (side_insets + Self::MARGIN * 2.0);
                let w = f32::min(available_width, Self::MAX_WIDTH);
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
        ui.style_mut().spacing.item_spacing = egui::vec2(Self::MARGIN, 0.0);
        // Setup vertical padding inside buttons.
        ui.style_mut().spacing.button_padding = egui::vec2(Self::MARGIN, 8.0);

        let button_rect = match self.mode {
            KeyboardLayout::TEXT => Self::text_ui(ui),
            KeyboardLayout::SYMBOLS => Self::symbols_ui(ui),
        };
        ui.add_space(View::TAB_ITEMS_PADDING);

        // Draw bottom keyboard buttons.
        let bottom_size = {
            let mut r = button_rect.clone();
            r.set_width(ui.available_width());
            r.size()
        };
        let button_width = button_rect.width();
        ui.allocate_ui_with_layout(bottom_size, Layout::right_to_left(Align::Max), |ui| {
            ui.horizontal_centered(|ui| {
                ui.set_max_width(button_width * 2.0 + Self::MARGIN);
                Self::button_ui(KEY_RETURN,
                               Colors::white_or_black(false),
                               Some(Colors::green()),
                               ui,
                               |_| {
                                   Self::input_event(KeyboardEvent::ENTER);
                               });
            });
            ui.horizontal_centered(|ui| {
                ui.set_max_width(button_width * 5.0 + 4.0 * Self::MARGIN);
                Self::button_ui(" ", Colors::inactive_text(), None, ui, |l| {
                    Self::input_event(KeyboardEvent::TEXT(l));
                });
            });
            ui.horizontal_centered(|ui| {
                ui.set_max_width(button_width);
                Self::button_ui(TRANSLATE,
                               Colors::text_button(),
                               Some(Colors::fill_lite()),
                               ui,
                               |_| {
                                   AppConfig::toggle_english_keyboard()
                               });
            });
            ui.horizontal_centered(|ui| {
                let label = if self.mode == KeyboardLayout::SYMBOLS {
                    "ABC"
                } else {
                    "?/ツ"
                };
                let mut mode = self.mode.clone();
                Self::button_ui(label, Colors::text_button(), Some(Colors::fill_lite()), ui, |_| {
                    if self.mode == KeyboardLayout::SYMBOLS {
                        mode = KeyboardLayout::TEXT;
                    } else {
                        mode = KeyboardLayout::SYMBOLS;
                    }
                });
                self.mode = mode;
            });
        });
    }

    /// Draw text content returning button [`Rect`].
    fn text_ui(ui: &mut egui::Ui) -> Rect {
        let mut button_rect = ui.available_rect_before_wrap();
        let tl_0: Vec<&str> = vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"];
        ui.columns(tl_0.len(), |columns| {
            for (index, s) in tl_0.iter().enumerate() {
                button_rect = Self::input_button_ui(s, &mut columns[index]);
            }
        });
        ui.add_space(View::TAB_ITEMS_PADDING);

        let tl_1: Vec<&str> = vec!["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"];
        ui.columns(tl_1.len(), |columns| {
            for (index, s) in tl_1.iter().enumerate() {
                Self::input_button_ui(s, &mut columns[index]);
            }
        });
        ui.add_space(View::TAB_ITEMS_PADDING);

        let tl_2: Vec<&str> = vec!["a", "s", "d", "f", "g", "h", "j", "k", "l", ":"];
        ui.columns(tl_2.len(), |columns| {
            for (index, s) in tl_2.iter().enumerate() {
                Self::input_button_ui(s, &mut columns[index]);
            }
        });
        ui.add_space(View::TAB_ITEMS_PADDING);

        let tl_3: Vec<&str> = vec![ARROW_FAT_UP, "z", "x", "c", "v", "b", "n", "m", ".", BACKSPACE];
        ui.columns(tl_3.len(), |columns| {
            for (index, s) in tl_3.iter().enumerate() {
                if index == 0 {
                    // Check for shift input.
                    let uppercase = UPPERCASE.load(Ordering::Relaxed);
                    let color = if uppercase {
                        Colors::yellow_dark()
                    } else {
                        Colors::inactive_text()
                    };
                    Self::button_ui(s, color, Some(Colors::fill_lite()), &mut columns[index], |_| {
                        UPPERCASE.store(!uppercase, Ordering::Relaxed);
                    });
                } else if index == tl_3.len() - 1 {
                    // Check for backspace input.
                    Self::button_ui(s,
                                   Colors::red(),
                                   Some(Colors::fill_lite()),
                                   &mut columns[index],
                                   |_| {
                                       Self::input_event(KeyboardEvent::CLEAR);
                                   });
                } else {
                    Self::input_button_ui(s, &mut columns[index]);
                }
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
                button_rect = Self::input_button_ui(s, &mut columns[index]);
            }
        });
        ui.add_space(View::TAB_ITEMS_PADDING);

        let tl_1: Vec<&str> = vec!["_", "\\", "|", "~", "<", ">", "`", "√", "π", "•"];
        ui.columns(tl_1.len(), |columns| {
            for (index, s) in tl_1.iter().enumerate() {
                Self::input_button_ui(s, &mut columns[index]);
            }
        });
        ui.add_space(View::TAB_ITEMS_PADDING);

        let tl_2: Vec<&str> = vec!["-", "/", ":", ";", "(", ")", "$", "&", "@", "\""];
        ui.columns(tl_2.len(), |columns| {
            for (index, s) in tl_2.iter().enumerate() {
                Self::input_button_ui(s, &mut columns[index]);
            }
        });
        ui.add_space(View::TAB_ITEMS_PADDING);

        let tl_3: Vec<&str> = vec![".", ",", "?", "!", "€", "£", "¥", "¢", "ツ", BACKSPACE];
        ui.columns(tl_3.len(), |columns| {
            for (index, s) in tl_3.iter().enumerate() {
                if index == tl_3.len() - 1 {
                    // Check for backspace input.
                    Self::button_ui(s,
                                   Colors::red(),
                                   Some(Colors::fill_lite()),
                                   &mut columns[index], |_| {
                            Self::input_event(KeyboardEvent::CLEAR);
                        });
                } else {
                    Self::input_button_ui(s, &mut columns[index]);
                }
            }
        });

        button_rect
    }

    /// Draw keyboard button.
    fn button_ui(s: &str,
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
            ui.visuals_mut().widgets.hovered.bg_stroke = View::hover_stroke();
            ui.visuals_mut().widgets.active.bg_stroke = Stroke::NONE;

            let label = if UPPERCASE.load(Ordering::Relaxed) {
                s.to_uppercase()
            } else {
                s.to_string()
            };
            let mut button = Button::new(RichText::new(label.clone()).size(17.0).color(color));
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
    fn input_button_ui(s: &str, ui: &mut egui::Ui) -> Rect {
        let rect = Self::button_ui(s, Colors::text_button(), None, ui, |l| {
            Self::input_event(KeyboardEvent::TEXT(l));
            UPPERCASE.store(false, Ordering::Relaxed);
        }).rect;
        rect
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
    pub fn showing() -> bool {
        SHOW_KEYBOARD.load(Ordering::Relaxed)
    }

    /// Show keyboard.
    pub fn show(numeric: bool) {
        NUMERIC.store(numeric, Ordering::Relaxed);
        SHOW_KEYBOARD.store(true, Ordering::Relaxed);
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
        SHOW_KEYBOARD.store(false, Ordering::Relaxed);
    }
}