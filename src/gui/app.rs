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

use egui::{Context, Stroke, Widget};
use egui::os::OperatingSystem;
use egui::style::Margin;

use crate::gui::colors::COLOR_LIGHT;
use crate::gui::Navigator;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::screens::Root;
use crate::gui::views::{Modal, ModalId, ModalLocation, ProgressLoading, View};
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
        let modal_open = Navigator::is_modal_open(ModalLocation::Global);
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: COLOR_LIGHT,
                .. Default::default()
            })
            .show(ctx, |ui| {
                if modal_open {
                    self.show_global_modal(ui, frame, cb);
                }
                self.root.ui(ui, frame, cb);
            }).response.enabled = !modal_open;
    }

    fn show_global_modal(&mut self,
                         ui: &mut egui::Ui,
                         frame: &mut eframe::Frame,
                         cb: &dyn PlatformCallbacks) {
        let location = ModalLocation::Global;
        Navigator::modal_ui(ui, frame, location, |ui, frame, modal| {
            match modal.id {
                ModalId::Exit => {
                    if self.show_exit_progress {
                        if !Node::is_running() {
                            Self::exit(frame, cb);
                        } else {
                            ui.add_space(10.0);
                            let text = Node::get_sync_status_text(Node::get_sync_status());
                            ProgressLoading::new(text).ui(ui);
                            ui.add_space(10.0);
                        }
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
                                View::modal_button(ui, t!("modal_exit.exit"), || {
                                    if !Node::is_running() {
                                        Self::exit(frame, cb);
                                        modal.close();
                                    } else {
                                        modal.disable_closing();
                                        Node::stop();
                                        self.show_exit_progress = true;
                                    }
                                });
                            });
                            columns[1].vertical_centered_justified(|ui| {
                                View::modal_button(ui, t!("modal.cancel"), || {
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
}

