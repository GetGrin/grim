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

use egui::{Color32, FontTweak, Visuals};
use egui::epaint::Shadow;

use jni::objects::{JObject, JPrimitiveArray};
use winit::platform::android::activity::AndroidApp;

use crate::gui::{PlatformApp, PlatformCallbacks};
use crate::gui::app::Screens;

#[derive(Clone)]
pub struct Android {
    pub android_app: AndroidApp,
    pub cutouts: [i32; 4],
    pub window_size: [f32; 2]
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

    }

    fn get_string_from_buffer(&mut self) -> String {
        "".to_string()
    }
}

impl PlatformApp<Android> {
    pub fn new(cc: &eframe::CreationContext<'_>, platform: Android) -> Self {
        setup_fonts(&cc.egui_ctx);
        Self {
            screens: Screens::default(),
            platform,
        }
    }
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "roboto".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../../../fonts/roboto.ttf"
        )).tweak(FontTweak {
            scale: 1.0,
            y_offset_factor: -0.20,
            y_offset: 0.0
        }),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "roboto".to_owned());
    ctx.set_fonts(fonts);
}

impl eframe::App for PlatformApp<Android> {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let _x = frame.info().window_info.size.x;
        let _y = frame.info().window_info.size.y;
        if _x != self.platform.window_size[0] || _y != self.platform.window_size[1] {
            self.platform.window_size[0] = _x;
            self.platform.window_size[1] = _y;
            self.platform.cutouts = get_display_cutouts(&self.platform.android_app);
        }

        let is_dark = ctx.style().visuals.dark_mode;
        egui::TopBottomPanel::top("top_padding_panel")
            .frame(egui::Frame {
                shadow: Shadow::NONE,
                fill: if is_dark {Color32::BLACK} else {Color32::WHITE},
                ..Default::default()
            })
            .show_separator_line(false)
            .resizable(false)
            .exact_height(self.platform.cutouts[0] as f32)
            .show(ctx, |ui| {});

        egui::TopBottomPanel::bottom("bottom_padding_panel")
            .frame(egui::Frame {
                shadow: Shadow::NONE,
                fill: if is_dark {Color32::BLACK} else {Color32::WHITE},
                ..Default::default()
            })
            .show_separator_line(false)
            .resizable(false)
            .exact_height(self.platform.cutouts[2] as f32)
            .show(ctx, |ui| {});

        egui::SidePanel::right("right_padding_panel")
            .frame(egui::Frame {
                shadow: Shadow::NONE,
                fill: if is_dark {Color32::BLACK} else {Color32::WHITE},
                ..Default::default()
            })
            .show_separator_line(false)
            .resizable(false)
            .default_width(self.platform.cutouts[1] as f32)
            .show(ctx, |ui| {});

        egui::SidePanel::left("left_padding_panel")
            .frame(egui::Frame {
                shadow: Shadow::NONE,
                fill: if is_dark {Color32::BLACK} else {Color32::WHITE},
                ..Default::default()
            })
            .show_separator_line(false)
            .resizable(false)
            .default_width(self.platform.cutouts[3] as f32)
            .show(ctx, |ui| {});

        egui::CentralPanel::default().show(ctx, |ui| {
            self.screens.ui(ctx, frame, &self.platform);
        });
    }
}

fn get_display_cutouts(app: &AndroidApp) -> [i32; 4] {
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
