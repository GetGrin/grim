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

use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use parking_lot::RwLock;
use lazy_static::lazy_static;

use egui::{Align, Button, CursorIcon, Layout, lerp, PointerState, Rect, Response, Rgba, RichText, Sense, SizeHint, Spinner, TextBuffer, TextStyle, TextureHandle, TextureOptions, Widget, UiBuilder};
use egui::epaint::{Color32, FontId, PathShape, PathStroke, RectShape, Rounding, Stroke};
use egui::epaint::text::TextWrapping;
use egui::load::SizedTexture;
use egui::os::OperatingSystem;
use egui::text::{LayoutJob, TextFormat};
use egui::text_edit::TextEditState;
use egui_extras::image::load_svg_bytes_with_size;

use crate::AppConfig;
use crate::gui::Colors;
use crate::gui::icons::{CHECK_SQUARE, CLIPBOARD_TEXT, COPY, EYE, EYE_SLASH, SCAN, SQUARE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::types::{LinePosition, TextEditOptions};

pub struct View;

impl View {
    /// Check if current platform is desktop
    pub fn is_desktop() -> bool {
        let os = OperatingSystem::from_target_os();
        os != OperatingSystem::Android && os != OperatingSystem::IOS
    }

    /// Format timestamp in seconds with local UTC offset.
    pub fn format_time(ts: i64) -> String {
        let utc_offset = chrono::Local::now().offset().local_minus_utc();
        let utc_time = ts + utc_offset as i64;
        let tx_time = chrono::DateTime::from_timestamp(utc_time, 0).unwrap();
        tx_time.format("%d/%m/%Y %H:%M:%S").to_string()
    }

    /// Get default stroke around views.
    pub fn default_stroke() -> Stroke {
        Stroke { width: 1.0, color: Colors::stroke() }
    }

    /// Get default stroke around item buttons.
    pub fn item_stroke() -> Stroke {
        Stroke { width: 1.0, color: Colors::item_stroke() }
    }

    /// Get stroke for hovered items and buttons.
    pub fn hover_stroke() -> Stroke {
        Stroke { width: 1.0, color: Colors::item_hover() }
    }

    /// Draw content with maximum width value.
    pub fn max_width_ui(ui: &mut egui::Ui,
                        max_width: f32,
                        add_content: impl FnOnce(&mut egui::Ui)) {
        // Setup content width.
        let mut width = ui.available_width();
        if width == 0.0 {
            return;
        }
        let mut rect = ui.available_rect_before_wrap();
        width = f32::min(width, max_width);
        rect.set_width(width);

        // Draw content.
        ui.vertical_centered(|ui| {
            ui.allocate_ui(rect.size(), |ui| {
                (add_content)(ui);
            });
        });
    }

    /// Get width and height of app window.
    pub fn window_size(ctx: &egui::Context) -> (f32, f32) {
        let rect = ctx.screen_rect();
        (rect.width(), rect.height())
    }

    /// Callback on Enter key press event.
    pub fn on_enter_key(ui: &mut egui::Ui, cb: impl FnOnce()) {
        if ui.ctx().input(|i| i.key_pressed(egui::Key::Enter)) {
            (cb)();
        }
    }

    /// Calculate margin for far left view based on display insets (cutouts).
    pub fn far_left_inset_margin(ui: &mut egui::Ui) -> f32 {
        if ui.available_rect_before_wrap().min.x == 0.0 {
            Self::get_left_inset()
        } else {
            0.0
        }
    }

    /// Calculate margin for far left view based on display insets (cutouts).
    pub fn far_right_inset_margin(ui: &mut egui::Ui) -> f32 {
        let container_width = ui.available_rect_before_wrap().max.x as i32;
        let window_size = Self::window_size(ui.ctx());
        let display_width = window_size.0 as i32;
        // Means end of the screen.
        if container_width == display_width {
            Self::get_right_inset()
        } else {
            0.0
        }
    }

    /// Cut long text with ﹍ character.
    fn ellipsize(text: String, size: f32, color: Color32) -> LayoutJob {
        let mut job = LayoutJob::single_section(text, TextFormat {
            font_id: FontId::proportional(size), color, ..Default::default()
        });
        job.wrap = TextWrapping {
            max_rows: 1,
            break_anywhere: true,
            overflow_character: Option::from('﹍'),
            ..Default::default()
        };
        job
    }

    /// Draw ellipsized text.
    pub fn ellipsize_text(ui: &mut egui::Ui, text: String, size: f32, color: Color32) {
        ui.label(Self::ellipsize(text, size, color));
    }

    /// Draw animated ellipsized text.
    pub fn animate_text(ui: &mut egui::Ui, text: String, size: f32, color: Color32, animate: bool) {
        // Setup text color animation if needed.
        let (dark, bright) = (0.3, 1.0);
        let color_factor = if animate {
            lerp(dark..=bright, ui.input(|i| i.time).cos().abs()) as f32
        } else {
            bright as f32
        };

        // Draw subtitle text.
        let sub_color_rgba = Rgba::from(color) * color_factor;
        let sub_color = Color32::from(sub_color_rgba);
        View::ellipsize_text(ui, text, size, sub_color);

        // Repaint delay based on animation status.
        if animate {
            ui.ctx().request_repaint();
        }
    }

    /// Draw horizontally centered sub-title with space below.
    pub fn sub_title(ui: &mut egui::Ui, text: String) {
        ui.vertical_centered_justified(|ui| {
            ui.label(RichText::new(text.to_uppercase()).size(16.0).color(Colors::text(false)));
        });
        ui.add_space(4.0);
    }

    /// Temporary click optimization for touch screens, return `true` if it was clicked.
    fn touched(ui: &mut egui::Ui, resp: Response) -> bool {
        let drag_resp = resp.interact(Sense::click_and_drag());
        // Clear pointer event if dragging is out of button area
        if drag_resp.dragged() && !ui.rect_contains_pointer(drag_resp.rect) {
            ui.input_mut(|i| i.pointer = PointerState::default());
        }
        if drag_resp.drag_stopped() || drag_resp.clicked() || drag_resp.secondary_clicked() {
            return true;
        }
        false
    }

    /// Draw big size title button.
    pub fn title_button_big(ui: &mut egui::Ui, icon: &str, action: impl FnOnce(&mut egui::Ui)) {
        Self::title_button(ui, 22.0, icon, action);
    }

    /// Draw small size title button.
    pub fn title_button_small(ui: &mut egui::Ui, icon: &str, action: impl FnOnce(&mut egui::Ui)) {
        Self::title_button(ui, 16.0, icon, action);
    }

    /// Draw title button with transparent background color, contains only icon.
    fn title_button(ui: &mut egui::Ui, size: f32, icon: &str, action: impl FnOnce(&mut egui::Ui)) {
        ui.scope(|ui| {
            // Disable strokes.
            ui.style_mut().visuals.widgets.inactive.bg_stroke = Stroke::NONE;
            ui.style_mut().visuals.widgets.hovered.bg_stroke = Stroke::NONE;
            ui.style_mut().visuals.widgets.active.bg_stroke = Stroke::NONE;
            ui.style_mut().visuals.widgets.active.rounding = Rounding::default();
            ui.style_mut().visuals.widgets.active.expansion = 0.0;

            // Setup text.
            let wt = RichText::new(icon.to_string()).size(size).color(Colors::title(true));
            // Draw button.
            let br = Button::new(wt)
                .fill(Colors::TRANSPARENT)
                .ui(ui)
                .on_hover_cursor(CursorIcon::PointingHand);
            br.surrender_focus();
            if Self::touched(ui, br) {
                (action)(ui);
            }
        });
    }

    /// Padding for tab items.
    pub const TAB_ITEMS_PADDING: f32 = 5.0;

    /// Tab button with white background fill color, contains only icon.
    pub fn tab_button(ui: &mut egui::Ui,
                      icon: &str,
                      active: bool,
                      action: impl FnOnce(&mut egui::Ui)) {
        ui.scope(|ui| {
            let text_color = match active {
                true => Colors::title(false),
                false => Colors::text(false)
            };

            let mut button = Button::new(RichText::new(icon).size(22.0).color(text_color));

            if !active {
                // Disable expansion on click/hover.
                ui.style_mut().visuals.widgets.hovered.expansion = 0.0;
                ui.style_mut().visuals.widgets.active.expansion = 0.0;
                // Setup fill colors.
                ui.visuals_mut().widgets.inactive.weak_bg_fill = Colors::white_or_black(false);
                ui.visuals_mut().widgets.hovered.weak_bg_fill = Colors::fill_lite();
                ui.visuals_mut().widgets.active.weak_bg_fill = Colors::fill();
                // Setup stroke colors.
                ui.visuals_mut().widgets.inactive.bg_stroke = Self::default_stroke();
                ui.visuals_mut().widgets.hovered.bg_stroke = Self::hover_stroke();
                ui.visuals_mut().widgets.active.bg_stroke = Self::item_stroke();
            } else {
                button = button.fill(Colors::fill()).stroke(Stroke::NONE);
            }

            let br = button.ui(ui).on_hover_cursor(CursorIcon::PointingHand);
            br.surrender_focus();
            if Self::touched(ui, br) {
                (action)(ui);
            }
        });
    }

    /// Draw [`Button`] with specified background fill and text color.
    fn button_resp(ui: &mut egui::Ui, text: String, text_color: Color32, bg: Color32) -> Response {
        let button_text = Self::ellipsize(text.to_uppercase(), 17.0, text_color);
        Button::new(button_text)
            .stroke(Self::default_stroke())
            .fill(bg)
            .ui(ui)
            .on_hover_cursor(CursorIcon::PointingHand)
    }

    /// Draw [`Button`] with specified background fill color and default text color.
    pub fn button(ui: &mut egui::Ui, text: String, fill: Color32, action: impl FnOnce()) {
        let br = Self::button_resp(ui, text, Colors::text_button(), fill);
        if Self::touched(ui, br) {
            (action)();
        }
    }

    /// Draw [`Button`] with specified background fill color and text color.
    pub fn colored_text_button(ui: &mut egui::Ui,
                               text: String,
                               text_color: Color32,
                               fill: Color32,
                               action: impl FnOnce()) {
        let br = Self::button_resp(ui, text, text_color, fill);
        if Self::touched(ui, br) {
            (action)();
        }
    }

    /// Draw [`Button`] with specified background fill color and text color.
    pub fn colored_text_button_ui(ui: &mut egui::Ui,
                                  text: String,
                                  text_color: Color32,
                                  fill: Color32,
                                  action: impl FnOnce(&mut egui::Ui)) {
        let br = Self::button_resp(ui, text, text_color, fill);
        if Self::touched(ui, br) {
            (action)(ui);
        }
    }

    /// Draw gold action [`Button`].
    pub fn action_button(ui: &mut egui::Ui,
                         text: String, action: impl FnOnce()) {
        Self::colored_text_button(ui, text, Colors::title(true), Colors::gold(), action);
    }

    /// Draw [`Button`] with specified background fill color and ui at callback.
    pub fn button_ui(ui: &mut egui::Ui,
                     text: String,
                     fill: Color32,
                     action: impl FnOnce(&mut egui::Ui)) {
        let button_text = Self::ellipsize(text.to_uppercase(), 17.0, Colors::text_button());
        let br = Button::new(button_text)
            .stroke(Self::default_stroke())
            .fill(fill)
            .ui(ui)
            .on_hover_cursor(CursorIcon::PointingHand);
        if Self::touched(ui, br) {
            (action)(ui);
        }
    }

    /// Draw list item [`Button`] with provided rounding.
    pub fn item_button(ui: &mut egui::Ui,
                       rounding: Rounding,
                       text: &'static str,
                       color: Option<Color32>,
                       action: impl FnOnce()) {
        // Setup button size.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_width(42.0);
        let button_size = rect.size();

        ui.scope(|ui| {
            // Setup padding for item buttons.
            ui.style_mut().spacing.button_padding = egui::vec2(14.0, 0.0);
            // Disable expansion on click/hover.
            ui.style_mut().visuals.widgets.hovered.expansion = 0.0;
            ui.style_mut().visuals.widgets.active.expansion = 0.0;
            // Setup fill colors.
            ui.visuals_mut().widgets.inactive.weak_bg_fill = Colors::white_or_black(false);
            ui.visuals_mut().widgets.hovered.weak_bg_fill = Colors::fill_lite();
            ui.visuals_mut().widgets.active.weak_bg_fill = Colors::fill();
            // Disable strokes.
            ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::NONE;
            ui.visuals_mut().widgets.hovered.bg_stroke = Stroke::NONE;
            ui.visuals_mut().widgets.active.bg_stroke = Stroke::NONE;

            // Setup button text color.
            let text_color = if let Some(c) = color { c } else { Colors::item_button() };

            // Show button.
            let br = Button::new(RichText::new(text).size(20.0).color(text_color))
                .rounding(rounding)
                .min_size(button_size)
                .ui(ui)
                .on_hover_cursor(CursorIcon::PointingHand);
            br.surrender_focus();
            if Self::touched(ui, br.clone()) {
                (action)();
            }

            // Draw stroke.
            let r = {
                let mut r = ui.available_rect_before_wrap();
                r.min = br.rect.min;
                r.min.x += 0.5;
                r
            };
            Self::line(ui, LinePosition::LEFT, &r, Colors::item_stroke());
        });
    }

    /// Default height of [`egui::TextEdit`] view.
    const TEXT_EDIT_HEIGHT: f32 = 37.0;

    /// Draw [`egui::TextEdit`] widget.
    pub fn text_edit(ui: &mut egui::Ui,
                     cb: &dyn PlatformCallbacks,
                     value: &mut String,
                     options: &mut TextEditOptions) {
        let mut layout_rect = ui.available_rect_before_wrap();
        layout_rect.set_height(Self::TEXT_EDIT_HEIGHT);
        ui.allocate_ui_with_layout(layout_rect.size(), Layout::right_to_left(Align::Center), |ui| {
            // Setup password button.
            let mut show_pass = false;
            if options.password {
                // Set password button state value.
                let show_pass_id = egui::Id::new(options.id).with("_show_pass");
                show_pass = ui.data(|data| {
                    data.get_temp(show_pass_id)
                }).unwrap_or(true);
                // Draw button to show/hide current password.
                let eye_icon = if show_pass { EYE } else { EYE_SLASH };
                let mut changed = false;
                View::button(ui, eye_icon.to_string(), Colors::white_or_black(false), || {
                    show_pass = !show_pass;
                    changed = true;
                });
                // Save state if changed.
                if changed {
                    ui.data_mut(|data| {
                        data.insert_temp(show_pass_id, show_pass);
                    });
                }
                ui.add_space(8.0);
            }

            // Setup copy button.
            if options.copy {
                let copy_icon = COPY.to_string();
                View::button(ui, copy_icon, Colors::white_or_black(false), || {
                    cb.copy_string_to_buffer(value.clone());
                });
                ui.add_space(8.0);
            }

            // Setup paste button.
            if options.paste {
                let paste_icon = CLIPBOARD_TEXT.to_string();
                View::button(ui, paste_icon, Colors::white_or_black(false), || {
                    *value = cb.get_string_from_buffer();
                });
                ui.add_space(8.0);
            }

            // Setup scan QR code button.
            if options.scan_qr {
                let scan_icon = SCAN.to_string();
                View::button(ui, scan_icon, Colors::white_or_black(false), || {
                    cb.start_camera();
                    options.scan_pressed = true;
                });
                ui.add_space(8.0);
            }

            let layout_size = ui.available_size();
            ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                // Setup text edit size.
                let mut edit_rect = ui.available_rect_before_wrap();
                edit_rect.set_height(Self::TEXT_EDIT_HEIGHT);

                // Show text edit.
                let text_edit_resp = egui::TextEdit::singleline(value)
                    .id(options.id)
                    .margin(egui::Vec2::new(2.0, 0.0))
                    .font(TextStyle::Heading)
                    .min_size(edit_rect.size())
                    .horizontal_align(if options.h_center { Align::Center } else { Align::Min })
                    .vertical_align(Align::Center)
                    .password(show_pass)
                    .cursor_at_end(true)
                    .ui(ui);
                // Show keyboard on click.
                if text_edit_resp.clicked() {
                    text_edit_resp.request_focus();
                    cb.show_keyboard();
                }
                // Setup focus on input field.
                if options.focus {
                    text_edit_resp.request_focus();
                    cb.show_keyboard();
                }

                // Apply text from input on Android as temporary fix for egui.
                if text_edit_resp.has_focus() {
                    Self::on_soft_input(ui, options.id, value);
                }
            });
        });
    }

    /// Apply soft keyboard input data to provided String.
    pub fn on_soft_input(ui: &mut egui::Ui, id: egui::Id, value: &mut String) {
        let os = OperatingSystem::from_target_os();
        if os == OperatingSystem::Android {
            let mut w_input = LAST_SOFT_KEYBOARD_INPUT.write();

            if !w_input.is_empty() {
                let mut state = TextEditState::load(ui.ctx(), id).unwrap();
                match state.cursor.char_range() {
                    None => {}
                    Some(range) => {
                        let mut r = range.clone();

                        let mut index = r.primary.index;

                        value.insert_text(w_input.as_str(), index);
                        index = index + 1;

                        if index == 0 {
                            r.primary.index = value.len();
                            r.secondary.index = r.primary.index;
                        } else {
                            r.primary.index = index;
                            r.secondary.index = r.primary.index;
                        }
                        state.cursor.set_char_range(Some(r));
                        TextEditState::store(state, ui.ctx(), id);
                    }
                }
            }

            *w_input = "".to_string();
            ui.ctx().request_repaint();
        }
    }

    /// Calculate item background/button rounding based on item index.
    pub fn item_rounding(index: usize, len: usize, is_button: bool) -> Rounding {
        let corners = if is_button {
            if len == 1 {
                [false, true, true, false]
            } else if index == 0 {
                [false, true, false, false]
            } else if index == len - 1 {
                [false, false, true, false]
            } else {
                [false, false, false, false]
            }
        } else {
            if len == 1 {
                [true, true, true, true]
            } else if index == 0 {
                [true, true, false, false]
            } else if index == len - 1 {
                [false, false, true, true]
            } else {
                [false, false, false, false]
            }
        };
        Rounding {
            nw: if corners[0] { 8.0 } else { 0.0 },
            ne: if corners[1] { 8.0 } else { 0.0 },
            sw: if corners[3] { 8.0 } else { 0.0 },
            se: if corners[2] { 8.0 } else { 0.0 },
        }
    }

    /// Draw rounded box with some value and label in the middle,
    /// where is r = (top_left, top_right, bottom_left, bottom_right).
    /// | VALUE |
    /// | label |
    pub fn label_box(ui: &mut egui::Ui, text: String, label: String, r: [bool; 4]) {
        let rect = ui.available_rect_before_wrap();

        // Create background shape.
        let mut bg_shape = RectShape::new(rect, Rounding {
            nw: if r[0] { 8.0 } else { 0.0 },
            ne: if r[1] { 8.0 } else { 0.0 },
            sw: if r[2] { 8.0 } else { 0.0 },
            se: if r[3] { 8.0 } else { 0.0 },
        }, Colors::fill_lite(), Self::item_stroke());
        let bg_idx = ui.painter().add(bg_shape);

        // Draw box content.
        let content_resp = ui.allocate_new_ui(UiBuilder::new().max_rect(rect), |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.add_space(4.0);
                ui.scope(|ui| {
                    // Correct vertical spacing between items.
                    ui.style_mut().spacing.item_spacing.y = -3.0;

                    // Draw box value.
                    let mut job = LayoutJob::single_section(text, TextFormat {
                        font_id: FontId::proportional(17.0),
                        color: Colors::white_or_black(true),
                        ..Default::default()
                    });
                    job.wrap = TextWrapping {
                        max_rows: 1,
                        break_anywhere: true,
                        overflow_character: Option::from('﹍'),
                        ..Default::default()
                    };
                    ui.label(job);

                    // Draw box label.
                    ui.label(RichText::new(label).color(Colors::gray()).size(15.0));
                });
                ui.add_space(2.0);
            });
        }).response;

        // Setup background shape size.
        bg_shape.rect = content_resp.rect;
        ui.painter().set(bg_idx, bg_shape);
    }

    /// Draw content in the center of current layout with specified width and height.
    pub fn center_content(ui: &mut egui::Ui, height: f32, content: impl FnOnce(&mut egui::Ui)) {
        ui.vertical_centered(|ui| {
            let mut rect = ui.available_rect_before_wrap();
            let side_margin = 28.0;
            rect.min += egui::emath::vec2(side_margin, ui.available_height() / 2.0 - height / 2.0);
            rect.max -= egui::emath::vec2(side_margin, 0.0);
            ui.allocate_new_ui(UiBuilder::new().max_rect(rect), |ui| {
                (content)(ui);
            });
        });
    }

    /// Size of big loading spinner.
    pub const BIG_SPINNER_SIZE: f32 = 104.0;

    /// Draw big gold loading spinner.
    pub fn big_loading_spinner(ui: &mut egui::Ui) {
        Spinner::new().size(Self::BIG_SPINNER_SIZE).color(Colors::gold()).ui(ui);
    }

    /// Draw small gold loading spinner.
    pub fn small_loading_spinner(ui: &mut egui::Ui) {
        Spinner::new().size(38.0).color(Colors::gold()).ui(ui);
    }

    /// Draw the button that looks like checkbox with callback on check.
    pub fn checkbox(ui: &mut egui::Ui, checked: bool, text: String, callback: impl FnOnce()) {
        let (text_value, color) = match checked {
            true => (format!("{} {}", CHECK_SQUARE, text), Colors::text_button()),
            false => (format!("{} {}", SQUARE, text), Colors::checkbox())
        };

        let br = Button::new(RichText::new(text_value).size(17.0).color(color))
            .frame(false)
            .stroke(Stroke::NONE)
            .fill(Colors::TRANSPARENT)
            .ui(ui)
            .on_hover_cursor(CursorIcon::PointingHand);
        if Self::touched(ui, br) {
            (callback)();
        }
    }

    /// Show a [`RadioButton`]. It is selected if `*current_value == selected_value`.
    /// If clicked, `selected_value` is assigned to `*current_value`.
    pub fn radio_value<T: PartialEq>(ui: &mut egui::Ui, current: &mut T, value: T, text: String) {
        ui.scope(|ui| {
            // Setup background color.
            ui.visuals_mut().widgets.inactive.bg_fill = Colors::fill_deep();
            // Draw radio button.
            let mut response = ui.radio(*current == value, text)
                .on_hover_cursor(CursorIcon::PointingHand);
            if Self::touched(ui, response.clone()) && *current != value {
                *current = value;
                response.mark_changed();
            }
        });
    }

    /// Draw horizontal line.
    pub fn horizontal_line(ui: &mut egui::Ui, color: Color32) {
        let line_size = egui::Vec2::new(ui.available_width(), 1.0);
        let (line_rect, _) = ui.allocate_exact_size(line_size, Sense::hover());
        let painter = ui.painter();
        painter.hline(line_rect.x_range(),
                      painter.round_to_pixel(line_rect.center().y),
                      Stroke { width: 1.0, color });
    }

    /// Draw line for panel content.
    pub fn line(ui: &mut egui::Ui, pos: LinePosition, rect: &Rect, color: Color32) {
        let points = match pos {
            LinePosition::RIGHT => {
                vec![{
                         let mut r = rect.clone();
                         r.min.x = r.max.x;
                         r.min
                }, rect.max]
            }
            LinePosition::BOTTOM => {
                vec![{
                        let mut r = rect.clone();
                        r.min.y = r.max.y;
                        r.min
                }, rect.max]
            }
            LinePosition::LEFT => {
                vec![rect.min, {
                        let mut r = rect.clone();
                        r.max.x = r.min.x;
                        r.max
                }]
            }
            LinePosition::TOP => {
                vec![rect.min, {
                        let mut r = rect.clone();
                        r.max.y = r.min.y;
                        r.max
                }]
            }
        };
        let stroke = PathShape {
            points,
            closed: false,
            fill: Default::default(),
            stroke: PathStroke::new(1.0, color),
        };
        ui.painter().add(stroke);
    }

    /// Draw SVG image from provided data with optional provided size.
    pub fn svg_image(ui: &mut egui::Ui,
                     name: &str,
                     svg: &[u8],
                     size: Option<SizeHint>) -> TextureHandle {
        let color_img = load_svg_bytes_with_size(svg, size).unwrap();
        // Create image texture.
        let texture_handle = ui.ctx().load_texture(name,
                                                   color_img.clone(),
                                                   TextureOptions::default());
        let img_size = egui::emath::vec2(color_img.width() as f32,
                                         color_img.height() as f32);
        let sized_img = SizedTexture::new(texture_handle.id(), img_size);
        // Add image to content.
        ui.add(egui::Image::from_texture(sized_img)
            .max_height(ui.available_width())
            .fit_to_original_size(1.0));
        texture_handle
    }

    /// Draw application logo image with name and version.
    pub fn app_logo_name_version(ui: &mut egui::Ui) {
        ui.add_space(-1.0);
        let logo = if AppConfig::dark_theme().unwrap_or(false) {
            egui::include_image!("../../../img/logo_light.png")
        } else {
            egui::include_image!("../../../img/logo.png")
        };
        // Show application logo and name.
        ui.scope(|ui| {
            ui.set_opacity(0.9);
            egui::Image::new(logo).fit_to_exact_size(egui::vec2(182.0, 182.0)).ui(ui);
        });
        ui.add_space(-11.0);
        ui.label(RichText::new("GRIM")
            .size(24.0)
            .color(Colors::white_or_black(true))
        );
        ui.add_space(-2.0);
        ui.label(RichText::new(crate::VERSION)
            .size(16.0)
            .color(Colors::title(false))
        );
    }

    /// Draw semi-transparent cover at specified area.
    pub fn content_cover_ui(ui: &mut egui::Ui,
                            rect: Rect,
                            id: impl std::hash::Hash,
                            mut on_click: impl FnMut()) {
        let resp = ui.interact(rect, egui::Id::new(id), Sense::click_and_drag());
        if resp.clicked() || resp.dragged() {
            on_click();
        }
        let shape = RectShape::filled(resp.rect, Rounding::ZERO, Colors::semi_transparent());
        ui.painter().add(shape);
    }

    /// Get top display inset (cutout) size.
    pub fn get_top_inset() -> f32 {
        TOP_DISPLAY_INSET.load(Ordering::Relaxed) as f32
    }

    /// Get right display inset (cutout) size.
    pub fn get_right_inset() -> f32 {
        RIGHT_DISPLAY_INSET.load(Ordering::Relaxed) as f32
    }

    /// Get bottom display inset (cutout) size.
    pub fn get_bottom_inset() -> f32 {
        BOTTOM_DISPLAY_INSET.load(Ordering::Relaxed) as f32
    }

    /// Get left display inset (cutout) size.
    pub fn get_left_inset() -> f32 {
        LEFT_DISPLAY_INSET.load(Ordering::Relaxed) as f32
    }

}

lazy_static! {
    static ref TOP_DISPLAY_INSET: AtomicI32 = AtomicI32::new(0);
    static ref RIGHT_DISPLAY_INSET: AtomicI32 = AtomicI32::new(0);
    static ref BOTTOM_DISPLAY_INSET: AtomicI32 = AtomicI32::new(0);
    static ref LEFT_DISPLAY_INSET: AtomicI32 = AtomicI32::new(0);
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Callback from Java code to update display insets (cutouts).
pub extern "C" fn Java_mw_gri_android_MainActivity_onDisplayInsets(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    cutouts: jni::sys::jarray
) {
    use jni::objects::{JObject, JPrimitiveArray};

    let mut array: [i32; 4] = [0; 4];
    unsafe {
        let j_obj = JObject::from_raw(cutouts);
        let j_arr = JPrimitiveArray::from(j_obj);
        _env.get_int_array_region(j_arr, 0, array.as_mut()).unwrap();
    }
    TOP_DISPLAY_INSET.store(array[0], Ordering::Relaxed);
    RIGHT_DISPLAY_INSET.store(array[1], Ordering::Relaxed);
    BOTTOM_DISPLAY_INSET.store(array[2], Ordering::Relaxed);
    LEFT_DISPLAY_INSET.store(array[3], Ordering::Relaxed);
}

lazy_static! {
    static ref LAST_SOFT_KEYBOARD_INPUT: Arc<RwLock<String>> = Arc::new(RwLock::new("".into()));
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Callback from Java code with last entered character from soft keyboard.
pub extern "C" fn Java_mw_gri_android_MainActivity_onInput(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    char: jni::sys::jstring
) {
    use jni::objects::JString;
    use std::ops::Add;

    unsafe {
        let j_obj = JString::from_raw(char);
        let j_str = _env.get_string_unchecked(j_obj.as_ref()).unwrap();
        match j_str.to_str() {
            Ok(str) => {
                let mut w_input = LAST_SOFT_KEYBOARD_INPUT.write();
                *w_input = w_input.clone().add(str);
            }
            Err(_) => {}
        }
    }
}