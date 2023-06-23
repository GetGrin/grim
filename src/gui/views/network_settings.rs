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

use egui::Ui;
use grin_core::global::ChainTypes;
use crate::gui::Colors;
use crate::gui::icons::COMPUTER_TOWER;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, NetworkTab, NetworkTabType, View};
use crate::Settings;

#[derive(Default)]
pub struct NetworkSettings;

impl NetworkTab for NetworkSettings {
    fn get_type(&self) -> NetworkTabType {
        NetworkTabType::Settings
    }

    fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        View::sub_title(ui, format!("{} {}", COMPUTER_TOWER, t!("network_settings.server")));
        View::horizontal_line(ui, Colors::ITEM_STROKE);
        ui.add_space(4.0);

    }

    fn on_modal_ui(&mut self, ui: &mut Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {

    }
}