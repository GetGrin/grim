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

use crate::gui::app::App;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::screens::{Account, Navigator, Screen, ScreenId};
use crate::gui::views::title_panel::TitlePanel;
use crate::gui::views::View;

#[derive(Default)]
pub struct Accounts {
    title: String,
}

impl Accounts {
    pub(crate) fn new() -> Self {
        Self {
            title: t!("accounts"),
        }
    }
}

impl Screen for Accounts {
    fn id(&self) -> ScreenId {
        ScreenId::Accounts
    }

    fn show(&mut self,
            ui: &mut egui::Ui,
            nav: Option<&mut Navigator>,
            cb: &dyn PlatformCallbacks) {
        TitlePanel::default()
            .title(self.title.to_owned())
            .ui(ui);
        if ui.button("test").clicked() {
            nav.unwrap().to(ScreenId::Account)
        };


        //TODO: content
    }

}