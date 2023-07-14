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

#[macro_use]
extern crate rust_i18n;

use std::sync::Arc;

use egui::{Context, Stroke};
#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

pub use settings::{AppConfig, Settings};

use crate::gui::{Colors, PlatformApp};
use crate::gui::platform::PlatformCallbacks;
use crate::node::Node;

i18n!("locales");

mod node;
mod wallet;
pub mod gui;

mod settings;

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[no_mangle]
/// Android platform entry point.
fn android_main(app: AndroidApp) {
    #[cfg(debug_assertions)]
    {
        std::env::set_var("RUST_BACKTRACE", "full");
        let log_config = android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("grim");
        android_logger::init_once(log_config);
    }

    use gui::platform::Android;
    use gui::PlatformApp;

    let platform = Android::new(app.clone());

    use winit::platform::android::EventLoopBuilderExtAndroid;
    let mut options = eframe::NativeOptions::default();
    // Setup limits that are guaranteed to be compatible with Android devices.
    options.wgpu_options.device_descriptor = Arc::new(|adapter| {
        let base_limits = wgpu::Limits::downlevel_webgl2_defaults();
        wgpu::DeviceDescriptor {
            label: Some("egui wgpu device"),
            features: wgpu::Features::default(),
            limits: wgpu::Limits {
                max_texture_dimension_2d: 8192,
                ..base_limits
            },
        }
    });
    options.event_loop_builder = Some(Box::new(move |builder| {
        builder.with_android_app(app);
    }));

    start(options, app_creator(PlatformApp::new(platform)));
}

/// [`PlatformApp`] setup for [`eframe`].
pub fn app_creator<T: 'static>(app: PlatformApp<T>) -> eframe::AppCreator
    where PlatformApp<T>: eframe::App, T: PlatformCallbacks {
    Box::new(|cc| {
        setup_visuals(&cc.egui_ctx);
        setup_fonts(&cc.egui_ctx);
        Box::new(app)
    })
}

/// Entry point to start ui with [`eframe`].
pub fn start(mut options: eframe::NativeOptions, app_creator: eframe::AppCreator) {
    options.default_theme = eframe::Theme::Light;
    options.renderer = eframe::Renderer::Wgpu;
    options.initial_window_size = Some(egui::Vec2::new(1200.0, 720.0));

    setup_i18n();

    if Settings::app_config_to_read().auto_start_node {
        Node::start();
    }

    let _ = eframe::run_native("Grim", options, app_creator);
}

/// Setup application [`egui::Style`] and [`egui::Visuals`].
pub fn setup_visuals(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    // Setup spacing for buttons.
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    // Make scroll-bar thinner.
    style.spacing.scroll_bar_width = 4.0;
    // Disable spacing between items.
    style.spacing.item_spacing = egui::vec2(0.0, 0.0);
    // Setup radio button/checkbox size and spacing.
    style.spacing.icon_width = 24.0;
    style.spacing.icon_width_inner = 14.0;
    style.spacing.icon_spacing = 10.0;
    // Setup style
    ctx.set_style(style);

    let mut visuals = egui::Visuals::light();
    // Setup selection color.
    visuals.selection.stroke = Stroke { width: 1.0, color: Colors::TEXT };
    visuals.selection.bg_fill = Colors::GOLD;
    // Disable stroke around panels by default
    visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;
    // Setup visuals
    ctx.set_visuals(visuals);
}

/// Setup application fonts.
pub fn setup_fonts(ctx: &Context) {
    use egui::FontFamily::Proportional;

    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "phosphor".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../fonts/phosphor.ttf"
        )).tweak(egui::FontTweak {
            scale: 1.0,
            y_offset_factor: -0.30,
            y_offset: 0.0,
            baseline_offset_factor: 0.30,
        }),
    );
    fonts
        .families
        .entry(Proportional)
        .or_default()
        .insert(0, "phosphor".to_owned());

    fonts.font_data.insert(
        "noto".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../fonts/noto_sc_reg.otf"
        )).tweak(egui::FontTweak {
            scale: 1.0,
            y_offset_factor: -0.25,
            y_offset: 0.0,
            baseline_offset_factor: 0.17,
        }),
    );
    fonts
        .families
        .entry(Proportional)
        .or_default()
        .insert(0, "noto".to_owned());

    ctx.set_fonts(fonts);

    use egui::FontId;
    use egui::TextStyle::*;

    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (Heading, FontId::new(19.0, Proportional)),
        (Body, FontId::new(16.0, Proportional)),
        (Button, FontId::new(17.0, Proportional)),
        (Small, FontId::new(15.0, Proportional)),
        (Monospace, FontId::new(16.0, Proportional)),
    ].into();

    ctx.set_style(style);
}

/// Setup translations.
fn setup_i18n() {
    const DEFAULT_LOCALE: &str = "en";
    let locale = sys_locale::get_locale().unwrap_or(String::from(DEFAULT_LOCALE));
    let locale_str = if locale.contains("-") {
        locale.split("-").next().unwrap_or(DEFAULT_LOCALE)
    } else {
        locale.as_str()
    };
    if _rust_i18n_available_locales().contains(&locale_str) {
        rust_i18n::set_locale(locale_str);
    }
}