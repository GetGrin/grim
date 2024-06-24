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
use lazy_static::lazy_static;
use egui::{Align, Context, Layout, Margin, Modifiers, Rect, Rounding, Stroke};
use egui::epaint::{RectShape, Shadow};

use crate::{AppConfig, built_info};
use crate::gui::Colors;
use crate::gui::icons::{ARROWS_IN, ARROWS_OUT, CARET_DOWN, MOON, SUN, X};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Root, View};

lazy_static! {
    /// State to check if platform Back button was pressed.
    static ref BACK_BUTTON_PRESSED: AtomicBool = AtomicBool::new(false);
}

/// Implements ui entry point and contains platform-specific callbacks.
pub struct App<Platform> {
    /// Platform specific callbacks handler.
    pub(crate) platform: Platform,
    /// Main ui content.
    root: Root
}

impl<Platform: PlatformCallbacks> App<Platform> {
    pub fn new(platform: Platform) -> Self {
        Self { platform, root: Root::default() }
    }

    /// Draw application content.
    pub fn ui(&mut self, ctx: &Context) {
        // Handle Esc keyboard key event and platform Back button key event.
        let back_button_pressed = BACK_BUTTON_PRESSED.load(Ordering::Relaxed);
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, egui::Key::Escape)) || back_button_pressed {
            self.root.on_back();
            if back_button_pressed {
                BACK_BUTTON_PRESSED.store(false, Ordering::Relaxed);
            }
            // Request repaint to update previous content.
            ctx.request_repaint();
        }

        // Handle Close event (on desktop).
        if ctx.input(|i| i.viewport().close_requested()) {
            if !self.root.exit_allowed {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                Root::show_exit_modal();
            } else {
                ctx.input(|i| {
                    if let Some(rect) = i.viewport().inner_rect {
                        AppConfig::save_window_size(rect.width(), rect.height());
                    }
                    if let Some(rect) = i.viewport().outer_rect {
                        AppConfig::save_window_pos(rect.left(), rect.top());
                    }
                });
            }
        }

        // Show main content with custom frame on desktop.
        if View::is_desktop() {
            self.window_frame_ui(ctx);
        } else {
            egui::CentralPanel::default()
                .frame(egui::Frame {
                    stroke: Stroke::NONE,
                    ..Default::default()
                })
                .show(ctx, |ui| {
                    self.root.ui(ui, &self.platform);
                });
        }
    }

    /// Draw custom resizeable window frame for desktop.
    fn window_frame_ui(&mut self, ctx: &Context) {
        egui::CentralPanel::default().frame(egui::Frame {
            inner_margin: Margin::same(Root::WINDOW_FRAME_MARGIN),
            ..Default::default()
        }).show(ctx, |ui| {
            self.custom_window_frame(ui);
        });
    }

    /// Draw custom window frame for desktop.
    fn custom_window_frame(&mut self, ui: &mut egui::Ui) {
        let is_fullscreen = ui.ctx().input(|i| {
            i.viewport().fullscreen.unwrap_or(false)
        });
        let panel_frame = if is_fullscreen {
            egui::Frame::default()
        } else {
            egui::Frame {
                shadow: Shadow {
                    offset: Default::default(),
                    blur: Root::WINDOW_FRAME_MARGIN,
                    spread: 0.5,
                    color: egui::Color32::from_black_alpha(25),
                },
                rounding: Rounding {
                    nw: 8.0,
                    ne: 8.0,
                    sw: 0.0,
                    se: 0.0,
                },
                ..Default::default()
            }
        };
        egui::CentralPanel::default().frame(panel_frame).show_inside(ui, |ui| {
            let app_rect = ui.max_rect();

            let window_title_height = Root::WINDOW_TITLE_HEIGHT;
            let window_title_rect = {
                let mut rect = app_rect;
                rect.max.y = rect.min.y + window_title_height;
                rect
            };

            let window_title_bg = RectShape {
                rect: window_title_rect,
                rounding: if is_fullscreen {
                    Rounding::ZERO
                } else {
                    Rounding {
                        nw: 8.0,
                        ne: 8.0,
                        sw: 0.0,
                        se: 0.0,
                    }
                },
                fill: Colors::yellow_dark(),
                stroke: Stroke::NONE,
                fill_texture_id: Default::default(),
                uv: Rect::ZERO
            };
            ui.painter().add(window_title_bg);

            // Draw window title.
            self.window_title_ui(ui, window_title_rect);

            let content_rect = {
                let mut rect = app_rect;
                rect.min.y = window_title_rect.max.y;
                rect
            };
            // Draw main content.
            let mut content_ui = ui.child_ui(content_rect, *ui.layout());
            self.root.ui(&mut content_ui, &self.platform);
        });
    }

    /// Draw custom window title content.
    fn window_title_ui(&self, ui: &mut egui::Ui, title_bar_rect: Rect) {
        let is_fullscreen = ui.ctx().input(|i| {
            i.viewport().fullscreen.unwrap_or(false)
        });

        let painter = ui.painter();

        let title_bar_response = ui.interact(
            title_bar_rect,
            egui::Id::new("title_bar"),
            egui::Sense::click_and_drag(),
        );

        // Paint the title.
        let dual_wallets_panel =
            ui.available_width() >= (Root::SIDE_PANEL_WIDTH * 3.0) + View::get_right_inset();
        let wallet_panel_opened = self.root.wallets.wallet_panel_opened();
        let hide_app_name = if dual_wallets_panel {
            !wallet_panel_opened || (AppConfig::show_wallets_at_dual_panel() &&
                self.root.wallets.showing_wallet() && !self.root.wallets.creating_wallet())
        } else if Root::is_dual_panel_mode(ui) {
            !wallet_panel_opened
        } else {
            !Root::is_network_panel_open() && !wallet_panel_opened
        };
        let title_text = if hide_app_name {
            "ツ".to_string()
        } else {
            format!("Grim {}", built_info::PKG_VERSION)
        };
        painter.text(
            title_bar_rect.center(),
            egui::Align2::CENTER_CENTER,
            title_text,
            egui::FontId::proportional(15.0),
            egui::Color32::from_gray(60),
        );

        // Interact with the window title (drag to move window):
        if title_bar_response.double_clicked() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Fullscreen(!is_fullscreen));
        }

        if title_bar_response.drag_started_by(egui::PointerButton::Primary) {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }

        ui.allocate_ui_at_rect(title_bar_rect, |ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Draw button to close window.
                View::title_button_small(ui, X, |_| {
                    Root::show_exit_modal();
                });

                // Draw fullscreen button.
                let fullscreen_icon = if is_fullscreen {
                    ARROWS_IN
                } else {
                    ARROWS_OUT
                };
                View::title_button_small(ui, fullscreen_icon, |ui| {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Fullscreen(!is_fullscreen));
                });

                // Draw button to minimize window.
                View::title_button_small(ui, CARET_DOWN, |ui| {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                });

                // Draw application icon.
                let layout_size = ui.available_size();
                ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                    // Draw button to minimize window.
                    let use_dark = AppConfig::dark_theme().unwrap_or(false);
                    let theme_icon = if use_dark {
                        SUN
                    } else {
                        MOON
                    };
                    View::title_button_small(ui, theme_icon, |ui| {
                        AppConfig::set_dark_theme(!use_dark);
                        crate::setup_visuals(ui.ctx());
                    });
                });
            });
        });
    }
}

/// To draw with egui`s eframe (for wgpu, glow backends and wasm target).
impl<Platform: PlatformCallbacks> eframe::App for App<Platform> {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        self.ui(ctx);
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Handle Back key code event from Android.
pub extern "C" fn Java_mw_gri_android_MainActivity_onBack(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    _activity: jni::objects::JObject,
) {
    BACK_BUTTON_PRESSED.store(true, Ordering::Relaxed);
}