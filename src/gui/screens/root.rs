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

use egui::Ui;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::screens::{Account, Accounts, Navigator, Screen, ScreenId};

pub struct Root {
    navigator: Navigator,
    // screens: Vec<Box<dyn Screen>>,
}

impl Default for Root {
    fn default() -> Self {
        Self {
            navigator: Navigator::new(vec![
                Box::new(Accounts::default()),
                Box::new(Account::default())
            ])
        }
    }
}

impl Screen for Root {
    fn id(&self) -> ScreenId {
        ScreenId::Root
    }

    fn show(&mut self, ui: &mut Ui, navigator: Option<&mut Navigator>, cb: &dyn PlatformCallbacks) {
        let screen = self.navigator.get_current_screen();
    }
}