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

use eframe::emath::lerp;
use eframe::epaint::{Color32, FontId, Rgba, Stroke};
use eframe::epaint::text::{LayoutJob, TextFormat, TextWrapping};
use egui::RichText;
use egui::style::Margin;
use egui_extras::{Size, StripBuilder};
use grin_chain::SyncStatus;
use grin_core::global::ChainTypes;

use crate::gui::app::is_dual_panel_mode;
use crate::gui::colors::{COLOR_DARK, COLOR_YELLOW};
use crate::gui::icons::{CARDHOLDER, DATABASE, DOTS_THREE_OUTLINE_VERTICAL, FACTORY, FADERS, GAUGE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::screens::Navigator;
use crate::gui::views::{DEFAULT_STROKE, NetworkTab};
use crate::gui::views::common::{tab_button, title_button};
use crate::gui::views::network_node::NetworkNode;
use crate::node::Node;

#[derive(PartialEq)]
enum Mode {
    Node,
    Metrics,
    Miner,
    Tuning
}

pub struct Network {
    node: Node,

    current_mode: Mode,
    node_view: NetworkNode,
}

impl Default for Network {
    fn default() -> Self {
        let node = Node::new(ChainTypes::Mainnet, true);
        Self {
            node,
            current_mode: Mode::Node,
            node_view: NetworkNode::default()
        }
    }
}

impl Network {
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              frame: &mut eframe::Frame,
              nav: &mut Navigator,
              cb: &dyn PlatformCallbacks) {

        egui::TopBottomPanel::top("network_title")
            .resizable(false)
            .frame(egui::Frame {
                fill: COLOR_YELLOW,
                inner_margin: Margin::same(0.0),
                outer_margin: Margin::same(0.0),
                stroke: Stroke::NONE,
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                self.draw_title(ui, frame, nav);
            });

        egui::TopBottomPanel::bottom("network_tabs")
            .frame(egui::Frame {
                outer_margin: Margin::same(6.0),
                .. Default::default()
            })
            .resizable(false)
            .show_inside(ui, |ui| {
                self.draw_tabs(ui);
            });

        egui::CentralPanel::default().frame(egui::Frame {
            stroke: DEFAULT_STROKE,
            inner_margin: Margin::same(4.0),
            fill: Color32::WHITE,
            .. Default::default()
        }).show_inside(ui, |ui| {
            self.draw_tab_content(ui);
        });


    }

    fn draw_tabs(&mut self, ui: &mut egui::Ui) {
        ui.columns(4, |columns| {
            columns[0].vertical_centered(|ui| {
                tab_button(ui, DATABASE, self.current_mode == Mode::Node, || {
                    self.current_mode = Mode::Node;
                });
            });
            columns[1].vertical_centered(|ui| {
                tab_button(ui, GAUGE, self.current_mode == Mode::Metrics, || {
                    self.current_mode = Mode::Metrics;
                });
            });
            columns[2].vertical_centered(|ui| {
                tab_button(ui, FACTORY, self.current_mode == Mode::Miner, || {
                    self.current_mode = Mode::Miner;
                });
            });
            columns[3].vertical_centered(|ui| {
                tab_button(ui, FADERS, self.current_mode == Mode::Tuning, || {
                    self.current_mode = Mode::Tuning;
                });
            });

        });
    }

    fn draw_tab_content(&mut self, ui: &mut egui::Ui) {
        match self.current_mode {
            Mode::Node => {
                self.node_view.ui(ui, &mut self.node);
            }
            Mode::Metrics => {}
            Mode::Tuning => {}
            Mode::Miner => {}
        }
    }

    fn draw_title(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, nav: &mut Navigator) {
        // Disable stroke around title buttons on hover
        ui.style_mut().visuals.widgets.active.bg_stroke = Stroke::NONE;

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
                                    title_button(ui, DOTS_THREE_OUTLINE_VERTICAL, || {
                                        //TODO: Actions for node
                                    });
                                });
                            });
                            strip.strip(|builder| {
                                self.draw_title_text(builder);
                            });
                            strip.cell(|ui| {
                                if !is_dual_panel_mode(frame) {
                                    ui.centered_and_justified(|ui| {
                                        title_button(ui, CARDHOLDER, || {
                                            nav.toggle_left_panel();
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
                self.node_view.name()
            }
            Mode::Miner => {
                self.node_view.name()
            }
            Mode::Tuning => {
                self.node_view.name()
            }
        };

        builder
            .size(Size::exact(19.0))
            .size(Size::remainder())
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    ui.centered_and_justified(|ui| {
                        ui.label(RichText::new(title_text.to_uppercase())
                            .size(19.0)
                            .color(COLOR_DARK));
                    });
                });
                strip.cell(|ui| {
                    ui.centered_and_justified(|ui| {
                        // Select sync status text
                        let sync_status = self.node.state.get_sync_status();
                        let status_text = if self.node.state.is_restarting() {
                            t!("server_restarting")
                        } else {
                            match sync_status {
                                None => {
                                    t!("server_down")
                                }
                                Some(ss) => {
                                    get_sync_status_text(ss).to_string()
                                }
                            }
                        };

                        // Setup text color animation based on sync status
                        let idle = match sync_status {
                            None => { !self.node.state.is_starting() }
                            Some(ss) => { ss == SyncStatus::NoSync }
                        };
                        let (dark, bright) = (0.3, 1.0);
                        let color_factor = if !idle {
                            lerp(dark..=bright, ui.input().time.cos().abs())
                        } else {
                            bright
                        };

                        // Repaint based on sync status
                        if idle {
                            ui.ctx().request_repaint_after(Duration::from_millis(600));
                        } else {
                            ui.ctx().request_repaint();
                        }

                        // Draw sync text
                        let mut job = LayoutJob::single_section(status_text, TextFormat {
                            font_id: FontId::proportional(15.0),
                            color: Color32::from(Rgba::from(COLOR_DARK) * color_factor as f32),
                            .. Default::default()
                        });
                        job.wrap = TextWrapping {
                            max_rows: 1,
                            break_anywhere: false,
                            overflow_character: Option::from('﹍'),
                            ..Default::default()
                        };
                        ui.label(job);
                    });
                });
            });
    }
}

fn get_sync_status_text(sync_status: SyncStatus) -> String {
    match sync_status {
        SyncStatus::Initial => t!("sync_status.initial"),
        SyncStatus::NoSync => t!("sync_status.no_sync"),
        SyncStatus::AwaitingPeers(_) => t!("sync_status.awaiting_peers"),
        SyncStatus::HeaderSync {
            sync_head,
            highest_height,
            ..
        } => {
            if highest_height == 0 {
                t!("sync_status.header_sync")
            } else {
                let percent = sync_head.height * 100 / highest_height;
                t!("sync_status.header_sync_percent", "percent" => percent)
            }
        }
        SyncStatus::TxHashsetDownload(stat) => {
            if stat.total_size > 0 {
                let percent = stat.downloaded_size * 100 / stat.total_size;
                t!("sync_status.tx_hashset_download_percent", "percent" => percent)
            } else {
                t!("sync_status.tx_hashset_download")
            }
        }
        SyncStatus::TxHashsetSetup => {
            t!("sync_status.tx_hashset_setup")
        }
        SyncStatus::TxHashsetRangeProofsValidation {
            rproofs,
            rproofs_total,
        } => {
            let r_percent = if rproofs_total > 0 {
                (rproofs * 100) / rproofs_total
            } else {
                0
            };
            t!("sync_status.tx_hashset_range_proofs_validation", "percent" => r_percent)
        }
        SyncStatus::TxHashsetKernelsValidation {
            kernels,
            kernels_total,
        } => {
            let k_percent = if kernels_total > 0 {
                (kernels * 100) / kernels_total
            } else {
                0
            };
            t!("sync_status.tx_hashset_kernels_validation", "percent" => k_percent)
        }
        SyncStatus::TxHashsetSave | SyncStatus::TxHashsetDone => {
            t!("sync_status.tx_hashset_save")
        }
        SyncStatus::BodySync {
            current_height,
            highest_height,
        } => {
            if highest_height == 0 {
                t!("sync_status.body_sync")
            } else {
                let percent = current_height * 100 / highest_height;
                t!("sync_status.body_sync_percent", "percent" => percent)
            }
        }
        SyncStatus::Shutdown => t!("sync_status.shutdown"),
    }
}


