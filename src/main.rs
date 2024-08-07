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

#![windows_subsystem = "windows"]

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

    // Setup callback on panic crash.
    std::panic::set_hook(Box::new(|info| {
        let backtrace = backtrace::Backtrace::new();
        // Format error.
        let time = grim::gui::views::View::format_time(chrono::Utc::now().timestamp());
        let target = egui::os::OperatingSystem::from_target_os();
        let ver = grim::VERSION;
        let msg = panic_message::panic_info_message(info);
        let err = format!("{} - {:?} - v{}\n\n{}\n\n{:?}", time, target, ver, msg, backtrace);
        // Save backtrace to file.
        let log = grim::Settings::crash_report_path();
        if log.exists() {
            std::fs::remove_file(log.clone()).unwrap();
        }
        std::fs::write(log, err.as_bytes()).unwrap();
        // Setup flag to show crash after app restart.
        grim::AppConfig::set_show_crash(true);
    }));

    // Start GUI.
    let _ = std::panic::catch_unwind(|| {
        start_desktop_gui();
    });
}

/// Start GUI with Desktop related setup.
#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn start_desktop_gui() {
    use grim::AppConfig;
    use dark_light::Mode;

    let platform = grim::gui::platform::Desktop::default();

    // Setup system theme if not set.
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
        .with_min_inner_size([AppConfig::MIN_WIDTH, AppConfig::MIN_HEIGHT])
        .with_inner_size([width, height]);
    // Setup an icon.
    if let Ok(icon) = eframe::icon_data::from_png_bytes(include_bytes!("../img/icon.png")) {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }
    // Setup window position.
    if let Some((x, y)) = AppConfig::window_pos() {
        viewport = viewport.with_position(egui::pos2(x, y));
    }
    // Setup window decorations.
    let is_mac = egui::os::OperatingSystem::from_target_os() == egui::os::OperatingSystem::Mac;
    viewport = viewport
        .with_fullsize_content_view(true)
        .with_title_shown(false)
        .with_titlebar_buttons_shown(false)
        .with_titlebar_shown(false)
        .with_transparent(true)
        .with_decorations(is_mac);

    let mut options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    // Use Glow renderer for Windows.
    let win = egui::os::OperatingSystem::from_target_os() == egui::os::OperatingSystem::Windows;
    options.renderer = if win {
        eframe::Renderer::Glow
    } else {
        eframe::Renderer::Wgpu
    };

    // Start GUI.
    match grim::start(options.clone(), grim::app_creator(grim::gui::App::new(platform.clone()))) {
        Ok(_) => {}
        Err(e) => {
            if win {
                panic!("{}", e);
            }
            // Start with another renderer on error.
            options.renderer = eframe::Renderer::Glow;
            match grim::start(options, grim::app_creator(grim::gui::App::new(platform))) {
                Ok(_) => {}
                Err(e) => {
                    panic!("{}", e);
                }
            }
        }
    }
}