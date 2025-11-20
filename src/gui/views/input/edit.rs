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
use egui::{Layout, TextBuffer, TextStyle, Widget, Align, ViewportCommand};
use egui::text_edit::TextEditState;
use lazy_static::lazy_static;
use parking_lot::RwLock;

use crate::gui::Colors;
use crate::gui::icons::{CLIPBOARD_TEXT, COPY, EYE, EYE_SLASH, SCAN};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::input::keyboard::KeyboardContent;
use crate::gui::views::{KeyboardEvent, View};

/// Text input content.
pub struct TextEdit {
    /// View identifier.
    id: egui::Id,
    /// Horizontal text centering is needed.
    h_center: bool,
    /// Focus is needed.
    focus: bool,
    /// Focus request was passed.
    focus_request: bool,
    /// Hide letters and draw button to show/hide letters.
    password: bool,
    /// Show copy button.
    copy: bool,
    /// Show paste button.
    paste: bool,
    /// Show button to scan QR code into text.
    scan_qr: bool,
    /// Scan button was pressed.
    pub scan_pressed: bool,
    /// Tab or Enter keys were pressed to focus on next line.
    pub enter_pressed: bool,
    /// Flag to enter only numbers.
    numeric: bool,
    /// Flag to not show soft keyboard.
    no_soft_keyboard: bool,
}

impl TextEdit {
    /// Default height of [`egui::TextEdit`] view.
    const TEXT_EDIT_HEIGHT: f32 = 41.0;

    pub fn new(id: egui::Id) -> Self {
        Self {
            id,
            h_center: false,
            focus: true,
            focus_request: false,
            password: false,
            copy: false,
            paste: false,
            scan_qr: false,
            scan_pressed: false,
            enter_pressed: false,
            numeric: false,
            no_soft_keyboard: is_android(),
        }
    }

    /// Draw text input content.
    pub fn ui(&mut self, ui: &mut egui::Ui, input: &mut String, cb: &dyn PlatformCallbacks) {
        let mut layout_rect = ui.available_rect_before_wrap();
        layout_rect.set_height(Self::TEXT_EDIT_HEIGHT);
        ui.allocate_ui_with_layout(layout_rect.size(), Layout::right_to_left(Align::Max), |ui| {
            let mut hide_input = false;
            if self.password {
                let show_pass_id = egui::Id::new(self.id).with("_show_pass");
                hide_input = ui.data(|data| {
                    data.get_temp(show_pass_id)
                }).unwrap_or(true);
                // Draw button to show/hide current password.
                let eye_icon = if hide_input { EYE } else { EYE_SLASH };
                View::button_ui(ui, eye_icon.to_string(), Colors::white_or_black(false), |ui| {
                    hide_input = !hide_input;
                    ui.data_mut(|data| {
                        data.insert_temp(show_pass_id, hide_input);
                    });
                });
                ui.add_space(8.0);
            }

            // Setup copy button.
            if self.copy {
                let copy_icon = COPY.to_string();
                View::button(ui, copy_icon, Colors::white_or_black(false), || {
                    cb.copy_string_to_buffer(input.clone());
                });
                ui.add_space(8.0);
            }

            // Setup paste button.
            if self.paste {
                let paste_icon = CLIPBOARD_TEXT.to_string();
                View::button(ui, paste_icon, Colors::white_or_black(false), || {
                    *input = cb.get_string_from_buffer();
                });
                ui.add_space(8.0);
            }

            // Setup scan QR code button.
            if self.scan_qr {
                let scan_icon = SCAN.to_string();
                View::button(ui, scan_icon, Colors::white_or_black(false), || {
                    cb.start_camera();
                    self.scan_pressed = true;
                });
                ui.add_space(8.0);
            }

            let layout_size = ui.available_size();
            ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Min), |ui| {
                // Setup text edit size.
                let mut edit_rect = ui.available_rect_before_wrap();
                edit_rect.set_height(Self::TEXT_EDIT_HEIGHT);

                // Setup focused input value to avoid dismiss when click on keyboard.
                let focused_input_id = egui::Id::new("focused_input_id");
                let focused = ui.data(|data| {
                    data.get_temp(focused_input_id)
                }).unwrap_or(egui::Id::new("")) == self.id;

                // Show text edit.
                let text_edit_resp = egui::TextEdit::singleline(input)
                    .id(self.id)
                    .font(TextStyle::Heading)
                    .min_size(edit_rect.size())
                    .horizontal_align(if self.h_center { Align::Center } else { Align::Min })
                    .vertical_align(Align::Center)
                    .password(hide_input)
                    .cursor_at_end(true)
                    .ui(ui);

                // Setup focus state.
                let clicked = text_edit_resp.clicked();
                if !text_edit_resp.has_focus() &&
                    (self.focus || self.focus_request || clicked || focused) {
                    text_edit_resp.request_focus();
                }

                // Reset keyboard state for newly focused.
                if clicked || self.focus_request {
                    ui.ctx().send_viewport_cmd(ViewportCommand::IMEAllowed(true));
                    KeyboardContent::reset_window_state();
                }

                // Apply text from software input.
                if text_edit_resp.has_focus() {
                    ui.data_mut(|data| {
                        data.insert_temp(focused_input_id, self.id);
                    });
                    self.enter_pressed = self.on_soft_input(ui, self.id, false, input);
                    // Check Enter or Tab keys press.
                    if !self.focus_request {
                        if ui.ctx().input(|i| i.key_pressed(egui::Key::Enter) ||
                            i.key_pressed(egui::Key::Tab)) {
                            self.enter_pressed = true;
                        }
                    }
                    if self.enter_pressed {
                        KeyboardContent::unshift();
                    }
                    if !self.no_soft_keyboard {
                        KeyboardContent::default().window_ui(self.numeric, ui.ctx());
                    }
                }
            });
        });
        // Repaint on Android to handle input from Java code without delays.
        if is_android() {
            ui.ctx().request_repaint();
        }
    }

    /// Apply soft keyboard input data to provided String, returns `true` if Enter was pressed.
    fn on_soft_input(&self, ui: &mut egui::Ui, id: egui::Id, multiline: bool, value: &mut String)
        -> bool {
        let event: Option<KeyboardEvent> = if is_android() {
            let mut w_input = LAST_SOFT_KEYBOARD_EVENT.write();
            w_input.take()
        } else {
            KeyboardContent::consume_event()
        };

        // Handle keyboard input event.
        if let Some(e) = event {
            let mut enter_pressed = false;
            let mut state = TextEditState::load(ui.ctx(), id).unwrap();
            match state.cursor.char_range() {
                None => {}
                Some(range) => {
                    let mut r = range.clone();
                    let mut index = r.primary.index;

                    let selected = r.primary.index != r.secondary.index;
                    let start_select = f32::min(r.primary.index as f32,
                                         r.secondary.index as f32) as usize;
                    let end_select = f32::max(r.primary.index as f32,
                                         r.secondary.index as f32) as usize;
                    match e {
                        KeyboardEvent::TEXT(text) => {
                            if selected {
                                *value = {
                                    let part1: String = value.chars()
                                        .skip(0)
                                        .take(start_select)
                                        .collect();
                                    let part2: String = value.chars()
                                        .skip(end_select)
                                        .take(value.len() - end_select)
                                        .collect();
                                    format!("{}{}{}", part1, text, part2)
                                };
                                index = start_select + 1;
                            } else {
                                value.insert_text(text.as_str(), index);
                                index = index + 1;
                            }
                        }
                        KeyboardEvent::CLEAR => {
                            if selected {
                                *value = {
                                    let part1: String = value.chars()
                                        .skip(0)
                                        .take(start_select)
                                        .collect();
                                    let part2: String = value.chars()
                                        .skip(end_select)
                                        .take(value.len() - end_select)
                                        .collect();
                                    format!("{}{}", part1, part2)
                                };
                                index = start_select;
                            } else if index != 0 {
                                *value = {
                                    let part1: String = value.chars()
                                        .skip(0)
                                        .take(index - 1)
                                        .collect();
                                    let part2: String = value.chars()
                                        .skip(index)
                                        .take(value.len() - index)
                                        .collect();
                                    format!("{}{}", part1, part2)
                                };
                                index = index - 1;
                            }
                        }
                        KeyboardEvent::ENTER => {
                            if multiline {
                                value.insert_text("\n", index);
                                index = index + 1;
                            } else {
                                enter_pressed = true;
                            }
                        }
                    }
                    // Setup cursor index.
                    r.primary.index = index;
                    r.secondary.index = r.primary.index;

                    state.cursor.set_char_range(Some(r));
                    TextEditState::store(state, ui.ctx(), id);
                }
            }
            return enter_pressed;
        }
        false
    }

    /// Center text horizontally.
    pub fn h_center(mut self) -> Self {
        self.h_center = true;
        self
    }

    /// Enable or disable constant focus.
    pub fn focus(mut self, focus: bool) -> Self {
        self.focus = focus;
        self
    }

    /// Focus on field.
    pub fn focus_request(&mut self) {
        self.focus_request = true;
    }

    /// Allow input of numbers only.
    pub fn numeric(mut self) -> Self {
        self.numeric = true;
        self
    }

    /// Hide letters and draw button to show/hide letters.
    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }

    /// Show button to copy text.
    pub fn copy(mut self) -> Self {
        self.copy = true;
        self
    }

    /// Show button to paste text.
    pub fn paste(mut self) -> Self {
        self.paste = true;
        self
    }

    /// Show button to scan QR code to text.
    pub fn scan_qr(mut self) -> Self {
        self.scan_qr = true;
        self.scan_pressed = false;
        self
    }

    /// Do not show soft keyboard for input.
    pub fn no_soft_keyboard(mut self) -> Self {
        self.no_soft_keyboard = true;
        self
    }
}

/// Check if current system is Android.
fn is_android() -> bool {
    egui::os::OperatingSystem::from_target_os() == egui::os::OperatingSystem::Android
}

lazy_static! {
    static ref LAST_SOFT_KEYBOARD_EVENT: Arc<RwLock<Option<KeyboardEvent>>> = Arc::new(RwLock::new(None));
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Callback from Java code with last entered character from soft keyboard.
pub extern "C" fn Java_mw_gri_android_MainActivity_onTextInput(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    char: jni::sys::jstring
) {
    use jni::objects::JString;

    unsafe {
        let j_obj = JString::from_raw(char);
        let j_str = _env.get_string_unchecked(j_obj.as_ref()).unwrap();
        match j_str.to_str() {
            Ok(str) => {
                let mut w_input = LAST_SOFT_KEYBOARD_EVENT.write();
                *w_input = Some(KeyboardEvent::TEXT(str.to_string()));
            }
            Err(_) => {}
        }
    }
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Callback from Java code when Clear key was pressed at soft keyboard.
pub extern "C" fn Java_mw_gri_android_MainActivity_onClearInput(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
) {
    let mut w_input = LAST_SOFT_KEYBOARD_EVENT.write();
    *w_input = Some(KeyboardEvent::CLEAR);
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Callback from Java code when Enter key was pressed at soft keyboard.
pub extern "C" fn Java_mw_gri_android_MainActivity_onEnterInput(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
) {
    let mut w_input = LAST_SOFT_KEYBOARD_EVENT.write();
    *w_input = Some(KeyboardEvent::ENTER);
}