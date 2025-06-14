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

use eframe::NativeOptions;
use egui::{Context, Stroke, Theme};
use lazy_static::lazy_static;
use std::sync::Arc;
use parking_lot::RwLock;

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

pub use settings::AppConfig;
pub use settings::Settings;

use crate::gui::{Colors, App};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::View;
use crate::node::Node;

i18n!("locales");

mod node;
mod wallet;
mod tor;
mod settings;
mod http;
pub mod gui;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Android platform entry point.
#[allow(dead_code)]
#[cfg(target_os = "android")]
#[no_mangle]
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
    let platform = Android::new(app.clone());
    use winit::platform::android::EventLoopBuilderExtAndroid;

    // Setup system theme if not set.
    if let None = AppConfig::dark_theme() {
        let use_dark = use_dark_theme(&platform);
        AppConfig::set_dark_theme(use_dark);
    }

    let width = app.config().screen_width_dp().unwrap() as f32;
    let height = app.config().screen_height_dp().unwrap() as f32;
    let size = egui::emath::vec2(width, height);
    let mut options = NativeOptions {
        android_app: Some(app.clone()),
        viewport: egui::ViewportBuilder::default().with_inner_size(size),
        ..Default::default()
    };
    options.event_loop_builder = Some(Box::new(move |builder| {
        builder.with_android_app(app);
    }));

    let app = App::new(platform);
    start(options, app_creator(app)).unwrap();
}

/// Check if system is using dark theme.
#[allow(dead_code)]
#[cfg(target_os = "android")]
fn use_dark_theme(platform: &gui::platform::Android) -> bool {
    let res = platform.call_java_method("useDarkTheme", "()Z", &[]).unwrap();
    unsafe { res.z != 0 }
}

/// [`App`] setup for [`eframe`].
pub fn app_creator<T: 'static>(app: App<T>) -> eframe::AppCreator<'static>
    where App<T>: eframe::App, T: PlatformCallbacks {
    Box::new(|cc| {
        setup_fonts(&cc.egui_ctx);
        // Setup images support.
        egui_extras::install_image_loaders(&cc.egui_ctx);
        Ok(Box::new(app))
    })
}

/// Entry point to start ui with [`eframe`].
pub fn start(options: NativeOptions, app_creator: eframe::AppCreator) -> eframe::Result<()> {
    // Setup translations.
    setup_i18n();
    // Start integrated node if needed.
    if AppConfig::autostart_node() {
        Node::start();
    }
    // Launch graphical interface.
    eframe::run_native("Grim", options, app_creator)
}

/// Setup application [`egui::Style`] and [`egui::Visuals`].
pub fn setup_visuals(ctx: &Context) {
    let use_dark = AppConfig::dark_theme().unwrap_or_else(|| {
        let use_dark = ctx.system_theme().unwrap_or(Theme::Dark) == Theme::Dark;
        AppConfig::set_dark_theme(use_dark);
        use_dark
    });

    let mut style = (*ctx.style()).clone();
    // Setup selection.
    style.interaction.selectable_labels = false;
    style.interaction.multi_widget_text_select = false;
    // Setup spacing for buttons.
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    // Make scroll-bar thinner and lighter.
    style.spacing.scroll.bar_width = 4.0;
    style.spacing.scroll.bar_outer_margin = -2.0;
    style.spacing.scroll.foreground_color = false;
    // Disable spacing between items.
    style.spacing.item_spacing = egui::vec2(0.0, 0.0);
    style.spacing.text_edit_width = 500.0;
    // Setup radio button/checkbox size and spacing.
    style.spacing.icon_width = 24.0;
    style.spacing.icon_width_inner = 14.0;
    style.spacing.icon_spacing = 10.0;
    // Setup style
    ctx.set_style(style);

    // Setup visuals based on app color theme.
    let mut visuals = if use_dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    // Setup selection color.
    visuals.selection.stroke = Stroke { width: 1.0, color: Colors::text(false) };
    visuals.selection.bg_fill = Colors::gold();
    // Disable stroke around panels by default.
    visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;
    // Setup stroke around inactive widgets.
    visuals.widgets.inactive.bg_stroke = View::default_stroke();
    // Setup background and foreground stroke color for widgets like pull-to-refresher.
    visuals.widgets.inactive.bg_fill = if use_dark {
        Colors::white_or_black(false)
    } else {
        Colors::yellow()
    };
    visuals.widgets.inactive.fg_stroke.color = Colors::item_button_text();
    // Setup visuals.
    ctx.set_visuals(visuals);
}

/// Setup application fonts.
pub fn setup_fonts(ctx: &Context) {
    use egui::FontFamily::Proportional;

    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "phosphor".to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../fonts/phosphor.ttf"
        )).tweak(egui::FontTweak {
            scale: 1.0,
            y_offset_factor: -0.20,
            y_offset: 0.0,
            baseline_offset_factor: 0.16,
        }),
    ));
    fonts
        .families
        .entry(Proportional)
        .or_default()
        .insert(0, "phosphor".to_owned());

    fonts.font_data.insert(
        "noto".to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../fonts/noto_sc_reg.otf"
        )).tweak(egui::FontTweak {
            scale: 1.0,
            y_offset_factor: -0.25,
            y_offset: 0.0,
            baseline_offset_factor: 0.17,
        }),
    ));
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
    // Set saved locale or get from system.
    if let Some(lang) = AppConfig::locale() {
        if rust_i18n::available_locales!().contains(&lang.as_str()) {
            rust_i18n::set_locale(lang.as_str());
        }
    } else {
        let locale = sys_locale::get_locale().unwrap_or(String::from(AppConfig::DEFAULT_LOCALE));
        let locale_str = if locale.contains("-") {
            locale.split("-").next().unwrap_or(AppConfig::DEFAULT_LOCALE)
        } else {
            locale.as_str()
        };

        // Set best possible locale.
        if rust_i18n::available_locales!().contains(&locale_str) {
            rust_i18n::set_locale(locale_str);
        } else {
            rust_i18n::set_locale(AppConfig::DEFAULT_LOCALE);
        }
    }
}

/// Get data from deeplink or opened file.
pub fn consume_incoming_data() -> Option<String> {
    let has_data = {
        let r_data = INCOMING_DATA.read();
        r_data.is_some()
    };
    if has_data {
        // Clear data.
        let mut w_data = INCOMING_DATA.write();
        let data = w_data.clone();
        *w_data = None;
        return data;
    }
    None
}

/// Provide data from deeplink or opened file.
pub fn on_data(data: String) {
    let mut w_data = INCOMING_DATA.write();
    *w_data = Some(data);
}

lazy_static! {
    /// Data provided from deeplink or opened file.
    pub static ref INCOMING_DATA: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));
}

/// Callback from Java code with with passed data.
#[allow(dead_code)]
#[allow(non_snake_case)]
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_mw_gri_android_MainActivity_onData(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    char: jni::sys::jstring
) {
    unsafe {
        let j_obj = jni::objects::JString::from_raw(char);
        if let Ok(j_str) = _env.get_string_unchecked(j_obj.as_ref()) {
            match j_str.to_str() {
                Ok(str) => {
                    let mut w_path = INCOMING_DATA.write();
                    *w_path = Some(str.to_string());
                }
                Err(_) => {}
            }
        };
    }
}