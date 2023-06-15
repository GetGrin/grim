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
use crate::gui::views::{Modal, ModalId, ModalLocation};

lazy_static! {
    /// Static [Navigator] state to be accessible from anywhere.
    static ref NAVIGATOR_STATE: RwLock<Navigator> = RwLock::new(Navigator::default());
}

/// Logic of navigation for UI, stores screen identifiers stack, open modals and side panel state.
pub struct Navigator {
    /// Screen identifiers in navigation stack.
    screen_stack: BTreeSet<ScreenId>,
    /// Indicator if side panel is open.
    side_panel_open: AtomicBool,
    /// Modal state to show globally above panel and screen.
    global_modal: Option<Modal>,
    /// Modal state to show on the side panel.
    side_panel_modal: Option<Modal>,
    /// Modal state to show on the screen.
    screen_modal: Option<Modal>,
}

impl Default for Navigator {
    fn default() -> Self {
        Self {
            screen_stack: BTreeSet::new(),
            side_panel_open: AtomicBool::new(false),
            global_modal: None,
            side_panel_modal: None,
            screen_modal: None,
        }
    }
}

impl Navigator {
    /// Initialize navigation from provided [ScreenId].
    pub fn init(from: ScreenId) {
        let mut w_nav = NAVIGATOR_STATE.write().unwrap();
        w_nav.screen_stack.clear();
        w_nav.screen_stack.insert(from);
    }

    /// Check if provided [ScreenId] is current.
    pub fn is_current(id: &ScreenId) -> bool {
        let r_nav = NAVIGATOR_STATE.read().unwrap();
        r_nav.screen_stack.last().unwrap() == id
    }

    /// Navigate to screen with provided [ScreenId].
    pub fn to(id: ScreenId) {
        NAVIGATOR_STATE.write().unwrap().screen_stack.insert(id);
    }

    /// Go back at navigation stack, close showing modals first.
    pub fn back() {
        let mut w_nav = NAVIGATOR_STATE.write().unwrap();

        // If global Modal is showing and closeable, remove it from Navigator.
        if w_nav.global_modal.is_some() {
            let global_modal = w_nav.global_modal.as_ref().unwrap();
            if global_modal.is_closeable() {
                w_nav.global_modal = None;
            }
            return;
        }

        // If side panel Modal is showing and closeable, remove it from Navigator.
        if w_nav.side_panel_modal.is_some() {
            let side_panel_modal = w_nav.side_panel_modal.as_ref().unwrap();
            if side_panel_modal.is_closeable() {
                w_nav.side_panel_modal = None;
            }
            return;
        }

        // If screen Modal is showing and closeable, remove it from Navigator.
        if w_nav.screen_modal.is_some() {
            let screen_modal = w_nav.screen_modal.as_ref().unwrap();
            if screen_modal.is_closeable() {
                w_nav.screen_modal = None;
            }
            return;
        }

        // Go back at screen stack or show exit confirmation Modal.
        if w_nav.screen_stack.len() > 1 {
            w_nav.screen_stack.pop_last();
        } else {
            Self::open_exit_modal_nav(w_nav);
        }
    }

    /// Open exit confirmation [Modal].
    pub fn open_exit_modal() {
        let w_nav = NAVIGATOR_STATE.write().unwrap();
        Self::open_exit_modal_nav(w_nav);
    }

    /// Open exit confirmation [Modal] with provided [NAVIGATOR_STATE] lock.
    fn open_exit_modal_nav(mut w_nav: RwLockWriteGuard<Navigator>) {
        let m = Modal::new(ModalId::Exit, ModalLocation::Global).title(t!("modal_exit.exit"));
        w_nav.global_modal = Some(m);
    }

    /// Open [Modal] at specified location.
    pub fn open_modal(modal: Modal) {
        let mut w_nav = NAVIGATOR_STATE.write().unwrap();
        match modal.location {
            ModalLocation::Global => {
                w_nav.global_modal = Some(modal);
            }
            ModalLocation::SidePanel => {
                w_nav.side_panel_modal = Some(modal);
            }
            ModalLocation::Screen => {
                w_nav.screen_modal = Some(modal);
            }
        }
    }

    /// Check if [Modal] is open at specified location and remove it from [Navigator] if closed.
    pub fn is_modal_open(location: ModalLocation) -> bool {
        // Check if Modal is showing.
        {
            let r_nav = NAVIGATOR_STATE.read().unwrap();
            let showing = match location {
                ModalLocation::Global => { r_nav.global_modal.is_some() }
                ModalLocation::SidePanel => { r_nav.side_panel_modal.is_some() }
                ModalLocation::Screen => { r_nav.screen_modal.is_some() }
            };
            if !showing {
                return false;
            }
        }

        // Check if Modal is open.
        let is_open = {
            let r_nav = NAVIGATOR_STATE.read().unwrap();
            match location {
                ModalLocation::Global => { r_nav.global_modal.as_ref().unwrap().is_open() }
                ModalLocation::SidePanel => { r_nav.side_panel_modal.as_ref().unwrap().is_open() }
                ModalLocation::Screen => {r_nav.screen_modal.as_ref().unwrap().is_open() }
            }
        };

        // If Modal is not open, remove it from navigator state.
        if !is_open {
            let mut w_nav = NAVIGATOR_STATE.write().unwrap();
            match location {
                ModalLocation::Global => { w_nav.global_modal = None }
                ModalLocation::SidePanel => { w_nav.side_panel_modal = None }
                ModalLocation::Screen => { w_nav.screen_modal = None }
            }
            return false;
        }
        true
    }

    /// Show [Modal] with provided location at app UI.
    pub fn modal_ui(ui: &mut egui::Ui,
                    location: ModalLocation,
                    add_content: impl FnOnce(&mut egui::Ui, &Modal)) {
        let r_nav = NAVIGATOR_STATE.read().unwrap();
        let modal = match location {
            ModalLocation::Global => { &r_nav.global_modal }
            ModalLocation::SidePanel => { &r_nav.side_panel_modal }
            ModalLocation::Screen => { &r_nav.screen_modal }
        };
        if modal.is_some() {
            modal.as_ref().unwrap().ui(ui, add_content);
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
