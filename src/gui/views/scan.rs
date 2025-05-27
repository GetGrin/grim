// Copyright 2024 The Grim Developers
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
use egui::{Id, ScrollArea};

use crate::gui::Colors;
use crate::gui::icons::COPY;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{CameraContent, Modal, View};
use crate::gui::views::types::QrScanResult;

/// QR code scan [`Modal`] content.
pub struct CameraScanModal {
    /// Camera content for QR scan [`Modal`].
    camera_content: Option<CameraContent>,
    /// QR code scan result
    qr_scan_result: Option<QrScanResult>,
}

impl Default for CameraScanModal {
    fn default() -> Self {
        Self {
            camera_content: Some(CameraContent::default()),
            qr_scan_result: None,
        }
    }
}

impl CameraScanModal {
    /// Draw [`Modal`] content.
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              cb: &dyn PlatformCallbacks,
              mut on_result: impl FnMut(&QrScanResult)) {
        // Show scan result if exists or show camera content while scanning.
        if let Some(result) = &self.qr_scan_result {
            let mut result_text = result.text();
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(3.0);
            ScrollArea::vertical()
                .id_salt(Id::from("qr_scan_result_input"))
                .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                .max_height(128.0)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.add_space(7.0);
                    egui::TextEdit::multiline(&mut result_text)
                        .font(egui::TextStyle::Small)
                        .desired_rows(5)
                        .interactive(false)
                        .desired_width(f32::INFINITY)
                        .show(ui);
                    ui.add_space(6.0);
                });
            ui.add_space(2.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(10.0);

            // Show copy button.
            ui.vertical_centered(|ui| {
                let copy_text = format!("{} {}", COPY, t!("copy"));
                View::button(ui, copy_text, Colors::white_or_black(false), || {
                    cb.copy_string_to_buffer(result_text.to_string());
                    self.qr_scan_result = None;
                    Modal::close();
                });
            });
            ui.add_space(10.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(6.0);

            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("close"), Colors::white_or_black(false), || {
                        self.qr_scan_result = None;
                        self.camera_content = None;
                        Modal::close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("repeat"), Colors::white_or_black(false), || {
                        Modal::set_title(t!("scan_qr"));
                        self.qr_scan_result = None;
                        self.camera_content = Some(CameraContent::default());
                        cb.start_camera();
                    });
                });
            });
        } else if let Some(camera_content) = self.camera_content.as_mut() {
            if let Some(result) = camera_content.qr_scan_result() {
                cb.stop_camera();
                self.camera_content = None;
                on_result(&result);

                // Set result and rename modal title.
                self.qr_scan_result = Some(result);
                Modal::set_title(t!("scan_result"));
            } else {
                // Draw camera content.
                ui.add_space(6.0);
                self.camera_content.as_mut().unwrap().ui(ui, cb);
                ui.add_space(12.0);
                ui.vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                        cb.stop_camera();
                        self.camera_content = None;
                        Modal::close();
                    });
                });
            }
        }
        ui.add_space(6.0);
    }
}