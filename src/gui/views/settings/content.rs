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
use eframe::epaint::StrokeKind;
use egui::{Layout, RichText};
use crate::AppConfig;
use crate::gui::Colors;
use crate::gui::icons::{CHECK, CHECK_FAT};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, View};
use crate::gui::views::types::ContentContainer;

/// Application settings content.
pub struct SettingsContent {

}


impl Default for SettingsContent {
    fn default() -> Self {
        Self {

        }
    }
}

impl ContentContainer for SettingsContent {
    fn modal_ids(&self) -> Vec<&'static str> {
        vec![]
    }

    fn modal_ui(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
    }

    fn on_back(&mut self, cb: &dyn PlatformCallbacks) -> bool {
        true
    }

    fn container_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);

        // Show theme selection.
        Self::theme_selection_ui(ui);

        ui.add_space(8.0);
        View::horizontal_line(ui, Colors::item_stroke());
        ui.add_space(6.0);

        ui.vertical_centered(|ui| {
            ui.label(RichText::new(format!("{}:", t!("language")))
                .size(16.0)
                .color(Colors::gray())
            );
        });
        ui.add_space(8.0);

        // Draw available list of languages to select.
        let locales = rust_i18n::available_locales!();
        for (index, locale) in locales.iter().enumerate() {
            Self::language_item_ui(locale, ui, index, locales.len());
        }
    }
}

impl SettingsContent {
    /// Draw theme selection content.
    fn theme_selection_ui(ui: &mut egui::Ui) {
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
        ui.add_space(8.0);

        if saved_use_dark != selected_use_dark {
            AppConfig::set_dark_theme(selected_use_dark);
            crate::setup_visuals(ui.ctx());
        }

        ui.add_space(6.0);
    }

    /// Draw language selection item content.
    fn language_item_ui(locale: &str, ui: &mut egui::Ui, index: usize, len: usize) {
        // Setup layout size.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(50.0);

        // Draw round background.
        let bg_rect = rect.clone();
        let item_rounding = View::item_rounding(index, len, false);
        ui.painter().rect(bg_rect,
                          item_rounding,
                          Colors::fill(),
                          View::item_stroke(),
                          StrokeKind::Middle);

        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            // Draw button to select language.
            let is_current = if let Some(lang) = AppConfig::locale() {
                lang == locale
            } else {
                rust_i18n::locale() == locale
            };
            if !is_current {
                View::item_button(ui, View::item_rounding(index, len, true), CHECK, None, || {
                    rust_i18n::set_locale(locale);
                    AppConfig::save_locale(locale);
                    Modal::close();
                });
            } else {
                ui.add_space(14.0);
                ui.label(RichText::new(CHECK_FAT).size(20.0).color(Colors::green()));
                ui.add_space(14.0);
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
                    ui.add_space(3.0);
                });
            });
        });
    }
}