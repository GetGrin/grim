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
use crate::gui::icons::{GLOBE, PLUS};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Root, TitleAction, TitlePanel, View};

/// Accounts content.
pub struct Accounts {
    /// List of accounts.
    list: Vec<String>
}

impl Default for Accounts {
    fn default() -> Self {
        Self {
            list: vec![],
        }
    }
}

impl Accounts {
    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, cb: &dyn PlatformCallbacks) {
        TitlePanel::ui(t!("accounts.title"), if !Root::is_dual_panel_mode(frame) {
            TitleAction::new(GLOBE, || {
                Root::toggle_side_panel();
            })
        } else {
            None
        }, TitleAction::new(PLUS, || {
            //TODO: add account
        }), ui);

        egui::CentralPanel::default()
            .frame(egui::Frame {
                stroke: View::DEFAULT_STROKE,
                fill: Colors::FILL_DARK,
                ..Default::default()
            })
            .show_inside(ui, |ui| {
               //TODO: accounts list
            });
    }
}