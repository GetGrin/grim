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

use std::time::Duration;
use egui::{Id, Margin, RichText, ScrollArea};
use egui::scroll_area::ScrollBarVisibility;

use crate::AppConfig;
use crate::gui::Colors;
use crate::gui::icons::{ARROWS_COUNTER_CLOCKWISE, BRIEFCASE, DATABASE, DOTS_THREE_OUTLINE_VERTICAL, FACTORY, FADERS, GAUGE, POWER};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{ConnectionsContent, NetworkMetrics, NetworkMining, NetworkNode, NetworkSettings, Content, TitlePanel, View};
use crate::gui::views::network::types::{NetworkTab, NetworkTabType};
use crate::gui::views::types::{TitleContentType, TitleType};
use crate::node::{Node, NodeError};
use crate::wallet::ExternalConnection;

/// Network content.
pub struct NetworkContent {
    /// Current integrated node tab content.
    node_tab_content: Box<dyn NetworkTab>,
    /// Connections content.
    connections: ConnectionsContent,
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
    pub fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        let show_connections = AppConfig::show_connections_network_panel();
        let dual_panel = Content::is_dual_panel_mode(ui);

        // Show title panel.
        self.title_ui(ui, show_connections);

        // Show integrated node tabs content.
        if !show_connections {
            egui::TopBottomPanel::bottom("node_tabs_content")
                .min_height(0.5)
                .resizable(false)
                .frame(egui::Frame {
                    inner_margin: Margin {
                        left: View::get_left_inset() + View::TAB_ITEMS_PADDING,
                        right: View::far_right_inset_margin(ui) + View::TAB_ITEMS_PADDING,
                        top: View::TAB_ITEMS_PADDING,
                        bottom: View::get_bottom_inset() + View::TAB_ITEMS_PADDING,
                    },
                    fill: Colors::fill(),
                    ..Default::default()
                })
                .show_inside(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        View::max_width_ui(ui, Content::SIDE_PANEL_WIDTH * 1.3, |ui| {
                            self.tabs_ui(ui);
                        });
                    });
                });
        }

        // Show current node tab content.
        egui::SidePanel::right("node_tab_content")
            .resizable(false)
            .exact_width(ui.available_width())
            .frame(egui::Frame {
                ..Default::default()
            })
            .show_animated_inside(ui, !show_connections, |ui| {
                egui::CentralPanel::default()
                    .frame(egui::Frame {
                        fill: Colors::white_or_black(false),
                        stroke: View::item_stroke(),
                        inner_margin: Margin {
                            left: View::get_left_inset() + 4.0,
                            right: View::far_right_inset_margin(ui) + 4.0,
                            top: 3.0,
                            bottom: 4.0,
                        },
                        ..Default::default()
                    })
                    .show_inside(ui, |ui| {
                        self.node_tab_content.ui(ui, cb);
                    });
            });

        // Show connections content.
        egui::CentralPanel::default()
            .frame(egui::Frame {
                stroke: View::item_stroke(),
                inner_margin: Margin {
                    left: if show_connections {
                        View::get_left_inset() + 4.0
                    } else {
                        0.0
                    },
                    right: if show_connections {
                        View::far_right_inset_margin(ui) + 4.0
                    } else {
                        0.0
                    },
                    top: 3.0,
                    bottom: if View::is_desktop() && show_connections {
                        6.0
                    } else {
                        4.0
                    },
                },
                fill: Colors::button(),
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                ScrollArea::vertical()
                    .id_source("connections_content")
                    .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.add_space(1.0);
                        ui.vertical_centered(|ui| {
                            let max_width = if !dual_panel {
                                Content::SIDE_PANEL_WIDTH * 1.3
                            } else {
                                ui.available_width()
                            };
                            View::max_width_ui(ui, max_width, |ui| {
                                self.connections.ui(ui, cb);
                            });
                        });
                    });
            });

        // Redraw after delay.
        if Node::is_running() {
            ui.ctx().request_repaint_after(Node::STATS_UPDATE_DELAY);
        } else if show_connections {
            ui.ctx().request_repaint_after(Duration::from_millis(1000));
        }
    }

    /// Draw tab buttons in the bottom of the screen.
    fn tabs_ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            // Setup spacing between tabs.
            ui.style_mut().spacing.item_spacing = egui::vec2(View::TAB_ITEMS_PADDING, 0.0);
            // Setup vertical padding inside tab button.
            ui.style_mut().spacing.button_padding = egui::vec2(0.0, 4.0);

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
    fn title_ui(&mut self, ui: &mut egui::Ui, show_connections: bool) {
        // Setup values for title panel.
        let title_text = self.node_tab_content.get_type().title().to_uppercase();
        let subtitle_text = Node::get_sync_status_text();
        let not_syncing = Node::not_syncing();
        let title_content = if !show_connections {
            TitleContentType::WithSubTitle(title_text, subtitle_text, !not_syncing)
        } else {
            TitleContentType::Title(t!("network.connections").to_uppercase())
        };

        // Draw title panel.
        TitlePanel::new(Id::from("network_title_panel")).ui(TitleType::Single(title_content), |ui| {
            if !show_connections {
                View::title_button_big(ui, DOTS_THREE_OUTLINE_VERTICAL, |_| {
                    AppConfig::toggle_show_connections_network_panel();
                    if AppConfig::show_connections_network_panel() {
                        ExternalConnection::check_ext_conn_availability(None);
                    }
                });
            }
        }, |ui| {
            if !Content::is_dual_panel_mode(ui) {
                View::title_button_big(ui, BRIEFCASE, |_| {
                    Content::toggle_network_panel();
                });
            }
        }, ui);
    }

    /// Content to draw when node is disabled.
    pub fn disabled_node_ui(ui: &mut egui::Ui) {
        View::center_content(ui, 156.0, |ui| {
            let text = t!("network.disabled_server", "dots" => DOTS_THREE_OUTLINE_VERTICAL);
            ui.label(RichText::new(text)
                .size(16.0)
                .color(Colors::inactive_text())
            );
            ui.add_space(8.0);
            View::action_button(ui, format!("{} {}", POWER, t!("network.enable_node")), || {
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
                        .color(Colors::inactive_text())
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

    /// Draw integrated node error content.
    pub fn node_error_ui(ui: &mut egui::Ui, e: NodeError) {
        match e {
            NodeError::Storage => {
                View::center_content(ui, 156.0, |ui| {
                    ui.label(RichText::new(t!("network_node.error_clean"))
                        .size(16.0)
                        .color(Colors::red())
                    );
                    ui.add_space(8.0);
                    let btn_txt = format!("{} {}",
                                          ARROWS_COUNTER_CLOCKWISE,
                                          t!("network_node.resync"));
                    View::action_button(ui, btn_txt, || {
                        Node::clean_up_data();
                        Node::start();
                    });
                    ui.add_space(2.0);
                });
                return;
            }
            NodeError::P2P | NodeError::API => {
                let msg_type = match e {
                    NodeError::API => "API",
                    _ => "P2P"
                };
                View::center_content(ui, 106.0, |ui| {
                    let text = t!(
                        "network_node.error_p2p_api",
                        "p2p_api" => msg_type,
                        "settings" => FADERS
                    );
                    ui.label(RichText::new(text)
                        .size(16.0)
                        .color(Colors::red())
                    );
                    ui.add_space(2.0);
                });
                return;
            }
            NodeError::Configuration => {
                View::center_content(ui, 106.0, |ui| {
                    ui.label(RichText::new(t!("network_node.error_config", "settings" => FADERS))
                        .size(16.0)
                        .color(Colors::red())
                    );
                    ui.add_space(2.0);
                });
            }
            NodeError::Unknown => {
                View::center_content(ui, 156.0, |ui| {
                    ui.label(RichText::new(t!("network_node.error_unknown", "settings" => FADERS))
                        .size(16.0)
                        .color(Colors::red())
                    );
                    ui.add_space(8.0);
                    let btn_txt = format!("{} {}",
                                          ARROWS_COUNTER_CLOCKWISE,
                                          t!("network_node.resync"));
                    View::action_button(ui, btn_txt, || {
                        Node::clean_up_data();
                        Node::start();
                    });
                    ui.add_space(2.0);
                });
            }
        }
    }
}