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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;

use lazy_static::lazy_static;

use crate::gui::screens::ScreenId;

lazy_static! {
    static ref NAVIGATOR_STATE: RwLock<Navigator> = RwLock::new(Navigator::default());
}

pub struct Navigator {
    screens_stack: BTreeSet<ScreenId>,
    side_panel_open: AtomicBool,
}

impl Default for Navigator {
    fn default() -> Self {
        Self {
            screens_stack: BTreeSet::new(),
            side_panel_open: AtomicBool::new(false)
        }
    }
}

impl Navigator {
    pub fn init(from: ScreenId) {
        let mut w_nav = NAVIGATOR_STATE.write().unwrap();
        w_nav.screens_stack.clear();
        w_nav.screens_stack.insert(from);
    }

    pub fn is_current(id: &ScreenId) -> bool {
        let r_nav = NAVIGATOR_STATE.read().unwrap();
        r_nav.screens_stack.last().unwrap() == id
    }

    pub fn to(id: ScreenId) {
        NAVIGATOR_STATE.write().unwrap().screens_stack.insert(id);
    }

    pub fn back() {
        let mut w_nav = NAVIGATOR_STATE.write().unwrap();
        if w_nav.screens_stack.len() > 1 {
            w_nav.screens_stack.pop_last();
        } else {

        }
    }

    pub fn toggle_side_panel() {
        let w_nav = NAVIGATOR_STATE.write().unwrap();
        w_nav.side_panel_open.store(
            !w_nav.side_panel_open.load(Ordering::Relaxed),
            Ordering::Relaxed
        );
    }

    pub fn is_side_panel_open() -> bool {
        let r_nav = NAVIGATOR_STATE.read().unwrap();
        r_nav.side_panel_open.load(Ordering::Relaxed)
    }
}
