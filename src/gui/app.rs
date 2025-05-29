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

use egui::epaint::RectShape;
use egui::{Align, Context, CornerRadius, CursorIcon, Layout, Modifiers, ResizeDirection, Stroke, StrokeKind, UiBuilder, ViewportCommand};
use lazy_static::lazy_static;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::gui::icons::{ARROWS_IN, ARROWS_OUT, CARET_DOWN, MOON, SUN, X};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Content, Modal, TitlePanel, View};
use crate::gui::Colors;
use crate::wallet::ExternalConnection;
use crate::AppConfig;

lazy_static! {
    /// State to check if platform Back button was pressed.
    static ref BACK_BUTTON_PRESSED: AtomicBool = AtomicBool::new(false);
}

/// Implements ui entry point and contains platform-specific callbacks.
pub struct App<Platform> {
    /// Handles platform-specific functionality.
    pub platform: Platform,

    /// Main content.
    content: Content,

    /// Last window resize direction.
    resize_direction: Option<ResizeDirection>,
    /// Flag to check if it's first draw.
    first_draw: bool
}

impl<Platform: PlatformCallbacks> App<Platform> {
    pub fn new(platform: Platform) -> Self {
        Self {
            platform,
            content: Content::default(),
            resize_direction: None,
            first_draw: true
        }
    }

    /// Called of first content draw.
    fn on_first_draw(&mut self, ctx: &Context) {
        // Set platform context.
        if View::is_desktop() {
            self.platform.set_context(ctx);
        }
        // Check connections availability at dual panel mode.
        if Content::is_dual_panel_mode(ctx) && AppConfig::show_connections_network_panel() {
            ExternalConnection::check(None, ctx);
        }
        // Setup visuals.
        crate::setup_visuals(ctx);
    }

    /// Draw application content.
    pub fn ui(&mut self, ctx: &Context) {
        if self.first_draw {
            self.on_first_draw(ctx);
            self.first_draw = false;
        }

        // Handle Esc keyboard key event.
        let back_pressed = BACK_BUTTON_PRESSED.load(Ordering::Relaxed);
        if back_pressed || ctx.input_mut(|i| i.consume_key(Modifiers::NONE, egui::Key::Escape)) {
            if Modal::on_back() {
                self.content.on_back(&self.platform);
            }
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
                let (w, h) = View::window_size(ctx);
                AppConfig::save_window_size(w, h);
                ctx.input(|i| {
                    if let Some(rect) = i.viewport().outer_rect {
                        AppConfig::save_window_pos(rect.left(), rect.top());
                    }
                });
            }
        }

        // Show main content.
        egui::CentralPanel::default()
            .frame(egui::Frame {
                ..Default::default()
            })
            .show(ctx, |ui| {
                if View::is_desktop() {
                    let is_fullscreen = ui.ctx().input(|i| {
                        i.viewport().fullscreen.unwrap_or(false)
                    });
                    let os = egui::os::OperatingSystem::from_target_os();
                    match os {
                        egui::os::OperatingSystem::Mac => {
                            self.window_title_ui(ui, is_fullscreen);
                            ui.add_space(-1.0);
                            Self::title_panel_bg(ui, true);
                            self.content.ui(ui, &self.platform);
                        }
                        egui::os::OperatingSystem::Windows => {
                            Self::title_panel_bg(ui, false);
                            self.content.ui(ui, &self.platform);
                        }
                        _ => {
                            self.custom_frame_ui(ui, is_fullscreen);
                        }
                    }
                } else {
                    Self::title_panel_bg(ui, false);
                    self.content.ui(ui, &self.platform);
                }

                // Provide incoming data to wallets.
                if let Some(data) = crate::consume_incoming_data() {
                    if !data.is_empty() {
                        self.content.wallets.on_data(ui, Some(data));
                    }
                }
            });

        // Check if desktop window was focused after requested attention.
        if self.platform.user_attention_required() &&
            ctx.input(|i| i.viewport().focused.unwrap_or(true)) {
            self.platform.clear_user_attention();
        }
    }

    /// Draw custom desktop window frame content.
    fn custom_frame_ui(&mut self, ui: &mut egui::Ui, is_fullscreen: bool) {
        let content_bg_rect = {
            let mut r = ui.max_rect();
            if !is_fullscreen {
                r = r.shrink(Content::WINDOW_FRAME_MARGIN);
            }
            r.min.y += Content::WINDOW_TITLE_HEIGHT + TitlePanel::HEIGHT;
            r
        };
        let content_bg = RectShape::new(content_bg_rect,
                                        CornerRadius::ZERO,
                                        Colors::fill_lite(),
                                        View::default_stroke(),
                                        StrokeKind::Middle);
        // Draw content background.
        ui.painter().add(content_bg);

        let mut content_rect = ui.max_rect();
        if !is_fullscreen {
            content_rect = content_rect.shrink(Content::WINDOW_FRAME_MARGIN);
        }
        // Draw window content.
        ui.scope_builder(UiBuilder::new().max_rect(content_rect), |ui| {
            // Draw window title.
            self.window_title_ui(ui, is_fullscreen);
            ui.add_space(-1.0);

            // Draw title panel background.
            Self::title_panel_bg(ui, true);

            let content_rect = {
                let mut rect = ui.max_rect();
                rect.min.y += Content::WINDOW_TITLE_HEIGHT;
                rect
            };
            let mut content_ui = ui.new_child(UiBuilder::new()
                .max_rect(content_rect)
                .layout(*ui.layout()));
            // Draw main content.
            self.content.ui(&mut content_ui, &self.platform);
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

    /// Draw title panel background.
    fn title_panel_bg(ui: &mut egui::Ui, window_title: bool) {
        let title_rect = {
            let mut rect = ui.max_rect();
            if window_title {
                rect.min.y += Content::WINDOW_TITLE_HEIGHT - 0.5;
            }
            rect.max.y = rect.min.y + View::get_top_inset() + TitlePanel::HEIGHT;
            rect
        };
        let title_bg = RectShape::filled(title_rect, CornerRadius::ZERO, Colors::yellow());
        ui.painter().add(title_bg);
    }

    /// Draw custom window title content.
    fn window_title_ui(&self, ui: &mut egui::Ui, is_fullscreen: bool) {
        let title_rect = {
            let mut rect = ui.max_rect();
            rect.max.y = rect.min.y + Content::WINDOW_TITLE_HEIGHT;
            rect
        };

        let title_bg_rect = {
            let mut r = title_rect.clone();
            r.max.y += TitlePanel::HEIGHT - 1.0;
            r
        };
        let is_mac = egui::os::OperatingSystem::from_target_os() == egui::os::OperatingSystem::Mac;
        let window_title_bg = RectShape::new(title_bg_rect, if is_fullscreen || is_mac {
            CornerRadius::ZERO
        } else {
            CornerRadius {
                nw: 8.0 as u8,
                ne: 8.0 as u8,
                sw: 0.0 as u8,
                se: 0.0 as u8,
            }
        }, Colors::yellow_dark(), Stroke::new(1.0, Colors::STROKE), StrokeKind::Middle);
        // Draw title background.
        ui.painter().add(window_title_bg);

        let painter = ui.painter();

        let interact_rect = {
            let mut rect = title_rect.clone();
            rect.max.x -= 128.0;
            rect.min.x += 85.0;
            if !is_fullscreen {
                rect.min.y += Content::WINDOW_FRAME_MARGIN;
            }
            rect
        };
        let title_resp = ui.interact(
            interact_rect,
            egui::Id::new("window_title"),
            egui::Sense::drag(),
        );
        // Interact with the window title (drag to move window):
        if !is_fullscreen && title_resp.drag_started_by(egui::PointerButton::Primary) {
            ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
        }

        // Paint the title.
        let dual_wallets_panel = ui.available_width() >= (Content::SIDE_PANEL_WIDTH * 3.0) +
            View::get_right_inset() + View::get_left_inset();
        let wallet_panel_opened = self.content.wallets.showing_wallet();
        let show_app_name = if dual_wallets_panel {
            wallet_panel_opened && !AppConfig::show_wallets_at_dual_panel()
        } else if Content::is_dual_panel_mode(ui.ctx()) {
            wallet_panel_opened
        } else {
            Content::is_network_panel_open() || wallet_panel_opened
        };
        let creating_wallet = self.content.wallets.creating_wallet();
        let title_text = if creating_wallet || show_app_name {
            format!("Grim {}", crate::VERSION)
        } else {
            "ツ".to_string()
        };
        painter.text(
            title_rect.center(),
            egui::Align2::CENTER_CENTER,
            title_text,
            egui::FontId::proportional(15.0),
            Colors::title(true),
        );

        ui.scope_builder(UiBuilder::new().max_rect(title_rect), |ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Draw button to close window.
                View::title_button_small(ui, X, |_| {
                    if Modal::opened().is_none() || Modal::opened_closeable() {
                        Content::show_exit_modal();
                    }
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
        let os = egui::os::OperatingSystem::from_target_os();
        let is_win = os == egui::os::OperatingSystem::Windows;
        let is_mac = os == egui::os::OperatingSystem::Mac;
        if !View::is_desktop() || is_win || is_mac {
            return Colors::fill_lite().to_normalized_gamma_f32();
        }
        Colors::TRANSPARENT.to_normalized_gamma_f32()
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