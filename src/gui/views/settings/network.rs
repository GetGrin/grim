// Copyright 2025 The Grim Developers
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

use crate::gui::icons::COMPUTER_TOWER;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::types::ContentContainer;
use crate::gui::views::{Modal, View};
use crate::gui::Colors;

/// Network communication settings content.
pub struct NetworkSettingsContent {

}

impl ContentContainer for NetworkSettingsContent {
    fn modal_ids(&self) -> Vec<&'static str> {
        vec![

        ]
    }

    fn modal_ui(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
    }

    fn container_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        View::sub_title(ui, format!("{} {}", COMPUTER_TOWER, t!("network.self")));
        View::horizontal_line(ui, Colors::stroke());
        ui.add_space(6.0);
    }
}

impl Default for NetworkSettingsContent {
    fn default() -> Self {
        Self {
            
        }
    }
}