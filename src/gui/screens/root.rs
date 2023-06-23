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

use crate::gui::{App, Colors, Navigator};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::screens::{Account, Accounts, Screen, ScreenId};
use crate::gui::views::{ModalLocation, Network, View};
use crate::node::Node;

pub struct Root {
    screens: Vec<Box<dyn Screen>>,
    network: Network,
    show_exit_progress: bool
}

impl Default for Root {
    fn default() -> Self {
        Navigator::init(ScreenId::Accounts);

        Self {
            screens: (vec![
                Box::new(Accounts::default()),
                Box::new(Account::default())
            ]),
            network: Network::default(),
            show_exit_progress: false
        }
    }
}

impl Root {
    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, cb: &dyn PlatformCallbacks) {
        if Navigator::is_modal_open(ModalLocation::Global) {
            self.show_global_modal(ui, frame, cb);
        }

        let (is_panel_open, panel_width) = self.dual_panel_state_width(frame);
        egui::SidePanel::left("network_panel")
            .resizable(false)
            .exact_width(panel_width)
            .frame(egui::Frame::default())
            .show_animated_inside(ui, is_panel_open, |ui| {
                self.network.ui(ui, frame, cb);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::default())
            .show_inside(ui, |ui| {
                self.show_current_screen(ui, frame, cb);
            });
    }

    fn show_global_modal(&mut self,
                         ui: &mut egui::Ui,
                         frame: &mut eframe::Frame,
                         cb: &dyn PlatformCallbacks) {
        Navigator::modal_ui(ui, ModalLocation::Global, |ui, modal| {
            match modal.id {
                Navigator::EXIT_MODAL => {
                    if self.show_exit_progress {
                        if !Node::is_running() {
                            App::exit(frame, cb);
                            modal.close();
                        }
                        ui.add_space(16.0);
                        ui.vertical_centered(|ui| {
                            View::small_loading_spinner(ui);
                            ui.add_space(12.0);
                            ui.label(t!("sync_status.shutdown"));
                        });
                        ui.add_space(10.0);
                    } else {
                        ui.add_space(8.0);
                        ui.vertical_centered(|ui| {
                            ui.label(t!("modal_exit.description"));
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
                                            App::exit(frame, cb);
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
                }
                _ => {}
            }
        });
    }

    fn show_current_screen(&mut self,
                           ui: &mut egui::Ui,
                           frame: &mut eframe::Frame,
                           cb: &dyn PlatformCallbacks) {
        let Self { screens, .. } = self;
        for screen in screens.iter_mut() {
            if Navigator::is_current(&screen.id()) {
                screen.ui(ui, frame, cb);
                break;
            }
        }
    }

    /// Get dual panel state and width
    fn dual_panel_state_width(&self, frame: &mut eframe::Frame) -> (bool, f32) {
        let dual_panel_mode = View::is_dual_panel_mode(frame);
        let is_panel_open = dual_panel_mode || Navigator::is_side_panel_open();
        let panel_width = if dual_panel_mode {
            min(frame.info().window_info.size.x as i64, View::SIDE_PANEL_MIN_WIDTH) as f32
        } else {
            frame.info().window_info.size.x
        };
        (is_panel_open, panel_width)
    }
}