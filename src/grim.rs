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
use log::LevelFilter::{Debug, Info, Trace, Warn};
#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

use crate::gui::PlatformApp;

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[no_mangle]
unsafe fn android_main(app: AndroidApp) {
    #[cfg(debug_assertions)]
    {
        std::env::set_var("RUST_BACKTRACE", "full");
        android_logger::init_once(
            android_logger::Config::default().with_max_level(Info).with_tag("grim"),
        );
    }

    use crate::gui::platform::Android;
    let platform = Android::new(app.clone());

    use winit::platform::android::EventLoopBuilderExtAndroid;
    let mut options = NativeOptions::default();
    options.event_loop_builder = Some(Box::new(move |builder| {
        builder.with_android_app(app);
    }));

    start(options, Box::new(|_cc| Box::new(
        PlatformApp::new(_cc, platform)
    )));
}

#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn main() {
    #[cfg(debug_assertions)]
    env_logger::builder()
        .filter_level(Debug)
        .parse_default_env()
        .init();

    use crate::gui::platform::Desktop;
    let options = NativeOptions::default();
    start(options, Box::new(|_cc| Box::new(
        PlatformApp::new(_cc, Desktop::default())
    )));
}

fn start(mut options: NativeOptions, app_creator: AppCreator) {
    options.default_theme = Theme::Light;
    options.renderer = Renderer::Wgpu;

    setup_i18n();

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
    if crate::available_locales().contains(&locale_str) {
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
