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

use std::ops::{Deref, DerefMut};
use eframe::epaint::{Color32, Stroke};

use egui::{Frame, Widget};

use crate::gui::{SYM_ARROW_BACK, SYM_NETWORK, SYM_SETTINGS};
use crate::gui::app::is_dual_panel_mode;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::screens::{Navigator, Screen, ScreenId};
use crate::gui::views::{TitlePanel, TitlePanelAction};

pub struct Accounts {
    title: String,
}

impl Default for Accounts {
    fn default() -> Self {
        Self {
            title: t!("accounts"),
        }
    }
}

impl Screen for Accounts {
    fn id(&self) -> ScreenId {
        ScreenId::Accounts
    }

    fn ui(&mut self,
          ui: &mut egui::Ui,
          frame: &mut eframe::Frame,
          nav: &mut Navigator,
          cb: &dyn PlatformCallbacks) {
        let Self { title } = self;

        let mut panel: TitlePanel = TitlePanel::default()
            .title(title)
            .right_action(TitlePanelAction {
                icon: SYM_SETTINGS.into(),
                on_click: Box::new(|nav| {
                    //TODO: open settings
                }),
            })
            .with_navigator(nav);
        if !is_dual_panel_mode(frame) {
            panel = panel.left_action(TitlePanelAction {
                icon: SYM_NETWORK.into(),
                on_click: Box::new(|nav|{
                    nav.as_mut().unwrap().toggle_left_panel();
                }),
            });
        }
        panel.ui(ui);

        egui::CentralPanel::default().frame(Frame {
            stroke: Stroke::new(1.0, Color32::from_gray(190)),
            .. Default::default()
        }).show_inside(ui, |ui| {
            ui.label(format!("{}Here we go 10000 ツ", SYM_ARROW_BACK));
            if ui.button("TEST").clicked() {
                nav.to(ScreenId::Account)
            };
            if ui.button(format!("{}BACK ", SYM_ARROW_BACK)).clicked() {
                nav.to(ScreenId::Account)
            };
        });
    }
}