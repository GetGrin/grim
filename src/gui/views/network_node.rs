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

use std::borrow::Cow;
use std::ptr::null;
use std::sync::mpsc;

use chrono::Utc;
use egui::{Ui, Widget};
use grin_chain::SyncStatus;
use grin_core::global::ChainTypes;
use grin_servers::ServerStats;

use crate::gui::views::NetworkTab;
use crate::node::Node;


pub struct NetworkNode {
    title: String
}

impl Default for NetworkNode {
    fn default() -> Self {
        Self {
            title: t!("node"),
        }
    }
}

impl NetworkTab for NetworkNode {
    fn ui(&mut self, ui: &mut Ui, node: &mut Node) {
        // ui.vertical_centered_justified(|ui| {
        //     let node_state = node.acquire_state();
        //     let stats = &node_state.stats;
        //     if stats.is_some() {
        //         ui.horizontal_wrapped(|ui| {
        //             let sync_status = stats.as_ref().unwrap().sync_status;
        //             ui.label(get_sync_progress_status(sync_status));
        //             ui.spinner();
        //         });
        //     } else {
        //         if node.stop_state.is_stopped() {
        //             ui.label("Stopped");
        //         } else {
        //             ui.label(get_sync_progress_status(SyncStatus::Initial));
        //         }
        //     }
        // });
        if ui.button("stop").clicked() {
            node.stop();
        }

        if ui.button("re-start").clicked() {
            node.restart(ChainTypes::Mainnet);
        }

        if ui.button("start").clicked() {
            node.start(ChainTypes::Mainnet);
        }
    }

    fn title(&self) -> &String {
        &self.title
    }
}