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

use crate::AppConfig;

/// Provides colors values based on current theme.
pub struct Colors;

const WHITE: Color32 = Color32::from_gray(253);
const BLACK: Color32 = Color32::from_gray(12);

const SEMI_TRANSPARENT: Color32 = Color32::from_black_alpha(100);
const DARK_SEMI_TRANSPARENT: Color32 = Color32::from_black_alpha(170);

const GOLD: Color32 = Color32::from_rgb(255, 215, 0);

const YELLOW: Color32 = Color32::from_rgb(254, 241, 2);
const YELLOW_DARK: Color32 = Color32::from_rgb(239, 229, 3);

const GREEN: Color32 = Color32::from_rgb(0, 0x64, 0);
const GREEN_DARK: Color32 = Color32::from_rgb(0, (0x64 as f32 * 1.3 + 0.5) as u8, 0);

const RED: Color32 = Color32::from_rgb(0x8B, 0, 0);
const RED_DARK: Color32 = Color32::from_rgb((0x8B as f32 * 1.3 + 0.5) as u8, 0, 0);

const BLUE: Color32 = Color32::from_rgb(0, 0x66, 0xE4);
const BLUE_DARK: Color32 =
    Color32::from_rgb(0, (0x66 as f32 * 1.3 + 0.5) as u8, (0xE4 as f32 * 1.3 + 0.5) as u8);

const FILL: Color32 = Color32::from_gray(244);
const FILL_DARK: Color32 = Color32::from_gray(24);

const FILL_DEEP: Color32 = Color32::from_gray(238);
const FILL_DEEP_DARK: Color32 = Color32::from_gray(18);

const FILL_LITE: Color32 = Color32::from_gray(249);
const FILL_LITE_DARK: Color32 = Color32::from_gray(16);

const TEXT: Color32 = Color32::from_gray(80);
const TEXT_DARK: Color32 = Color32::from_gray(185);

const CHECKBOX: Color32 = Color32::from_gray(100);
const CHECKBOX_DARK: Color32 = Color32::from_gray(175);

const TEXT_BUTTON: Color32 = Color32::from_gray(70);
const TEXT_BUTTON_DARK: Color32 = Color32::from_gray(195);

const TITLE: Color32 = Color32::from_gray(60);
const TITLE_DARK: Color32 = Color32::from_gray(205);

const GRAY: Color32 = Color32::from_gray(120);
const GRAY_DARK: Color32 = Color32::from_gray(145);

const STROKE_DARK: Color32 = Color32::from_gray(50);

const INACTIVE_TEXT: Color32 = Color32::from_gray(150);
const INACTIVE_TEXT_DARK: Color32 = Color32::from_gray(115);

const ITEM_BUTTON: Color32 = Color32::from_gray(90);
const ITEM_BUTTON_DARK: Color32 = Color32::from_gray(175);

const ITEM_STROKE: Color32 = Color32::from_gray(220);
const ITEM_STROKE_DARK: Color32 = Color32::from_gray(40);

const ITEM_HOVER: Color32 = Color32::from_gray(205);
const ITEM_HOVER_DARK: Color32 = Color32::from_gray(48);

/// Check if dark theme should be used.
fn use_dark() -> bool {
    AppConfig::dark_theme().unwrap_or(false)
}

impl Colors {
    pub const TRANSPARENT: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 0);
    pub const STROKE: Color32 = Color32::from_gray(200);

    pub fn white_or_black(black_in_white: bool) -> Color32 {
        if use_dark() {
            if black_in_white {
                WHITE
            } else {
                BLACK
            }
        } else {
            if black_in_white {
                BLACK
            } else {
                WHITE
            }
        }
    }

    pub fn semi_transparent() -> Color32 {
        if use_dark() {
            DARK_SEMI_TRANSPARENT
        } else {
            SEMI_TRANSPARENT
        }
    }

    pub fn gold() -> Color32 {
        if use_dark() {
            GOLD.gamma_multiply(0.9)
        } else {
            GOLD
        }
    }

    pub fn yellow() -> Color32 {
        YELLOW
    }

    pub fn yellow_dark() -> Color32 {
        YELLOW_DARK
    }

    pub fn green() -> Color32 {
        if use_dark() {
            GREEN_DARK
        } else {
            GREEN
        }
    }

    pub fn red() -> Color32 {
        if use_dark() {
            RED_DARK
        } else {
            RED
        }
    }

    pub fn blue() -> Color32 {
        if use_dark() {
            BLUE_DARK
        } else {
            BLUE
        }
    }

    pub fn fill() -> Color32 {
        if use_dark() {
            FILL_DARK
        } else {
            FILL
        }
    }

    pub fn fill_deep() -> Color32 {
        if use_dark() {
            FILL_DEEP_DARK
        } else {
            FILL_DEEP
        }
    }

    pub fn fill_lite() -> Color32 {
        if use_dark() {
            FILL_LITE_DARK
        } else {
            FILL_LITE
        }
    }

    pub fn checkbox() -> Color32 {
        if use_dark() {
            CHECKBOX_DARK
        } else {
            CHECKBOX
        }
    }

    pub fn text(always_light: bool) -> Color32 {
        if use_dark() && !always_light {
            TEXT_DARK
        } else {
            TEXT
        }
    }

    pub fn text_button() -> Color32 {
        if use_dark() {
            TEXT_BUTTON_DARK
        } else {
            TEXT_BUTTON
        }
    }

    pub fn title(always_light: bool) -> Color32 {
        if use_dark() && !always_light {
            TITLE_DARK
        } else {
            TITLE
        }
    }

    pub fn gray() -> Color32 {
        if use_dark() {
            GRAY_DARK
        } else {
            GRAY
        }
    }

    pub fn stroke() -> Color32 {
        if use_dark() {
            STROKE_DARK
        } else {
            Self::STROKE
        }
    }

    pub fn inactive_text() -> Color32 {
        if use_dark() {
            INACTIVE_TEXT_DARK
        } else {
            INACTIVE_TEXT
        }
    }

    pub fn item_button() -> Color32 {
        if use_dark() {
            ITEM_BUTTON_DARK
        } else {
            ITEM_BUTTON
        }
    }

    pub fn item_stroke() -> Color32 {
        if use_dark() {
            ITEM_STROKE_DARK
        } else {
            ITEM_STROKE
        }
    }

    pub fn item_hover() -> Color32 {
        if use_dark() {
            ITEM_HOVER_DARK
        } else {
            ITEM_HOVER
        }
    }
}