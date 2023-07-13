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

use egui::{Color32, lerp, Rgba, RichText};
use egui::style::Margin;
use egui_extras::{Size, StripBuilder};
use grin_chain::SyncStatus;

use crate::AppConfig;
use crate::gui::Colors;
use crate::gui::icons::{CARDHOLDER, DATABASE, DOTS_THREE_OUTLINE_VERTICAL, FACTORY, FADERS, GAUGE, POWER};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, ModalContainer, NetworkMetrics, NetworkMining, NetworkNode, NetworkSettings, Root, TitleAction, TitleContent, TitlePanel, View};
use crate::gui::views::network::setup::{DandelionSetup, NodeSetup, P2PSetup, PoolSetup, StratumSetup};
use crate::node::Node;

pub trait NetworkTab {
    fn get_type(&self) -> NetworkTabType;
    fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks);
    fn on_modal_ui(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks);
}


#[derive(PartialEq)]
pub enum NetworkTabType {
    Node,
    Metrics,
    Mining,
    Settings
}

impl NetworkTabType {
    pub fn title(&self) -> String {
        match *self {
            NetworkTabType::Node => { t!("network.node") }
            NetworkTabType::Metrics => { t!("network.metrics") }
            NetworkTabType::Mining => { t!("network.mining") }
            NetworkTabType::Settings => { t!("network.settings") }
        }
    }
}

/// Network content.
pub struct Network {
    /// Current tab view to show at ui.
    current_tab: Box<dyn NetworkTab>,
    /// [`Modal`] ids allowed at this ui container.
    modal_ids: Vec<&'static str>,
}

impl Default for Network {
    fn default() -> Self {
        Self {
            current_tab: Box::new(NetworkNode::default()),
            modal_ids: vec![
                // Network settings modals.
                NetworkSettings::NODE_RESTART_REQUIRED_MODAL,
                NetworkSettings::RESET_SETTINGS_MODAL,
                // Node setup modals.
                NodeSetup::API_PORT_MODAL,
                NodeSetup::API_SECRET_MODAL,
                NodeSetup::FOREIGN_API_SECRET_MODAL,
                NodeSetup::FTL_MODAL,
                // P2P setup modals.
                P2PSetup::PORT_MODAL,
                P2PSetup::CUSTOM_SEED_MODAL,
                P2PSetup::ALLOW_PEER_MODAL,
                P2PSetup::DENY_PEER_MODAL,
                P2PSetup::PREFER_PEER_MODAL,
                P2PSetup::BAN_WINDOW_MODAL,
                P2PSetup::MAX_INBOUND_MODAL,
                P2PSetup::MAX_OUTBOUND_MODAL,
                P2PSetup::MIN_OUTBOUND_MODAL,
                // Stratum setup modals.
                StratumSetup::STRATUM_PORT_MODAL,
                StratumSetup::ATTEMPT_TIME_MODAL,
                StratumSetup::MIN_SHARE_DIFF_MODAL,
                // Pool setup modals.
                PoolSetup::FEE_BASE_MODAL,
                PoolSetup::REORG_PERIOD_MODAL,
                PoolSetup::POOL_SIZE_MODAL,
                PoolSetup::STEMPOOL_SIZE_MODAL,
                PoolSetup::MAX_WEIGHT_MODAL,
                // Dandelion setup modals.
                DandelionSetup::EPOCH_MODAL,
                DandelionSetup::EMBARGO_MODAL,
                DandelionSetup::AGGREGATION_MODAL,
                DandelionSetup::STEM_PROBABILITY_MODAL,
            ]
        }
    }
}

impl ModalContainer for Network {
    fn modal_ids(&self) -> &Vec<&'static str> {
        self.modal_ids.as_ref()
    }
}

impl Network {
    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, cb: &dyn PlatformCallbacks) {
        // Show modal content if it's opened.
        if self.can_draw_modal() {
            Modal::ui(ui, |ui, modal| {
                self.current_tab.as_mut().on_modal_ui(ui, modal, cb);
            });
        }

        egui::TopBottomPanel::top("network_title")
            .exact_height(TitlePanel::DEFAULT_HEIGHT)
            .resizable(false)
            .frame(egui::Frame {
                fill: Colors::YELLOW,
                inner_margin: Margin::same(0.0),
                outer_margin: Margin::same(0.0),
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                self.title_ui(ui, frame);
            });

        egui::TopBottomPanel::bottom("network_tabs")
            .frame(egui::Frame {
                fill: Colors::FILL,
                inner_margin: Margin::same(4.0),
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                self.tabs_ui(ui);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame {
                stroke: View::DEFAULT_STROKE,
                inner_margin: Margin {
                    left: 4.0,
                    right: 4.0,
                    top: 3.0,
                    bottom: 4.0,
                },
                fill: Colors::WHITE,
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                self.current_tab.ui(ui, cb);
            });
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
        let title_content = TitleContent::Custom("network_title".to_string(), Box::new(|ui| {
        }));
        TitlePanel::test_ui(title_content, TitleAction::new(DOTS_THREE_OUTLINE_VERTICAL, || {
            //TODO: Show connections
        }), if !Root::is_dual_panel_mode(frame) {
            TitleAction::new(CARDHOLDER, || {
                Root::toggle_side_panel();
            })
        } else {
            None
        }, ui);

        // StripBuilder::new(ui)
        //     .size(Size::exact(52.0))
        //     .size(Size::remainder())
        //     .size(Size::exact(52.0))
        //     .horizontal(|mut strip| {
        //         strip.cell(|ui| {
        //             ui.centered_and_justified(|ui| {
        //                 View::title_button(ui, DOTS_THREE_OUTLINE_VERTICAL, || {
        //                     //TODO: Show connections
        //                 });
        //             });
        //         });
        //         strip.strip(|builder| {
        //             self.title_text_ui(builder);
        //         });
        //         strip.cell(|ui| {
        //             if !Root::is_dual_panel_mode(frame) {
        //                 ui.centered_and_justified(|ui| {
        //                     View::title_button(ui, CARDHOLDER, || {
        //                         Root::toggle_side_panel();
        //                     });
        //                 });
        //             }
        //         });
        //     });
    }

    /// Draw title text.
    fn title_text_ui(&self, builder: StripBuilder) {
        builder
            .size(Size::remainder())
            .size(Size::exact(28.0))
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    ui.add_space(4.0);
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new(self.current_tab.get_type().title().to_uppercase())
                            .size(19.0)
                            .color(Colors::TITLE));
                    });
                });
                strip.cell(|ui| {
                    ui.centered_and_justified(|ui| {
                        let sync_status = Node::get_sync_status();

                        // Setup text color animation based on sync status
                        let idle = match sync_status {
                            None => !Node::is_starting(),
                            Some(ss) => ss == SyncStatus::NoSync
                        };
                        let (dark, bright) = (0.3, 1.0);
                        let color_factor = if !idle {
                            lerp(dark..=bright, ui.input(|i| i.time).cos().abs()) as f32
                        } else {
                            bright as f32
                        };

                        // Draw sync status text.
                        let status_color_rgba = Rgba::from(Colors::TEXT) * color_factor;
                        let status_color = Color32::from(status_color_rgba);
                        View::ellipsize_text(ui, Node::get_sync_status_text(), 15.0, status_color);

                        // Repaint delay based on sync status.
                        if idle {
                            ui.ctx().request_repaint_after(Node::STATS_UPDATE_DELAY);
                        } else {
                            ui.ctx().request_repaint();
                        }
                    });
                });
            });
    }

    /// Content to draw when node is disabled.
    pub fn disabled_node_ui(ui: &mut egui::Ui) {
        View::center_content(ui, 162.0, |ui| {
            let text = t!("network.disabled_server", "dots" => DOTS_THREE_OUTLINE_VERTICAL);
            ui.label(RichText::new(text)
                .size(16.0)
                .color(Colors::INACTIVE_TEXT)
            );
            ui.add_space(10.0);
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