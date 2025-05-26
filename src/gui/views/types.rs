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

/// Stroke position against content.
pub enum LinePosition {
    TOP, LEFT, RIGHT, BOTTOM
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
                modal: &Modal,
                cb: &dyn PlatformCallbacks);

    /// Draw [`Modal`] for current ui container if it's possible.
    fn current_modal_ui(&mut self,
                        ui: &mut egui::Ui,
                        cb: &dyn PlatformCallbacks) {
        let modal_id = Modal::opened();
        let draw = modal_id.is_some() && self.modal_ids().contains(&modal_id.unwrap());
        if draw {
            Modal::ui(ui.ctx(), |ui, modal| {
                self.modal_ui(ui, modal, cb);
            });
        }
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

/// QR code scanning state.
pub struct QrScanState {
    /// Flag to check if image is processing to find QR code.
    pub image_processing: bool,
    /// Processed QR code result.
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

/// QR code image data state.
pub struct QrImageState {
    /// Flag to check if QR code image is loading.
    pub loading: bool,
    /// Flag to check if QR code image is exporting.
    pub exporting: bool,

    /// Created GIF data from animated QR code.
    pub gif_data: Option<Vec<u8>>,
    /// Flag to check if GIF is creating.
    pub gif_creating: bool,

    /// Vector image data.
    pub svg: Option<Vec<u8>>,
    /// Multiple vector image data for animated QR code.
    pub svg_list: Option<Vec<Vec<u8>>>
}

impl Default for QrImageState {
    fn default() -> Self {
        Self {
            loading: false,
            exporting: false,
            gif_data: None,
            gif_creating: false,
            svg: None,
            svg_list: None,
        }
    }
}