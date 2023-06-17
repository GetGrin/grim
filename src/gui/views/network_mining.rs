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

use crate::gui::Colors;
use crate::gui::views::{Network, NetworkTab, NetworkTabType, View};
use crate::node::Node;

#[derive(Default)]
pub struct NetworkMining;

impl NetworkTab for NetworkMining {
    fn get_type(&self) -> NetworkTabType {
        NetworkTabType::Mining
    }

    fn name(&self) -> String {
        t!("network.mining")
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let server_stats = Node::get_stats();
        // Show loading spinner when stats are not available or message when server is not enabled.
        if !server_stats.is_some() {
            if !Node::is_running() {
                Network::disabled_server_content(ui);
            } else {
                ui.centered_and_justified(|ui| {
                    View::big_loading_spinner(ui);
                });
            }
            return;
        }

        let stats = server_stats.as_ref().unwrap();

    }
}