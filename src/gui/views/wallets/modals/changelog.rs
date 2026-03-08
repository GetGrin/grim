// Copyright 2026 The Grim Developers
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

use egui::scroll_area::ScrollBarVisibility;
use egui::{Id, OpenUrl, RichText, ScrollArea};

use crate::gui::views::{Modal, View};
use crate::gui::Colors;
use crate::gui::icons::{BRACKETS_CURLY, GITHUB_LOGO, TELEGRAM_LOGO};

/// Application release changelog content.
pub struct ChangelogContent {
    /// Changelog text.
    changelog: String,
}

/// Endpoint for GitHub repository.
const GITHUB_URL: &'static str = "https://github.com/GetGrin/grim";
/// Endpoint for Telegram releases channel.
const TELEGRAM_URL: &'static str = "https://t.me/grim_releases";
/// Endpoint for git repository.
const GIT_URL: &'static str = "https://code.gri.mw/GUI/grim";

impl ChangelogContent {
    /// Create new content instance.
    pub fn new(changelog: String) -> Self {
        Self { changelog }
    }

    /// Identifier for [`Modal`].
    pub const MODAL_ID: &'static str = "release_changelog_modal";

    /// Draw changelog [`Modal`] content.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("changelog")).size(16.0).color(Colors::gray()));
        });
        ui.add_space(6.0);

        // Show changelog text.
        ui.vertical_centered(|ui| {
            let scroll_id = Id::from("release_changelog");
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(3.0);
            ScrollArea::vertical()
                .id_salt(scroll_id)
                .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                .max_height(128.0)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.add_space(7.0);
                    let input_id = scroll_id.with("_input");
                    egui::TextEdit::multiline(&mut self.changelog)
                        .id(input_id)
                        .font(egui::TextStyle::Small)
                        .desired_rows(5)
                        .interactive(false)
                        .desired_width(f32::INFINITY)
                        .show(ui);
                    ui.add_space(6.0);
                });
        });

        ui.add_space(2.0);
        View::horizontal_line(ui, Colors::item_stroke());
        ui.add_space(8.0);

        // Setup spacing between buttons.
        ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

        ui.columns(3, |columns| {
            columns[0].vertical_centered_justified(|ui| {
                // Draw button to open GitHub link.
                let mut github_clicked = false;
                View::button(ui, GITHUB_LOGO, Colors::white_or_black(false), || {
                    github_clicked = true;
                });
                if github_clicked {
                    ui.ctx().open_url(OpenUrl {
                        url: GITHUB_URL.into(),
                        new_tab: true,
                    });
                }
            });
            columns[1].vertical_centered_justified(|ui| {
                // Draw button to open Telegram link.
                let mut tg_clicked = false;
                View::button(ui, TELEGRAM_LOGO, Colors::white_or_black(false), || {
                    tg_clicked = true;
                });
                if tg_clicked {
                    ui.ctx().open_url(OpenUrl {
                        url: TELEGRAM_URL.into(),
                        new_tab: true,
                    });
                }
            });
            columns[2].vertical_centered_justified(|ui| {
                // Draw button to open repository link.
                let mut git_clicked = false;
                View::button(ui, BRACKETS_CURLY, Colors::white_or_black(false), || {
                    git_clicked = true;
                });
                if git_clicked {
                    ui.ctx().open_url(OpenUrl {
                        url: GIT_URL.into(),
                        new_tab: true,
                    });
                }
            });
        });

        ui.add_space(8.0);
        View::horizontal_line(ui, Colors::item_stroke());
        ui.add_space(8.0);

        // Show button to close modal.
        ui.vertical_centered_justified(|ui| {
            View::button(ui, t!("close"), Colors::white_or_black(false), || {
                Modal::close();
            });
        });
        ui.add_space(6.0);
    }
}