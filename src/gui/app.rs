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

use egui::{Context, RichText, Stroke};
use egui::os::OperatingSystem;

use crate::gui::{Colors, Navigator};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::screens::Root;
use crate::gui::views::{ModalContainer, View};
use crate::node::Node;

/// To be implemented at platform-specific module.
pub struct PlatformApp<Platform> {
    pub(crate) app: App,
    pub(crate) platform: Platform,
}

/// Contains main ui content, handles application exit and visual style setup.
pub struct App {
    /// Main ui container.
    root: Root,

    /// Check if app exit is allowed on close event callback.
    pub(crate) exit_allowed: bool,
    /// Called from callback of [`eframe::App`] platform implementation on close event.
    pub(crate) exit_requested: bool,

    /// Flag to show exit progress at modal.
    show_exit_progress: bool,
    /// List of allowed modal ids for this [`ModalContainer`].
    allowed_modal_ids: Vec<&'static str>
}

impl Default for App {
    fn default() -> Self {
        let os = OperatingSystem::from_target_os();
        // Exit from eframe only for non-mobile platforms.
        let allow_to_exit = os == OperatingSystem::Android || os == OperatingSystem::IOS;
        Self {
            root: Root::default(),
            exit_allowed: allow_to_exit,
            exit_requested: false,
            show_exit_progress: false,
            allowed_modal_ids: vec![
                Navigator::EXIT_MODAL
            ]
        }
    }
}

impl ModalContainer for App {
    fn modal_ids(&self) -> &Vec<&'static str> {
        &self.allowed_modal_ids
    }
}

impl App {
    /// Draw content on main screen panel.
    pub fn ui(&mut self, ctx: &Context, frame: &mut eframe::Frame, cb: &dyn PlatformCallbacks) {
        // Show exit modal if window closing was requested.
        if self.exit_requested {
            Navigator::show_exit_modal();
            self.exit_requested = false;
        }
        // Draw main content.
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: Colors::FILL,
                ..Default::default()
            })
            .show(ctx, |ui| {
                // Draw exit modal content if it's open or exit requested.
                let modal_id = Navigator::is_modal_open();
                if modal_id.is_some() && self.can_show_modal(modal_id.unwrap()) {
                    self.exit_modal_content(ui, frame, cb);
                }
                self.root.ui(ui, frame, cb);
            });
    }

    /// Draw exit confirmation modal content.
    fn exit_modal_content(&mut self,
                          ui: &mut egui::Ui,
                          frame: &mut eframe::Frame,
                          cb: &dyn PlatformCallbacks) {
        Navigator::modal_ui(ui, |ui, modal| {
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
                //TODO: exit on iOS
            }
            OperatingSystem::Nix | OperatingSystem::Mac | OperatingSystem::Windows => {
                self.exit_allowed = true;
                frame.close();
            }
            // Web
            OperatingSystem::Unknown => {}
        }
    }

    /// Setup application styles.
    pub fn setup_visuals(ctx: &Context) {
        let mut style = (*ctx.style()).clone();
        // Setup spacing for buttons.
        style.spacing.button_padding = egui::vec2(12.0, 8.0);
        // Make scroll-bar thinner.
        style.spacing.scroll_bar_width = 4.0;
        // Disable spacing between items.
        style.spacing.item_spacing = egui::vec2(0.0, 0.0);
        // Setup radio button/checkbox size and spacing.
        style.spacing.icon_width = 24.0;
        style.spacing.icon_width_inner = 14.0;
        style.spacing.icon_spacing = 10.0;
        // Setup style
        ctx.set_style(style);

        let mut visuals = egui::Visuals::light();
        // Setup selection color.
        visuals.selection.stroke = Stroke { width: 1.0, color: Colors::TEXT };
        visuals.selection.bg_fill = Colors::GOLD;
        // Disable stroke around panels by default
        visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;
        // Setup visuals
        ctx.set_visuals(visuals);
    }

    /// Setup application fonts.
    pub fn setup_fonts(ctx: &Context) {
        use egui::FontFamily::Proportional;

        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "phosphor".to_owned(),
            egui::FontData::from_static(include_bytes!(
                "../../fonts/phosphor.ttf"
            )).tweak(egui::FontTweak {
                scale: 1.0,
                y_offset_factor: -0.30,
                y_offset: 0.0,
                baseline_offset_factor: 0.30,
            }),
        );
        fonts
            .families
            .entry(Proportional)
            .or_default()
            .insert(0, "phosphor".to_owned());

        fonts.font_data.insert(
            "noto".to_owned(),
            egui::FontData::from_static(include_bytes!(
                "../../fonts/noto_sc_reg.otf"
            )).tweak(egui::FontTweak {
                scale: 1.0,
                y_offset_factor: -0.25,
                y_offset: 0.0,
                baseline_offset_factor: 0.17,
            }),
        );
        fonts
            .families
            .entry(Proportional)
            .or_default()
            .insert(0, "noto".to_owned());

        ctx.set_fonts(fonts);

        use egui::FontId;
        use egui::TextStyle::*;

        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (Heading, FontId::new(20.0, Proportional)),
            (Body, FontId::new(16.0, Proportional)),
            (Button, FontId::new(18.0, Proportional)),
            (Small, FontId::new(12.0, Proportional)),
            (Monospace, FontId::new(16.0, Proportional)),
        ].into();

        ctx.set_style(style);
    }
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Calling when back button is pressed on Android.
pub extern "C" fn Java_mw_gri_android_MainActivity_onBackButtonPress(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    _activity: jni::objects::JObject,
) {
    Navigator::back();
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
#[no_mangle]
/// Calling on unexpected Android application termination (removal from recent apps).
pub extern "C" fn Java_mw_gri_android_MainActivity_onTermination(
    _env: jni::JNIEnv,
    _class: jni::objects::JObject,
    _activity: jni::objects::JObject,
) {
    Node::stop(false);
}

