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
use egui::{Align, Context, CursorIcon, Layout, Modifiers, Rect, ResizeDirection, Rounding, Stroke, ViewportCommand};
use egui::epaint::{RectShape};
use egui::os::OperatingSystem;

use crate::AppConfig;
use crate::gui::Colors;
use crate::gui::icons::{ARROWS_IN, ARROWS_OUT, CARET_DOWN, MOON, SUN, X};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Content, TitlePanel, View};

lazy_static! {
    /// State to check if platform Back button was pressed.
    static ref BACK_BUTTON_PRESSED: AtomicBool = AtomicBool::new(false);
}

/// Implements ui entry point and contains platform-specific callbacks.
pub struct App<Platform> {
    /// Platform specific callbacks handler.
    pub platform: Platform,
    /// Main content.
    content: Content,
    /// Last window resize direction.
    resize_direction: Option<ResizeDirection>,
    /// Flag to check if it's first draw.
    first_draw: bool,
}

impl<Platform: PlatformCallbacks> App<Platform> {
    pub fn new(platform: Platform) -> Self {
        Self {
            platform,
            content: Content::default(),
            resize_direction: None,
            first_draw: true,
        }
    }

    /// Draw application content.
    pub fn ui(&mut self, ctx: &Context) {
        // Set Desktop platform context on first draw.
        if self.first_draw {
            if View::is_desktop() {
                self.platform.set_context(ctx);
            }
            self.first_draw = false;
        }

        // Handle Esc keyboard key event and platform Back button key event.
        let back_pressed = BACK_BUTTON_PRESSED.load(Ordering::Relaxed);
        if back_pressed || ctx.input_mut(|i| i.consume_key(Modifiers::NONE, egui::Key::Escape)) {
            self.content.on_back();
            if back_pressed {
                BACK_BUTTON_PRESSED.store(false, Ordering::Relaxed);
            }
            // Request repaint to update previous content.
            ctx.request_repaint();
        }

        // Handle Close event on desktop.
        if View::is_desktop() && ctx.input(|i| i.viewport().close_requested()) {
            if !self.content.exit_allowed {
                ctx.send_viewport_cmd(ViewportCommand::CancelClose);
                Content::show_exit_modal();
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
        egui::CentralPanel::default()
            .frame(egui::Frame {
                ..Default::default()
            })
            .show(ctx, |ui| {
                let is_mac_os = OperatingSystem::from_target_os() == OperatingSystem::Mac;
                if View::is_desktop() && !is_mac_os {
                    self.desktop_window_ui(ui);
                } else {
                    if is_mac_os {
                        self.window_title_ui(ui);
                        ui.add_space(-1.0);
                    }
                    self.content.ui(ui, &self.platform);
                }

                // Provide incoming data to wallets.
                if let Some(data) = self.platform.consume_data() {
                    self.content.wallets.on_data(ui, Some(data), &self.platform);
                }
            });
    }

    /// Draw custom resizeable window content.
    fn desktop_window_ui(&mut self, ui: &mut egui::Ui) {
        let is_fullscreen = ui.ctx().input(|i| {
            i.viewport().fullscreen.unwrap_or(false)
        });

        let title_stroke_rect = {
            let mut rect = ui.max_rect();
            if !is_fullscreen {
                rect = rect.shrink(Content::WINDOW_FRAME_MARGIN);
            }
            rect.max.y = if !is_fullscreen {
                Content::WINDOW_FRAME_MARGIN
            } else {
                0.0
            } + Content::WINDOW_TITLE_HEIGHT + TitlePanel::DEFAULT_HEIGHT + 0.5;
            rect
        };
        let title_stroke = RectShape {
            rect: title_stroke_rect,
            rounding: Rounding {
                nw: 8.0,
                ne: 8.0,
                sw: 0.0,
                se: 0.0,
            },
            fill: Colors::yellow(),
            stroke: Stroke {
                width: 1.0,
                color: egui::Color32::from_gray(200)
            },
            blur_width: 0.0,
            fill_texture_id: Default::default(),
            uv: Rect::ZERO
        };
        // Draw title stroke.
        ui.painter().add(title_stroke);

        let content_stroke_rect = {
            let mut rect = ui.max_rect();
            if !is_fullscreen {
                rect = rect.shrink(Content::WINDOW_FRAME_MARGIN);
            }
            let top = Content::WINDOW_TITLE_HEIGHT + TitlePanel::DEFAULT_HEIGHT + 0.5;
            rect.min += egui::vec2(0.0, top);
            rect
        };
        let content_stroke = RectShape {
            rect: content_stroke_rect,
            rounding: Rounding::ZERO,
            fill: Colors::fill(),
            stroke: Stroke {
                width: 1.0,
                color: Colors::stroke()
            },
            blur_width: 0.0,
            fill_texture_id: Default::default(),
            uv: Rect::ZERO
        };
        // Draw content stroke.
        ui.painter().add(content_stroke);

        // Draw window content.
        let mut content_rect = ui.max_rect();
        if !is_fullscreen {
            content_rect = content_rect.shrink(Content::WINDOW_FRAME_MARGIN);
        }
        ui.allocate_ui_at_rect(content_rect, |ui| {
            self.window_title_ui(ui);
            self.window_content(ui);
        });

        // Setup resize areas.
        if !is_fullscreen {
            self.resize_area_ui(ui, ResizeDirection::North);
            self.resize_area_ui(ui, ResizeDirection::East);
            self.resize_area_ui(ui, ResizeDirection::South);
            self.resize_area_ui(ui, ResizeDirection::West);
            self.resize_area_ui(ui, ResizeDirection::NorthWest);
            self.resize_area_ui(ui, ResizeDirection::NorthEast);
            self.resize_area_ui(ui, ResizeDirection::SouthEast);
            self.resize_area_ui(ui, ResizeDirection::SouthWest);
        }
    }

    /// Draw window content for desktop.
    fn window_content(&mut self, ui: &mut egui::Ui) {
        let content_rect = {
            let mut rect = ui.max_rect();
            rect.min.y += Content::WINDOW_TITLE_HEIGHT;
            rect
        };
        // Draw main content.
        let mut content_ui = ui.child_ui(content_rect, *ui.layout(), None);
        self.content.ui(&mut content_ui, &self.platform);
    }

    /// Draw custom window title content.
    fn window_title_ui(&self, ui: &mut egui::Ui) {
        let content_rect = ui.max_rect();

        let title_rect = {
            let mut rect = content_rect;
            rect.max.y = rect.min.y + Content::WINDOW_TITLE_HEIGHT;
            rect
        };

        let is_fullscreen = ui.ctx().input(|i| {
            i.viewport().fullscreen.unwrap_or(false)
        });

        let window_title_bg = RectShape {
            rect: title_rect,
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
            blur_width: 0.0,
            fill_texture_id: Default::default(),
            uv: Rect::ZERO
        };
        // Draw title background.
        ui.painter().add(window_title_bg);

        let painter = ui.painter();

        let interact_rect = {
            let mut rect = title_rect;
            if !is_fullscreen {
                rect.min.y += Content::WINDOW_FRAME_MARGIN;
            }
            rect
        };
        let title_resp = ui.interact(
            interact_rect,
            egui::Id::new("window_title"),
            egui::Sense::click_and_drag(),
        );

        // Paint the title.
        let dual_wallets_panel =
            ui.available_width() >= (Content::SIDE_PANEL_WIDTH * 3.0) + View::get_right_inset();
        let wallet_panel_opened = self.content.wallets.wallet_panel_opened();
        let hide_app_name = if dual_wallets_panel {
            !wallet_panel_opened || (AppConfig::show_wallets_at_dual_panel() &&
                self.content.wallets.showing_wallet() && !self.content.wallets.creating_wallet())
        } else if Content::is_dual_panel_mode(ui) {
            !wallet_panel_opened
        } else {
            !Content::is_network_panel_open() && !wallet_panel_opened
        };
        let title_text = if hide_app_name {
            "ツ".to_string()
        } else {
            format!("Grim {}", crate::VERSION)
        };
        painter.text(
            title_rect.center(),
            egui::Align2::CENTER_CENTER,
            title_text,
            egui::FontId::proportional(15.0),
            Colors::title(true),
        );

        // Interact with the window title (drag to move window):
        if !is_fullscreen && title_resp.double_clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Fullscreen(!is_fullscreen));
        }

        if !is_fullscreen && title_resp.drag_started_by(egui::PointerButton::Primary) {
            ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
        }

        ui.allocate_ui_at_rect(title_rect, |ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Draw button to close window.
                View::title_button_small(ui, X, |_| {
                    Content::show_exit_modal();
                });

                // Draw fullscreen button.
                let fullscreen_icon = if is_fullscreen {
                    ARROWS_IN
                } else {
                    ARROWS_OUT
                };
                View::title_button_small(ui, fullscreen_icon, |ui| {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Fullscreen(!is_fullscreen));
                });

                // Draw button to minimize window.
                View::title_button_small(ui, CARET_DOWN, |ui| {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
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

    /// Setup window resize area.
    fn resize_area_ui(&mut self, ui: &egui::Ui, direction: ResizeDirection) {
        let mut rect = ui.max_rect();

        // Setup area id, cursor and area rect based on direction.
        let (id, cursor, rect) = match direction {
            ResizeDirection::North => ("n", CursorIcon::ResizeNorth, {
                rect.min.x += Content::WINDOW_FRAME_MARGIN * 2.0;
                rect.max.y = rect.min.y + Content::WINDOW_FRAME_MARGIN;
                rect.max.x -= Content::WINDOW_FRAME_MARGIN * 2.0;
                rect
            }),
            ResizeDirection::East => ("e", CursorIcon::ResizeEast, {
                rect.min.y += Content::WINDOW_FRAME_MARGIN * 2.0;
                rect.min.x = rect.max.x - Content::WINDOW_FRAME_MARGIN;
                rect.max.y -= Content::WINDOW_FRAME_MARGIN * 2.0;
                rect
            }),
            ResizeDirection::South => ("s", CursorIcon::ResizeSouth, {
                rect.min.x += Content::WINDOW_FRAME_MARGIN * 2.0;
                rect.min.y = rect.max.y - Content::WINDOW_FRAME_MARGIN;
                rect.max.x -= Content::WINDOW_FRAME_MARGIN * 2.0;
                rect
            }),
            ResizeDirection::West => ("w", CursorIcon::ResizeWest, {
                rect.min.y += Content::WINDOW_FRAME_MARGIN * 2.0;
                rect.max.x = rect.min.x + Content::WINDOW_FRAME_MARGIN;
                rect.max.y -= Content::WINDOW_FRAME_MARGIN * 2.0;
                rect
            }),
            ResizeDirection::NorthWest => ("nw", CursorIcon::ResizeNorthWest, {
                rect.max.y = rect.min.y + Content::WINDOW_FRAME_MARGIN * 2.0;
                rect.max.x = rect.max.y + Content::WINDOW_FRAME_MARGIN * 2.0;
                rect
            }),
            ResizeDirection::NorthEast => ("ne", CursorIcon::ResizeNorthEast, {
                rect.min.x = rect.max.x - Content::WINDOW_FRAME_MARGIN * 2.0;
                rect.max.y = Content::WINDOW_FRAME_MARGIN * 2.0;
                rect
            }),
            ResizeDirection::SouthEast => ("se", CursorIcon::ResizeSouthEast, {
                rect.min.y = rect.max.y - Content::WINDOW_FRAME_MARGIN * 2.0;
                rect.min.x = rect.max.x - Content::WINDOW_FRAME_MARGIN * 2.0;
                rect
            }),
            ResizeDirection::SouthWest => ("sw", CursorIcon::ResizeSouthWest, {
                rect.min.y = rect.max.y - Content::WINDOW_FRAME_MARGIN * 2.0;
                rect.max.x = rect.min.x + Content::WINDOW_FRAME_MARGIN * 2.0;
                rect
            }),
        };

        // Setup resize area.
        let id = egui::Id::new("window_resize").with(id);
        let sense = egui::Sense::drag();
        let area_resp = ui.interact(rect, id, sense).on_hover_cursor(cursor);
        if area_resp.dragged() {
            if self.resize_direction.is_none() {
                self.resize_direction = Some(direction.clone());
                ui.ctx().send_viewport_cmd(ViewportCommand::BeginResize(direction));
            }
        }
        if area_resp.drag_stopped() {
            self.resize_direction = None;
        }
    }
}

/// To draw with egui`s eframe (for wgpu, glow backends and wasm target).
impl<Platform: PlatformCallbacks> eframe::App for App<Platform> {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        self.ui(ctx);
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        if View::is_desktop() {
            let is_mac_os = OperatingSystem::from_target_os() == OperatingSystem::Mac;
            if is_mac_os {
                Colors::fill().to_normalized_gamma_f32()
            } else {
                egui::Rgba::TRANSPARENT.to_array()
            }
        } else {
            Colors::fill().to_normalized_gamma_f32()
        }
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