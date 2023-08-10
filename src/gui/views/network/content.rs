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

use egui::{RichText, ScrollArea, Stroke};
use egui::style::Margin;

use crate::AppConfig;
use crate::gui::Colors;
use crate::gui::icons::{CARDHOLDER, DATABASE, DOTS_THREE_OUTLINE_VERTICAL, FACTORY, FADERS, GAUGE, PLUS_CIRCLE, POWER};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{ConnectionsContent, NetworkMetrics, NetworkMining, NetworkNode, NetworkSettings, Root, TitlePanel, View};
use crate::gui::views::network::types::{NetworkTab, NetworkTabType};
use crate::gui::views::types::TitleType;
use crate::node::Node;
use crate::wallet::ExternalConnection;

/// Network content.
pub struct NetworkContent {
    /// Current integrated node tab content.
    node_tab_content: Box<dyn NetworkTab>,
    /// Connections content.
    connections: ConnectionsContent
}

impl Default for NetworkContent {
    fn default() -> Self {
        Self {
            node_tab_content: Box::new(NetworkNode::default()),
            connections: ConnectionsContent::default(),
        }
    }
}

impl NetworkContent {
    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, cb: &dyn PlatformCallbacks) {
        // Flag to show connections or integrated node content.
        let show_connections = AppConfig::show_connections_network_panel();

        // Show title panel.
        self.title_ui(ui, frame, show_connections, cb);

        // Show integrated node tabs content.
        egui::TopBottomPanel::bottom("node_tabs_panel")
            .resizable(false)
            .frame(egui::Frame {
                fill: Colors::FILL,
                inner_margin: Margin {
                    left: View::get_left_inset() + 4.0,
                    right: View::far_right_inset_margin(ui, frame) + 4.0,
                    top: 4.0,
                    bottom: View::get_bottom_inset() + 4.0,
                },
                ..Default::default()
            })
            .show_animated_inside(ui, !show_connections, |ui| {
                self.tabs_ui(ui);
            });

        // Show current node tab content.
        egui::SidePanel::right("node_tab_content_panel")
            .resizable(false)
            .exact_width(ui.available_width())
            .frame(egui::Frame {
                stroke: View::DEFAULT_STROKE,
                ..Default::default()
            })
            .show_animated_inside(ui, !show_connections, |ui| {
                egui::CentralPanel::default()
                    .frame(egui::Frame {
                        fill: Colors::WHITE,
                        stroke: View::DEFAULT_STROKE,
                        inner_margin: Margin {
                            left: View::get_left_inset() + 4.0,
                            right: View::far_right_inset_margin(ui, frame) + 4.0,
                            top: 3.0,
                            bottom: 4.0,
                        },
                        ..Default::default()
                    })
                    .show_inside(ui, |ui| {
                        self.node_tab_content.ui(ui, frame, cb);
                    });
            });

        let content_width = ui.available_width();

        // Show connections content.
        egui::CentralPanel::default()
            .frame(egui::Frame {
                stroke: if show_connections{
                    View::DEFAULT_STROKE
                } else {
                    Stroke::NONE
                },
                inner_margin: Margin {
                    left: if show_connections {
                        View::get_left_inset() + 4.0
                    } else {
                        0.0
                    },
                    right: if show_connections {
                        View::far_right_inset_margin(ui, frame) + 4.0
                    } else {
                        0.0
                    },
                    top: 3.0,
                    bottom: View::get_bottom_inset() + 4.0,
                },
                fill: Colors::BUTTON,
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                if !show_connections {
                    return;
                }
                ScrollArea::vertical()
                    .id_source("connections_content")
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.add_space(1.0);
                        ui.vertical_centered(|ui| {
                            // Setup wallet list width.
                            let mut rect = ui.available_rect_before_wrap();
                            let mut width = ui.available_width();
                            if !Root::is_dual_panel_mode(frame) {
                                width = f32::min(width, Root::SIDE_PANEL_WIDTH * 1.3)
                            }
                            rect.set_width(width);

                            ui.allocate_ui(rect.size(), |ui| {
                                self.connections.ui(ui, frame, cb);
                            });
                        });
                    });
            });

        // Redraw after delay if node is not syncing to update stats.
        if Node::not_syncing() {
            ui.ctx().request_repaint_after(Node::STATS_UPDATE_DELAY);
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
            let current_type = self.node_tab_content.get_type();
            ui.columns(4, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::tab_button(ui, DATABASE, current_type == NetworkTabType::Node, || {
                        self.node_tab_content = Box::new(NetworkNode::default());
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::tab_button(ui, GAUGE, current_type == NetworkTabType::Metrics, || {
                        self.node_tab_content = Box::new(NetworkMetrics::default());
                    });
                });
                columns[2].vertical_centered_justified(|ui| {
                    View::tab_button(ui, FACTORY, current_type == NetworkTabType::Mining, || {
                        self.node_tab_content = Box::new(NetworkMining::default());
                    });
                });
                columns[3].vertical_centered_justified(|ui| {
                    View::tab_button(ui, FADERS, current_type == NetworkTabType::Settings, || {
                        self.node_tab_content = Box::new(NetworkSettings::default());
                    });
                });
            });
        });
    }

    /// Draw title content.
    fn title_ui(&mut self,
                ui: &mut egui::Ui,
                frame: &mut eframe::Frame,
                show_connections: bool,
                cb: &dyn PlatformCallbacks) {
        // Setup values for title panel.
        let title_text = self.node_tab_content.get_type().title().to_uppercase();
        let subtitle_text = Node::get_sync_status_text();
        let not_syncing = Node::not_syncing();
        let title_content = if !show_connections {
            TitleType::WithSubTitle(title_text, subtitle_text, !not_syncing)
        } else {
            TitleType::Single(t!("network.connections").to_uppercase())
        };

        // Draw title panel.
        TitlePanel::ui(title_content, |ui, _| {
            if !show_connections {
                View::title_button(ui, DOTS_THREE_OUTLINE_VERTICAL, || {
                    AppConfig::toggle_show_connections_network_panel();
                    if AppConfig::show_connections_network_panel() {
                        ExternalConnection::start_ext_conn_availability_check();
                    }
                });
            } else {
                View::title_button(ui, PLUS_CIRCLE, || {
                    self.connections.show_add_ext_conn_modal(cb);
                });
            }
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

    /// Content to draw on loading.
    pub fn loading_ui(ui: &mut egui::Ui, text: Option<String>) {
        match text {
            None => {
                ui.centered_and_justified(|ui| {
                    View::big_loading_spinner(ui);
                });
            }
            Some(t) => {
                View::center_content(ui, 162.0, |ui| {
                    View::big_loading_spinner(ui);
                    ui.add_space(18.0);
                    ui.label(RichText::new(t)
                        .size(16.0)
                        .color(Colors::INACTIVE_TEXT)
                    );
                });
            }
        }
    }

    /// Draw checkbox to run integrated node on application launch.
    pub fn autorun_node_ui(ui: &mut egui::Ui) {
        let autostart = AppConfig::autostart_node();
        View::checkbox(ui, autostart, t!("network.autorun"), || {
            AppConfig::toggle_node_autostart();
        });
    }
}