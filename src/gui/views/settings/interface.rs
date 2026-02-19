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

use egui::scroll_area::ScrollBarVisibility;
use egui::{Align, Layout, RichText, ScrollArea, StrokeKind};

use crate::gui::icons::{CHECK, CHECK_FAT, PENCIL, TRANSLATE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::types::{ContentContainer, ModalPosition};
use crate::gui::views::{Modal, View};
use crate::gui::Colors;
use crate::AppConfig;

/// User interface settings content.
pub struct InterfaceSettingsContent {
    /// Current locale.
    locale: String,
}

/// Identifier for language selection [`Modal`].
const LANGUAGE_SELECTION_MODAL: &'static str = "language_selection_modal";

impl ContentContainer for InterfaceSettingsContent {
    fn modal_ids(&self) -> Vec<&'static str> {
        vec![
            LANGUAGE_SELECTION_MODAL
        ]
    }

    fn modal_ui(&mut self, ui: &mut egui::Ui, modal: &Modal, _: &dyn PlatformCallbacks) {
        match modal.id {
            LANGUAGE_SELECTION_MODAL => self.language_selection_ui(ui),
            _ => {}
        }
    }

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
        self.language_item_ui(self.locale.clone().as_str(), ui, true, 0, 1);
        ui.add_space(4.0);
    }
}

impl Default for InterfaceSettingsContent {
    fn default() -> Self {
        let locale = if let Some(lang) = AppConfig::locale() {
            lang
        } else {
            rust_i18n::locale()
        };
        Self {
            locale,
        }
    }
}

impl InterfaceSettingsContent {
    /// Draw language selection content.
    fn language_selection_ui(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ScrollArea::vertical()
            .max_height(373.0)
            .id_salt("select_language_scroll")
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .auto_shrink([true; 2])
            .show(ui, |ui| {
                ui.add_space(2.0);
                ui.vertical_centered(|ui| {
                    let locales = rust_i18n::available_locales!();
                    for (index, locale) in locales.iter().enumerate() {
                        self.language_item_ui(locale, ui, false, index, locales.len());
                    }
                });
            });

        ui.add_space(6.0);
        View::horizontal_line(ui, Colors::item_stroke());
        ui.add_space(6.0);

        // Show button to close modal.
        ui.vertical_centered_justified(|ui| {
            View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                Modal::close();
            });
        });
        ui.add_space(6.0);
    }

    /// Draw language selection item content.
    fn language_item_ui(&mut self, locale: &str, ui: &mut egui::Ui, edit: bool, index: usize, len: usize) {
        // Setup layout size.
        let mut rect = ui.available_rect_before_wrap();
        if edit {
            rect.set_height(56.0);
        } else {
            rect.set_height(50.0);
        }

        // Draw round background.
        let bg_rect = rect.clone();
        let item_rounding = View::item_rounding(index, len, false);
        ui.painter().rect(bg_rect,
                          item_rounding,
                          Colors::fill(),
                          View::item_stroke(),
                          StrokeKind::Outside);

        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            if edit {
                View::item_button(ui, View::item_rounding(index, len, true), PENCIL, None, || {
                    // Show language selection modal.
                    Modal::new(LANGUAGE_SELECTION_MODAL)
                        .position(ModalPosition::Center)
                        .title(t!("language"))
                        .show();
                });
                let layout_size = ui.available_size();
                ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                    ui.add_space(12.0);
                    ui.vertical(|ui| {
                        ui.add_space(4.0);
                        View::ellipsize_text(ui,
                                             t!("lang_name", locale = locale),
                                             18.0,
                                             Colors::title(false));
                        ui.add_space(1.0);
                        let value = format!("{} {}",
                                            TRANSLATE,
                                            t!("language"));
                        ui.label(RichText::new(value).size(15.0).color(Colors::gray()));
                        ui.add_space(3.0);
                    });
                });
            } else {
                // Draw button to select language.
                let is_current = self.locale == locale;
                if !is_current {
                    View::item_button(ui, View::item_rounding(index, len, true), CHECK, None, || {
                        rust_i18n::set_locale(locale);
                        AppConfig::save_locale(locale);
                        self.locale = locale.to_string();
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
            }
        });
    }
}