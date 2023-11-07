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

use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::Modal;

/// Title type, can be single or dual title in the row.
pub enum TitleType {
    /// Single title content.
    Single(TitleContentType),
    /// Dual title content, will align first content for default panel size width.
    Dual(TitleContentType, TitleContentType),
}

/// Title content type, can be single title or with animated subtitle.
pub enum TitleContentType {
    /// Single text.
    Title(String),
    /// With optionally animated subtitle text.
    WithSubTitle(String, String, bool)
}

/// Position of [`Modal`] on the screen.
pub enum ModalPosition {
    CenterTop,
    Center
}

/// Global [`Modal`] state.
#[derive(Default)]
pub struct ModalState {
    pub modal: Option<Modal>
}

/// Contains identifiers to draw opened [`Modal`] content for current ui container.
pub trait ModalContainer {
    /// List of allowed [`Modal`] identifiers.
    fn modal_ids(&self) -> &Vec<&'static str>;

    /// Draw modal ui content.
    fn modal_ui(&mut self,
                ui: &mut egui::Ui,
                frame: &mut eframe::Frame,
                modal: &Modal,
                cb: &dyn PlatformCallbacks);

    /// Draw [`Modal`] for current ui container if it's possible.
    fn current_modal_ui(&mut self,
                        ui: &mut egui::Ui,
                        frame: &mut eframe::Frame,
                        cb: &dyn PlatformCallbacks) {
        let modal_id = Modal::opened();
        let draw = modal_id.is_some() && self.modal_ids().contains(&modal_id.unwrap());
        if draw {
            Modal::ui(ui.ctx(), |ui, modal| {
                self.modal_ui(ui, frame, modal, cb);
            });
        }
    }
}

/// Options for [`egui::TextEdit`] view.
pub struct TextEditOptions {
    /// View identifier.
    pub id: egui::Id,
    /// Flag to check if horizontal centering is needed.
    pub h_center: bool,
    /// Flag to check if initial focus on field is needed.
    pub focus: bool,
    /// Flag to hide letters and draw button to show/hide letters.
    pub password: bool,
    /// Flag to show copy button.
    pub copy: bool,
    /// Flag to show paste button.
    pub paste: bool
}

impl TextEditOptions {
    pub fn new(id: egui::Id) -> Self {
        Self {
            id,
            h_center: false,
            focus: true,
            password: false,
            copy: false,
            paste: false,
        }
    }

    /// Center text horizontally.
    pub fn h_center(mut self) -> Self {
        self.h_center = true;
        self
    }

    /// Disable initial focus.
    pub fn no_focus(mut self) -> Self {
        self.focus = false;
        self
    }

    /// Hide letters and draw button to show/hide letters.
    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }

    /// Show button to copy text.
    pub fn copy(mut self) -> Self {
        self.copy = true;
        self
    }

    /// Show button to paste text.
    pub fn paste(mut self) -> Self {
        self.paste = true;
        self
    }
}