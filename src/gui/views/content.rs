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

use std::sync::atomic::{AtomicBool, Ordering};
use egui::os::OperatingSystem;
use egui::{Align, Layout, RichText};
use lazy_static::lazy_static;

use crate::gui::Colors;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, View};
use crate::gui::views::types::ModalContainer;
use crate::node::Node;
use crate::AppConfig;
use crate::gui::icons::{CHECK, CHECK_FAT};
use crate::gui::views::network::{NetworkContent, NodeSetup};
use crate::gui::views::wallets::WalletsContent;

lazy_static! {
    /// Global state to check if [`NetworkContent`] panel is open.
    static ref NETWORK_PANEL_OPEN: AtomicBool = AtomicBool::new(false);
}

/// Contains main ui content, handles side panel state.
pub struct Content {
    /// Side panel [`NetworkContent`] content.
    network: NetworkContent,
    /// Central panel [`WalletsContent`] content.
    pub wallets: WalletsContent,

    /// Check if app exit is allowed on close event of [`eframe::App`] implementation.
    pub(crate) exit_allowed: bool,
    /// Flag to show exit progress at [`Modal`].
    show_exit_progress: bool,

    /// Flag to check it's first draw of content.
    first_draw: bool,

    /// List of allowed [`Modal`] ids for this [`ModalContainer`].
    allowed_modal_ids: Vec<&'static str>
}

impl Default for Content {
    fn default() -> Self {
        // Exit from eframe only for non-mobile platforms.
        let os = OperatingSystem::from_target_os();
        let exit_allowed = os == OperatingSystem::Android || os == OperatingSystem::IOS;
        Self {
            network: NetworkContent::default(),
            wallets: WalletsContent::default(),
            exit_allowed,
            show_exit_progress: false,
            first_draw: true,
            allowed_modal_ids: vec![
                Self::EXIT_MODAL_ID,
                Self::SETTINGS_MODAL,
                Self::ANDROID_INTEGRATED_NODE_WARNING_MODAL,
            ],
        }
    }
}

impl ModalContainer for Content {
    fn modal_ids(&self) -> &Vec<&'static str> {
        &self.allowed_modal_ids
    }

    fn modal_ui(&mut self,
                ui: &mut egui::Ui,
                modal: &Modal,
                _: &dyn PlatformCallbacks) {
        match modal.id {
            Self::EXIT_MODAL_ID => self.exit_modal_content(ui, modal),
            Self::SETTINGS_MODAL => self.settings_modal_ui(ui, modal),
            Self::ANDROID_INTEGRATED_NODE_WARNING_MODAL => self.android_warning_modal_ui(ui, modal),
            _ => {}
        }
    }
}

impl Content {
    /// Identifier for exit confirmation [`Modal`].
    pub const EXIT_MODAL_ID: &'static str = "exit_confirmation_modal";
    /// Identifier for wallet opening [`Modal`].
    pub const SETTINGS_MODAL: &'static str = "settings_modal";

    /// Identifier for integrated node warning [`Modal`] on Android.
    const ANDROID_INTEGRATED_NODE_WARNING_MODAL: &'static str = "android_node_warning_modal";

    /// Default width of side panel at application UI.
    pub const SIDE_PANEL_WIDTH: f32 = 400.0;
    /// Desktop window title height.
    pub const WINDOW_TITLE_HEIGHT: f32 = 38.0;
    /// Margin of window frame at desktop.
    pub const WINDOW_FRAME_MARGIN: f32 = 6.0;

    pub fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        // Draw modal content for current ui container.
        self.current_modal_ui(ui, cb);

        let dual_panel = Self::is_dual_panel_mode(ui);
        let (is_panel_open, panel_width) = Self::network_panel_state_width(ui, dual_panel);

        // Show network content.
        egui::SidePanel::left("network_panel")
            .resizable(false)
            .exact_width(panel_width)
            .frame(egui::Frame {
                ..Default::default()
            })
            .show_animated_inside(ui, is_panel_open, |ui| {
                self.network.ui(ui, cb);
            });

        // Show wallets content.
        egui::CentralPanel::default()
            .frame(egui::Frame {
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                self.wallets.ui(ui, cb);
            });

        // Show integrated node warning on Android if needed.
        if self.first_draw && OperatingSystem::from_target_os() == OperatingSystem::Android &&
            AppConfig::android_integrated_node_warning_needed() {
            Modal::new(Self::ANDROID_INTEGRATED_NODE_WARNING_MODAL)
                .title(t!("network.node"))
                .show();
        }

        // Setup first draw flag.
        if self.first_draw {
            self.first_draw = false;
        }
    }

    /// Get [`NetworkContent`] panel state and width.
    fn network_panel_state_width(ui: &mut egui::Ui, dual_panel: bool) -> (bool, f32) {
        let is_panel_open = dual_panel || Self::is_network_panel_open();
        let panel_width = if dual_panel {
            Self::SIDE_PANEL_WIDTH + View::get_left_inset()
        } else {
            let is_fullscreen = ui.ctx().input(|i| {
                i.viewport().fullscreen.unwrap_or(false)
            });
            View::window_size(ui).0 - if View::is_desktop() && !is_fullscreen &&
                OperatingSystem::from_target_os() != OperatingSystem::Mac {
                Self::WINDOW_FRAME_MARGIN * 2.0
            } else {
                0.0
            }
        };
        (is_panel_open, panel_width)
    }

    /// Check if ui can show [`NetworkContent`] and [`WalletsContent`] at same time.
    pub fn is_dual_panel_mode(ui: &egui::Ui) -> bool {
        let (w, h) = View::window_size(ui);
        // Screen is wide if width is greater than height or just 20% smaller.
        let is_wide_screen = w > h || w + (w * 0.2) >= h;
        // Dual panel mode is available when window is wide and its width is at least 2 times
        // greater than minimal width of the side panel plus display insets from both sides.
        let side_insets = View::get_left_inset() + View::get_right_inset();
        is_wide_screen && w >= (Self::SIDE_PANEL_WIDTH * 2.0) + side_insets
    }

    /// Toggle [`NetworkContent`] panel state.
    pub fn toggle_network_panel() {
        let is_open = NETWORK_PANEL_OPEN.load(Ordering::Relaxed);
        NETWORK_PANEL_OPEN.store(!is_open, Ordering::Relaxed);
    }

    /// Check if [`NetworkContent`] panel is open.
    pub fn is_network_panel_open() -> bool {
        NETWORK_PANEL_OPEN.load(Ordering::Relaxed)
    }

    /// Show exit confirmation modal.
    pub fn show_exit_modal() {
        Modal::new(Self::EXIT_MODAL_ID)
            .title(t!("modal.confirmation"))
            .show();
    }

    /// Draw exit confirmation modal content.
    fn exit_modal_content(&mut self, ui: &mut egui::Ui, modal: &Modal) {
        if self.show_exit_progress {
            if !Node::is_running() {
                self.exit_allowed = true;
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                modal.close();
            }
            ui.add_space(16.0);
            ui.vertical_centered(|ui| {
                View::small_loading_spinner(ui);
                ui.add_space(12.0);
                ui.label(RichText::new(t!("sync_status.shutdown"))
                    .size(17.0)
                    .color(Colors::text(false)));
            });
            ui.add_space(10.0);
        } else {
            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(t!("modal_exit.description"))
                    .size(17.0)
                    .color(Colors::text(false)));
            });
            ui.add_space(10.0);

            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button_ui(ui, t!("modal_exit.exit"), Colors::white_or_black(false), |ui| {
                        if !Node::is_running() {
                            self.exit_allowed = true;
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                            modal.close();
                        } else {
                            Node::stop(true);
                            modal.disable_closing();
                            Modal::set_title(t!("modal_exit.exit"));
                            self.show_exit_progress = true;
                        }
                    });
                });
            });
            ui.add_space(6.0);
        }
    }

    /// Handle Back key event.
    pub fn on_back(&mut self) {
        if Modal::on_back() {
            if self.wallets.on_back() {
                Self::show_exit_modal()
            }
        }
    }

    /// Draw creating wallet name/password input [`Modal`] content.
    pub fn settings_modal_ui(&mut self, ui: &mut egui::Ui, modal: &Modal) {
        ui.add_space(6.0);

        // Draw chain type selection.
        NodeSetup::chain_type_ui(ui);

        ui.add_space(8.0);
        View::horizontal_line(ui, Colors::item_stroke());
        ui.add_space(8.0);

        // Draw theme selection.
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
            Self::language_item_ui(locale, ui, index, locales.len(), modal);
        }

        ui.add_space(8.0);

        // Show button to close modal.
        ui.vertical_centered_justified(|ui| {
            View::button(ui, t!("close"), Colors::white_or_black(false), || {
                modal.close();
            });
        });
        ui.add_space(6.0);
    }

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
    }

    /// Draw language selection item content.
    fn language_item_ui(locale: &str, ui: &mut egui::Ui, index: usize, len: usize, modal: &Modal) {
        // Setup layout size.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(50.0);

        // Draw round background.
        let bg_rect = rect.clone();
        let item_rounding = View::item_rounding(index, len, false);
        ui.painter().rect(bg_rect, item_rounding, Colors::fill(), View::item_stroke());

        ui.vertical(|ui| {
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
                        modal.close();
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
        });
    }

    /// Draw content for integrated node warning [`Modal`] on Android.
    fn android_warning_modal_ui(&mut self, ui: &mut egui::Ui, modal: &Modal) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network.android_warning"))
                .size(15.0)
                .color(Colors::text(false)));
        });
        ui.add_space(8.0);
        ui.vertical_centered_justified(|ui| {
            View::button(ui, t!("continue"), Colors::white_or_black(false), || {
                AppConfig::show_android_integrated_node_warning();
                modal.close();
            });
        });
        ui.add_space(6.0);
    }
}