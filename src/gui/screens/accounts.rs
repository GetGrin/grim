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

use crate::gui::app::is_dual_panel_mode;
use crate::gui::icons::{ARROW_CIRCLE_LEFT, GEAR_SIX, GLOBE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::screens::{Navigator, Screen, ScreenId};
use crate::gui::views::{DEFAULT_STROKE, TitlePanel, TitlePanelAction};

#[derive(Default)]
pub struct Accounts {}

impl Screen for Accounts {
    fn id(&self) -> ScreenId {
        ScreenId::Accounts
    }

    fn ui(&mut self,
          ui: &mut egui::Ui,
          frame: &mut eframe::Frame,
          nav: &mut Navigator,
          cb: &dyn PlatformCallbacks) {
        let mut panel: TitlePanel = TitlePanel::new(nav)
            .title(t!("accounts"))
            .right_action(TitlePanelAction {
                icon: GEAR_SIX.into(),
                on_click: Box::new(|nav| {
                    //TODO: open settings
                }),
            });
        if !is_dual_panel_mode(frame) {
            panel = panel.left_action(TitlePanelAction {
                icon: GLOBE.into(),
                on_click: Box::new(|nav|{
                    nav.toggle_left_panel();
                }),
            });
        }
        panel.ui(ui);

        egui::CentralPanel::default().frame(Frame {
            stroke: DEFAULT_STROKE,
            .. Default::default()
        }).show_inside(ui, |ui| {
            ui.label(format!("{}Here we go 10000 ツ", ARROW_CIRCLE_LEFT));
            if ui.button("TEST").clicked() {
                nav.to(ScreenId::Account)
            };
            if ui.button(format!("{}BACK ", ARROW_CIRCLE_LEFT)).clicked() {
                nav.to(ScreenId::Account)
            };
        });
    }
}