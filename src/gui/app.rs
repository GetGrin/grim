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

use egui::{Color32, Context, RichText, Spinner, Stroke, Widget};
use egui::os::OperatingSystem;
use egui::style::Margin;

use crate::gui::{Colors, Navigator};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::screens::Root;
use crate::gui::views::{Modal, ModalId, ModalLocation, View};
use crate::node::Node;

pub struct PlatformApp<Platform> {
    pub(crate) app: App,
    pub(crate) platform: Platform,
}

#[derive(Default)]
pub struct App {
    root: Root,
    show_exit_progress: bool
}

impl App {
    pub fn ui(&mut self, ctx: &Context, frame: &mut eframe::Frame, cb: &dyn PlatformCallbacks) {
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: Colors::FILL,
                .. Default::default()
            })
            .show(ctx, |ui| {
                if Navigator::is_modal_open(ModalLocation::Global) {
                    self.show_global_modal(ui, frame, cb);
                }
                self.root.ui(ui, frame, cb);
            });
    }

    fn show_global_modal(&mut self,
                         ui: &mut egui::Ui,
                         frame: &mut eframe::Frame,
                         cb: &dyn PlatformCallbacks) {
        let location = ModalLocation::Global;
        Navigator::modal_ui(ui, location, |ui, modal| {
            match modal.id {
                ModalId::Exit => {
                    if self.show_exit_progress {
                        if !Node::is_running() {
                            Self::exit(frame, cb);
                            modal.close();
                        }
                        ui.add_space(16.0);
                        ui.vertical_centered(|ui| {
                            Spinner::new().size(42.0).color(Colors::GRAY).ui(ui);
                            ui.add_space(10.0);
                            ui.label(RichText::new(Node::get_sync_status_text())
                                .size(18.0)
                                .color(Colors::INACTIVE_TEXT)
                            );
                        });
                        ui.add_space(12.0);
                    } else {
                        ui.add_space(8.0);
                        ui.vertical_centered(|ui| {
                            ui.label(t!("modal_exit.description"));
                        });
                        ui.add_space(10.0);
                        // Setup spacing between buttons
                        ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 0.0);
                        ui.columns(2, |columns| {
                            columns[0].vertical_centered_justified(|ui| {
                                View::button(ui, t!("modal_exit.exit"), Colors::WHITE, || {
                                    if !Node::is_running() {
                                        Self::exit(frame, cb);
                                        modal.close();
                                    } else {
                                        Node::stop();
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
                    }
                }
            }
        });
    }

    fn exit(frame: &mut eframe::Frame, cb: &dyn PlatformCallbacks) {
        match OperatingSystem::from_target_os() {
            OperatingSystem::Android => {
                cb.exit();
            }
            OperatingSystem::IOS => {
                //TODO: exit on iOS
            }
            OperatingSystem::Nix | OperatingSystem::Mac | OperatingSystem::Windows => {
                frame.close();
            }
            // Web
            OperatingSystem::Unknown => {}
        }
    }

    pub fn setup_visuals(ctx: &Context) {
        // Setup style
        let mut style = (*ctx.style()).clone();
        // Setup spacing for buttons.
        style.spacing.button_padding = egui::vec2(12.0, 8.0);
        // Make scroll-bar thinner.
        style.spacing.scroll_bar_width = 4.0;
        // Disable spacing between items.
        style.spacing.item_spacing = egui::vec2(0.0, 0.0);

        ctx.set_style(style);

        // Setup visuals
        let mut visuals = egui::Visuals::light();

        // Disable stroke around panels by default
        visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;
        ctx.set_visuals(visuals);
    }

    pub fn setup_fonts(ctx: &Context) {
        use egui::FontFamily::Proportional;

        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "phosphor".to_owned(),
            egui::FontData::from_static(include_bytes!(
                "../../fonts/phosphor.ttf"
            )).tweak(egui::FontTweak {
                scale: 1.0,
                y_offset_factor: 0.14,
                y_offset: 0.0
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
                y_offset: 0.0
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

