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

use eframe::icon_data::from_png_bytes;

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
    use grim::AppConfig;

    use std::sync::Arc;
    use egui::pos2;

    let platform = Desktop::default();

    // Setup system theme if not set.
    use dark_light::Mode;
    if let None = AppConfig::dark_theme() {
        let dark = match dark_light::detect() {
            Mode::Dark => true,
            Mode::Light => false,
            Mode::Default => false
        };
        AppConfig::set_dark_theme(dark);
    }

    // Setup window size.
    let (width, height) = AppConfig::window_size();

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([width, height]);

    // Setup an icon.
    if let Ok(icon) = from_png_bytes(include_bytes!("../img/icon.png")) {
        viewport = viewport.with_icon(Arc::new(icon));
    }

    // Setup window position.
    if let Some((x, y)) = AppConfig::window_pos() {
        viewport = viewport.with_position(pos2(x, y));
    }

    let mut options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    options.renderer = eframe::Renderer::Wgpu;
    match grim::start(options.clone(), grim::app_creator(PlatformApp::new(platform.clone()))) {
        Ok(_) => {}
        Err(_) => {
            // Start with Glow renderer on error.
            options.renderer = eframe::Renderer::Glow;
            grim::start(options, grim::app_creator(PlatformApp::new(platform))).unwrap();
        }
    }
}