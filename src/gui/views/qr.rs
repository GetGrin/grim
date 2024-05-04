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

use std::sync::{Arc, RwLock};
use std::thread;
use egui::{SizeHint, TextureHandle, TextureOptions};
use egui::load::SizedTexture;
use egui_extras::image::{load_svg_bytes, load_svg_bytes_with_size};

use crate::gui::views::types::QrCreationState;
use crate::gui::views::View;

/// QR code image from text.
pub struct QrCodeContent {
    /// Text to create QR code.
    pub(crate) text: String,

    /// Texture handle to show image when created.
    texture_handle: Option<TextureHandle>,
    /// QR code image creation progress and result.
    qr_creation_state: Arc<RwLock<QrCreationState>>
}

impl QrCodeContent {
    pub fn new(text: String) -> Self {
        Self {
            text,
            texture_handle: None,
            qr_creation_state: Arc::new(RwLock::new(QrCreationState::default())),
        }
    }

    /// Draw QR code.
    pub fn ui(&mut self, ui: &mut egui::Ui, text: String) {
        // Get saved QR code image or load new one.
        if !self.has_image() {
            ui.add_space(38.0);
            View::small_loading_spinner(ui);
            ui.add_space(38.0);

            // Create image from text if not loading.
            self.create_image(text);
        } else {
            // Create image from SVG data.
            let r_create = self.qr_creation_state.read().unwrap();
            let svg = r_create.svg.as_ref().unwrap();
            let size = SizeHint::Size(ui.available_width() as u32, ui.available_width() as u32);
            let color_img = load_svg_bytes_with_size(svg, Some(size)).unwrap();
            // Create image texture.
            let texture_handle = ui.ctx().load_texture("qr_code",
                                                       color_img.clone(),
                                                       TextureOptions::default());
            self.texture_handle = Some(texture_handle.clone());
            let img_size = egui::emath::vec2(color_img.width() as f32,
                                             color_img.height() as f32);
            let sized_img = SizedTexture::new(texture_handle.id(), img_size);
            // Add image to content.
            ui.add(egui::Image::from_texture(sized_img)
                .max_height(ui.available_width())
                .fit_to_original_size(1.0));
        }
    }

    /// Check if image is creating.
    fn creating(&self) -> bool {
        let r_create = self.qr_creation_state.read().unwrap();
        r_create.creating
    }

    /// Check if image was created.
    fn has_image(&self) -> bool {
        let r_create = self.qr_creation_state.read().unwrap();
        r_create.svg.is_some()
    }

    /// Create QR code image at separate thread.
    fn create_image(&self, text: String) {
        let qr_creation_state = self.qr_creation_state.clone();
        if !self.creating() {
            thread::spawn(move || {
                let qr = qrcodegen::QrCode::encode_text(text.as_str(),
                                                        qrcodegen::QrCodeEcc::Medium).unwrap();
                let svg = Self::qr_to_svg(qr, 0);
                let mut w_create = qr_creation_state.write().unwrap();
                w_create.creating = false;
                w_create.svg = Some(svg.into_bytes());
            });
        }
    }

    /// Convert QR code to SVG string.
    fn qr_to_svg(qr: qrcodegen::QrCode, border: i32) -> String {
        let mut result = String::new();
        let dimension = qr.size().checked_add(border.checked_mul(2).unwrap()).unwrap();
        result += "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n";
        result += "<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1//EN\" \"http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd\">\n";
        result += &format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\" viewBox=\"0 0 {0} {0}\" stroke=\"none\">\n", dimension);
        result += "\t<rect width=\"100%\" height=\"100%\" fill=\"#FFFFFF\"/>\n";
        result += "\t<path d=\"";
        for y in 0 .. qr.size() {
            for x in 0 .. qr.size() {
                if qr.get_module(x, y) {
                    if x != 0 || y != 0 {
                        result += " ";
                    }
                    result += &format!("M{},{}h1v1h-1z", x + border, y + border);
                }
            }
        }
        result += "\" fill=\"#000000\"/>\n";
        result += "</svg>\n";
        result
    }

    /// Reset QR code image content state to default.
    pub fn clear_state(&mut self) {
        let mut w_create = self.qr_creation_state.write().unwrap();
        *w_create = QrCreationState::default();
    }
}