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

pub mod buttons;

mod title_panel;
pub use crate::gui::views::title_panel::{TitlePanel, TitlePanelAction, TitlePanelActions};

mod network;
pub use crate::gui::views::network::Network;

mod network_node;
mod network_tuning;
mod network_metrics;

pub trait NetworkTab {
    fn ui(&mut self, ui: &mut egui::Ui, node: &mut crate::node::Node);
    fn name(&self) -> &String;
}
