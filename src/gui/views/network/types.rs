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

use serde_derive::{Deserialize, Serialize};
use crate::gui::platform::PlatformCallbacks;

/// Integrated node tab content interface.
pub trait NodeTab {
    fn get_type(&self) -> NodeTabType;
    fn tab_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks);
}

/// Type of [`NodeTab`] content.
#[derive(PartialEq)]
pub enum NodeTabType {
    Info,
    Metrics,
    Mining,
    Settings
}

impl NodeTabType {
    pub fn title(&self) -> String {
        match *self {
            NodeTabType::Info => t!("network.node").into(),
            NodeTabType::Metrics => t!("network.metrics").into(),
            NodeTabType::Mining => t!("network.mining").into(),
            NodeTabType::Settings => t!("network.settings").into()
        }
    }
}

/// Connection details to share.
#[derive(Serialize, Deserialize, Clone)]
pub struct ShareConnection {
    #[serde(rename(serialize = "ipPort", deserialize = "ipPort"))]
    pub url: String,
    pub username: String,
    pub secret: String
}