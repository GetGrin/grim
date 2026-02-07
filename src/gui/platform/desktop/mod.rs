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
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use parking_lot::RwLock;
use lazy_static::lazy_static;
use egui::{UserAttentionType, ViewportCommand, WindowLevel};
use rfd::FileDialog;

use crate::gui::platform::PlatformCallbacks;

/// Desktop platform related actions.
#[derive(Clone)]
pub struct Desktop {
    /// Context to repaint content and handle viewport commands.
    ctx: Arc<RwLock<Option<egui::Context>>>,

    /// Cameras amount.
    cameras_amount: Arc<AtomicUsize>,
    /// Camera index.
    camera_index: Arc<AtomicUsize>,
    /// Flag to check if camera stop is needed.
    stop_camera: Arc<AtomicBool>,

    /// Flag to check if attention required after window focusing.
    attention_required: Arc<AtomicBool>,
}

impl Desktop {
    pub fn new() -> Self {
        Self {
            ctx: Arc::new(RwLock::new(None)),
            cameras_amount: Arc::new(AtomicUsize::new(0)),
            camera_index: Arc::new(AtomicUsize::new(0)),
            stop_camera: Arc::new(AtomicBool::new(false)),
            attention_required: Arc::new(AtomicBool::new(false)),
        }
    }

    // #[allow(dead_code)]
    #[cfg(not(target_os = "macos"))]
    fn start_camera_capture(cameras_amount: Arc<AtomicUsize>,
                            camera_index: Arc<AtomicUsize>,
                            stop_camera: Arc<AtomicBool>) {
        use nokhwa::Camera;
        use nokhwa::pixel_format::RgbFormat;
        use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
        use nokhwa::utils::ApiBackend;

        let devices = nokhwa::query(ApiBackend::Auto).unwrap();
        cameras_amount.store(devices.len(), Ordering::Relaxed);
        let index = camera_index.load(Ordering::Relaxed);
        if devices.is_empty() || index >= devices.len() {
            return;
        }

        thread::spawn(move || {
            let index = CameraIndex::Index(camera_index.load(Ordering::Relaxed) as u32);
            let requested = RequestedFormat::new::<RgbFormat>(
                RequestedFormatType::AbsoluteHighestFrameRate
            );
            // Create and open camera.
            if let Ok(mut camera) = Camera::new(index, requested) {
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
        });
    }

    #[allow(dead_code)]
    #[cfg(target_os = "macos")]
    fn start_camera_capture(cameras_amount: Arc<AtomicUsize>,
                            camera_index: Arc<AtomicUsize>,
                            stop_camera: Arc<AtomicBool>) {
        use nokhwa::nokhwa_initialize;
        use nokhwa::pixel_format::RgbFormat;
        use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
        use nokhwa::utils::ApiBackend;
        use nokhwa::query;
        use nokhwa::CallbackCamera;

        // Ask permission to open camera.
        nokhwa_initialize(|_| {});

        thread::spawn(move || {
            let cameras = query(ApiBackend::Auto).unwrap();
            cameras_amount.store(cameras.len(), Ordering::Relaxed);
            let index = camera_index.load(Ordering::Relaxed);
            if cameras.is_empty() || index >= cameras.len() {
                return;
            }
            // Start camera.
            let camera_index = CameraIndex::Index(camera_index.load(Ordering::Relaxed) as u32);
            let camera_callback = CallbackCamera::new(
                camera_index,
                RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate),
                |_| {}
            );
            if let Ok(mut cb) = camera_callback {
                if cb.open_stream().is_ok() {
                    loop {
                        // Stop if camera was stopped.
                        if stop_camera.load(Ordering::Relaxed) {
                            stop_camera.store(false, Ordering::Relaxed);
                            // Clear image.
                            let mut w_image = LAST_CAMERA_IMAGE.write();
                            *w_image = None;
                            break;
                        }
                        // Get image from camera.
                        if let Ok(frame) = cb.poll_frame() {
                            let image = frame.decode_image::<RgbFormat>().unwrap();
                            let mut bytes: Vec<u8> = Vec::new();
                            let format = image::ImageFormat::Jpeg;
                            // Convert image to Jpeg format.
                            image.write_to(&mut std::io::Cursor::new(&mut bytes), format).unwrap();
                            let mut w_image = LAST_CAMERA_IMAGE.write();
                            *w_image = Some((bytes, 0));
                        } else {
                            // Clear image.
                            let mut w_image = LAST_CAMERA_IMAGE.write();
                            *w_image = None;
                            break;
                        }
                    }
                }
            }
        });
    }
}

impl PlatformCallbacks for Desktop {
    fn set_context(&mut self, ctx: &egui::Context) {
        let mut w_ctx = self.ctx.write();
        *w_ctx = Some(ctx.clone());
    }

    fn exit(&self) {
        let r_ctx = self.ctx.read();
        if r_ctx.is_some() {
            let ctx = r_ctx.as_ref().unwrap();
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    }

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

        Self::start_camera_capture(self.cameras_amount.clone(),
                                   self.camera_index.clone(),
                                   stop_camera);
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
        let amount = self.cameras_amount.load(Ordering::Relaxed);
        amount > 1
    }

    fn switch_camera(&self) {
        self.stop_camera();
        let index = self.camera_index.load(Ordering::Relaxed);
        let amount = self.cameras_amount.load(Ordering::Relaxed);
        if index == amount - 1 {
            self.camera_index.store(0, Ordering::Relaxed);
        } else {
            self.camera_index.store(index + 1, Ordering::Relaxed);
        }
        self.start_camera();
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

    fn pick_folder(&self) -> Option<String> {
        let file = FileDialog::new()
            .set_title(t!("choose_folder"))
            .set_directory(dirs::home_dir().unwrap())
            .pick_folder();
        if let Some(file) = file {
            return Some(file.to_str().unwrap_or_default().to_string());
        }
        None
    }

    fn picked_file(&self) -> Option<String> {
        None
    }

    fn request_user_attention(&self) {
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
        self.attention_required.store(true, Ordering::Relaxed);
    }

    fn user_attention_required(&self) -> bool {
        self.attention_required.load(Ordering::Relaxed)
    }

    fn clear_user_attention(&self) {
        let r_ctx = self.ctx.read();
        if r_ctx.is_some() {
            let ctx = r_ctx.as_ref().unwrap();
            ctx.send_viewport_cmd(
                ViewportCommand::RequestUserAttention(UserAttentionType::Reset)
            );
            ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::Normal));
        }
        self.attention_required.store(false, Ordering::Relaxed);
    }
}

lazy_static! {
    /// Last captured image from started camera.
    static ref LAST_CAMERA_IMAGE: Arc<RwLock<Option<(Vec<u8>, u32)>>> = Arc::new(RwLock::new(None));
}
