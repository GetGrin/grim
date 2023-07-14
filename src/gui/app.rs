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

use egui::Context;
use lazy_static::lazy_static;

use crate::gui::Colors;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::Root;

/// Implements ui entry point and contains platform-specific callbacks.
pub struct PlatformApp<Platform> {
    /// Platform specific callbacks handler.
    pub(crate) platform: Platform,
    /// Main ui content.
    root: Root
}

impl<Platform> PlatformApp<Platform> {
    pub fn new(platform: Platform) -> Self {
        Self { platform, root: Root::default() }
    }
}

impl<Platform: PlatformCallbacks> eframe::App for PlatformApp<Platform> {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        // Show panels to support display cutouts (insets).
        padding_panels(ctx);

        // Show main content.
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: Colors::YELLOW,
                ..Default::default()
            })
            .show(ctx, |ui| {
                self.root.ui(ui, frame, &self.platform);
            });
    }

    fn on_close_event(&mut self) -> bool {
        Root::show_exit_modal();
        self.root.exit_allowed
    }
}

/// Draw panels to support display cutouts (insets).
fn padding_panels(ctx: &Context) {
    egui::TopBottomPanel::top("top_padding_panel")
        .frame(egui::Frame {
            inner_margin: egui::style::Margin::same(0.0),
            fill: Colors::YELLOW,
            ..Default::default()
        })
        .show_separator_line(false)
        .resizable(false)
        .exact_height(get_top_display_cutout())
        .show(ctx, |_ui| {});

    egui::TopBottomPanel::bottom("bottom_padding_panel")
        .frame(egui::Frame {
            inner_margin: egui::style::Margin::same(0.0),
            fill: Colors::BLACK,
            ..Default::default()
        })
        .show_separator_line(false)
        .resizable(false)
        .exact_height(get_bottom_display_cutout())
        .show(ctx, |_ui| {});

    egui::SidePanel::right("right_padding_panel")
        .frame(egui::Frame {
            inner_margin: egui::style::Margin::same(0.0),
            fill: Colors::YELLOW,
            ..Default::default()
        })
        .show_separator_line(false)
        .resizable(false)
        .max_width(get_right_display_cutout())
        .show(ctx, |_ui| {});

    egui::SidePanel::left("left_padding_panel")
        .frame(egui::Frame {
            inner_margin: egui::style::Margin::same(0.0),
            fill: Colors::YELLOW,
            ..Default::default()
        })
        .show_separator_line(false)
        .resizable(false)
        .max_width(get_left_display_cutout())
        .show(ctx, |_ui| {});
}

/// Get top display cutout (inset) size.
pub fn get_top_display_cutout() -> f32 {
    TOP_DISPLAY_CUTOUT.load(Ordering::Relaxed) as f32
}

/// Get right display cutout (inset) size.
pub fn get_right_display_cutout() -> f32 {
    RIGHT_DISPLAY_CUTOUT.load(Ordering::Relaxed) as f32
}

/// Get bottom display cutout (inset) size.
pub fn get_bottom_display_cutout() -> f32 {
    BOTTOM_DISPLAY_CUTOUT.load(Ordering::Relaxed) as f32
}

/// Get left display cutout (inset) size.
pub fn get_left_display_cutout() -> f32 {
    LEFT_DISPLAY_CUTOUT.load(Ordering::Relaxed) as f32
}

/// Fields to handle platform-specific display cutouts (insets).
lazy_static! {
    static ref TOP_DISPLAY_CUTOUT: AtomicI32 = AtomicI32::new(0);
    static ref RIGHT_DISPLAY_CUTOUT: AtomicI32 = AtomicI32::new(0);
    static ref BOTTOM_DISPLAY_CUTOUT: AtomicI32 = AtomicI32::new(0);
    static ref LEFT_DISPLAY_CUTOUT: AtomicI32 = AtomicI32::new(0);
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Callback from Java code to update display cutouts (insets).
pub extern "C" fn Java_mw_gri_android_MainActivity_onDisplayCutouts(
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
    TOP_DISPLAY_CUTOUT.store(array[0], Ordering::Relaxed);
    RIGHT_DISPLAY_CUTOUT.store(array[1], Ordering::Relaxed);
    BOTTOM_DISPLAY_CUTOUT.store(array[2], Ordering::Relaxed);
    LEFT_DISPLAY_CUTOUT.store(array[3], Ordering::Relaxed);
}

