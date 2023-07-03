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

use eframe::{AppCreator, NativeOptions, Renderer, Theme};
#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

use crate::gui::{App, PlatformApp};
use crate::node::Node;
use crate::Settings;

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

    use crate::gui::platform::Android;
    let platform = Android::new(app.clone());

    use winit::platform::android::EventLoopBuilderExtAndroid;
    let mut options = NativeOptions::default();
    options.event_loop_builder = Some(Box::new(move |builder| {
        builder.with_android_app(app);
    }));

    start(options, app_creator(PlatformApp::new(platform)));
}

#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn main() {
    #[cfg(debug_assertions)]
    env_logger::builder()
        .filter_level(Info)
        .parse_default_env()
        .init();

    use crate::gui::platform::Desktop;
    let options = NativeOptions::default();
    start(options, app_creator(Desktop::default()));
}

fn app_creator<T: 'static>(app: PlatformApp<T>) -> AppCreator where PlatformApp<T>: eframe::App {
    Box::new(|cc| {
        App::setup_visuals(&cc.egui_ctx);
        App::setup_fonts(&cc.egui_ctx);
        //TODO: Setup storage
        Box::new(app)
    })
}

fn start(mut options: NativeOptions, app_creator: AppCreator) {
    options.default_theme = Theme::Light;
    options.renderer = Renderer::Wgpu;

    setup_i18n();

    if Settings::app_config_to_read().auto_start_node {
        Node::start();
    }

    eframe::run_native("Grim", options, app_creator);
}

fn setup_i18n() {
    const DEFAULT_LOCALE: &str = "en";
    let locale = sys_locale::get_locale().unwrap_or(String::from(DEFAULT_LOCALE));
    let locale_str = if locale.contains("-") {
        locale.split("-").next().unwrap_or(DEFAULT_LOCALE)
    } else {
        DEFAULT_LOCALE
    };
    if crate::_rust_i18n_available_locales().contains(&locale_str) {
        rust_i18n::set_locale(locale_str);
    }
}

mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub fn info_strings() -> (String, String) {
    (
        format!(
            "This is Grim version {}{}, built for {} by {}.",
            built_info::PKG_VERSION,
            built_info::GIT_VERSION.map_or_else(|| "".to_owned(), |v| format!(" (git {})", v)),
            built_info::TARGET,
            built_info::RUSTC_VERSION,
        ),
        format!(
            "Built with profile \"{}\", features \"{}\".",
            built_info::PROFILE,
            built_info::FEATURES_STR,
        ),
    )
}