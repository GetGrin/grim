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

use egui::{Context, Stroke};
use egui::os::OperatingSystem;

use crate::gui::{Colors, Navigator};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::screens::Root;
use crate::node::Node;

/// To be implemented by platform-specific application.
pub struct PlatformApp<Platform> {
    pub(crate) app: App,
    pub(crate) platform: Platform,
}

#[derive(Default)]
/// Contains main screen panel and ui setup.
pub struct App {
    root: Root,
}

impl App {
    /// Draw content on main screen panel.
    pub fn ui(&mut self, ctx: &Context, frame: &mut eframe::Frame, cb: &dyn PlatformCallbacks) {
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: Colors::FILL,
                .. Default::default()
            })
            .show(ctx, |ui| {
                self.root.ui(ui, frame, cb);
            });
    }

    /// Exit from the app.
    pub fn exit(frame: &mut eframe::Frame, cb: &dyn PlatformCallbacks) {
        match OperatingSystem::from_target_os() {
            OperatingSystem::Android => {
                cb.exit();
            }
            OperatingSystem::IOS => {
                //TODO: exit on iOS
            }
            OperatingSystem::Nix | OperatingSystem::Mac | OperatingSystem::Windows => {
                frame.close();
            }
            // Web
            OperatingSystem::Unknown => {}
        }
    }

    /// Setup application styles.
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
                "../../fonts/phosphor.ttf"
            )).tweak(egui::FontTweak {
                scale: 1.0,
                y_offset_factor: 0.14,
                y_offset: 0.0
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
                "../../fonts/noto_sc_reg.otf"
            )).tweak(egui::FontTweak {
                scale: 1.0,
                y_offset_factor: -0.25,
                y_offset: 0.0
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
            (Heading, FontId::new(20.0, Proportional)),
            (Body, FontId::new(16.0, Proportional)),
            (Button, FontId::new(18.0, Proportional)),
            (Small, FontId::new(12.0, Proportional)),
            (Monospace, FontId::new(16.0, Proportional)),
        ].into();

        ctx.set_style(style);
    }
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Calling when back button is pressed on Android.
pub extern "C" fn Java_mw_gri_android_MainActivity_onBackButtonPress(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    _activity: jni::objects::JObject,
) {
    Navigator::back();
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Calling on unexpected Android application termination (removal from recent apps).
pub extern "C" fn Java_mw_gri_android_MainActivity_onTermination(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    _activity: jni::objects::JObject,
) {
    Node::stop(false);
}

