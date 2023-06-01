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
pub mod app;

pub trait PlatformCallbacks {
    fn show_keyboard(&self);
    fn hide_keyboard(&self);
    fn copy_string_to_buffer(&self, data: String);
    fn get_string_from_buffer(&self) -> String;
    fn exit(&self);
}