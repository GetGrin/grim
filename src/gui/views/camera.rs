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
use egui::load::SizedTexture;
use egui::{Pos2, Rect, RichText, TextureOptions, UiBuilder, Widget};
use image::{DynamicImage, EncodableLayout};
use grin_util::ZeroingString;
use grin_wallet_libwallet::SlatepackAddress;
use grin_keychain::mnemonic::WORDS;

use crate::gui::Colors;
use crate::gui::icons::CAMERA_ROTATE;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::types::{QrScanResult, QrScanState};
use crate::gui::views::View;
use crate::wallet::types::PhraseSize;
use crate::wallet::WalletUtils;

/// Camera QR code scanner.
pub struct CameraContent {
    /// QR code scanning progress and result.
    qr_scan_state: Arc<RwLock<QrScanState>>,
    /// Uniform Resources URIs collected from QR code scanning.
    ur_data: Arc<RwLock<Option<(Vec<String>, usize)>>>
}

impl Default for CameraContent {
    fn default() -> Self {
        Self {
            qr_scan_state: Arc::new(RwLock::new(QrScanState::default())),
            ur_data: Arc::new(RwLock::new(None))
        }
    }
}

impl CameraContent {
    /// Draw camera content.
    pub fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        ui.ctx().request_repaint();
        if let Some(img_data) = cb.camera_image() {
            if let Ok(img) =
                image::load_from_memory(&*img_data.0) {
                // Process image to find QR code.
                self.scan_qr(&img);

                // Draw image.
                let img_rect = self.image_ui(ui, img, img_data.1);

                // Show UR scan progress.
                self.ur_progress_ui(ui);

                // Show button to switch cameras.
                if cb.can_switch_camera() {
                    let r = {
                        let mut r = img_rect.clone();
                        r.min.y = r.max.y - 52.0;
                        r.min.x = r.max.x - 52.0;
                        r
                    };
                    ui.allocate_new_ui(UiBuilder::new().max_rect(r), |ui| {
                        let rotate_img = CAMERA_ROTATE.to_string();
                        View::button(ui, rotate_img, Colors::white_or_black(false), || {
                            cb.switch_camera();
                        });
                    });
                }
            } else {
                self.loading_ui(ui);
            }
        } else {
            self.loading_ui(ui);
        }
    }

    /// Draw camera image.
    fn image_ui(&mut self, ui: &mut egui::Ui, mut img: DynamicImage, rotation: u32) -> Rect {
        // Setup image rotation.
        img = match rotation {
            90 => img.rotate90(),
            180 => img.rotate180(),
            270 => img.rotate270(),
            _ => img
        };
        if View::is_desktop() {
            img = img.fliph();
        }
        // Convert to ColorImage.
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
        egui::Image::from_texture(sized_img)
            // Setup to crop image at square.
            .uv(Rect::from([
                Pos2::new(1.0 - (img_size.y / img_size.x), 0.0),
                Pos2::new(1.0, 1.0)
            ]))
            .max_height(ui.available_width())
            .maintain_aspect_ratio(false)
            .shrink_to_fit()
            .ui(ui).rect
    }

    /// Draw animated QR code scanning progress.
    fn ur_progress_ui(&self, ui: &mut egui::Ui) {
        let show_ur_progress = {
            self.ur_data.as_ref().read().is_some()
        };
        if show_ur_progress {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new(format!("{}%", self.ur_progress()))
                    .size(17.0)
                    .color(Colors::green()));
            });
        }
    }

    /// Draw camera loading progress content.
    fn loading_ui(&self, ui: &mut egui::Ui) {
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

    /// Get UR scanning progress in percents.
    fn ur_progress(&self) -> i32 {
        // Setup data.
        let r_data = self.ur_data.read();
        let (data, total) = r_data.clone().unwrap_or((vec![], 0));
        if data.is_empty() {
            return 0;
        }
        // Calculate progress.
        let mut complete = 0;
        for i in &data {
            if !i.is_empty() {
                complete += 1;
            }
        }
        (100 * complete / total) as i32
    }

    /// Parse QR code from provided image data.
    fn scan_qr(&self, image_data: &DynamicImage) {
        // Do not scan when another image is processing.
        if self.image_processing() {
            return;
        }
        // Setup scanning flag.
        {
            let mut w_scan = self.qr_scan_state.write();
            w_scan.image_processing = true;
        }

        let image_data = image_data.clone();
        let qr_scan_state = self.qr_scan_state.clone();
        let ur_data = self.ur_data.clone();

        let on_scan = async move {
            // Prepare image data.
            let img = image_data.to_luma8();
            let mut img: rqrr::PreparedImage<image::GrayImage>
                = rqrr::PreparedImage::prepare(img);
            // Scan and save results.
            let grids = img.detect_grids();
            if let Some(g) = grids.get(0) {
                let mut qr_data = vec![];
                if let Ok(_) = g.decode_to(&mut qr_data) {
                    // Setup scanned data into text.
                    let text = String::from_utf8(qr_data.clone()).unwrap_or("".to_string());
                    // Setup current text.
                    let cur_text = {
                        let r_scan = qr_scan_state.read();
                        let text = if let Some(res) = r_scan.qr_scan_result.clone() {
                            res.text()
                        } else {
                            "".to_string()
                        };
                        text
                    };
                    // Parse non-empty data if parsed text is different from saved.
                    if !qr_data.is_empty() && (cur_text.is_empty() || text != cur_text) {
                        let res = Self::parse_qr_code(qr_data);
                        match res {
                            QrScanResult::URPart(uri, index, total) => {
                                // Setup current UR data.
                                let mut cur_data = {
                                    let r_data = ur_data.read();
                                    let mut cur_data = vec!["".to_string(); total];
                                    if let Some((d, _)) = r_data.clone() {
                                        cur_data = d;
                                    }
                                    cur_data
                                };
                                if !cur_data.contains(&uri) {
                                    // Save part of UR data.
                                    {
                                        cur_data.insert(index, uri);
                                        let mut w_data = ur_data.write();
                                        *w_data = Some((cur_data.clone(), total));
                                    }
                                    // Setup UR decoder.
                                    let mut decoder = ur::Decoder::default();
                                    for m in cur_data {
                                        if !m.is_empty() {
                                            if let Ok(_) = decoder.receive(m.as_str()) {
                                                continue;
                                            } else {
                                                break;
                                            }
                                        }
                                    }
                                    // Check if UR data is complete.
                                    if decoder.complete() {
                                        if let Ok(data) = decoder.message() {
                                            // Parse complete data.
                                            let res = Self::parse_qr_code(data.unwrap_or(vec![]));
                                            // Clean UR data.
                                            let mut w_data = ur_data.write();
                                            *w_data = None;
                                            // Save scan result.
                                            let mut w_scan = qr_scan_state.write();
                                            w_scan.qr_scan_result = Some(res);
                                            return;
                                        }
                                    }
                                }
                            }
                            _ => {
                                // Clean UR data.
                                let mut w_data = ur_data.write();
                                *w_data = None;
                                // Save scan result.
                                let mut w_scan = qr_scan_state.write();
                                w_scan.qr_scan_result = Some(res);
                                return;
                            }
                        }
                    }
                }
            }
            // Reset scanning flag to process again.
            {
                let mut w_scan = qr_scan_state.write();
                w_scan.image_processing = false;
            }
        };

        // Launch scanner at separate thread.
        thread::spawn(move || {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(on_scan);
        });
    }

    /// Parse QR code scan result.
    fn parse_qr_code(data: Vec<u8>) -> QrScanResult {
        // Check if string starts with Grin address prefix.
        let text_string = String::from_utf8(data.clone()).unwrap_or("".to_string());
        let text = text_string.trim();
        if text.starts_with("tgrin") || text.starts_with("grin") {
            if SlatepackAddress::try_from(text).is_ok() {
                return QrScanResult::Address(ZeroingString::from(text));
            }
        }

        // Check if string contains Slatepack message prefix and postfix.
        if text.starts_with("BEGINSLATEPACK.") && text.ends_with("ENDSLATEPACK.") {
            return QrScanResult::Slatepack(ZeroingString::from(text));
        }

        // Check Uniform Resource data.
        // https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2020-005-ur.md
        if text.starts_with("ur:bytes/") {
            let split = text.split("/").collect::<Vec<_>>();
            if let Some(index_total) = split.get(1) {
                if let Some((index, total)) = index_total.split_once("-") {
                    let index = index.parse::<usize>();
                    let total = total.parse::<usize>();
                    if index.is_ok() && total.is_ok() {
                        let index = index.unwrap() - 1;
                        let total = total.unwrap();
                        return QrScanResult::URPart(text_string, index, total);
                    }
                }
            }
        }

        // Check Compact SeedQR format.
        // https://github.com/SeedSigner/seedsigner/blob/dev/docs/seed_qr/README.md#compactseedqr-specification
        if data.len() <= 32 && 16 <= data.len() && data.len() % 4 == 0 {
            // Setup words amount.
            let total_bits = data.len() * 8;
            let checksum_bits = total_bits / 32;
            let total_words = (total_bits + checksum_bits) / 11;
            // Setup entropy.
            let mut entropy = data.clone();
            WalletUtils::setup_checksum(&mut entropy);
            // Setup bits.
            let mut bits = vec![false; entropy.len() * 8];
            for i in 0..entropy.len() {
                for j in 0..8 {
                    bits[(i * 8) + j] = (entropy[i] & (1 << (7 - j))) != 0;
                }
            }
            // Extract word index.
            let extract_index = |i: usize| -> usize {
                let mut index = 0;
                for j in 0..11 {
                    index = index << 1;
                    if bits[(i * 11) + j] {
                        index += 1;
                    }
                }
                return index;
            };
            // Setup words.
            let mut words = "".to_string();
            for n in 0..total_words {
                // Setup word index.
                let index = extract_index(n);
                // Setup word.
                let empty_word = "".to_string();
                let word = WORDS.get(index).clone().unwrap_or(&empty_word).clone();
                if word.is_empty() {
                    words = empty_word;
                    break;
                }
                words = if words.is_empty() {
                    format!("{}", word)
                } else {
                    format!("{} {}", words, word)
                };
            }
            if !words.is_empty() {
                return QrScanResult::SeedQR(ZeroingString::from(words));
            }
        }

        // Check Standard SeedQR format.
        // https://github.com/SeedSigner/seedsigner/blob/dev/docs/seed_qr/README.md#standard-seedqr-specification
        let only_numbers = || {
            for c in text.chars() {
                if !c.is_numeric() {
                    return false;
                }
            }
            true
        };
        if !text.is_empty() && data.len() <= 96 && data.len() % 4 == 0 && only_numbers() {
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
}