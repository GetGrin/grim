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

use egui::Color32;

pub struct Colors;

impl Colors {
    pub const WHITE: Color32 = Color32::from_gray(253);
    pub const BLACK: Color32 = Color32::from_gray(2);
    pub const TRANSPARENT: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 0);
    pub const SEMI_TRANSPARENT: Color32 = Color32::from_black_alpha(100);
    pub const YELLOW: Color32 = Color32::from_rgb(254, 241, 2);
    pub const GOLD: Color32 = Color32::from_rgb(255, 215, 0);
    pub const GREEN: Color32 = Color32::from_rgb(0, 0x64, 0);
    pub const RED: Color32 = Color32::from_rgb(0x8B, 0, 0);
    pub const FILL: Color32 = Color32::from_gray(244);
    pub const FILL_DARK: Color32 = Color32::from_gray(232);
    pub const TITLE: Color32 = Color32::from_gray(60);
    pub const TEXT: Color32 = Color32::from_gray(80);
    pub const TEXT_BUTTON: Color32 = Color32::from_gray(70);
    pub const BUTTON: Color32 = Color32::from_gray(249);
    pub const GRAY: Color32 = Color32::from_gray(120);
    pub const STROKE: Color32 = Color32::from_gray(190);
    pub const INACTIVE_TEXT: Color32 = Color32::from_gray(150);
    pub const ITEM_STROKE: Color32 = Color32::from_gray(220);
}
