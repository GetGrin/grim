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

pub use self::platform::*;

#[cfg(target_os = "android")]
#[path = "android/mod.rs"]
pub mod platform;
#[cfg(not(target_os = "android"))]
#[path = "desktop/mod.rs"]
pub mod platform;

pub trait PlatformCallbacks {
    fn set_context(&mut self, ctx: &egui::Context);
    fn show_keyboard(&self);
    fn hide_keyboard(&self);
    fn copy_string_to_buffer(&self, data: String);
    fn get_string_from_buffer(&self) -> String;
    fn start_camera(&self);
    fn stop_camera(&self);
    fn camera_image(&self) -> Option<(Vec<u8>, u32)>;
    fn can_switch_camera(&self) -> bool;
    fn switch_camera(&self);
    fn share_data(&self, name: String, data: Vec<u8>) -> Result<(), std::io::Error>;
    fn pick_file(&self) -> Option<String>;
    fn picked_file(&self) -> Option<String>;
    fn request_user_attention(&self);
    fn user_attention_required(&self) -> bool;
    fn clear_user_attention(&self);
}