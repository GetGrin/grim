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
use std::io:: Write;
use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use nokhwa::Camera;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use rfd::FileDialog;

use crate::gui::platform::PlatformCallbacks;

/// Desktop platform related actions.
#[derive(Clone)]
pub struct Desktop {
    /// Flag to check if camera stop is needed.
    stop_camera: Arc<AtomicBool>,
}

impl Default for Desktop {
    fn default() -> Self {
        Self {
            stop_camera: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl PlatformCallbacks for Desktop {
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
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
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
                });
        });
    }

    fn stop_camera(&self) {
        // Stop camera.
        self.stop_camera.store(true, Ordering::Relaxed);
        // Clear image.
        let mut w_image = LAST_CAMERA_IMAGE.write();
        *w_image = None;
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
}

lazy_static! {
    /// Last captured image from started camera.
    static ref LAST_CAMERA_IMAGE: Arc<RwLock<Option<(Vec<u8>, u32)>>> = Arc::new(RwLock::new(None));
}
