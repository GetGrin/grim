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

use std::cmp::min;
use std::sync::atomic::{AtomicBool, Ordering};
use egui::os::OperatingSystem;
use egui::RichText;

use lazy_static::lazy_static;
use crate::gui::app::{get_left_display_cutout, get_right_display_cutout};
use crate::gui::Colors;

use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Accounts, Modal, ModalContainer, Network, View};
use crate::node::Node;

lazy_static! {
    /// To check if side panel is open from any part of ui.
    static ref SIDE_PANEL_OPEN: AtomicBool = AtomicBool::new(false);
}

/// Contains main ui content, handles side panel state.
pub struct Root {
    /// Side panel content.
    side_panel: Network,
    /// Central panel content.
    central_content: Accounts,

    /// Check if app exit is allowed on close event of [`eframe::App`] platform implementation.
    pub(crate) exit_allowed: bool,

    /// Flag to show exit progress at [`Modal`].
    show_exit_progress: bool,

    /// List of allowed [`Modal`] ids for this [`ModalContainer`].
    allowed_modal_ids: Vec<&'static str>
}

impl Default for Root {
    fn default() -> Self {
        // Exit from eframe only for non-mobile platforms.
        let os = OperatingSystem::from_target_os();
        let exit_allowed = os == OperatingSystem::Android || os == OperatingSystem::IOS;
        Self {
            side_panel: Network::default(),
            central_content: Accounts::default(),
            exit_allowed,
            show_exit_progress: false,
            allowed_modal_ids: vec![
                Self::EXIT_MODAL_ID
            ],
        }
    }
}

impl ModalContainer for Root {
    fn modal_ids(&self) -> &Vec<&'static str> {
        &self.allowed_modal_ids
    }
}

impl Root {
    /// Identifier for exit confirmation [`Modal`].
    pub const EXIT_MODAL_ID: &'static str = "exit_confirmation";

    /// Default width of side panel at application UI.
    pub const SIDE_PANEL_MIN_WIDTH: i64 = 400;

    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, cb: &dyn PlatformCallbacks) {
        // Show opened exit confirmation Modal content.
        if self.can_draw_modal() {
            self.exit_modal_content(ui, frame, cb);
        }

        // Show network content on side panel.
        let (is_panel_open, panel_width) = Self::side_panel_state_width(frame);
        egui::SidePanel::left("network_panel")
            .resizable(false)
            .exact_width(panel_width)
            .frame(egui::Frame::none())
            .show_animated_inside(ui, is_panel_open, |ui| {
                self.side_panel.ui(ui, frame, cb);
            });

        // Show accounts content on central panel.
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show_inside(ui, |ui| {
                self.central_content.ui(ui, frame, cb);
            });
    }

    /// Get side panel state and width.
    fn side_panel_state_width(frame: &mut eframe::Frame) -> (bool, f32) {
        let dual_panel_mode = Self::is_dual_panel_mode(frame);
        let is_panel_open = dual_panel_mode || Self::is_side_panel_open();
        let side_cutouts = get_left_display_cutout() + get_right_display_cutout();
        let panel_width = if dual_panel_mode {
            let available_width = (frame.info().window_info.size.x - side_cutouts) as i64;
            min(available_width, Self::SIDE_PANEL_MIN_WIDTH) as f32
        } else {
            frame.info().window_info.size.x - side_cutouts
        };
        (is_panel_open, panel_width)
    }

    /// Check if ui can show [`Network`] and [`Accounts`] at same time.
    pub fn is_dual_panel_mode(frame: &mut eframe::Frame) -> bool {
        let w = frame.info().window_info.size.x;
        let h = frame.info().window_info.size.y;
        // Screen is wide if width is greater than height or just 20% smaller.
        let is_wide_screen = w > h || w + (w * 0.2) >= h;
        // Dual panel mode is available when window is wide and its width is at least 2 times
        // greater than minimal width of the side panel plus display cutouts from both sides.
        let side_cutouts = get_left_display_cutout() + get_right_display_cutout();
        is_wide_screen && w >= (Self::SIDE_PANEL_MIN_WIDTH as f32 * 2.0) + side_cutouts
    }

    /// Toggle [`Network`] panel state.
    pub fn toggle_side_panel() {
        let is_open = SIDE_PANEL_OPEN.load(Ordering::Relaxed);
        SIDE_PANEL_OPEN.store(!is_open, Ordering::Relaxed);
    }

    /// Check if side panel is open.
    pub fn is_side_panel_open() -> bool {
        SIDE_PANEL_OPEN.load(Ordering::Relaxed)
    }

    /// Show exit confirmation modal.
    pub fn show_exit_modal() {
        let exit_modal = Modal::new(Self::EXIT_MODAL_ID).title(t!("modal_exit.exit"));
        Modal::show(exit_modal);
    }

    /// Draw exit confirmation modal content.
    fn exit_modal_content(&mut self,
                          ui: &mut egui::Ui,
                          frame: &mut eframe::Frame,
                          cb: &dyn PlatformCallbacks) {
        Modal::ui(ui, |ui, modal| {
            if self.show_exit_progress {
                if !Node::is_running() {
                    self.exit(frame, cb);
                    modal.close();
                }
                ui.add_space(16.0);
                ui.vertical_centered(|ui| {
                    View::small_loading_spinner(ui);
                    ui.add_space(12.0);
                    ui.label(RichText::new(t!("sync_status.shutdown"))
                        .size(18.0)
                        .color(Colors::TEXT));
                });
                ui.add_space(10.0);
            } else {
                ui.add_space(8.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(t!("modal_exit.description"))
                        .size(18.0)
                        .color(Colors::TEXT));
                });
                ui.add_space(10.0);

                // Show modal buttons.
                ui.scope(|ui| {
                    // Setup spacing between buttons.
                    ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 0.0);

                    ui.columns(2, |columns| {
                        columns[0].vertical_centered_justified(|ui| {
                            View::button(ui, t!("modal_exit.exit"), Colors::WHITE, || {
                                if !Node::is_running() {
                                    self.exit(frame, cb);
                                    modal.close();
                                } else {
                                    Node::stop(true);
                                    modal.disable_closing();
                                    self.show_exit_progress = true;
                                }
                            });
                        });
                        columns[1].vertical_centered_justified(|ui| {
                            View::button(ui, t!("modal.cancel"), Colors::WHITE, || {
                                modal.close();
                            });
                        });
                    });
                    ui.add_space(6.0);
                });
            }
        });
    }

    /// Platform-specific exit from the application.
    fn exit(&mut self, frame: &mut eframe::Frame, cb: &dyn PlatformCallbacks) {
        match OperatingSystem::from_target_os() {
            OperatingSystem::Android => {
                cb.exit();
            }
            OperatingSystem::IOS => {
                //TODO: exit on iOS.
            }
            OperatingSystem::Nix | OperatingSystem::Mac | OperatingSystem::Windows => {
                self.exit_allowed = true;
                frame.close();
            }
            OperatingSystem::Unknown => {}
        }
    }

    /// Handle platform-specific Back key code event.
    pub fn on_back() {
        if Modal::on_back() {
            Self::show_exit_modal()
        }
    }
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Handle Back key code event from Android.
pub extern "C" fn Java_mw_gri_android_MainActivity_onBack(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    _activity: jni::objects::JObject,
) {
    Root::on_back();
}



