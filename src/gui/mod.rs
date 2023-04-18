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

mod app;
pub use app::PlatformApp;

pub mod screens;
pub mod platform;

pub trait Ui {
    fn ui(&mut self, ui: &mut egui::Ui);
}

pub trait PlatformCallbacks {
    fn show_keyboard(&mut self);
    fn hide_keyboard(&mut self);
    fn copy_string_to_buffer(&mut self, data: String);
    fn get_string_from_buffer(&mut self) -> String;
}