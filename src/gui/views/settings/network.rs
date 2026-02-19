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

use egui::{Align, Id, Layout, RichText, StrokeKind};
use url::Url;

use crate::gui::icons::{CLOUD_CHECK, CLOUD_SLASH, PENCIL};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::types::{ContentContainer, ModalPosition};
use crate::gui::views::{Modal, TextEdit, View};
use crate::gui::Colors;
use crate::AppConfig;

/// Network communication settings content.
pub struct NetworkSettingsContent {
    /// Proxy URL input value for [`Modal`].
    proxy_url_edit: String,
    /// Flag to check if entered proxy address was correct.
    proxy_url_error: bool,
}

/// Identifier for proxy URL edit [`Modal`].
const PROXY_URL_EDIT_MODAL: &'static str = "settings_proxy_edit_modal";

impl ContentContainer for NetworkSettingsContent {
    fn modal_ids(&self) -> Vec<&'static str> {
        vec![
            PROXY_URL_EDIT_MODAL
        ]
    }

    fn modal_ui(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        match modal.id {
            PROXY_URL_EDIT_MODAL => self.proxy_modal_ui(ui, cb),
            _ => {}
        }
    }

    fn container_ui(&mut self, ui: &mut egui::Ui, _: &dyn PlatformCallbacks) {
        let use_proxy = AppConfig::use_proxy();
        View::checkbox(ui, use_proxy, t!("app_settings.proxy"), || {
            // Show edit modal when both URLs are empty.
            if AppConfig::http_proxy_url().is_none() && AppConfig::socks_proxy_url().is_none() &&
                !use_proxy {
                Modal::new(PROXY_URL_EDIT_MODAL)
                    .position(ModalPosition::CenterTop)
                    .title(t!("app_settings.proxy"))
                    .show();
            } else {
                AppConfig::toggle_use_proxy();
            }
        });
        if !use_proxy {
            ui.add_space(4.0);
            ui.label(RichText::new(t!("app_settings.proxy_desc"))
                .size(16.0)
                .color(Colors::inactive_text())
            );
            ui.add_space(8.0);
        } else {
            ui.add_space(8.0);

            // Draw proxy type selection.
            Self::proxy_type_ui(ui);

            // Draw proxy URL info.
            self.proxy_item_ui(ui);
            ui.add_space(6.0);
        }
    }
}

impl Default for NetworkSettingsContent {
    fn default() -> Self {
        Self {
            proxy_url_edit: "".to_string(),
            proxy_url_error: false,
        }
    }
}

impl NetworkSettingsContent {
    /// Draw proxy edit modal content.
    fn proxy_modal_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        let on_save = |c: &mut NetworkSettingsContent| {
            let proxy = c.proxy_url_edit.trim().to_string();
            let use_socks = AppConfig::use_socks_proxy();
            // Clear value if empty.
            if proxy.is_empty() {
                if use_socks {
                    AppConfig::save_socks_proxy_url(None);
                } else {
                    AppConfig::save_http_proxy_url(None);
                }
                Modal::close();
                return;
            }
            // Format URL.
            let http = "http://";
            let socks = "socks5://";
            let url = if use_socks {
                let p = proxy.replace(http, "");
                if !p.contains(socks) {
                    format!("{}{}", socks, p)
                } else {
                    p
                }
            } else {
                let p = proxy.replace(socks, "");
                if !p.contains(http) {
                    format!("{}{}", http, p)
                } else {
                    p
                }
            };
            c.proxy_url_error = Url::parse(url.as_str()).is_err();
            if !c.proxy_url_error {
                // Save result when no error.
                if !AppConfig::use_proxy() {
                    AppConfig::toggle_use_proxy();
                }
                if use_socks {
                    AppConfig::save_socks_proxy_url(Some(url))
                } else {
                    AppConfig::save_http_proxy_url(Some(url));
                }
                Modal::close();
            }
        };

        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            let label = format!("{}:", t!("enter_url"));
            ui.label(RichText::new(label).size(17.0).color(Colors::gray()));
            ui.add_space(8.0);

            // Draw proxy URL text edit.
            let mut edit = TextEdit::new(
                Id::from("proxy_url_edit")
                    .with(PROXY_URL_EDIT_MODAL)
                    .with(if AppConfig::use_proxy() {
                        "socks5"
                    } else {
                        "http"
                    })
            ).paste();
            edit.ui(ui, &mut self.proxy_url_edit, cb);
            if edit.enter_pressed {
                on_save(self);
            }

            // Show error when specified address is incorrect.
            if self.proxy_url_error {
                ui.add_space(10.0);
                ui.label(RichText::new(t!("wallets.invalid_url"))
                    .size(16.0)
                    .color(Colors::red()));
            }
            ui.add_space(12.0);

            // Show type selection when both URLs are empty.
            if AppConfig::socks_proxy_url().is_none() && AppConfig::http_proxy_url().is_none() {
                ui.add_space(6.0);
                ui.vertical_centered(|ui| {
                    Self::proxy_type_ui(ui);
                });
                ui.add_space(4.0);
            }

            // Show modal buttons.
            ui.scope(|ui| {
                // Setup spacing between buttons.
                ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

                ui.columns(2, |columns| {
                    columns[0].vertical_centered_justified(|ui| {
                        View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                            Modal::close();
                        });
                    });
                    columns[1].vertical_centered_justified(|ui| {
                        View::button(ui, t!("modal.save"), Colors::white_or_black(false), || {
                            on_save(self);
                        });
                    });
                });
                ui.add_space(6.0);
            });
        });
    }

    /// Draw proxy item content.
    fn proxy_item_ui(&mut self, ui: &mut egui::Ui) {
        // Setup layout size.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(56.0);

        // Draw round background.
        let bg_rect = rect.clone();
        let item_rounding = View::item_rounding(0, 1, false);
        ui.painter().rect(bg_rect,
                          item_rounding,
                          Colors::fill(),
                          View::item_stroke(),
                          StrokeKind::Outside);

        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            View::item_button(ui, View::item_rounding(0, 1, true), PENCIL, None, || {
                let url = if AppConfig::use_socks_proxy() {
                    AppConfig::socks_proxy_url().unwrap_or("".to_string())
                } else {
                    AppConfig::http_proxy_url().unwrap_or("".to_string())
                };
                self.proxy_url_edit = url;
                // Show proxy URL edit modal.
                Modal::new(PROXY_URL_EDIT_MODAL)
                    .position(ModalPosition::CenterTop)
                    .title(t!("app_settings.proxy"))
                    .show();
            });
            let layout_size = ui.available_size();
            ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    let use_socks = AppConfig::use_socks_proxy();
                    let proxy_url = if use_socks {
                        AppConfig::socks_proxy_url()
                    } else {
                        AppConfig::http_proxy_url()
                    };
                    let (url, color, icon, text) = if let Some(url) = proxy_url {
                        (url, Colors::title(false), CLOUD_CHECK, t!("network_settings.enabled"))
                    } else {
                        (
                            t!("enter_url"),
                            Colors::inactive_text(),
                            CLOUD_SLASH,
                            t!("network_settings.disabled")
                        )
                    };
                    View::ellipsize_text(ui, url, 18.0, color);
                    ui.add_space(1.0);

                    let value = format!("{} {}", icon, text);
                    ui.label(RichText::new(value).size(15.0).color(Colors::gray()));
                    ui.add_space(3.0);
                });
            });
        });
    }

    /// Draw proxy type selection.
    fn proxy_type_ui(ui: &mut egui::Ui) {
        // Draw proxy type selection.
        let saved_use_socks = AppConfig::use_socks_proxy();
        let mut selected_use_socks = saved_use_socks;
        ui.columns(2, |columns| {
            columns[0].vertical_centered(|ui| {
                View::radio_value(ui, &mut selected_use_socks, true, "SOCKS5".to_string());
            });
            columns[1].vertical_centered(|ui| {
                View::radio_value(ui, &mut selected_use_socks, false, "HTTP".to_string());
            })
        });
        ui.add_space(14.0);
        if saved_use_socks != selected_use_socks {
            AppConfig::toggle_use_socks_proxy();
        }
    }
}