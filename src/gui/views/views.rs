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

use eframe::epaint::{Color32, FontId, Rounding, Stroke};
use eframe::epaint::text::{LayoutJob, TextFormat, TextWrapping};
use egui::{RichText, Sense, Widget};

use crate::gui::colors::{COLOR_DARK, COLOR_GRAY, COLOR_LIGHT};

pub struct View;

impl View {
    pub const DEFAULT_STROKE: Stroke = Stroke { width: 1.0, color: Color32::from_gray(190) };

    pub fn title_button(ui: &mut egui::Ui, icon: &str, action: impl FnOnce()) {
        let b = egui::widgets::Button::new(
            RichText::new(icon.to_string()).size(24.0).color(COLOR_DARK)
        ).fill(Color32::TRANSPARENT)
            .ui(ui).interact(Sense::click_and_drag());

        // Click optimization for touch screens
        if b.drag_released() || b.clicked() {
            (action)();
        };
    }

    pub fn tab_button(ui: &mut egui::Ui, icon: &str, active: bool, mut action: impl FnMut()) {
        let stroke = match active {
            true => { Stroke::NONE }
            false => { Self::DEFAULT_STROKE }
        };

        let color = match active {
            true => { COLOR_LIGHT }
            false => { Color32::WHITE }
        };

        let b = egui::widgets::Button::new(
            RichText::new(icon.to_string()).size(24.0).color(COLOR_DARK)
        ).min_size(ui.available_size_before_wrap())
            .stroke(stroke)
            .fill(color)
            .ui(ui).interact(Sense::click_and_drag());


        let vel_y = ui.ctx().input().pointer.delta().y;
        let vel_x = ui.ctx().input().pointer.delta().x;
        println!("12345, vel {}, {}", vel_y, vel_x);

        // Click optimization for touch screens
        if b.drag_released() || b.clicked() {
            (action)();
        };
    }

    pub fn sub_title(ui: &mut egui::Ui, text: String, color: Color32) {
        ui.label(RichText::new(text).size(17.0).color(color));
    }

    /// Draw rounded box with some value and label in the middle
    /// where is r = [top_left, top_right, bottom_left, bottom_right]
    /// | VALUE |
    /// | label |
    pub fn rounded_box(ui: &mut egui::Ui, value: String, label: String, r: [bool; 4]) {
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(46.0);

        // Draw box background
        ui.painter().rect(
            rect,
            Rounding {
                nw: if r[0] { 8.0 } else { 0.0 },
                ne: if r[1] { 8.0 } else { 0.0 },
                sw: if r[2] { 8.0 } else { 0.0 },
                se: if r[3] { 8.0 } else { 0.0 },
            },
            Color32::WHITE,
            Stroke { width: 1.0, color: Color32::from_gray(230) },
        );

        ui.vertical_centered_justified(|ui| {
            // Correct vertical spacing between items
            ui.style_mut().spacing.item_spacing.y = -4.0;

            // Draw box value
            let mut job = LayoutJob::single_section(value, TextFormat {
                font_id: FontId::proportional(18.0),
                color: Color32::BLACK,
                .. Default::default()
            });
            job.wrap = TextWrapping {
                max_rows: 1,
                break_anywhere: false,
                overflow_character: Option::from('﹍'),
                ..Default::default()
            };
            ui.label(job);

            // Draw box label
            ui.label(RichText::new(label).color(COLOR_GRAY).size(15.0));
        });
    }
}