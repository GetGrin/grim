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

use std::sync::Arc;
use parking_lot::RwLock;
use std::thread;
use eframe::emath::Align;
use egui::load::SizedTexture;
use egui::{Layout, Pos2, Rect, TextureOptions, Widget};
use image::{DynamicImage, EncodableLayout, ImageFormat};

use grin_util::ZeroingString;
use grin_wallet_libwallet::SlatepackAddress;
use grin_keychain::mnemonic::WORDS;

use crate::gui::Colors;
use crate::gui::icons::CAMERA_ROTATE;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::types::{QrScanResult, QrScanState};
use crate::gui::views::View;
use crate::wallet::types::PhraseSize;

/// Camera QR code scanner.
pub struct CameraContent {
    /// QR code scanning progress and result.
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
                    270 => img.rotate270(),
                    _ => img
                };
                // Convert to ColorImage to add at content.
                let color_img = match &img {
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
                                                    color_img.clone(),
                                                    TextureOptions::default());
                let img_size = egui::emath::vec2(color_img.width() as f32,
                                                 color_img.height() as f32);
                let sized_img = SizedTexture::new(texture.id(), img_size);
                // Add image to content.
                ui.vertical_centered(|ui| {
                    egui::Image::from_texture(sized_img)
                        // Setup to crop image at square.
                        .uv(Rect::from([
                            Pos2::new(1.0 - (img_size.y / img_size.x), 0.0),
                            Pos2::new(1.0, 1.0)
                        ]))
                        .max_height(ui.available_width())
                        .maintain_aspect_ratio(false)
                        .shrink_to_fit()
                        .ui(ui);
                });

                // Show button to switch cameras.
                if cb.can_switch_camera() {
                    ui.add_space(-52.0);
                    let mut size = ui.available_size();
                    size.y = 48.0;
                    ui.allocate_ui_with_layout(size, Layout::right_to_left(Align::Max), |ui| {
                        ui.add_space(4.0);
                        View::button(ui, CAMERA_ROTATE.to_string(), Colors::WHITE, || {
                            cb.switch_camera();
                        });
                    });
                }
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
        let r_scan = self.qr_scan_state.read();
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
            let mut w_scan = self.qr_scan_state.write();
            w_scan.image_processing = true;
        }
        // Clear previous scanning result.
        {
            let mut w_scan = self.qr_scan_state.write();
            w_scan.qr_scan_result = None;
        }
        // Launch scanner at separate thread.
        let data = data.clone();
        let qr_scan_state = self.qr_scan_state.clone();
        thread::spawn(move || {
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
                        if let Ok((_, text)) = g.decode() {
                            let text = text.trim();
                            let cur_text = {
                                let r_scan = qr_scan_state.read();
                                let text = if let Some(res) = r_scan.qr_scan_result.clone() {
                                    res.value()
                                } else {
                                    "".to_string()
                                };
                                text
                            };
                            if !text.is_empty() && text != cur_text {
                                let result = Self::parse_scan_result(text);
                                let mut w_scan = qr_scan_state.write();
                                w_scan.qr_scan_result = Some(result);
                            }
                        }
                    }
                    // Setup scanning flag.
                    {
                        let mut w_scan = qr_scan_state.write();
                        w_scan.image_processing = false;
                    }
                });
        });
    }

    fn parse_scan_result(text: &str) -> QrScanResult {
        // Check if string starts with Grin address prefix.
        if text.starts_with("tgrin") || text.starts_with("grin") {
            if SlatepackAddress::try_from(text).is_ok() {
                return QrScanResult::Address(ZeroingString::from(text));
            }
        }

        // Check if string contains Slatepack message prefix and postfix.
        if text.starts_with("BEGINSLATEPACK.") && text.ends_with("ENDSLATEPACK.") {
            return QrScanResult::Slatepack(ZeroingString::from(text));
        }

        // Check SeedQR format.
        let only_numbers = || {
            for c in text.chars() {
                if !c.is_numeric() {
                    return false;
                }
            }
            true
        };
        if only_numbers() {
            if let Some(_) = PhraseSize::type_for_value(text.len() / 4) {
                let chars: Vec<char> = text.trim().chars().collect();
                let split = &chars.chunks(4)
                    .map(|chunk| chunk.iter().collect::<String>()
                        .trim()
                        .trim_start_matches("0")
                        .to_string()
                    )
                    .collect::<Vec<_>>();
                let mut words = "".to_string();
                for i in split {
                    let index = if i.is_empty() {
                        0usize
                    } else {
                        i.parse::<usize>().unwrap_or(WORDS.len())
                    };
                    let empty_word = "".to_string();
                    let word = WORDS.get(index).clone().unwrap_or(&empty_word).clone();
                    // Return text result when BIP39 word was not found.
                    if word.is_empty() {
                        return QrScanResult::Text(ZeroingString::from(text));
                    }
                    words = if words.is_empty() {
                        format!("{}", word)
                    } else {
                        format!("{} {}", words, word)
                    };
                }
                return QrScanResult::SeedQR(ZeroingString::from(words));
            }
        }

        // Return default text result.
        QrScanResult::Text(ZeroingString::from(text))
    }

    /// Get QR code scan result.
    pub fn qr_scan_result(&self) -> Option<QrScanResult> {
        let r_scan = self.qr_scan_state.read();
        if r_scan.qr_scan_result.is_some() {
            return Some(r_scan.qr_scan_result.clone().unwrap());
        }
        None
    }

    /// Reset camera content state to default.
    pub fn clear_state(&mut self) {
        let mut w_scan = self.qr_scan_state.write();
        *w_scan = QrScanState::default();
    }
}