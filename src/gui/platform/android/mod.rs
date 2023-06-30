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

use std::sync::atomic::{AtomicI32, Ordering};
use lazy_static::lazy_static;
use winit::platform::android::activity::AndroidApp;

use crate::gui::{App, PlatformApp};
use crate::gui::platform::PlatformCallbacks;

#[derive(Clone)]
pub struct Android {
    android_app: AndroidApp,
}

impl Android {
    pub fn new(app: AndroidApp) -> Self {
        Self {
            android_app: app,
        }
    }
}

impl PlatformCallbacks for Android {
    fn show_keyboard(&self) {
        self.android_app.show_soft_input(true);
    }

    fn hide_keyboard(&self) {
        self.android_app.hide_soft_input(true);
    }

    fn copy_string_to_buffer(&self, data: String) {
        use jni::objects::{JObject, JValue};

        let vm = unsafe { jni::JavaVM::from_raw(self.android_app.vm_as_ptr() as _) }.unwrap();
        let mut env = vm.attach_current_thread().unwrap();
        let activity = unsafe {
            JObject::from_raw(self.android_app.activity_as_ptr() as jni::sys::jobject)
        };
        let arg_value = env.new_string(data).unwrap();
        let _ = env.call_method(
            activity,
            "copyText",
            "(Ljava/lang/String;)V",
            &[JValue::Object(&JObject::from(arg_value))]
        ).unwrap();
    }

    fn get_string_from_buffer(&self) -> String {
        use jni::objects::{JObject, JValue, JString};

        let vm = unsafe { jni::JavaVM::from_raw(self.android_app.vm_as_ptr() as _) }.unwrap();
        let mut env = vm.attach_current_thread().unwrap();
        let activity = unsafe {
            JObject::from_raw(self.android_app.activity_as_ptr() as jni::sys::jobject)
        };
        let result = env.call_method(
            activity,
            "pasteText",
            "()Ljava/lang/String;",
            &[]
        ).unwrap();
        let j_object: jni::sys::jobject = unsafe { result.as_jni().l };
        let paste_data: String = unsafe {
            env.get_string(JString::from(JObject::from_raw(j_object)).as_ref()).unwrap().into()
        };
        paste_data
    }

    fn exit(&self) {
        use jni::objects::{JObject};

        let vm = unsafe { jni::JavaVM::from_raw(self.android_app.vm_as_ptr() as _) }.unwrap();
        let mut env = vm.attach_current_thread().unwrap();
        let activity = unsafe {
            JObject::from_raw(self.android_app.activity_as_ptr() as jni::sys::jobject)
        };
        env.call_method(activity, "onExit", "()V", &[]).unwrap();
    }
}

impl PlatformApp<Android> {
    pub fn new(platform: Android) -> Self {
        Self {
            app: App::default(),
            platform,
        }
    }
}

impl eframe::App for PlatformApp<Android> {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        padding_panels(ctx);
        self.app.ui(ctx, frame, &self.platform);
    }
}

fn padding_panels(ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_padding_panel")
        .frame(egui::Frame {
            inner_margin: egui::style::Margin::same(0.0),
            fill: ctx.style().visuals.panel_fill,
            ..Default::default()
        })
        .show_separator_line(false)
        .resizable(false)
        .exact_height(DISPLAY_CUTOUT_TOP.load(Ordering::Relaxed) as f32)
        .show(ctx, |_ui| {});

    egui::TopBottomPanel::bottom("bottom_padding_panel")
        .frame(egui::Frame {
            inner_margin: egui::style::Margin::same(0.0),
            fill: ctx.style().visuals.panel_fill,
            ..Default::default()
        })
        .show_separator_line(false)
        .resizable(false)
        .exact_height(DISPLAY_CUTOUT_BOTTOM.load(Ordering::Relaxed) as f32)
        .show(ctx, |_ui| {});

    egui::SidePanel::right("right_padding_panel")
        .frame(egui::Frame {
            inner_margin: egui::style::Margin::same(0.0),
            fill: ctx.style().visuals.panel_fill,
            ..Default::default()
        })
        .show_separator_line(false)
        .resizable(false)
        .max_width(DISPLAY_CUTOUT_RIGHT.load(Ordering::Relaxed) as f32)
        .show(ctx, |_ui| {});

    egui::SidePanel::left("left_padding_panel")
        .frame(egui::Frame {
            inner_margin: egui::style::Margin::same(0.0),
            fill: ctx.style().visuals.panel_fill,
            ..Default::default()
        })
        .show_separator_line(false)
        .resizable(false)
        .max_width(DISPLAY_CUTOUT_LEFT.load(Ordering::Relaxed) as f32)
        .show(ctx, |_ui| {});
}

lazy_static! {
    static ref DISPLAY_CUTOUT_TOP: AtomicI32 = AtomicI32::new(0);
    static ref DISPLAY_CUTOUT_RIGHT: AtomicI32 = AtomicI32::new(0);
    static ref DISPLAY_CUTOUT_BOTTOM: AtomicI32 = AtomicI32::new(0);
    static ref DISPLAY_CUTOUT_LEFT: AtomicI32 = AtomicI32::new(0);
}

#[allow(non_snake_case)]
#[no_mangle]
/// Callback from Java code to update display cutouts.
pub extern "C" fn Java_mw_gri_android_MainActivity_onDisplayCutoutsChanged(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    cutouts: jni::sys::jarray
) {
    use jni::objects::{JObject, JPrimitiveArray};

    let mut array: [i32; 4] = [0; 4];
    unsafe {
        let j_obj = JObject::from_raw(cutouts);
        let j_arr = JPrimitiveArray::from(j_obj);
        _env.get_int_array_region(j_arr, 0, array.as_mut()).unwrap();
    }
    DISPLAY_CUTOUT_TOP.store(array[0], Ordering::Relaxed);
    DISPLAY_CUTOUT_RIGHT.store(array[1], Ordering::Relaxed);
    DISPLAY_CUTOUT_BOTTOM.store(array[2], Ordering::Relaxed);
    DISPLAY_CUTOUT_LEFT.store(array[3], Ordering::Relaxed);
}