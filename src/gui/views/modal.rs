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

use std::cmp::min;
use std::sync::atomic::{AtomicBool, Ordering};

use egui::{Align2, RichText, Rounding, Sense, Stroke, Vec2};
use egui::epaint::RectShape;

use crate::gui::Colors;
use crate::gui::views::View;

/// Location for [`Modal`] at application UI.
pub enum ModalLocation {
    /// To draw globally above side panel and screen.
    Global,
    /// To draw on the side panel.
    SidePanel,
    /// To draw on the screen.
    Screen
}

/// Position of [`Modal`] on the screen at provided [`ModalLocation`].
pub enum ModalPosition {
    /// Center-top position.
    CenterTop,
    /// Center of the location.
    Center
}

/// Stores data to draw dialog box/popup at UI, powered by [`egui::Window`].
pub struct Modal {
    /// Identifier for modal.
    pub(crate) id: &'static str,
    /// Location at UI.
    pub(crate) location: ModalLocation,
    /// Position on the screen.
    position: ModalPosition,
    /// Flag to show the content.
    open: AtomicBool,
    /// To check if it can be closed.
    closeable: AtomicBool,
    /// Title text
    title: Option<String>
}

impl Modal {
    /// Default width of the content.
    const DEFAULT_WIDTH: i64 = 380;

    /// Create open and closeable Modal with center position.
    pub fn new(id: &'static str, location: ModalLocation) -> Self {
        Self {
            id,
            location,
            position: ModalPosition::Center,
            open: AtomicBool::new(true),
            closeable: AtomicBool::new(true),
            title: None
        }
    }

    /// Setup position of Modal on the screen.
    pub fn position(mut self, position: ModalPosition) -> Self {
        self.position = position;
        self
    }

    /// Check if Modal is open.
    pub fn is_open(&self) -> bool {
        self.open.load(Ordering::Relaxed)
    }

    /// Mark Modal closed.
    pub fn close(&self) {
        self.open.store(false, Ordering::Relaxed);
    }

    /// Setup possibility to close Modal.
    pub fn closeable(self, closeable: bool) -> Self {
        self.closeable.store(closeable, Ordering::Relaxed);
        self
    }

    /// Disable possibility to close Modal.
    pub fn disable_closing(&self) {
        self.closeable.store(false, Ordering::Relaxed);
    }

    /// Check if Modal is closeable.
    pub fn is_closeable(&self) -> bool {
        self.closeable.load(Ordering::Relaxed)
    }

    /// Set title text on Modal creation.
    pub fn title(mut self, title: String) -> Self {
        self.title = Some(title.to_uppercase());
        self
    }

    /// Show Modal with provided content.
    pub fn ui(&self, ui: &mut egui::Ui, add_content: impl FnOnce(&mut egui::Ui, &Modal)) {
        // Show background Window at full available size.
        egui::Window::new(self.window_id(true))
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .fixed_pos(ui.next_widget_position())
            .fixed_size(ui.available_size())
            .frame(egui::Frame {
                fill: Colors::SEMI_TRANSPARENT,
                ..Default::default()
            })
            .show(ui.ctx(), |ui| {
                ui.set_min_size(ui.available_size());
            });

        // Choose width of modal content.
        let width = min(ui.available_width() as i64 - 20, Self::DEFAULT_WIDTH) as f32;

        // Show main content Window at given position.
        let (content_align, content_offset) = self.modal_position();
        let layer_id = egui::Window::new(self.window_id(false))
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .default_width(width)
            .anchor(content_align, content_offset)
            .frame(egui::Frame {
                rounding: Rounding::same(8.0),
                fill: Colors::YELLOW,
                ..Default::default()
            })
            .show(ui.ctx(), |ui| {
                if self.title.is_some() {
                    self.draw_title(ui);
                }
                self.draw_content(ui, add_content);
            }).unwrap().response.layer_id;

        // Always show main content Window above background Window.
        ui.ctx().move_to_top(layer_id);

    }

    /// Generate identifier for inner [`egui::Window`] parts based on [`ModalLocation`].
    fn window_id(&self, background: bool) -> &'static str {
        match self.location {
            ModalLocation::Global => {
                if background { "global.bg" } else { "global" }
            }
            ModalLocation::SidePanel => {
                if background { "side_panel.bg" } else { "side_panel" }
            }
            ModalLocation::Screen => {
                if background { "global.bg" } else { "global" }
            }
        }
    }

    /// Get [`egui::Window`] position based on [`ModalPosition`].
    fn modal_position(&self) -> (Align2, Vec2) {
        let align = match self.position {
            ModalPosition::CenterTop => { Align2::CENTER_TOP }
            ModalPosition::Center => { Align2::CENTER_CENTER }
        };
        let offset = match self.position {
            ModalPosition::CenterTop => { Vec2::new(0.0, 20.0) }
            ModalPosition::Center => { Vec2::new(0.0, 0.0) }
        };
        (align, offset)
    }

    /// Draw provided content.
    fn draw_content(&self, ui: &mut egui::Ui, add_content: impl FnOnce(&mut egui::Ui, &Modal)) {
        let mut rect = ui.available_rect_before_wrap();
        rect.min += egui::emath::vec2(6.0, 0.0);
        rect.max -= egui::emath::vec2(6.0, 0.0);

        // Create background shape.
        let rounding = if self.title.is_some() {
            Rounding {
                nw: 0.0,
                ne: 0.0,
                sw: 8.0,
                se: 8.0,
            }
        } else {
            Rounding::same(8.0)
        };
        let mut bg_shape = RectShape {
            rect,
            rounding,
            fill: Colors::FILL,
            stroke: Stroke::NONE,
        };
        let bg_idx = ui.painter().add(bg_shape);

        // Draw main content.
        let mut content_rect = ui.allocate_ui_at_rect(rect, |ui| {
            (add_content)(ui, self);
        }).response.rect;

        // Setup background shape to be painted behind main content.
        content_rect.min -= egui::emath::vec2(6.0, 0.0);
        content_rect.max += egui::emath::vec2(6.0, 0.0);
        bg_shape.rect = content_rect;
        ui.painter().set(bg_idx, bg_shape);
    }

    /// Draw the title.
    fn draw_title(&self, ui: &mut egui::Ui) {
        let rect = ui.available_rect_before_wrap();

        // Create background shape.
        let mut bg_shape = RectShape {
            rect,
            rounding: Rounding {
                nw: 8.0,
                ne: 8.0,
                sw: 0.0,
                se: 0.0,
            },
            fill: Colors::YELLOW,
            stroke: Stroke::NONE,
        };
        let bg_idx = ui.painter().add(bg_shape);

        // Draw title content.
        let title_resp = ui.allocate_ui_at_rect(rect, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.add_space(8.0);
                ui.label(RichText::new(self.title.as_ref().unwrap())
                    .size(20.0)
                    .color(Colors::TITLE)
                );
                ui.add_space(8.0);
            });
        }).response;

        // Setup background shape to be painted behind title content.
        bg_shape.rect = title_resp.rect;
        ui.painter().set(bg_idx, bg_shape);

        // Draw line below title.
        View::horizontal_line(ui, Colors::STROKE);
    }
}