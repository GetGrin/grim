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

use egui::ScrollArea;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, NetworkTab, NetworkTabType};
use crate::gui::views::settings_node::NodeSetup;

#[derive(Default)]
pub struct NetworkSettings {
    node_setup: NodeSetup
}

impl NetworkTab for NetworkSettings {
    fn get_type(&self) -> NetworkTabType {
        NetworkTabType::Settings
    }

    fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        ScrollArea::vertical()
            .id_source("network_settings")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                self.node_setup.ui(ui, cb);
            });
    }

    fn on_modal_ui(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        match modal.id {
            NodeSetup::API_PORT_MODAL => {
                self.node_setup.api_port_modal_ui(ui, modal, cb);
            },
            _ => {}
        }
    }
}