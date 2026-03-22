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

use eframe::epaint::RectShape;
use egui::{Align, CursorIcon, Layout, RichText, Sense, StrokeKind, UiBuilder};

use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::types::ContentContainer;
use crate::gui::views::{Modal, View};
use crate::gui::Colors;
use crate::AppConfig;

/// User interface settings content.
pub struct InterfaceSettingsContent {
    /// Current locale.
    locale: String,
}

impl ContentContainer for InterfaceSettingsContent {
    fn modal_ids(&self) -> Vec<&'static str> { vec![] }

    fn modal_ui(&mut self, _: &mut egui::Ui, _: &Modal, _: &dyn PlatformCallbacks) {}

    fn container_ui(&mut self, ui: &mut egui::Ui, _: &dyn PlatformCallbacks) {
        ui.add_space(5.0);

        // Draw theme selection.
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("theme")).size(16.0).color(Colors::gray()));
        });

        let saved_use_dark = AppConfig::dark_theme().unwrap_or(false);
        let mut selected_use_dark = saved_use_dark;

        ui.add_space(8.0);
        ui.columns(2, |columns| {
            columns[0].vertical_centered(|ui| {
                View::radio_value(ui, &mut selected_use_dark, false, t!("light"));
            });
            columns[1].vertical_centered(|ui| {
                View::radio_value(ui, &mut selected_use_dark, true, t!("dark"));
            })
        });
        ui.add_space(14.0);
        if saved_use_dark != selected_use_dark {
            AppConfig::set_dark_theme(selected_use_dark);
            crate::setup_visuals(ui.ctx());
        }

        // Draw language selection.
        let locales = rust_i18n::available_locales!();
        for (index, locale) in locales.iter().enumerate() {
            self.language_item_ui(locale, ui, index, locales.len());
        }
        ui.add_space(4.0);
    }
}

impl Default for InterfaceSettingsContent {
    fn default() -> Self {
        let locale = if let Some(lang) = AppConfig::locale() {
            lang
        } else {
            rust_i18n::locale().to_string()
        };
        Self {
            locale,
        }
    }
}

impl InterfaceSettingsContent {
    /// Draw language selection item content.
    fn language_item_ui(&mut self, locale: &str, ui: &mut egui::Ui, index: usize, len: usize) {
        let is_current = self.locale == locale;
        // Draw round background.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(50.0);
        let r = View::item_rounding(index, len, false);
        let bg = if is_current {
            Colors::fill()
        } else {
            Colors::fill_lite()
        };
        let mut bg_shape = RectShape::new(rect, r, bg, View::item_stroke(), StrokeKind::Outside);
        let bg_idx = ui.painter().add(bg_shape.clone());

        let res = ui.scope_builder(
            UiBuilder::new()
                .sense(Sense::click())
                .layout(Layout::right_to_left(Align::Center))
                .max_rect(rect), |ui| {
                if is_current {
                    View::selected_item_check(ui);
                }
                let layout_size = ui.available_size();
                ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                    ui.add_space(12.0);
                    ui.vertical(|ui| {
                        // Draw language name.
                        ui.add_space(12.0);
                        let color = if is_current {
                            Colors::title(false)
                        } else {
                            Colors::gray()
                        };
                        ui.label(RichText::new(t!("lang_name", locale = locale))
                            .size(17.0)
                            .color(color));
                        ui.add_space(14.0);
                    });
                });
            }
        ).response;
        let clicked = res.clicked() || res.long_touched();
        // Setup background and cursor.
        if res.hovered() && !is_current {
            res.on_hover_cursor(CursorIcon::PointingHand);
            bg_shape.fill = Colors::fill();
        }
        ui.painter().set(bg_idx, bg_shape);
        // Handle clicks on layout.
        if clicked && !is_current {
            rust_i18n::set_locale(locale);
            AppConfig::save_locale(locale);
            self.locale = locale.to_string();
        }
    }
}