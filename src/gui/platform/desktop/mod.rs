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

use std::fs::File;
use std::io::Write;
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use parking_lot::RwLock;
use lazy_static::lazy_static;
use egui::{UserAttentionType, ViewportCommand, WindowLevel};
use rfd::FileDialog;

use crate::gui::platform::PlatformCallbacks;

/// Desktop platform related actions.
#[derive(Clone)]
pub struct Desktop {
    /// Flag to check if camera stop is needed.
    stop_camera: Arc<AtomicBool>,
    /// Context to repaint content and handle viewport commands.
    ctx: Arc<RwLock<Option<egui::Context>>>,
}

impl PlatformCallbacks for Desktop {
    fn set_context(&mut self, ctx: &egui::Context) {
        let mut w_ctx = self.ctx.write();
        *w_ctx = Some(ctx.clone());
    }

    fn show_keyboard(&self) {}

    fn hide_keyboard(&self) {}

    fn copy_string_to_buffer(&self, data: String) {
        let mut clipboard = arboard::Clipboard::new().unwrap();
        clipboard.set_text(data).unwrap();
    }

    fn get_string_from_buffer(&self) -> String {
        let mut clipboard = arboard::Clipboard::new().unwrap();
        clipboard.get_text().unwrap_or("".to_string())
    }

    fn start_camera(&self) {
        // Clear image.
        {
            let mut w_image = LAST_CAMERA_IMAGE.write();
            *w_image = None;
        }

        // Setup stop camera flag.
        let stop_camera = self.stop_camera.clone();
        stop_camera.store(false, Ordering::Relaxed);

        // Capture images at separate thread.
        thread::spawn(move || {
            Self::start_camera_capture(stop_camera);
        });
    }

    fn stop_camera(&self) {
        // Stop camera.
        self.stop_camera.store(true, Ordering::Relaxed);
    }

    fn camera_image(&self) -> Option<(Vec<u8>, u32)> {
        let r_image = LAST_CAMERA_IMAGE.read();
        if r_image.is_some() {
            return r_image.clone();
        }
        None
    }

    fn can_switch_camera(&self) -> bool {
        false
    }

    fn switch_camera(&self) {
        return;
    }

    fn share_data(&self, name: String, data: Vec<u8>) -> Result<(), std::io::Error> {
        let folder = FileDialog::new()
            .set_title(t!("share"))
            .set_directory(dirs::home_dir().unwrap())
            .set_file_name(name.clone())
            .save_file();
        if let Some(folder) = folder {
            let mut image = File::create(folder)?;
            image.write_all(data.as_slice())?;
            image.sync_all()?;
        }
        Ok(())
    }

    fn pick_file(&self) -> Option<String> {
        let file = FileDialog::new()
            .set_title(t!("choose_file"))
            .set_directory(dirs::home_dir().unwrap())
            .pick_file();
        if let Some(file) = file {
            return Some(file.to_str().unwrap_or_default().to_string());
        }
        None
    }

    fn picked_file(&self) -> Option<String> {
        None
    }

    fn consume_data(&mut self) -> Option<String> {
        let has_data = {
            let r_data = PASSED_DATA.read();
            r_data.is_some()
        };
        if has_data {
            // Clear data.
            let mut w_data = PASSED_DATA.write();
            let data = w_data.clone();
            *w_data = None;
            return data;
        }
        None
    }
}

impl Desktop {
    /// Create new instance with provided extra data from app opening.
    pub fn new(data: Option<String>) -> Self {
        let mut w_data = PASSED_DATA.write();
        *w_data = data;
        Self {
            stop_camera: Arc::new(AtomicBool::new(false)),
            ctx: Arc::new(RwLock::new(None)),
        }
    }

    /// Handle data passed to application.
    pub fn on_data(&self, data: String) {
        let mut w_data = PASSED_DATA.write();
        *w_data = Some(data);

        let r_ctx = self.ctx.read();
        if r_ctx.is_some() {
            let ctx = r_ctx.as_ref().unwrap();
            // Request attention on taskbar.
            ctx.send_viewport_cmd(
                ViewportCommand::RequestUserAttention(UserAttentionType::Informational)
            );
            // Un-minimize window.
            if ctx.input(|i| i.viewport().minimized.unwrap_or(false)) {
                ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
            }
            // Focus to window.
            if !ctx.input(|i| i.viewport().focused.unwrap_or(false)) {
                ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::AlwaysOnTop));
                ctx.send_viewport_cmd(ViewportCommand::Focus);
            }
            ctx.request_repaint();
        }
    }

    #[allow(dead_code)]
    #[cfg(target_os = "windows")]
    fn start_camera_capture(stop_camera: Arc<AtomicBool>) {
        use nokhwa::Camera;
        use nokhwa::pixel_format::RgbFormat;
        use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
        let index = CameraIndex::Index(0);
        let requested = RequestedFormat::new::<RgbFormat>(
            RequestedFormatType::AbsoluteHighestFrameRate
        );
        // Create and open camera.
        let mut camera = Camera::new(index, requested).unwrap();
        if let Ok(_) = camera.open_stream() {
            loop {
                // Stop if camera was stopped.
                if stop_camera.load(Ordering::Relaxed) {
                    stop_camera.store(false, Ordering::Relaxed);
                    // Clear image.
                    let mut w_image = LAST_CAMERA_IMAGE.write();
                    *w_image = None;
                    break;
                }
                // Get a frame.
                if let Ok(frame) = camera.frame() {
                    // Save image.
                    let mut w_image = LAST_CAMERA_IMAGE.write();
                    *w_image = Some((frame.buffer().to_vec(), 0));
                } else {
                    // Clear image.
                    let mut w_image = LAST_CAMERA_IMAGE.write();
                    *w_image = None;
                    break;
                }
            }
            camera.stop_stream().unwrap();
        };
    }

    #[allow(dead_code)]
    #[cfg(not(target_os = "windows"))]
    fn start_camera_capture(stop_camera: Arc<AtomicBool>) {
        use eye::hal::{traits::{Context, Device, Stream}, PlatformContext};
        use image::ImageEncoder;

        let ctx = PlatformContext::default();
        let devices = ctx.devices().unwrap();
        let dev = ctx.open_device(&devices[0].uri).unwrap();

        let streams = dev.streams().unwrap();
        let stream_desc = streams[0].clone();
        let w = stream_desc.width;
        let h = stream_desc.height;

        let mut stream = dev.start_stream(&stream_desc).unwrap();

        loop {
            // Stop if camera was stopped.
            if stop_camera.load(Ordering::Relaxed) {
                stop_camera.store(false, Ordering::Relaxed);
                // Clear image.
                let mut w_image = LAST_CAMERA_IMAGE.write();
                *w_image = None;
                break;
            }
            // Get a frame.
            let frame = stream.next().expect("Stream is dead").expect("Failed to capture a frame");
            let mut out = vec![];
            if let Some(buf) = image::ImageBuffer::<image::Rgb<u8>, &[u8]>::from_raw(w, h, &frame) {
                image::codecs::jpeg::JpegEncoder::new(&mut out)
                    .write_image(buf.as_raw(), w, h, image::ExtendedColorType::Rgb8).unwrap();
            } else {
                out = frame.to_vec();
            }
            // Save image.
            let mut w_image = LAST_CAMERA_IMAGE.write();
            *w_image = Some((out, 0));
        }
    }
}

lazy_static! {
    /// Last captured image from started camera.
    static ref LAST_CAMERA_IMAGE: Arc<RwLock<Option<(Vec<u8>, u32)>>> = Arc::new(RwLock::new(None));

    /// Data passed from deeplink or opened file.
    static ref PASSED_DATA: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));
}
