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

use egui::{Button, PointerState, Response, RichText, Sense, Spinner, Widget};
use egui::epaint::{CircleShape, Color32, FontId, RectShape, Rounding, Stroke};
use egui::epaint::text::TextWrapping;
use egui::text::{LayoutJob, TextFormat};
use lazy_static::lazy_static;

use crate::gui::Colors;
use crate::gui::icons::{CHECK_SQUARE, SQUARE};

pub struct View;

impl View {
    /// Default stroke around views.
    pub const DEFAULT_STROKE: Stroke = Stroke { width: 1.0, color: Colors::STROKE };
    /// Stroke for items.
    pub const ITEM_STROKE: Stroke = Stroke { width: 1.0, color: Colors::ITEM_STROKE };
    /// Stroke for hovered items and buttons.
    pub const ITEM_HOVER_STROKE: Stroke = Stroke { width: 1.0, color: Colors::ITEM_HOVER };

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
    pub fn far_right_inset_margin(ui: &mut egui::Ui, frame: &mut eframe::Frame) -> f32 {
        let container_width = ui.available_rect_before_wrap().max.x as i32;
        let display_width = frame.info().window_info.size.x as i32;
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
            break_anywhere: false,
            overflow_character: Option::from('﹍'),
            ..Default::default()
        };
        job
    }

    /// Show ellipsized text.
    pub fn ellipsize_text(ui: &mut egui::Ui, text: String, size: f32, color: Color32) {
        ui.label(Self::ellipsize(text, size, color));
    }

    /// Draw horizontally centered sub-title with space below.
    pub fn sub_title(ui: &mut egui::Ui, text: String) {
        ui.vertical_centered_justified(|ui| {
            ui.label(RichText::new(text.to_uppercase()).size(16.0).color(Colors::TEXT));
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
        if drag_resp.drag_released() || drag_resp.clicked() {
            return true;
        }
        false
    }

    /// Title button with transparent background fill color, contains only icon.
    pub fn title_button(ui: &mut egui::Ui, icon: &str, action: impl FnOnce()) {
        ui.scope(|ui| {
            // Setup stroke around title buttons on click.
            ui.style_mut().visuals.widgets.hovered.bg_stroke = Self::ITEM_HOVER_STROKE;
            ui.style_mut().visuals.widgets.active.bg_stroke = Self::DEFAULT_STROKE;
            // Disable rounding.
            ui.style_mut().visuals.widgets.hovered.rounding = Rounding::none();
            ui.style_mut().visuals.widgets.active.rounding = Rounding::none();
            // Disable expansion.
            ui.style_mut().visuals.widgets.hovered.expansion = 0.0;
            ui.style_mut().visuals.widgets.active.expansion = 0.0;

            // Setup text.
            let wt = RichText::new(icon.to_string()).size(22.0).color(Colors::TITLE);
            // Draw button.
            let br = Button::new(wt)
                .fill(Colors::TRANSPARENT)
                .ui(ui);
            if Self::touched(ui, br) {
                (action)();
            }
        });
    }

    /// Tab button with white background fill color, contains only icon.
    pub fn tab_button(ui: &mut egui::Ui, icon: &str, active: bool, action: impl FnOnce()) {
        let text_color = match active {
            true => Colors::TITLE,
            false => Colors::TEXT
        };
        let stroke = match active {
            true => Stroke::NONE,
            false => Self::DEFAULT_STROKE
        };
        let color = match active {
            true => Colors::FILL,
            false => Colors::WHITE
        };
        let br = Button::new(RichText::new(icon.to_string()).size(22.0).color(text_color))
            .stroke(stroke)
            .fill(color)
            .ui(ui);
        if Self::touched(ui, br) {
            (action)();
        }
    }

    /// Draw [`Button`] with specified background fill color.
    pub fn button(ui: &mut egui::Ui, text: String, fill: Color32, action: impl FnOnce()) {
        let button_text = Self::ellipsize(text.to_uppercase(), 17.0, Colors::TEXT_BUTTON);
        let br = Button::new(button_text)
            .stroke(Self::DEFAULT_STROKE)
            .fill(fill)
            .ui(ui);
        if Self::touched(ui, br) {
            (action)();
        }
    }

    /// Draw list item [`Button`] with given vertical padding and rounding on left and right sides.
    pub fn item_button(ui: &mut egui::Ui, r: [bool; 2], icon: &'static str, action: impl FnOnce()) {
        let rounding = Self::get_rounding([r[0], r[1], r[1], r[0]]);

        // Setup button size.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_width(32.0);
        let button_size = rect.size();

        ui.scope(|ui| {
            // Disable expansion on click/hover.
            ui.style_mut().visuals.widgets.hovered.expansion = 0.0;
            ui.style_mut().visuals.widgets.active.expansion = 0.0;

            // Setup fill colors.
            ui.visuals_mut().widgets.inactive.weak_bg_fill = Colors::WHITE;
            ui.visuals_mut().widgets.hovered.weak_bg_fill = Colors::BUTTON;
            ui.visuals_mut().widgets.active.weak_bg_fill = Colors::FILL;

            // Setup stroke colors.
            ui.visuals_mut().widgets.inactive.bg_stroke = Self::DEFAULT_STROKE;
            ui.visuals_mut().widgets.hovered.bg_stroke = Self::ITEM_HOVER_STROKE;
            ui.visuals_mut().widgets.active.bg_stroke = Self::ITEM_STROKE;

            // Show button.
            let br = Button::new(RichText::new(icon).size(20.0).color(Colors::CHECKBOX))
                .rounding(rounding)
                .min_size(button_size)
                .ui(ui);
            if Self::touched(ui, br) {
                (action)();
            }
        });
    }

    /// Draw circle [`Button`] with icon.
    pub fn circle_button(ui: &mut egui::Ui, icon: &'static str, action: impl FnOnce()) {
        ui.scope(|ui| {
            // Setup colors.
            ui.visuals_mut().widgets.inactive.bg_fill = Colors::GOLD;
            ui.visuals_mut().widgets.hovered.bg_fill = Colors::GOLD;
            ui.visuals_mut().widgets.active.bg_fill = Colors::YELLOW;

            // Setup radius.
            let mut r = 44.0 * 0.5;
            let size = egui::Vec2::splat(2.0 * r + 5.0);
            let (rect, br) = ui.allocate_at_least(size, Sense::click_and_drag());

            let mut icon_color = Colors::TEXT;

            // Increase radius and change icon size and color on-hover.
            if br.hovered() {
                r = r * 1.04;
                icon_color = Colors::TITLE;
            }

            let visuals = ui.style().interact(&br);
            ui.painter().add(CircleShape {
                center: rect.center(),
                radius: r,
                fill: visuals.bg_fill,
                stroke: Self::DEFAULT_STROKE
            });
            ui.allocate_ui_at_rect(rect, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.label(RichText::new(icon).color(icon_color).size(25.0));
                });
            });
            if Self::touched(ui, br) {
                (action)();
            }
        });
    }

    /// Get rounding for provided corners clockwise.
    fn get_rounding(corners: [bool; 4]) -> Rounding {
        Rounding {
            nw: if corners[0] { 8.0 } else { 0.0 },
            ne: if corners[1] { 8.0 } else { 0.0 },
            sw: if corners[3] { 8.0 } else { 0.0 },
            se: if corners[2] { 8.0 } else { 0.0 },
        }
    }

    /// Calculate list item rounding based on item index.
    pub fn item_rounding(index: usize, len: usize) -> Rounding {
        let rounding = if len == 1 {
            [true, true, true, true]
        } else if index == 0 {
            [true, true, false, false]
        } else if index == len - 1 {
            [false, false, true, true]
        } else {
            [false, false, false, false]
        };
        Self::get_rounding(rounding)
    }

    /// Draw rounded box with some value and label in the middle,
    /// where is r = (top_left, top_right, bottom_left, bottom_right).
    /// | VALUE |
    /// | label |
    pub fn rounded_box(ui: &mut egui::Ui, value: String, label: String, r: [bool; 4]) {
        let rect = ui.available_rect_before_wrap();

        // Create background shape.
        let mut bg_shape = RectShape {
            rect,
            rounding: Rounding {
                nw: if r[0] { 8.0 } else { 0.0 },
                ne: if r[1] { 8.0 } else { 0.0 },
                sw: if r[2] { 8.0 } else { 0.0 },
                se: if r[3] { 8.0 } else { 0.0 },
            },
            fill: Colors::WHITE,
            stroke: Self::ITEM_STROKE,
        };
        let bg_idx = ui.painter().add(bg_shape);

        // Draw box content.
        let content_resp = ui.allocate_ui_at_rect(rect, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.add_space(2.0);

                ui.scope(|ui| {
                    // Correct vertical spacing between items.
                    ui.style_mut().spacing.item_spacing.y = -3.0;

                    // Draw box value.
                    let mut job = LayoutJob::single_section(value, TextFormat {
                        font_id: FontId::proportional(17.0),
                        color: Colors::BLACK,
                        ..Default::default()
                    });
                    job.wrap = TextWrapping {
                        max_rows: 1,
                        break_anywhere: false,
                        overflow_character: Option::from('﹍'),
                        ..Default::default()
                    };
                    ui.label(job);

                    // Draw box label.
                    ui.label(RichText::new(label).color(Colors::GRAY).size(15.0));
                });

                ui.add_space(2.0);
            });
        }).response;

        // Setup background shape to be painted behind box content.
        bg_shape.rect = content_resp.rect;
        ui.painter().set(bg_idx, bg_shape);
    }

    /// Draw content in the center of current layout with specified width and height.
    pub fn center_content(ui: &mut egui::Ui, height: f32, content: impl FnOnce(&mut egui::Ui)) {
        ui.vertical_centered(|ui| {
            let mut rect = ui.available_rect_before_wrap();
            let side_margin = 24.0;
            rect.min += egui::emath::vec2(side_margin, ui.available_height() / 2.0 - height / 2.0);
            rect.max -= egui::emath::vec2(side_margin, 0.0);
            ui.allocate_ui_at_rect(rect, |ui| {
                (content)(ui);
            });
        });
    }

    /// Draw big gold loading spinner.
    pub fn big_loading_spinner(ui: &mut egui::Ui) {
        Spinner::new().size(104.0).color(Colors::GOLD).ui(ui);
    }

    /// Draw small gold loading spinner.
    pub fn small_loading_spinner(ui: &mut egui::Ui) {
        Spinner::new().size(38.0).color(Colors::GOLD).ui(ui);
    }

    /// Draw the button that looks like checkbox with callback on check.
    pub fn checkbox(ui: &mut egui::Ui, checked: bool, text: String, callback: impl FnOnce()) {
        let (text_value, color) = match checked {
            true => (format!("{} {}", CHECK_SQUARE, text), Colors::TEXT_BUTTON),
            false => (format!("{} {}", SQUARE, text), Colors::CHECKBOX)
        };

        let br = Button::new(RichText::new(text_value).size(17.0).color(color))
            .frame(false)
            .stroke(Stroke::NONE)
            .fill(Colors::TRANSPARENT)
            .ui(ui);
        if Self::touched(ui, br) {
            (callback)();
        }
    }

    /// Show a [`RadioButton`]. It is selected if `*current_value == selected_value`.
    /// If clicked, `selected_value` is assigned to `*current_value`.
    pub fn radio_value<T: PartialEq>(ui: &mut egui::Ui, current: &mut T, value: T, text: String) {
        let mut response = ui.radio(*current == value, text);
        if Self::touched(ui, response.clone()) && *current != value {
            *current = value;
            response.mark_changed();
        }
    }

    /// Draw horizontal line
    pub fn horizontal_line(ui: &mut egui::Ui, color: Color32) {
        let line_size = egui::Vec2::new(ui.available_width(), 1.0);
        let (line_rect, _) = ui.allocate_exact_size(line_size, Sense::hover());
        let painter = ui.painter();
        painter.hline(line_rect.x_range(),
                      painter.round_to_pixel(line_rect.center().y),
                      Stroke { width: 1.0, color });
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