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




use std::collections::BTreeSet;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::screens::{Accounts, Screen, ScreenId};

pub struct Navigator {
    stack: BTreeSet<ScreenId>,
    screens: Vec<Box<dyn Screen>>,
}

impl Navigator {
    pub fn new(screens: Vec<Box<dyn Screen>>) -> Self {
        let mut stack = BTreeSet::new();
        stack.insert(ScreenId::Accounts);
        Self { stack, screens }
    }

    pub fn to(&mut self, id: ScreenId) {
        self.stack.insert(id);
    }

    pub fn back(&mut self) {
        self.stack.pop_last();
    }

    pub fn get_current_screen(&mut self) -> Option<&Box<dyn Screen>> {
        let Self { stack, screens } = self;
        let current = stack.last().unwrap();
        let mut result = screens.get(0);
        for screen in screens.iter() {
            if screen.id() == *current {
                result = Some(screen);
                break;
            }
        }
        return result;
    }
}