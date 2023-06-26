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
use std::sync::{RwLock, RwLockWriteGuard};
use std::sync::atomic::{AtomicBool, Ordering};

use lazy_static::lazy_static;

use crate::gui::screens::ScreenId;
use crate::gui::views::Modal;

lazy_static! {
    /// Static [`Navigator`] state to be accessible from anywhere.
    static ref NAVIGATOR_STATE: RwLock<Navigator> = RwLock::new(Navigator::default());
}

/// Logic of navigation at ui, stores screen identifiers stack, showing modal and side panel state.
pub struct Navigator {
    /// Screen identifiers in navigation stack.
    screen_stack: BTreeSet<ScreenId>,
    /// Indicator if side panel is open.
    side_panel_open: AtomicBool,
    /// Modal window to show.
    modal: Option<Modal>,
}

impl Default for Navigator {
    fn default() -> Self {
        Self {
            screen_stack: BTreeSet::new(),
            side_panel_open: AtomicBool::new(false),
            modal: None,
        }
    }
}

impl Navigator {
    /// Identifier for exit [`Modal`].
    pub const EXIT_MODAL: &'static str = "exit";

    /// Initialize navigation from provided [`ScreenId`].
    pub fn init(from: ScreenId) {
        let mut w_nav = NAVIGATOR_STATE.write().unwrap();
        w_nav.screen_stack.clear();
        w_nav.screen_stack.insert(from);
    }

    /// Check if provided [`ScreenId`] is current.
    pub fn is_current(id: &ScreenId) -> bool {
        let r_nav = NAVIGATOR_STATE.read().unwrap();
        r_nav.screen_stack.last().unwrap() == id
    }

    /// Navigate to screen with provided [`ScreenId`].
    pub fn to(id: ScreenId) {
        NAVIGATOR_STATE.write().unwrap().screen_stack.insert(id);
    }

    /// Go back at navigation stack, close showing modals first.
    pub fn back() {
        let mut w_nav = NAVIGATOR_STATE.write().unwrap();

        // If Modal is showing and closeable, remove it from Navigator.
        if w_nav.modal.is_some() {
            let modal = w_nav.modal.as_ref().unwrap();
            if modal.is_closeable() {
                w_nav.modal = None;
            }
            return;
        }

        // Go back at screen stack or set exit confirmation Modal.
        if w_nav.screen_stack.len() > 1 {
            w_nav.screen_stack.pop_last();
        } else {
            Self::show_exit_modal_nav(w_nav);
        }
    }

    /// Set exit confirmation [`Modal`].
    pub fn show_exit_modal() {
        let w_nav = NAVIGATOR_STATE.write().unwrap();
        Self::show_exit_modal_nav(w_nav);
    }

    /// Set exit confirmation [`Modal`] with provided [NAVIGATOR_STATE] lock.
    fn show_exit_modal_nav(mut w_nav: RwLockWriteGuard<Navigator>) {
        let m = Modal::new(Self::EXIT_MODAL).title(t!("modal_exit.exit"));
        w_nav.modal = Some(m);
    }

    /// Set [`Modal`] to show.
    pub fn show_modal(modal: Modal) {
        let mut w_nav = NAVIGATOR_STATE.write().unwrap();
        w_nav.modal = Some(modal);
    }

    /// Check if [`Modal`] is open by returning its id, remove it from [`Navigator`] if it's closed.
    pub fn is_modal_open() -> Option<&'static str> {
        // Check if Modal is showing.
        {
            if NAVIGATOR_STATE.read().unwrap().modal.is_none() {
                return None;
            }
        }

        // Check if Modal is open.
        let (is_open, id) = {
            let r_nav = NAVIGATOR_STATE.read().unwrap();
            let modal = r_nav.modal.as_ref().unwrap();
            (modal.is_open(), modal.id)
        };

        // If Modal is not open, remove it from navigator state.
        if !is_open {
            let mut w_nav = NAVIGATOR_STATE.write().unwrap();
            w_nav.modal = None;
            return None;
        }
        Some(id)
    }

    /// Draw showing [`Modal`] content if it's opened.
    pub fn modal_ui(ui: &mut egui::Ui, add_content: impl FnOnce(&mut egui::Ui, &Modal)) {
        if let Some(modal) = &NAVIGATOR_STATE.read().unwrap().modal {
            if modal.is_open() {
                modal.ui(ui, add_content);
            }
        }
    }

    /// Change state of side panel to opposite.
    pub fn toggle_side_panel() {
        let r_nav = NAVIGATOR_STATE.read().unwrap();
        let is_open = r_nav.side_panel_open.load(Ordering::Relaxed);
        r_nav.side_panel_open.store(!is_open, Ordering::Relaxed);
    }

    /// Check if side panel is open.
    pub fn is_side_panel_open() -> bool {
        let r_nav = NAVIGATOR_STATE.read().unwrap();
        r_nav.side_panel_open.load(Ordering::Relaxed)
    }
}
