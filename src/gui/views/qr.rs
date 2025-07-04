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
use egui::{SizeHint, TextureHandle, UiBuilder};
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ExtendedColorType, ImageEncoder};
use parking_lot::RwLock;
use qrcodegen::QrCode;
use std::mem::size_of;
use std::sync::Arc;
use std::thread;

use crate::gui::icons::{COPY, IMAGES_SQUARE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::types::QrImageState;
use crate::gui::views::View;
use crate::gui::Colors;

/// QR code image from text.
pub struct QrCodeContent {
    /// QR code text.
    text: String,

    /// Maximum QR code size.
    max_size: f32,

    /// Flag to draw animated QR with Uniform Resources
    /// https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2020-005-ur.md
    animated: bool,
    /// Index of current image at animation.
    animated_index: Option<usize>,
    /// Time of last image draw.
    animation_time: Option<i64>,

    /// Texture handle to show image when created.
    texture_handle: Option<TextureHandle>,
    /// QR code view data state.
    qr_image_state: Arc<RwLock<QrImageState>>,
}

const DEFAULT_QR_SIZE: u32 = 512;

impl QrCodeContent {
    pub fn new(text: String, animated: bool) -> Self {
        Self {
            text,
            max_size: DEFAULT_QR_SIZE as f32,
            animated,
            animated_index: None,
            animation_time: None,
            texture_handle: None,
            qr_image_state: Arc::new(RwLock::new(QrImageState::default())),
        }
    }

    /// Setup maximum QR code size.
    pub fn with_max_size(mut self, max_size: f32) -> Self {
        self.max_size = max_size;
        self
    }

    /// Draw QR code.
    pub fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        if self.animated {
            // Show animated QR code.
            self.animated_ui(ui, cb);
        } else {
            // Show static QR code.
            self.static_ui(ui, cb);
        }
        ui.add_space(8.0);
        View::horizontal_line(ui, Colors::item_stroke());
        ui.add_space(8.0);
    }

    /// Draw animated QR code content.
    fn animated_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        if !self.has_image() {
            let space = (ui.available_width() - View::BIG_SPINNER_SIZE) / 2.0;
            ui.vertical_centered(|ui| {
                ui.add_space(space);
                View::big_loading_spinner(ui);
                ui.add_space(space);
            });

            // Create multiple vector images from text if not creating.
            if !self.loading() {
                self.create_svg_list();
            }
        } else {
            let svg_list = {
                let r_create = self.qr_image_state.read();
                r_create.svg_list.clone().unwrap()
            };

            // Setup animated index.
            let now = chrono::Utc::now().timestamp_millis();
            if now - *self.animation_time.get_or_insert(now) > 100 {
                if let Some(i) = self.animated_index {
                    self.animated_index = Some(i + 1);
                }
                if *self.animated_index.get_or_insert(0) == svg_list.len() {
                    self.animated_index = Some(0);
                }
                self.animation_time = Some(now);
            }

            let svg = svg_list[self.animated_index.unwrap_or(0)].clone();

            // Create images from SVG data.
            self.qr_image_ui(svg, ui);

            // Show QR code text.
            self.text_ui(ui);

            ui.vertical_centered(|ui| {
                let sharing = {
                    let r_state = self.qr_image_state.read();
                    r_state.exporting || r_state.gif_creating
                };
                if !sharing {
                    ui.vertical_centered(|ui| {
                        // Show button to share QR.
                        let share_text = format!("{} {}", IMAGES_SQUARE, t!("share"));
                        View::colored_text_button(ui,
                                                  share_text,
                                                  Colors::blue(),
                                                  Colors::white_or_black(false), || {
                                {
                                    let mut w_state = self.qr_image_state.write();
                                    w_state.exporting = true;
                                }
                                // Create GIF to export.
                                self.create_qr_gif();
                            });
                    });
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(2.0);
                        View::small_loading_spinner(ui);
                        ui.add_space(1.0);
                    });
                }

                // Check if GIF was created to share.
                let has_gif = {
                    let r_state = self.qr_image_state.read();
                    r_state.gif_data.is_some()
                };
                if has_gif {
                    let data = {
                        let r_state = self.qr_image_state.read();
                        r_state.gif_data.clone().unwrap()
                    };
                    let name = format!("{}.gif", chrono::Utc::now().timestamp());
                    cb.share_data(name, data).unwrap_or_default();
                    // Clear GIF data and exporting flag.
                    {
                        let mut w_state = self.qr_image_state.write();
                        w_state.gif_data = None;
                        w_state.exporting = false;
                    }
                }
            });

            ui.ctx().request_repaint();
        }
    }

    /// Draw static QR code content.
    fn static_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        if !self.has_image() {
            let space = (ui.available_width() - View::BIG_SPINNER_SIZE) / 2.0;
            ui.vertical_centered(|ui| {
                ui.add_space(space);
                View::big_loading_spinner(ui);
                ui.add_space(space);
            });

            // Create vector image from text if not creating.
            if !self.loading() {
                self.create_svg();
            }
        } else {
            // Create image from SVG data.
            let svg = {
                let r_state = self.qr_image_state.read();
                r_state.svg.clone().unwrap()
            };
            self.qr_image_ui(svg, ui);

            // Show QR code text.
            self.text_ui(ui);

            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    // Draw copy button.
                    let copy_text = format!("{} {}", COPY, t!("copy"));
                    View::button(ui, copy_text, Colors::white_or_black(false), || {
                        cb.copy_string_to_buffer(self.text.clone());
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    let share_text = format!("{} {}", IMAGES_SQUARE, t!("share"));
                    View::colored_text_button(ui,
                                              share_text,
                                              Colors::blue(),
                                              Colors::white_or_black(false), || {
                            self.share_static(cb);
                        });
                });
            });
        }
    }

    /// Share static QR code image.
    fn share_static(&self, cb: &dyn PlatformCallbacks) {
        let text = self.text.as_str();
        if let Ok(qr) = QrCode::encode_text(text, qrcodegen::QrCodeEcc::Low) {
            let size = DEFAULT_QR_SIZE as usize;
            if let Some(data) = Self::qr_to_image_data(qr, size) {
                let mut png = vec![];
                let png_enc = PngEncoder::new_with_quality(&mut png,
                                                           CompressionType::Best,
                                                           FilterType::NoFilter);
                if let Ok(()) = png_enc.write_image(data.as_slice(),
                                                    DEFAULT_QR_SIZE,
                                                    DEFAULT_QR_SIZE,
                                                    ExtendedColorType::L8) {
                    let name = format!("{}.png", chrono::Utc::now().timestamp());
                    cb.share_data(name, png).unwrap_or_default();
                }
            }
        }
    }

    /// Draw QR code image content.
    fn qr_image_ui(&mut self, svg: Vec<u8>, ui: &mut egui::Ui) {
        View::max_width_ui(ui, self.max_size, |ui| {
            let mut rect = ui.available_rect_before_wrap();
            rect.min += egui::emath::vec2(10.0, 0.0);
            rect.max -= egui::emath::vec2(10.0, 0.0);

            // Create background shape.
            let mut bg_shape = RectShape::new(
                rect,
                egui::CornerRadius::default(),
                egui::Color32::WHITE,
                egui::Stroke::NONE,
                egui::StrokeKind::Outside
            );
            let bg_idx = ui.painter().add(bg_shape.clone());

            // Draw QR code image.
            let mut content_rect = ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
                ui.add_space(10.0);
                let size = SizeHint::Size {
                    width: ui.available_width() as u32,
                    height: ui.available_width() as u32,
                    maintain_aspect_ratio: true,
                };
                self.texture_handle = Some(View::svg_image(ui, "qr", svg.as_slice(), size));
                ui.add_space(10.0);
            }).response.rect;

            // Setup background size.
            content_rect.min -= egui::emath::vec2(10.0, 0.0);
            content_rect.max += egui::emath::vec2(10.0, 0.0);
            bg_shape.rect = content_rect;
            ui.painter().set(bg_idx, bg_shape);
        });
    }

    /// Draw QR code text.
    fn text_ui(&self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        View::ellipsize_text(ui, self.text.clone(), 15.0, Colors::inactive_text());
        ui.add_space(6.0);
    }

    /// Check if QR code is loading.
    fn loading(&self) -> bool {
        let r_state = self.qr_image_state.read();
        r_state.loading
    }

    /// Create multiple vector QR code images at separate thread.
    fn create_svg_list(&self) {
        let qr_state = self.qr_image_state.clone();
        let text = self.text.clone();
        thread::spawn(move || {
            let mut encoder = ur::Encoder::bytes(text.as_bytes(), 100).unwrap();
            let mut data = Vec::with_capacity(encoder.fragment_count());
            for _ in 0..encoder.fragment_count() {
                let ur = encoder.next_part().unwrap();
                if let Ok(qr) = QrCode::encode_text(ur.as_str(), qrcodegen::QrCodeEcc::Low) {
                    let svg = Self::qr_to_svg(qr, 0);
                    data.push(svg.into_bytes());
                }
            }
            let mut w_state = qr_state.write();
            if !data.is_empty() {
                w_state.svg_list = Some(data);
            }
            w_state.loading = false;
        });
    }

    /// Check if image was created.
    fn has_image(&self) -> bool {
        let r_state = self.qr_image_state.read();
        r_state.svg.is_some() || r_state.svg_list.is_some()
    }

    /// Create vector QR code image at separate thread.
    fn create_svg(&self) {
        let qr_state = self.qr_image_state.clone();
        let text = self.text.clone();
        thread::spawn(move || {
            if let Ok(qr) = QrCode::encode_text(text.as_str(), qrcodegen::QrCodeEcc::Low) {
                let svg = Self::qr_to_svg(qr, 0);
                let mut w_state = qr_state.write();
                w_state.loading = false;
                w_state.svg = Some(svg.into_bytes());
            }
        });
    }

    /// Convert QR code to SVG string.
    fn qr_to_svg(qr: QrCode, border: i32) -> String {
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

    /// Create GIF image at separate thread.
    fn create_qr_gif(&self) {
        {
            let mut w_state = self.qr_image_state.write();
            w_state.gif_creating = true;

        }
        let qr_state = self.qr_image_state.clone();
        let text = self.text.clone();
        thread::spawn(move || {
            // Setup GIF image encoder.
            let mut gif = vec![];
            {
                // Generate QR codes from text.
                let mut qrs = vec![];
                let mut ur_enc = ur::Encoder::bytes(text.as_bytes(), 100).unwrap();
                for _ in 0..ur_enc.fragment_count() {
                    let ur = ur_enc.next_part().unwrap();
                    if let Ok(qr) = qrcode::QrCode::with_error_correction_level(
                        ur.as_bytes(),
                        qrcode::EcLevel::L
                    ) {
                        // Create an image from QR data.
                        let image = qr.render()
                            .max_dimensions(DEFAULT_QR_SIZE, DEFAULT_QR_SIZE)
                            .dark_color(image::Rgb([0, 0, 0]))
                            .light_color(image::Rgb([255, 255, 255]))
                            .build();
                        qrs.push(image);
                    }
                }

                if !qrs.is_empty() {
                    // Generate GIF data.
                    let color_map = &[0, 0, 0, 0xFF, 0xFF, 0xFF];
                    let mut gif_enc = gif::Encoder::new(&mut gif,
                                                        qrs[0].width() as u16,
                                                        qrs[0].height() as u16,
                                                        color_map).unwrap();
                    gif_enc.set_repeat(gif::Repeat::Infinite).unwrap();
                    for qr in qrs {
                        let mut frame = gif::Frame::from_rgb(qr.width() as u16,
                                                             qr.height() as u16,
                                                             qr.as_raw().as_slice());
                        frame.delay = 10;
                        // Write an image to GIF encoder.
                        if let Ok(_) = gif_enc.write_frame(&frame) {
                            continue;
                        }
                        // Exit on error.
                        let mut w_state = qr_state.write();
                        w_state.gif_creating = false;
                        return;
                    }
                }
            }
            // Setup GIF image data.
            let mut w_state = qr_state.write();
            if !gif.is_empty() {
                w_state.gif_data = Some(gif);
            }
            w_state.gif_creating = false;
        });
    }

    /// Convert QR code to image data.
    fn qr_to_image_data(qr: QrCode, size: usize) -> Option<Vec<u8>> {
        if size >= 2usize.pow((size_of::<usize>() * 4) as u32) {
            return None;
        }
        let margin_size = 1;
        let s = qr.size();
        let data_length = s as usize;
        let data_length_with_margin = data_length + 2 * margin_size;
        let point_size = size / data_length_with_margin;
        if point_size == 0 {
            return None;
        }
        let margin = (size - (point_size * data_length)) / 2;
        let length = size * size;
        let mut img_raw: Vec<u8> = vec![255u8; length];
        for i in 0..s {
            for j in 0..s {
                if qr.get_module(i, j) {
                    let x = i as usize * point_size + margin;
                    let y = j as usize * point_size + margin;

                    for j in y..(y + point_size) {
                        let offset = j * size;
                        for i in x..(x + point_size) {
                            img_raw[offset + i] = 0;
                        }
                    }
                }
            }
        }
        Some(img_raw)
    }
}