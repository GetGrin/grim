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

use eframe::epaint::{Color32, Stroke};
use egui::{RichText, Sense, Widget};

use crate::gui::colors::{COLOR_DARK, COLOR_LIGHT};
use crate::gui::views::DEFAULT_STROKE;

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
        false => { DEFAULT_STROKE }
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

    // Click optimization for touch screens
    if b.drag_released() || b.clicked() {
        (action)();
    };
}

pub fn sub_title(ui: &mut egui::Ui, text: String, color: Color32) {
    ui.label(RichText::new(text).size(17.0).color(color));
}