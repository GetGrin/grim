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

use egui::Frame;

use crate::gui::icons::{ARROW_CIRCLE_LEFT, GLOBE, PLUS};
use crate::gui::{Colors, Navigator};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::screens::{Screen, ScreenId};
use crate::gui::views::{TitlePanel, TitleAction, View};

#[derive(Default)]
pub struct Accounts;

impl Screen for Accounts {
    fn id(&self) -> ScreenId {
        ScreenId::Accounts
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, cb: &dyn PlatformCallbacks) {
        TitlePanel::ui(t!("screen_accounts.title"), if !View::is_dual_panel_mode(frame) {
            TitleAction::new(GLOBE, || {
                Navigator::toggle_side_panel();
            })
        } else {
            None
        }, TitleAction::new(PLUS, || {
            //TODO: add account
        }), ui);

        egui::CentralPanel::default()
            .frame(Frame {
                stroke: View::DEFAULT_STROKE,
                fill: Colors::FILL_DARK,
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                ui.label(format!("{}Here we go 10000 ツ", ARROW_CIRCLE_LEFT));
                if ui.button("TEST").clicked() {
                    Navigator::to(ScreenId::Account)
                };
                if ui.button(format!("{}BACK ", ARROW_CIRCLE_LEFT)).clicked() {
                    Navigator::to(ScreenId::Account)
                };
            });
    }
}