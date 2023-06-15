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

use egui::{Color32, lerp, Rgba, RichText, Stroke};
use egui::style::Margin;
use egui_extras::{Size, StripBuilder};
use grin_chain::SyncStatus;

use crate::gui::{Colors, Navigator};
use crate::gui::icons::{CARDHOLDER, DATABASE, DOTS_THREE_OUTLINE_VERTICAL, FACTORY, FADERS, GAUGE, POWER};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{NetworkTab, View};
use crate::gui::views::network_metrics::NetworkMetrics;
use crate::gui::views::network_mining::NetworkMining;
use crate::gui::views::network_node::NetworkNode;
use crate::node::Node;

#[derive(PartialEq)]
enum Mode {
    Node,
    Metrics,
    Mining,
    Tuning
}

pub struct Network {
    current_mode: Mode,

    node_view: NetworkNode,
    metrics_view: NetworkMetrics,
    mining_view: NetworkMining,
}

impl Default for Network {
    fn default() -> Self {
        Self {
            current_mode: Mode::Node,
            node_view: NetworkNode::default(),
            metrics_view: NetworkMetrics::default(),
            mining_view: NetworkMining::default()
        }
    }
}

impl Network {
    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, _: &dyn PlatformCallbacks) {
        egui::TopBottomPanel::top("network_title")
            .resizable(false)
            .frame(egui::Frame {
                fill: Colors::YELLOW,
                inner_margin: Margin::same(0.0),
                outer_margin: Margin::same(0.0),
                stroke: Stroke::NONE,
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                self.draw_title(ui, frame);
            });

        egui::TopBottomPanel::bottom("network_tabs")
            .frame(egui::Frame {
                outer_margin: Margin::same(5.0),
                .. Default::default()
            })
            .show_inside(ui, |ui| {
                self.draw_tabs(ui);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame {
                stroke: View::DEFAULT_STROKE,
                inner_margin: Margin::same(4.0),
                fill: Colors::WHITE,
                .. Default::default()
            })
            .show_inside(ui, |ui| {
                self.draw_tab_content(ui);
            });
    }

    fn draw_tabs(&mut self, ui: &mut egui::Ui) {
        ui.scope(|ui| {
            // Setup spacing between tabs.
            ui.style_mut().spacing.item_spacing = egui::vec2(5.0, 0.0);
            // Setup vertical padding inside tab button.
            ui.style_mut().spacing.button_padding = egui::vec2(0.0, 3.0);

            ui.columns(4, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::tab_button(ui, DATABASE, self.current_mode == Mode::Node, || {
                        self.current_mode = Mode::Node;
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::tab_button(ui, GAUGE, self.current_mode == Mode::Metrics, || {
                        self.current_mode = Mode::Metrics;
                    });
                });
                columns[2].vertical_centered_justified(|ui| {
                    View::tab_button(ui, FACTORY, self.current_mode == Mode::Mining, || {
                        self.current_mode = Mode::Mining;
                    });
                });
                columns[3].vertical_centered_justified(|ui| {
                    View::tab_button(ui, FADERS, self.current_mode == Mode::Tuning, || {
                        self.current_mode = Mode::Tuning;
                    });
                });
            });
        });
    }

    fn draw_tab_content(&mut self, ui: &mut egui::Ui) {
        match self.current_mode {
            Mode::Node => {
                self.node_view.ui(ui);
            }
            Mode::Metrics => {
                self.metrics_view.ui(ui);
            }
            Mode::Tuning => {
                self.node_view.ui(ui);
            }
            Mode::Mining => {}
        }
    }

    fn draw_title(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        StripBuilder::new(ui)
            .size(Size::exact(52.0))
            .vertical(|mut strip| {
                strip.strip(|builder| {
                    builder
                        .size(Size::exact(52.0))
                        .size(Size::remainder())
                        .size(Size::exact(52.0))
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui.centered_and_justified(|ui| {
                                    View::title_button(ui, DOTS_THREE_OUTLINE_VERTICAL, || {
                                        //TODO: Actions for node
                                    });
                                });
                            });
                            strip.strip(|builder| {
                                self.draw_title_text(builder);
                            });
                            strip.cell(|ui| {
                                if !View::is_dual_panel_mode(frame) {
                                    ui.centered_and_justified(|ui| {
                                        View::title_button(ui, CARDHOLDER, || {
                                            Navigator::toggle_side_panel();
                                        });
                                    });
                                }
                            });
                        });
                });
            });
    }

    fn draw_title_text(&self, builder: StripBuilder) {
        let title_text = match &self.current_mode {
            Mode::Node => {
                self.node_view.name()
            }
            Mode::Metrics => {
                self.metrics_view.name()
            }
            Mode::Mining => {
                self.mining_view.name()
            }
            Mode::Tuning => {
                self.node_view.name()
            }
        };

        builder
            .size(Size::remainder())
            .size(Size::exact(32.0))
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    ui.add_space(2.0);
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new(title_text.to_uppercase())
                            .size(18.0)
                            .color(Colors::TITLE));
                    });
                });
                strip.cell(|ui| {
                    ui.centered_and_justified(|ui| {
                        let sync_status = Node::get_sync_status();

                        // Setup text color animation based on sync status
                        let idle = match sync_status {
                            None => { !Node::is_starting() }
                            Some(ss) => { ss == SyncStatus::NoSync }
                        };
                        let (dark, bright) = (0.3, 1.0);
                        let color_factor = if !idle {
                            lerp(dark..=bright, ui.input().time.cos().abs()) as f32
                        } else {
                            bright as f32
                        };

                        // Draw sync text
                        let status_color_rgba = Rgba::from(Colors::TEXT) * color_factor;
                        let status_color = Color32::from(status_color_rgba);
                        View::ellipsize_text(ui, Node::get_sync_status_text(), 15.0, status_color);

                        // Repaint based on sync status
                        if idle {
                            ui.ctx().request_repaint_after(Duration::from_millis(600));
                        } else {
                            ui.ctx().request_repaint();
                        }
                    });
                });
            });
    }

    pub fn disabled_server_content(ui: &mut egui::Ui) {
        View::center_content(ui, 142.0, |ui| {
            let text = t!("network.disabled_server", "dots" => DOTS_THREE_OUTLINE_VERTICAL);
            ui.label(RichText::new(text)
                .size(16.0)
                .color(Colors::INACTIVE_TEXT)
            );

            ui.add_space(10.0);

            View::button(ui, format!("{} {}", POWER, t!("network.enable")), Colors::GOLD, || {
                Node::start();
            });
        });
    }
}

