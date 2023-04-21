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

use std::ops::Deref;
use eframe::Frame;
use egui::{ScrollArea, Ui};

use crate::gui::app::Screens;
use crate::gui::{PlatformCallbacks};

pub struct Wallets {

}

impl Default for Wallets {
    fn default() -> Self {
        Self {
        }
    }
}

impl super::Screen for Wallets {
    fn name(&self) -> String {
        t!("wallets")
    }

    fn show(&mut self, ui: &mut Ui, frame: &mut Frame, cb: &dyn PlatformCallbacks) {
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for item in 1..=55 {
                    ui.heading(format!("This is longest future Wallet #{}", item));
                }
            });
    }
}