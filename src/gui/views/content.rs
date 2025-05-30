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

use egui::os::OperatingSystem;
use egui::RichText;
use lazy_static::lazy_static;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::gui::icons::FILE_X;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::network::NetworkContent;
use crate::gui::views::types::{ContentContainer, ModalPosition};
use crate::gui::views::wallets::WalletsContent;
use crate::gui::views::{Modal, View};
use crate::gui::Colors;
use crate::node::Node;
use crate::{AppConfig, Settings};

lazy_static! {
    /// Global state to check if [`NetworkContent`] panel is open.
    static ref NETWORK_PANEL_OPEN: AtomicBool = AtomicBool::new(false);
}

/// Contains main ui content, handles side panel state.
pub struct Content {
    /// Side panel [`NetworkContent`] content.
    network: NetworkContent,

    /// Central panel [`WalletsContent`] content.
    wallets: WalletsContent,

    /// Check if app exit is allowed on Desktop close event.
    pub exit_allowed: bool,
    /// Flag to show exit progress at [`Modal`].
    show_exit_progress: bool,

    /// Flag to check it's first draw of content.
    first_draw: bool,
}

impl Default for Content {
    fn default() -> Self {
        // Exit from eframe only for non-mobile platforms.
        let os = OperatingSystem::from_target_os();
        let exit_allowed = os == OperatingSystem::Android || os == OperatingSystem::IOS;
        Self {
            network: NetworkContent::default(),
            wallets: WalletsContent::default(),
            exit_allowed,
            show_exit_progress: false,
            first_draw: true,
        }
    }
}

/// Identifier for integrated node warning [`Modal`] on Android.
const ANDROID_INTEGRATED_NODE_WARNING_MODAL: &'static str = "android_node_warning_modal";
/// Identifier for crash report [`Modal`].
const CRASH_REPORT_MODAL: &'static str = "crash_report_modal";

impl ContentContainer for Content {
    fn modal_ids(&self) -> Vec<&'static str> {
        vec![
            Self::EXIT_CONFIRMATION_MODAL,
            ANDROID_INTEGRATED_NODE_WARNING_MODAL,
            CRASH_REPORT_MODAL
        ]
    }

    fn modal_ui(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        match modal.id {
            Self::EXIT_CONFIRMATION_MODAL => self.exit_modal_content(ui, modal, cb),
            ANDROID_INTEGRATED_NODE_WARNING_MODAL => self.android_warning_modal_ui(ui),
            CRASH_REPORT_MODAL => self.crash_report_modal_ui(ui, cb),
            _ => {}
        }
    }

    fn on_back(&mut self, cb: &dyn PlatformCallbacks) -> bool {
        if Modal::on_back() {
            if self.wallets.on_back(cb) {
                Self::show_exit_modal();
                return false;
            }
        }
        true
    }

    fn container_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        self.content_ui(ui, cb);
    }
}

impl Content {
    /// Default width of side panel at application UI.
    pub const SIDE_PANEL_WIDTH: f32 = 400.0;
    /// Desktop window title height.
    pub const WINDOW_TITLE_HEIGHT: f32 = 38.0;
    /// Margin of window frame at desktop.
    pub const WINDOW_FRAME_MARGIN: f32 = 6.0;

    /// Identifier for exit confirmation [`Modal`].
    pub const EXIT_CONFIRMATION_MODAL: &'static str = "exit_confirmation_modal";

    fn content_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        let dual_panel = Self::is_dual_panel_mode(ui.ctx());
        let (is_panel_open, mut panel_width) = network_panel_state_width(ui.ctx(), dual_panel);
        if self.network.showing_settings() {
            panel_width = ui.available_width();
        }

        // Show network content.
        egui::SidePanel::left("network_panel")
            .resizable(false)
            .exact_width(panel_width)
            .frame(egui::Frame {
                ..Default::default()
            })
            .show_animated_inside(ui, is_panel_open, |ui| {
                self.network.ui(ui, cb);
            });

        // Show wallets content.
        egui::CentralPanel::default()
            .frame(egui::Frame {
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                self.wallets.ui(ui, cb);
            });

        if self.first_draw {
            // Show crash report or integrated node Android warning.
            if Settings::crash_report_path().exists() {
                Modal::new(CRASH_REPORT_MODAL)
                    .closeable(false)
                    .position(ModalPosition::Center)
                    .title(t!("crash_report"))
                    .show();
            } else if OperatingSystem::from_target_os() == OperatingSystem::Android &&
                    AppConfig::android_integrated_node_warning_needed() {
                    Modal::new(ANDROID_INTEGRATED_NODE_WARNING_MODAL)
                        .title(t!("network.node"))
                        .show();
            }
            self.first_draw = false;
        }
    }

    /// Check if ui can show [`NetworkContent`] and [`WalletsContent`] at same time.
    pub fn is_dual_panel_mode(ctx: &egui::Context) -> bool {
        let (w, h) = View::window_size(ctx);
        // Screen is wide if width is greater than height or just 20% smaller.
        let is_wide_screen = w > h || w + (w * 0.2) >= h;
        // Dual panel mode is available when window is wide and its width is at least 2 times
        // greater than minimal width of the side panel plus display insets from both sides.
        let side_insets = View::get_left_inset() + View::get_right_inset();
        is_wide_screen && w >= (Self::SIDE_PANEL_WIDTH * 2.0) + side_insets
    }

    /// Toggle [`NetworkContent`] panel state.
    pub fn toggle_network_panel() {
        let is_open = NETWORK_PANEL_OPEN.load(Ordering::Relaxed);
        NETWORK_PANEL_OPEN.store(!is_open, Ordering::Relaxed);
    }

    /// Check if [`NetworkContent`] panel is open.
    pub fn is_network_panel_open() -> bool {
        NETWORK_PANEL_OPEN.load(Ordering::Relaxed)
    }

    /// Show exit confirmation [`Modal`].
    pub fn show_exit_modal() {
        Modal::new(Self::EXIT_CONFIRMATION_MODAL)
            .title(t!("confirmation"))
            .show();
    }

    /// Draw exit confirmation modal content.
    fn exit_modal_content(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        if self.show_exit_progress {
            if !Node::is_running() {
                self.exit_allowed = true;
                cb.exit();
                Modal::close();
            }
            ui.add_space(16.0);
            ui.vertical_centered(|ui| {
                View::small_loading_spinner(ui);
                ui.add_space(12.0);
                ui.label(RichText::new(t!("sync_status.shutdown"))
                    .size(17.0)
                    .color(Colors::text(false)));
            });
            ui.add_space(10.0);
        } else {
            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(t!("modal_exit.description"))
                    .size(17.0)
                    .color(Colors::text(false)));
            });
            ui.add_space(10.0);

            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                        Modal::close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button_ui(ui, t!("modal_exit.exit"), Colors::white_or_black(false), |_| {
                        if !Node::is_running() {
                            self.exit_allowed = true;
                            cb.exit();
                            Modal::close();
                        } else {
                            Node::stop(true);
                            modal.disable_closing();
                            Modal::set_title(t!("modal_exit.exit"));
                            self.show_exit_progress = true;
                        }
                    });
                });
            });
            ui.add_space(6.0);
        }
    }

    /// Draw content for integrated node warning [`Modal`] on Android.
    fn android_warning_modal_ui(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network.android_warning"))
                .size(16.0)
                .color(Colors::text(false)));
        });
        ui.add_space(8.0);
        ui.vertical_centered_justified(|ui| {
            View::button(ui, t!("continue"), Colors::white_or_black(false), || {
                AppConfig::show_android_integrated_node_warning();
                Modal::close();
            });
        });
        ui.add_space(6.0);
    }

    /// Draw content for integrated node warning [`Modal`] on Android.
    fn crash_report_modal_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("crash_report_warning"))
                .size(16.0)
                .color(Colors::text(false)));
            ui.add_space(6.0);
            // Draw button to share crash report.
            let text = format!("{} {}", FILE_X, t!("share"));
            View::colored_text_button(ui, text, Colors::blue(), Colors::white_or_black(false), || {
                if let Ok(data) = fs::read_to_string(Settings::crash_report_path()) {
                    let name = Settings::CRASH_REPORT_FILE_NAME.to_string();
                    let _ = cb.share_data(name, data.as_bytes().to_vec());
                }
                Settings::delete_crash_report();
                Modal::close();
            });
        });
        ui.add_space(8.0);
        View::horizontal_line(ui, Colors::item_stroke());
        ui.add_space(8.0);
        ui.vertical_centered_justified(|ui| {
            View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                Settings::delete_crash_report();
                Modal::close();
            });
        });
        ui.add_space(6.0);
    }
}

/// Get [`NetworkContent`] panel state and width.
fn network_panel_state_width(ctx: &egui::Context, dual_panel: bool) -> (bool, f32) {
    let is_panel_open = dual_panel || Content::is_network_panel_open();
    let panel_width = if dual_panel {
        Content::SIDE_PANEL_WIDTH + View::get_left_inset()
    } else {
        let is_fullscreen = ctx.input(|i| {
            i.viewport().fullscreen.unwrap_or(false)
        });
        View::window_size(ctx).0 - if View::is_desktop() && !is_fullscreen &&
            OperatingSystem::from_target_os() != OperatingSystem::Mac {
            Content::WINDOW_FRAME_MARGIN * 2.0
        } else {
            0.0
        }
    };
    (is_panel_open, panel_width)
}