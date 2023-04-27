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

use crate::gui::screens::ScreenId;

pub struct Navigator {
    pub(crate) stack: BTreeSet<ScreenId>,
    pub(crate) left_panel_open: bool,
}

impl Default for Navigator {
    fn default() -> Self {
        let mut stack = BTreeSet::new();
        stack.insert(ScreenId::Accounts);
        Self {
            stack,
            left_panel_open: false
        }
    }
}

impl Navigator {
    pub fn to(&mut self, id: ScreenId) {
        self.stack.insert(id);
    }

    pub fn back(&mut self) {
        self.stack.pop_last();
    }

    pub fn toggle_left_panel(&mut self) {
        self.left_panel_open = !self.left_panel_open;
    }
}