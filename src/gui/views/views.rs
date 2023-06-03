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

use egui::{Button, PointerState, Response, RichText, Sense, Vec2, Widget};
use egui::epaint::{Color32, FontId, RectShape, Rounding, Stroke};
use egui::epaint::text::TextWrapping;
use egui::text::{LayoutJob, TextFormat};
use egui_extras::{Size, StripBuilder};

use crate::gui::Colors;

pub struct View;

impl View {
    /// Default stroke around views.
    pub const DEFAULT_STROKE: Stroke = Stroke { width: 1.0, color: Colors::STROKE };

    /// Default width of side panel at application UI.
    pub const SIDE_PANEL_MIN_WIDTH: i64 = 400;

    /// Check if UI can show side panel and screen at same time.
    pub fn is_dual_panel_mode(frame: &mut eframe::Frame) -> bool {
        let w = frame.info().window_info.size.x;
        let h = frame.info().window_info.size.y;
        // Screen is wide if width is greater than height or just 20% smaller.
        let is_wide_screen = w > h || w + (w * 0.2) >= h;
        // Dual panel mode is available when window is wide and its width is at least 2 times
        // greater than minimal width of the side panel.
        is_wide_screen && w >= Self::SIDE_PANEL_MIN_WIDTH as f32 * 2.0
    }

    /// Show and cut long text with ﹍ character.
    pub fn ellipsize_text(ui: &mut egui::Ui, text: String, size: f32, color: Color32) {
        let mut job = LayoutJob::single_section(text, TextFormat {
            font_id: FontId::proportional(size), color, .. Default::default()
        });
        job.wrap = TextWrapping {
            max_rows: 1,
            break_anywhere: false,
            overflow_character: Option::from('﹍'),
            ..Default::default()
        };
        ui.label(job);
    }

    /// Sub-header with uppercase characters and more lighter color.
    pub fn sub_header(ui: &mut egui::Ui, text: String) {
        ui.label(RichText::new(text.to_uppercase()).size(16.0).color(Colors::TEXT));
    }

    /// Temporary button click optimization for touch screens.
    fn on_button_click(ui: &mut egui::Ui, resp: Response, action: impl FnOnce()) {
        // Clear pointer event if dragging is out of button area
        if resp.dragged() && !ui.rect_contains_pointer(resp.rect) {
            ui.input_mut().pointer = PointerState::default();
        }
        // Call click action if button is clicked or drag released
        if resp.drag_released() || resp.clicked() {
            (action)();
        };
    }

    /// Title button with transparent background fill color, contains only icon.
    pub fn title_button(ui: &mut egui::Ui, icon: &str, action: impl FnOnce()) {
        ui.scope(|ui| {
            // Disable stroke around title buttons on hover
            ui.style_mut().visuals.widgets.active.bg_stroke = Stroke::NONE;

            let wt = RichText::new(icon.to_string()).size(24.0).color(Colors::TITLE);
            let br = Button::new(wt)
                .fill(Colors::TRANSPARENT)
                .ui(ui).interact(Sense::click_and_drag());

            Self::on_button_click(ui, br, action);
        });
    }

    /// Tab button with white background fill color, contains only icon.
    pub fn tab_button(ui: &mut egui::Ui, icon: &str, active: bool, action: impl FnOnce()) {
        let text_color = match active {
            true => { Colors::TITLE }
            false => { Colors::TEXT }
        };
        let wt = RichText::new(icon.to_string()).size(24.0).color(text_color);

        let stroke = match active {
            true => { Stroke::NONE }
            false => { Self::DEFAULT_STROKE }
        };

        let color = match active {
            true => { Colors::FILL }
            false => { Colors::WHITE }
        };
        let br = Button::new(wt)
            .stroke(stroke)
            .fill(color)
            .ui(ui).interact(Sense::click_and_drag());

        Self::on_button_click(ui, br, action);
    }

    /// Draw [`Button`] with specified background fill color.
    pub fn button(ui: &mut egui::Ui, text: String, fill_color: Color32, action: impl FnOnce()) {
        let wt = RichText::new(text.to_uppercase()).size(18.0).color(Colors::BUTTON);
        let br = Button::new(wt)
            .stroke(Self::DEFAULT_STROKE)
            .fill(fill_color)
            .ui(ui).interact(Sense::click_and_drag());

        Self::on_button_click(ui, br, action);
    }

    /// Draw rounded box with some value and label in the middle,
    /// where is r = (top_left, top_right, bottom_left, bottom_right).
    /// | VALUE |
    /// | label |
    pub fn rounded_box(ui: &mut egui::Ui, value: String, label: String, r: [bool; 4]) {
        let mut rect = ui.available_rect_before_wrap();

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
            stroke: Stroke { width: 1.0, color: Colors::ITEM_STROKE },
        };
        let bg_idx = ui.painter().add(bg_shape);

        // Draw box content.
        let content_resp = ui.allocate_ui_at_rect(rect, |ui| {
            ui.vertical_centered_justified(|ui| {
                // Correct vertical spacing between items.
                ui.style_mut().spacing.item_spacing.y = -4.0;

                // Draw box value.
                let mut job = LayoutJob::single_section(value, TextFormat {
                    font_id: FontId::proportional(18.0),
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
}