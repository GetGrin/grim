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

use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::Modal;

/// Network tab content interface.
pub trait NetworkTab {
    fn get_type(&self) -> NetworkTabType;
    fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks);
    fn on_modal_ui(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks);
}

/// Type of [`NetworkTab`] content.
#[derive(PartialEq)]
pub enum NetworkTabType {
    Node,
    Metrics,
    Mining,
    Settings
}

impl NetworkTabType {
    pub fn title(&self) -> String {
        match *self {
            NetworkTabType::Node => { t!("network.node") }
            NetworkTabType::Metrics => { t!("network.metrics") }
            NetworkTabType::Mining => { t!("network.mining") }
            NetworkTabType::Settings => { t!("network.settings") }
        }
    }
}