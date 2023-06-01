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

use egui::{Response, RichText, Spinner, Ui, Widget};

use crate::gui::colors::COLOR_DARK;

pub struct ProgressLoading {
    text: String
}

impl ProgressLoading {
    pub fn new(text: String) -> Self {
        Self {
            text
        }
    }
}

impl Widget for ProgressLoading {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical_centered_justified(|ui| {
            ui.add_space(10.0);
            Spinner::new().size(36.0).color(COLOR_DARK).ui(ui);
            ui.add_space(10.0);
            ui.label(RichText::new(self.text).size(18.0).color(COLOR_DARK));
        }).response
    }
}