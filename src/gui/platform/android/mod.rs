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

use eframe::epaint::Stroke;
use winit::platform::android::activity::AndroidApp;

use crate::gui::{App, PlatformApp};
use crate::gui::platform::PlatformCallbacks;

#[derive(Clone)]
pub struct Android {
    android_app: AndroidApp,
    cutouts: [i32; 4],
    window_size: [f32; 2]
}

impl Android {
    pub fn new(app: AndroidApp) -> Self {
        Self {
            android_app: app,
            cutouts: Default::default(),
            window_size: Default::default()
        }
    }
}

impl PlatformCallbacks for Android {
    fn show_keyboard(&mut self) {
        self.android_app.show_soft_input(true);
    }

    fn hide_keyboard(&mut self) {
        self.android_app.hide_soft_input(true);
    }

    fn copy_string_to_buffer(&mut self, data: String) {
        //TODO
    }

    fn get_string_from_buffer(&mut self) -> String {
        //TODO
        "".to_string()
    }
}

//TODO

// pub trait PlatformSetup<T> {
//     fn new(cc: &eframe::CreationContext<'_>, platform: T) -> Box<Self> {
//         Self::setup_visuals(&cc.egui_ctx);
//         return Self {
//             app: App::default(),
//             platform
//         }
//     }
//     fn setup_visuals(ctx: &egui::Context);
//
// }
//
// impl PlatformSetup<Android> for PlatformApp<Android> {
//     fn setup_visuals(ctx: &Context) {
//
//     }
// }

impl PlatformApp<Android> {
    pub fn new(cc: &eframe::CreationContext<'_>, platform: Android) -> Self {
        Self::setup_visuals(&cc.egui_ctx);
        Self::setup_fonts(&cc.egui_ctx);
        Self {
            app: App::default(),
            platform,
        }
    }

    fn setup_visuals(ctx: &egui::Context) {
        // Setup style
        let mut style = (*ctx.style()).clone();

        // Make scroll-bar thinner
        style.spacing.scroll_bar_width = 4.0;
        //
        style.spacing.item_spacing = egui::vec2(0.0, 0.0);
        ctx.set_style(style);

        // Setup visuals
        let mut visuals = egui::Visuals::light();

        // Disable stroke around panels by default
        visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;
        ctx.set_visuals(visuals);
    }

    fn setup_fonts(ctx: &egui::Context) {
        use egui::FontFamily::Proportional;

        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "phosphor".to_owned(),
            egui::FontData::from_static(include_bytes!(
                "../../../../fonts/phosphor.ttf"
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
                "../../../../fonts/noto_sc_reg.otf"
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

    fn get_display_cutouts(app: &AndroidApp) -> [i32; 4] {
        use jni::objects::{JObject, JPrimitiveArray};

        let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr() as _) }.unwrap();
        let mut env = vm.attach_current_thread().unwrap();
        let activity = unsafe {
            JObject::from_raw(app.activity_as_ptr() as jni::sys::jobject)
        };
        let _res = env
            .call_method(
                activity,
                "getDisplayCutouts",
                "()[I",
                &[],
            )
            .unwrap();
        let mut array: [i32; 4] = [0; 4];
        let object: jni::sys::jobject = unsafe { _res.as_jni().l };
        unsafe {
            env.get_int_array_region(JPrimitiveArray::from(
                JObject::from_raw(object)), 0, array.as_mut()
            ).unwrap();
        }
        array
    }
}

impl eframe::App for PlatformApp<Android> {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let _x = frame.info().window_info.size.x;
        let _y = frame.info().window_info.size.y;
        if _x != self.platform.window_size[0] || _y != self.platform.window_size[1] {
            self.platform.window_size[0] = _x;
            self.platform.window_size[1] = _y;
            self.platform.cutouts = Self::get_display_cutouts(&self.platform.android_app);
        }
        padding_panels(self, ctx);
        self.app.ui(ctx, frame, &self.platform);
    }
}

fn padding_panels(app: &PlatformApp<Android>, ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_padding_panel")
        .frame(egui::Frame {
            inner_margin: egui::style::Margin::same(0.0),
            fill: ctx.style().visuals.panel_fill,
            ..Default::default()
        })
        .show_separator_line(false)
        .resizable(false)
        .exact_height(app.platform.cutouts[0] as f32)
        .show(ctx, |ui| {});

    egui::TopBottomPanel::bottom("bottom_padding_panel")
        .frame(egui::Frame {
            inner_margin: egui::style::Margin::same(0.0),
            fill: ctx.style().visuals.panel_fill,
            ..Default::default()
        })
        .show_separator_line(false)
        .resizable(false)
        .exact_height(app.platform.cutouts[2] as f32)
        .show(ctx, |ui| {});

    egui::SidePanel::right("right_padding_panel")
        .frame(egui::Frame {
            inner_margin: egui::style::Margin::same(0.0),
            fill: ctx.style().visuals.panel_fill,
            ..Default::default()
        })
        .show_separator_line(false)
        .resizable(false)
        .max_width(app.platform.cutouts[1] as f32)
        .show(ctx, |ui| {});

    egui::SidePanel::left("left_padding_panel")
        .frame(egui::Frame {
            inner_margin: egui::style::Margin::same(0.0),
            fill: ctx.style().visuals.panel_fill,
            ..Default::default()
        })
        .show_separator_line(false)
        .resizable(false)
        .max_width(app.platform.cutouts[3] as f32)
        .show(ctx, |ui| {});
}