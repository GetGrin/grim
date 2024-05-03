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

use std::sync::{Arc, RwLock};
use egui::load::SizedTexture;
use egui::{Pos2, Rect, TextureOptions, Widget};
use image::{DynamicImage, EncodableLayout, ImageFormat};

use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::types::QrScanState;
use crate::gui::views::View;

/// Camera scanner content.
pub struct CameraContent {
    // QR code scanning progress and result.
    qr_scan_state: Arc<RwLock<QrScanState>>
}

impl Default for CameraContent {
    fn default() -> Self {
        Self {
            qr_scan_state: Arc::new(RwLock::new(QrScanState::default())),
        }
    }
}

impl CameraContent {
    pub fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        // Draw last image from camera or loader.
        if let Some(img_data) = cb.camera_image() {
            // Load image to draw.
            if let Ok(mut img) =
                image::load_from_memory_with_format(&*img_data.0, ImageFormat::Jpeg) {
                // Process image to find QR code.
                self.scan_qr(&img);
                // Setup image rotation.
                img = match img_data.1 {
                    90 => img.rotate90(),
                    180 => img.rotate180(),
                    279 => img.rotate270(),
                    _ => img
                };
                // Convert to ColorImage to add at content.
                let color_image = match &img {
                    DynamicImage::ImageRgb8(image) => {
                        egui::ColorImage::from_rgb(
                            [image.width() as usize, image.height() as usize],
                            image.as_bytes(),
                        )
                    },
                    other => {
                        let image = other.to_rgba8();
                        egui::ColorImage::from_rgba_unmultiplied(
                            [image.width() as usize, image.height() as usize],
                            image.as_bytes(),
                        )
                    },
                };
                // Create image texture.
                let texture = ui.ctx().load_texture("camera_image",
                                                    color_image.clone(),
                                                    TextureOptions::default());
                let image_size = egui::emath::vec2(color_image.width() as f32,
                                                   color_image.height() as f32);
                let sized_image = SizedTexture::new(texture.id(), image_size);
                // Add image to content.
                ui.vertical_centered(|ui| {
                    egui::Image::from_texture(sized_image)
                        // Setup to make image cropped at center of square.
                        .uv(Rect::from([Pos2::new(0.125, 0.0), Pos2::new(1.125, 1.0)]))
                        .max_height(ui.available_width())
                        .maintain_aspect_ratio(false)
                        .shrink_to_fit()
                        .ui(ui);
                });
            } else {
                self.loading_content_ui(ui);
            }
        } else {
            self.loading_content_ui(ui);
        }

        // Request redraw.
        ui.ctx().request_repaint();
    }

    /// Draw camera loading progress content.
    fn loading_content_ui(&self, ui: &mut egui::Ui) {
        let space = (ui.available_width() - View::BIG_SPINNER_SIZE) / 2.0;
        ui.vertical_centered(|ui| {
            ui.add_space(space);
            View::big_loading_spinner(ui);
            ui.add_space(space);
        });
    }

    /// Check if image is processing to find QR code.
    fn image_processing(&self) -> bool {
        let mut r_scan = self.qr_scan_state.read().unwrap();
        r_scan.image_processing
    }

    /// Parse QR code from provided image data.
    fn scan_qr(&self, data: &DynamicImage) {
        // Do not scan when another image is processing.
        if self.image_processing() {
            return;
        }
        // Setup scanning flag.
        {
            let mut w_scan = self.qr_scan_state.write().unwrap();
            w_scan.image_processing = true;
        }
        // Launch scanner at separate thread.
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                // Prepare image data.
                let img = data.to_luma8();
                let mut img: rqrr::PreparedImage<image::GrayImage>
                    = rqrr::PreparedImage::prepare(img);
                // Scan and save results.
                let grids = img.detect_grids();
                for g in grids {
                    if let Ok((meta, text)) = g.decode() {
                        println!("12345 ecc: {}, text: {}", meta.ecc_level, text.clone());
                        if !text.is_empty() {
                            let mut w_scan = self.qr_scan_state.write().unwrap();
                            w_scan.qr_scan_result = Some(text);
                        }
                    }
                }
                // Setup scanning flag.
                {
                    let mut w_scan = self.qr_scan_state.write().unwrap();
                    w_scan.image_processing = false;
                }
            });
    }

    /// Get QR code scan result.
    pub fn qr_scan_result(&self) -> Option<String> {
        let r_scan = self.qr_scan_state.read().unwrap();
        if r_scan.qr_scan_result.is_some() {
            return Some(r_scan.qr_scan_result.as_ref().unwrap().clone());
        }
        None
    }

    /// Reset camera content state to default.
    pub fn clear_state(&mut self) {
        let mut w_scan = self.qr_scan_state.write().unwrap();
        *w_scan = QrScanState::default();
    }
}