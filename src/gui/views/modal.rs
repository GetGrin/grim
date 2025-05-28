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

use egui::epaint::{RectShape, Shadow};
use egui::os::OperatingSystem;
use egui::{Align2, CornerRadius, RichText, Stroke, StrokeKind, UiBuilder, Vec2};
use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::gui::views::types::{ModalPosition, ModalState};
use crate::gui::views::{Content, View};
use crate::gui::Colors;

lazy_static! {
    /// Showing [`Modal`] state to be accessible from different ui parts.
    static ref MODAL_STATE: Arc<RwLock<ModalState>> = Arc::new(RwLock::new(ModalState::default()));
}

/// Modal [`egui::Window`] container.
#[derive(Clone)]
pub struct Modal {
    /// Identifier for modal.
    pub(crate) id: &'static str,
    /// Position on the screen.
    pub position: ModalPosition,
    /// Flag to check if modal can be closed by keys.
    closeable: Arc<AtomicBool>,
    /// Title text.
    title: Option<String>,
    /// Flag to check first content render.
    first_draw: Arc<AtomicBool>,
}

impl Modal {
    /// Margin from [`Modal`] window at top/left/right.
    const DEFAULT_MARGIN: f32 = 8.0;
    /// Maximum width of the content.
    const DEFAULT_WIDTH: f32 = Content::SIDE_PANEL_WIDTH - (2.0 * Self::DEFAULT_MARGIN);

    /// Create closeable [`Modal`] with center position.
    pub fn new(id: &'static str) -> Self {
        Self {
            id,
            position: ModalPosition::Center,
            closeable: Arc::new(AtomicBool::new(true)),
            title: None,
            first_draw: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Setup position of [`Modal`] on the screen.
    pub fn position(mut self, position: ModalPosition) -> Self {
        self.position = position;
        self
    }

    /// Change [`Modal`] position on the screen.
    pub fn change_position(position: ModalPosition) {
        let mut w_state = MODAL_STATE.write();
        w_state.modal.as_mut().unwrap().position = position;
    }

    /// Close [`Modal`] by clearing its state.
    pub fn close() {
        let mut w_nav = MODAL_STATE.write();
        w_nav.modal = None;
    }

    /// Setup possibility to close [`Modal`].
    pub fn closeable(self, closeable: bool) -> Self {
        self.closeable.store(closeable, Ordering::Relaxed);
        self
    }

    /// Disable possibility to close [`Modal`].
    pub fn disable_closing(&self) {
        self.closeable.store(false, Ordering::Relaxed);
    }

    /// Enable possibility to close [`Modal`].
    pub fn enable_closing(&self) {
        self.closeable.store(true, Ordering::Relaxed);
    }

    /// Check if [`Modal`] is closeable.
    pub fn is_closeable(&self) -> bool {
        self.closeable.load(Ordering::Relaxed)
    }

    /// Set title text on [`Modal`] creation.
    pub fn title(mut self, title: String) -> Self {
        self.title = Some(title.to_uppercase());
        self
    }

    /// Set [`Modal`] instance into state to show at ui.
    pub fn show(self) {
        let mut w_nav = MODAL_STATE.write();
        w_nav.modal = Some(self);
    }

    /// Remove [`Modal`] from [`ModalState`] if it's showing and can be closed.
    /// Return `false` if modal existed in state before call.
    pub fn on_back() -> bool {
        if Self::opened().is_some() {
            Self::close();
            return false;
        }
        true
    }

    /// Return identifier of opened [`Modal`].
    pub fn opened() -> Option<&'static str> {
        // Check if modal is showing.
        {
            if MODAL_STATE.read().modal.is_none() {
                return None;
            }
        }

        // Get identifier of opened modal.
        let r_state = MODAL_STATE.read();
        let modal = r_state.modal.as_ref().unwrap();
        Some(modal.id)
    }

    /// Check if [`Modal`] is opened and can be closed.
    pub fn opened_closeable() -> bool {
        // Check if modal is showing.
        {
            if MODAL_STATE.read().modal.is_none() {
                return false;
            }
        }
        let r_state = MODAL_STATE.read();
        let modal = r_state.modal.as_ref().unwrap();
        modal.closeable.load(Ordering::Relaxed)
    }

    /// Set title text for current opened [`Modal`].
    pub fn set_title(title: String) {
        let mut w_state = MODAL_STATE.write();
        if w_state.modal.is_some() {
            let mut modal = w_state.modal.clone().unwrap();
            modal.title = Some(title.to_uppercase());
            w_state.modal = Some(modal);
        }
    }

    /// Check for first [`Modal`] content rendering.
    pub fn first_draw() -> bool {
        if Self::opened().is_none() {
            return false;
        }
        let r_state = MODAL_STATE.read();
        let modal = r_state.modal.as_ref().unwrap();
        modal.first_draw.load(Ordering::Relaxed)
    }

    /// Draw opened [`Modal`] content.
    pub fn ui(ctx: &egui::Context, add_content: impl FnOnce(&mut egui::Ui, &Modal)) {
        let has_modal = {
            MODAL_STATE.read().modal.is_some()
        };
        if has_modal {
            let modal = {
                let r_state = MODAL_STATE.read();
                r_state.modal.clone().unwrap()
            };
            modal.window_ui(ctx, add_content);
        }
    }

    /// Draw [`egui::Window`] with provided content.
    fn window_ui(&self, ctx: &egui::Context, add_content: impl FnOnce(&mut egui::Ui, &Modal)) {
        let is_fullscreen = ctx.input(|i| {
            i.viewport().fullscreen.unwrap_or(false)
        });

        // Setup background rect.
        let is_win = OperatingSystem::Windows == OperatingSystem::from_target_os();
        let bg_rect = if View::is_desktop() && !is_win {
            let mut r = ctx.screen_rect();
            let is_mac = OperatingSystem::Mac == OperatingSystem::from_target_os();
            if !is_mac && !is_fullscreen {
                r = r.shrink(Content::WINDOW_FRAME_MARGIN - 1.0);
            }
            r.min.y += Content::WINDOW_TITLE_HEIGHT;
            r
        } else {
            ctx.screen_rect()
        };

        // Draw modal background.
        egui::Window::new("modal_bg_window")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .fixed_rect(bg_rect)
            .frame(egui::Frame {
                fill: Colors::semi_transparent(),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.set_min_size(bg_rect.size());
            });

        // Setup width of modal content.
        let side_insets = View::get_left_inset() + View::get_right_inset();
        let available_width = ctx.screen_rect().width() - (side_insets + Self::DEFAULT_MARGIN);
        let width = f32::min(available_width, Self::DEFAULT_WIDTH);

        // Show main content window at given position.
        let (content_align, content_offset) = self.modal_position();
        let layer_id = egui::Window::new("modal_window")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .min_width(width)
            .default_width(width)
            .anchor(content_align, content_offset)
            .frame(egui::Frame {
                shadow: Shadow {
                    offset: Default::default(),
                    blur: 30.0 as u8,
                    spread: 3.0 as u8,
                    color: egui::Color32::from_black_alpha(32),
                },
                corner_radius: CornerRadius::same(8.0 as u8),
                ..Default::default()
            })
            .show(ctx, |ui| {
                if let Some(title) = &self.title {
                    title_ui(title, ui);
                }
                self.content_ui(ui, add_content);
            }).unwrap().response.layer_id;

        // Always show main content window above background window.
        ctx.move_to_top(layer_id);
        
        // Setup first draw flag.
        if Self::first_draw() {
            let r_state = MODAL_STATE.read();
            let modal = r_state.modal.as_ref().unwrap();
            modal.first_draw.store(false, Ordering::Relaxed);
        }
    }

    /// Get [`egui::Window`] position based on [`ModalPosition`].
    fn modal_position(&self) -> (Align2, Vec2) {
        let align = match self.position {
            ModalPosition::CenterTop => Align2::CENTER_TOP,
            ModalPosition::Center => Align2::CENTER_CENTER
        };

        let x_align = View::get_left_inset() - View::get_right_inset();
        let is_mac = OperatingSystem::Mac == OperatingSystem::from_target_os();
        let is_win = OperatingSystem::Windows == OperatingSystem::from_target_os();
        let extra_y = if View::is_desktop() && !is_win {
            Content::WINDOW_TITLE_HEIGHT + if !is_mac {
                Content::WINDOW_FRAME_MARGIN
            } else {
                0.0
            }
        } else {
            0.0
        };
        let y_align = View::get_top_inset() + Self::DEFAULT_MARGIN / 2.0 + extra_y;

        let offset = match self.position {
            ModalPosition::CenterTop => Vec2::new(x_align, y_align),
            ModalPosition::Center => Vec2::new(x_align, 0.0)
        };
        (align, offset)
    }

    /// Draw provided content.
    fn content_ui(&self, ui: &mut egui::Ui, add_content: impl FnOnce(&mut egui::Ui, &Modal)) {
        let mut rect = ui.available_rect_before_wrap();

        // Create background shape.
        let mut bg_shape = RectShape::new(rect, if self.title.is_none() {
            CornerRadius::same(8.0 as u8)
        } else {
            CornerRadius {
                nw: 0.0 as u8,
                ne: 0.0 as u8,
                sw: 8.0 as u8,
                se: 8.0 as u8,
            }
        }, Colors::fill(), Stroke::NONE, StrokeKind::Middle);
        let bg_idx = ui.painter().add(bg_shape.clone());

        rect.min += egui::emath::vec2(6.0, 0.0);
        rect.max -= egui::emath::vec2(6.0, 0.0);
        let resp = ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            (add_content)(ui, self);
        }).response;

        // Setup background size.
        let bg_rect = {
            let mut r = resp.rect.clone();
            r.min -= egui::emath::vec2(6.0, 0.0);
            r.max += egui::emath::vec2(6.0, 0.0);
            r
        };
        bg_shape.rect = bg_rect;
        ui.painter().set(bg_idx, bg_shape);
    }
}

/// Draw title content.
fn title_ui(title: &String, ui: &mut egui::Ui) {
    let rect = ui.available_rect_before_wrap();

    // Create background shape.
    let mut bg_shape = RectShape::new(rect, CornerRadius {
        nw: 8.0 as u8,
        ne: 8.0 as u8,
        sw: 0.0 as u8,
        se: 0.0 as u8,
    }, Colors::yellow(), Stroke::NONE, StrokeKind::Middle);
    let bg_idx = ui.painter().add(bg_shape.clone());

    // Draw title content.
    let resp = ui.vertical_centered(|ui| {
        ui.add_space(Modal::DEFAULT_MARGIN + 2.0);
        ui.label(RichText::new(title)
            .size(19.0)
            .color(Colors::title(true))
        );
        ui.add_space(Modal::DEFAULT_MARGIN + 1.0);
        // Draw line below title.
        View::horizontal_line(ui, Colors::item_stroke());
    }).response;

    // Setup background size.
    bg_shape.rect = resp.rect;
    ui.painter().set(bg_idx, bg_shape);
}