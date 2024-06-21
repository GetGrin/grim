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

use egui::{Align, Context, Layout, Modifiers, Rect, Rounding, Stroke};
use egui::epaint::RectShape;
use egui::os::OperatingSystem;
use lazy_static::lazy_static;

use crate::AppConfig;
use crate::gui::Colors;
use crate::gui::icons::{ARROWS_IN, ARROWS_OUT, CARET_DOWN, MOON, SUN, X};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Root, TitlePanel, View};

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
        let os = OperatingSystem::from_target_os();
        let custom_window = os != OperatingSystem::Android;
        if custom_window {
            custom_window_frame(ctx, |ui| {
                self.root.ui(ui, &self.platform);
            });
        } else {
            egui::CentralPanel::default()
                .frame(egui::Frame {
                    ..Default::default()
                })
                .show(ctx, |ui| {
                    self.root.ui(ui, &self.platform);
                });
        }
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

/// Draw custom window frame for desktop.
fn custom_window_frame(ctx: &Context, add_contents: impl FnOnce(&mut egui::Ui)) {
    let panel_frame = egui::Frame {
        fill: Colors::fill(),
        rounding: Rounding {
            nw: 8.0,
            ne: 8.0,
            sw: 0.0,
            se: 0.0,
        },
        // stroke: ctx.style().visuals.widgets.noninteractive.fg_stroke,
        // outer_margin: 0.5.into(), // so the stroke is within the bounds
        ..Default::default()
    };

    egui::CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
        let app_rect = ui.max_rect();

        let window_title_height = 38.0;
        let window_title_rect = {
            let mut rect = app_rect;
            rect.max.y = rect.min.y + window_title_height;
            rect
        };

        let window_title_bg = RectShape {
            rect: window_title_rect,
            rounding: panel_frame.rounding,
            fill: Colors::yellow_dark(),
            stroke: Stroke::NONE,
            fill_texture_id: Default::default(),
            uv: Rect::ZERO
        };
        let bg_idx = ui.painter().add(window_title_bg);

        // Draw window title.
        window_title_ui(ui, window_title_rect);

        // Setup window title background.
        ui.painter().set(bg_idx, window_title_bg);

        let mut title_bar_rect = window_title_rect.clone();
        title_bar_rect.min += egui::emath::vec2(0.0, window_title_height);
        title_bar_rect.max += egui::emath::vec2(0.0, TitlePanel::DEFAULT_HEIGHT - 0.5);
        let title_bar_bg = RectShape {
            rect: title_bar_rect,
            rounding: Rounding::ZERO,
            fill: Colors::yellow(),
            stroke: Stroke::NONE,
            fill_texture_id: Default::default(),
            uv: Rect::ZERO
        };
        let bg_idx = ui.painter().add(title_bar_bg);

        // Draw main content.
        let mut content_rect = {
            let mut rect = app_rect;
            rect.min.y = window_title_rect.max.y;
            rect
        };
        content_rect.min += egui::emath::vec2(4.0, 0.0);
        content_rect.max -= egui::emath::vec2(4.0, 4.0);
        let mut content_ui = ui.child_ui(content_rect, *ui.layout());
        add_contents(&mut content_ui);

        // Setup title bar background.
        ui.painter().set(bg_idx, title_bar_bg);
    });
}

/// Draw custom window title content.
fn window_title_ui(ui: &mut egui::Ui, title_bar_rect: egui::epaint::Rect) {
    let painter = ui.painter();

    let title_bar_response = ui.interact(
        title_bar_rect,
        egui::Id::new("title_bar"),
        egui::Sense::click_and_drag(),
    );

    // Paint the title.
    painter.text(
        title_bar_rect.center(),
        egui::Align2::CENTER_CENTER,
        "Grim 0.1.0",
        egui::FontId::proportional(15.0),
        egui::Color32::from_gray(60),
    );

    // Interact with the title bar (drag to move window):
    if title_bar_response.double_clicked() {
        let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
        ui.ctx()
            .send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
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
            let is_fullscreen = ui.ctx().input(|i| {
                i.viewport().fullscreen.unwrap_or(false)
            });
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