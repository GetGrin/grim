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
use egui::RichText;
use crate::gui::{App, Colors, Navigator};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::screens::{Account, Accounts, Screen, ScreenId};
use crate::gui::views::{ModalContainer, NetworkContainer, View};
use crate::node::Node;

pub struct Root {
    screens: Vec<Box<dyn Screen>>,
    network_panel: NetworkContainer,
    show_exit_progress: bool,
    allowed_modal_ids: Vec<&'static str>
}

impl Default for Root {
    fn default() -> Self {
        Navigator::init(ScreenId::Accounts);

        Self {
            screens: vec![
                Box::new(Accounts::default()),
                Box::new(Account::default())
            ],
            network_panel: NetworkContainer::default(),
            show_exit_progress: false,
            allowed_modal_ids: vec![
                Navigator::EXIT_MODAL
            ]
        }
    }
}

impl ModalContainer for Root {
    fn modal_ids(&self) -> &Vec<&'static str> {
        self.allowed_modal_ids.as_ref()
    }
}

impl Root {
    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, cb: &dyn PlatformCallbacks) {
        // Draw exit modal content if it's open.
        let modal_id = Navigator::is_modal_open();
        if modal_id.is_some() && self.can_show_modal(modal_id.unwrap()) {
            self.exit_modal_content(ui, frame, cb);
        }

        let (is_panel_open, panel_width) = Self::side_panel_state_width(frame);
        egui::SidePanel::left("network_panel")
            .resizable(false)
            .exact_width(panel_width)
            .frame(egui::Frame::default())
            .show_animated_inside(ui, is_panel_open, |ui| {
                self.network_panel.ui(ui, frame, cb);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::default())
            .show_inside(ui, |ui| {
                self.show_current_screen(ui, frame, cb);
            });
    }

    fn exit_modal_content(&mut self,
                          ui: &mut egui::Ui,
                          frame: &mut eframe::Frame,
                          cb: &dyn PlatformCallbacks) {
        Navigator::modal_ui(ui, |ui, modal| {
            if self.show_exit_progress {
                if !Node::is_running() {
                    App::exit(frame, cb);
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

    /// Get side panel state and width.
    fn side_panel_state_width(frame: &mut eframe::Frame) -> (bool, f32) {
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