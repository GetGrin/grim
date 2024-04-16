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

use std::sync::Arc;
use egui::{IconData, pos2};
use grim::AppConfig;

pub fn main() {
    #[allow(dead_code)]
    #[cfg(not(target_os = "android"))]
    real_main();
}

#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn real_main() {
    #[cfg(debug_assertions)]
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    use grim::gui::platform::Desktop;
    use grim::gui::PlatformApp;

    let platform = Desktop::default();

    // Desktop window size.
    let (width, height) = AppConfig::window_size();

    // Setup an icon.
    let icon = image::open("img/icon.png").expect("Failed to open icon path").to_rgba8();
    let (icon_width, icon_height) = icon.dimensions();

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([width, height])
        .with_icon(Arc::new(IconData {
            rgba: icon.into_raw(),
            width: icon_width,
            height: icon_height,
        }));

    // Desktop window position.
    if let Some((x, y)) = AppConfig::window_pos() {
        viewport = viewport.with_position(pos2(x, y));
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    grim::start(options, grim::app_creator(PlatformApp::new(platform)));
}