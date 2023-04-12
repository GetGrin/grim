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

use egui::FontTweak;

#[derive(Default)]
pub struct MainApp {
    root: egui_demo_lib::DemoWindows,
    status_bar_height: Option<f32>
}

impl MainApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_fonts(&cc.egui_ctx);
        Self::default()
    }
}

#[cfg(target_os = "android")]
impl crate::gui::AndroidUi for MainApp {
    fn with_status_bar_height(mut self, value: f32) -> Self {
        self.status_bar_height = Some(value);
        self
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_padding_panel")
            .resizable(false)
            .exact_height(self.status_bar_height.unwrap_or(0.0))
            .show(ctx, |ui| {});
        self.root.ui(ctx);
    }
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "jura".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../fonts/jura.ttf"
        )).tweak(FontTweak {
            scale: 1.0,
            y_offset_factor: -0.25,
            y_offset: 0.0
        }),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "jura".to_owned());
    ctx.set_fonts(fonts);
}