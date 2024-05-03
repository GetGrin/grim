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

use lazy_static::lazy_static;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::thread;
use eye::hal::PlatformContext;
use eye::hal::traits::{Context, Device, Stream};

use crate::gui::platform::PlatformCallbacks;

/// Desktop platform related actions.
pub struct Desktop {
    /// Camera index.
    camera_index: AtomicI32,
    /// Flag to check if camera stop is needed.
    stop_camera: Arc<AtomicBool>
}

impl Default for Desktop {
    fn default() -> Self {
        Self {
            camera_index: AtomicI32::new(0),
            stop_camera: Arc::new(AtomicBool::new(false))
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

    fn cameras_amount(&self) -> u32 {
        let devices = PlatformContext::default().devices();
        if devices.is_ok() {
            return devices.unwrap().len() as u32;
        }
        0
    }

    fn switch_camera(&self) {
        let amount = self.cameras_amount();
        if amount < 2 {
            return;
        }
    }

    fn start_camera(&self) {
        // Clear image.
        {
            let mut w_image = LAST_CAMERA_IMAGE.write().unwrap();
            *w_image = None;
        }

        // Query for available devices.
        let devices = PlatformContext::default().devices();
        if devices.is_err() {
            return;
        }
        let devices = devices.unwrap();

        // Setup camera index.
        let saved_index = self.camera_index.load(Ordering::Relaxed);
        let camera_index = if devices.len() <= self.camera_index.load(Ordering::Relaxed) as usize {
            self.camera_index.store(0, Ordering::Relaxed);
            0
        } else {
            saved_index
        };

        // Setup stop camera flag.
        let stop_camera = self.stop_camera.clone();
        stop_camera.store(false, Ordering::Relaxed);

        let devices = devices.clone();

        // Capture images at separate thread.
        thread::spawn(move || {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    // Open camera.
                    let context = PlatformContext::default();
                    if let Ok(dev) = context.open_device(&devices[camera_index as usize].uri) {
                        let streams = dev.streams().unwrap();
                        let stream_desc = streams[0].clone();
                        println!("Camera stream: {:?}", stream_desc);
                        let mut stream = dev.start_stream(&stream_desc).unwrap();
                        loop {
                            // Stop if camera was stopped.
                            if stop_camera.load(Ordering::Relaxed) {
                                stop_camera.store(false, Ordering::Relaxed);
                                break;
                            }
                            // Get frame.
                            if let Some(frame) = stream.next() {
                                // Get data from frame.
                                if let Ok(frame_data) = frame {
                                    // Save image.
                                    let mut w_image = LAST_CAMERA_IMAGE.write().unwrap();
                                    *w_image = Some((frame_data.to_vec(), 0));
                                } else {
                                    // Clear image.
                                    let mut w_image = LAST_CAMERA_IMAGE.write().unwrap();
                                    *w_image = None;
                                    break;
                                }
                            } else {
                                // Clear image.
                                let mut w_image = LAST_CAMERA_IMAGE.write().unwrap();
                                *w_image = None;
                                break;
                            }
                        }
                    };
                });
        });
    }

    fn stop_camera(&self) {
        // Stop camera.
        self.stop_camera.store(true, Ordering::Relaxed);
        // Clear image.
        let mut w_image = LAST_CAMERA_IMAGE.write().unwrap();
        *w_image = None;
    }

    fn camera_image(&self) -> Option<(Vec<u8>, u32)> {
        let r_image = LAST_CAMERA_IMAGE.read().unwrap();
        if r_image.is_some() {
            return Some(r_image.clone().unwrap());
        }
        None
    }
}

/// Last captured image from started camera.
lazy_static! {
    static ref LAST_CAMERA_IMAGE: Arc<RwLock<Option<(Vec<u8>, u32)>>> = Arc::new(RwLock::new(None));
}
