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

use egui::RichText;
use egui::style::Margin;

use crate::AppConfig;
use crate::gui::Colors;
use crate::gui::icons::{CARDHOLDER, DATABASE, DOTS_THREE_OUTLINE_VERTICAL, FACTORY, FADERS, GAUGE, POWER};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{NetworkMetrics, NetworkMining, NetworkNode, NetworkSettings, Root, TitlePanel, View};
use crate::gui::views::network::types::{NetworkTab, NetworkTabType};
use crate::gui::views::types::TitleType;
use crate::node::Node;

/// Network content.
pub struct NetworkContent {
    /// Current tab view to show at ui.
    current_tab: Box<dyn NetworkTab>,
}

impl Default for NetworkContent {
    fn default() -> Self {
        Self {
            current_tab: Box::new(NetworkNode::default())
        }
    }
}

impl NetworkContent {
    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, cb: &dyn PlatformCallbacks) {
        // Show title panel.
        self.title_ui(ui, frame);

        // Show bottom tabs.
        egui::TopBottomPanel::bottom("network_tabs")
            .frame(egui::Frame {
                fill: Colors::FILL,
                inner_margin: Self::tabs_inner_margin(ui, frame),
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                self.tabs_ui(ui);
            });

        // Show tab content.
        egui::CentralPanel::default()
            .frame(egui::Frame {
                stroke: View::DEFAULT_STROKE,
                inner_margin: Self::content_inner_margin(ui, frame),
                fill: Colors::WHITE,
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                self.current_tab.ui(ui, frame, cb);
            });

        // Redraw content after delay if node is not syncing to update stats.
        if Node::not_syncing() {
            ui.ctx().request_repaint_after(Node::STATS_UPDATE_DELAY);
        }
    }

    /// Calculate tabs inner margin based on display insets (cutouts).
    fn tabs_inner_margin(ui: &mut egui::Ui, frame: &mut eframe::Frame) -> Margin {
        Margin {
            left: View::get_left_inset() + 4.0,
            right: View::far_right_inset_margin(ui, frame) + 4.0,
            top: 4.0,
            bottom: View::get_bottom_inset() + 4.0,
        }
    }

    /// Calculate content inner margin based on display insets (cutouts).
    fn content_inner_margin(ui: &mut egui::Ui, frame: &mut eframe::Frame) -> Margin {
        Margin {
            left: View::get_left_inset() + 4.0,
            right: View::far_right_inset_margin(ui, frame) + 4.0,
            top: 3.0,
            bottom: 4.0,
        }
    }

    /// Draw tab buttons in the bottom of the screen.
    fn tabs_ui(&mut self, ui: &mut egui::Ui) {
        ui.scope(|ui| {
            // Setup spacing between tabs.
            ui.style_mut().spacing.item_spacing = egui::vec2(4.0, 0.0);
            // Setup vertical padding inside tab button.
            ui.style_mut().spacing.button_padding = egui::vec2(0.0, 8.0);

            // Draw tab buttons.
            let current_type = self.current_tab.get_type();
            ui.columns(4, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::tab_button(ui, DATABASE, current_type == NetworkTabType::Node, || {
                            self.current_tab = Box::new(NetworkNode::default());
                        });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::tab_button(ui, GAUGE, current_type == NetworkTabType::Metrics, || {
                            self.current_tab = Box::new(NetworkMetrics::default());
                        });
                });
                columns[2].vertical_centered_justified(|ui| {
                    View::tab_button(ui, FACTORY, current_type == NetworkTabType::Mining, || {
                            self.current_tab = Box::new(NetworkMining::default());
                        });
                });
                columns[3].vertical_centered_justified(|ui| {
                    View::tab_button(ui, FADERS, current_type == NetworkTabType::Settings, || {
                            self.current_tab = Box::new(NetworkSettings::default());
                        });
                });
            });
        });
    }

    /// Draw title content.
    fn title_ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // Setup values for title panel.
        let title_text = self.current_tab.get_type().title().to_uppercase();
        let subtitle_text = Node::get_sync_status_text();
        let not_syncing = Node::not_syncing();
        let title_content = TitleType::WithSubTitle(title_text, subtitle_text, !not_syncing);

        // Draw title panel.
        TitlePanel::ui(title_content, |ui, _| {
            View::title_button(ui, DOTS_THREE_OUTLINE_VERTICAL, || {
                //TODO: Show connections
            });
        }, |ui, frame| {
            if !Root::is_dual_panel_mode(frame) {
                View::title_button(ui, CARDHOLDER, || {
                    Root::toggle_network_panel();
                });
            }
        }, ui, frame);
    }

    /// Content to draw when node is disabled.
    pub fn disabled_node_ui(ui: &mut egui::Ui) {
        View::center_content(ui, 156.0, |ui| {
            let text = t!("network.disabled_server", "dots" => DOTS_THREE_OUTLINE_VERTICAL);
            ui.label(RichText::new(text)
                .size(16.0)
                .color(Colors::INACTIVE_TEXT)
            );
            ui.add_space(8.0);
            View::button(ui, format!("{} {}", POWER, t!("network.enable_node")), Colors::GOLD, || {
                Node::start();
            });
            ui.add_space(2.0);
            Self::autorun_node_ui(ui);
        });
    }

    /// Draw checkbox to run integrated node on application launch.
    pub fn autorun_node_ui(ui: &mut egui::Ui) {
        let autostart = AppConfig::autostart_node();
        View::checkbox(ui, autostart, t!("network.autorun"), || {
            AppConfig::toggle_node_autostart();
        });
    }
}