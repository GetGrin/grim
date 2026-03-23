// Copyright 2024 The Grim Developers
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

use crate::AppConfig;
use crate::gui::Colors;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::network::types::ShareConnection;
use crate::gui::views::{Modal, QrCodeContent, View};

/// [`Modal`] content to share connection with QR code.
pub struct ShareConnectionContent {
    /// QR code content.
    pub qr_details_content: QrCodeContent,
}

impl ShareConnectionContent {
    /// Create new content instance from connection details.
    pub fn new(details: ShareConnection) -> Result<Self, serde_json::Error> {
        let details = serde_json::to_string_pretty(&details)?;
        let c = Self {
            qr_details_content: QrCodeContent::new(details, false).hide_text().no_copy(),
        };
        Ok(c)
    }

    /// Draw QR code content.
    pub fn ui(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        let dark_theme = AppConfig::dark_theme().unwrap_or(false);
        // Set light theme for better scanning.
        AppConfig::set_dark_theme(false);
        modal.set_background_color(Colors::FILL_DEEP);
        crate::setup_visuals(ui.ctx());
        // Draw QR code content.
        ui.add_space(6.0);
        self.qr_details_content.ui(ui, cb);
        ui.vertical_centered_justified(|ui| {
            View::button(ui, t!("close"), Colors::white_or_black(false), || {
                Modal::close();
            });
        });
        ui.add_space(6.0);
        // Set color theme back.
        AppConfig::set_dark_theme(dark_theme);
        crate::setup_visuals(ui.ctx());
    }
}