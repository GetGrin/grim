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

use grin_util::ZeroingString;
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
#[derive(Clone)]
pub enum ModalPosition {
    CenterTop,
    Center
}

/// Global [`Modal`] state.
#[derive(Default)]
pub struct ModalState {
    /// Opened [`Modal`].
    pub modal: Option<Modal>,
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
    /// Check if horizontal centering is needed.
    pub h_center: bool,
    /// Check if initial focus on field is needed.
    pub focus: bool,
    /// Hide letters and draw button to show/hide letters.
    pub password: bool,
    /// Show copy button.
    pub copy: bool,
    /// Show paste button.
    pub paste: bool,
    /// Show button to scan QR code into text.
    pub scan_qr: bool,
    /// Callback when scan button was pressed.
    pub scan_pressed: bool,
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
            scan_qr: false,
            scan_pressed: false,
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

    /// Show button to scan QR code to text.
    pub fn scan_qr(mut self) -> Self {
        self.scan_qr = true;
        self.scan_pressed = false;
        self
    }
}

/// QR code scan result.
#[derive(Clone)]
pub enum QrScanResult {
    /// Slatepack message.
    Slatepack(ZeroingString),
    /// Slatepack address.
    Address(ZeroingString),
    /// Parsed text.
    Text(ZeroingString),
    /// Recovery phrase in standard or compact SeedQR format.
    /// https://github.com/SeedSigner/seedsigner/blob/dev/docs/seed_qr/README.md
    SeedQR(ZeroingString),
    /// Part of Uniform Resources as URI with current index and total messages amount.
    /// https://github.com/BlockchainCommons/Research/blob/master/papers/bcr-2020-005-ur.md
    URPart(String, usize, usize),
}

impl QrScanResult {
    /// Get text scanning result.
    pub fn text(&self) -> String {
        match self {
            QrScanResult::Slatepack(text) => text.to_string(),
            QrScanResult::Address(text) => text.to_string(),
            QrScanResult::Text(text) => text.to_string(),
            QrScanResult::SeedQR(text) => text.to_string(),
            QrScanResult::URPart(uri, _, _) => uri.to_string(),
        }
    }
}

/// QR code scan state.
pub struct QrScanState {
    // Flag to check if image is processing to find QR code.
    pub(crate) image_processing: bool,
    // Found QR code content.
    pub qr_scan_result: Option<QrScanResult>
}

impl Default for QrScanState {
    fn default() -> Self {
        Self {
            image_processing: false,
            qr_scan_result: None,
        }
    }
}

/// QR code image creation state.
pub struct QrCreationState {
    // Flag to check if QR code image is creating.
    pub creating: bool,
    // Vector image data.
    pub svg: Option<Vec<u8>>,
    // Multiple vector image data.
    pub svg_list: Option<Vec<Vec<u8>>>
}

impl Default for QrCreationState {
    fn default() -> Self {
        Self {
            creating: false,
            svg: None,
            svg_list: None,
        }
    }
}