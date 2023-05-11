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

use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::sync::atomic::Ordering;
use std::time::Duration;
use chrono::Utc;
use eframe::epaint::{Color32, FontId, Stroke};
use eframe::epaint::text::{LayoutJob, TextFormat, TextWrapping};
use egui::{Response, RichText, Sense, Spinner, Widget};
use egui::style::Margin;
use egui_extras::{Size, StripBuilder};
use grin_chain::SyncStatus;

use grin_core::global::ChainTypes;
use grin_servers::ServerStats;
use crate::gui::app::is_dual_panel_mode;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::screens::Navigator;
use crate::gui::{COLOR_DARK, COLOR_LIGHT, COLOR_YELLOW, SYM_ACCOUNTS, SYM_METRICS, SYM_NETWORK};

use crate::gui::views::{NetworkTab, TitlePanel, TitlePanelAction};
use crate::gui::views::network_node::NetworkNode;
use crate::node;
use crate::node::Node;

enum Mode {
    Node,
    // Miner,
    Metrics,
    Tuning
}

pub struct Network {
    current_mode: Mode,
    node: Node,
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

        egui::CentralPanel::default().frame(egui::Frame {
            stroke: Stroke::new(1.0, Color32::from_gray(190)),
            fill: Color32::WHITE,
            .. Default::default()
        }).show_inside(ui, |ui| {
            self.draw_tab_content(ui);
        });

        egui::TopBottomPanel::bottom("network_tabs")
            .frame(egui::Frame {
                stroke: Stroke::new(1.0, Color32::from_gray(190)),
                .. Default::default()
            })
            .resizable(false)
            .show_inside(ui, |ui| {
                self.draw_tabs(ui);
            });

        ui.ctx().request_repaint_after(Duration::from_millis(1000));
    }

    fn draw_tabs(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.columns(3, |columns| {
                columns[0].horizontal_wrapped(|ui| {
                });
                columns[1].vertical_centered(|ui| {
                });
                columns[2].horizontal_wrapped(|ui| {
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
                            strip.empty();
                            strip.strip(|builder| {
                                self.draw_title_text(builder);
                            });
                            strip.cell(|ui| {
                                if !is_dual_panel_mode(frame) {
                                    ui.centered_and_justified(|ui| {
                                        let b = egui::widgets::Button::new(
                                            RichText::new(SYM_ACCOUNTS)
                                                .size(24.0)
                                                .color(COLOR_DARK)
                                        ).fill(Color32::TRANSPARENT)
                                            .ui(ui).interact(Sense::click_and_drag());
                                        if b.drag_released() || b.clicked() {
                                            nav.toggle_left_panel();
                                        };
                                    });
                                }
                            });
                        });
                });
            });
    }

    fn draw_title_text(&self, mut builder: StripBuilder) {
        let Self { node, ..} = self;

        let title_text = match &self.current_mode {
            Mode::Node => {
                self.node_view.name()
            }
            Mode::Metrics => {
                self.node_view.name()
            }
            Mode::Tuning => {
                self.node_view.name()
            }
        };

        let syncing = node.state.is_syncing();

        let mut b = builder.size(Size::remainder());
        if syncing {
            b = b.size(Size::remainder());
        }
        b.vertical(|mut strip| {
            strip.cell(|ui| {
                ui.centered_and_justified(|ui| {
                    ui.heading(title_text.to_uppercase());
                });
            });
            if syncing {
                strip.cell(|ui| {
                    ui.centered_and_justified(|ui| {
                        let status_text = if node.state.is_restarting() {
                            "Restarting".to_string()
                        } else {
                            let sync_status = node.state.get_sync_status();
                            get_sync_status_text(sync_status.unwrap()).to_string()
                        };
                        let mut job = LayoutJob::single_section(status_text, TextFormat {
                            font_id: FontId::proportional(15.0),
                            color: COLOR_DARK,
                            .. Default::default()
                        });
                        job.wrap = TextWrapping {
                            max_rows: 1,
                            break_anywhere: false,
                            overflow_character: Option::from('…'),
                            ..Default::default()
                        };
                        ui.label(job);
                    });
                });
            }
        });
    }
}

fn get_sync_status_text(sync_status: SyncStatus) -> Cow<'static, str> {
    match sync_status {
        SyncStatus::Initial => Cow::Borrowed("Initializing"),
        SyncStatus::NoSync => Cow::Borrowed("Running"),
        SyncStatus::AwaitingPeers(_) => Cow::Borrowed("Waiting for peers"),
        SyncStatus::HeaderSync {
            sync_head,
            highest_height,
            ..
        } => {
            if highest_height == 0 {
                Cow::Borrowed("Downloading headers")
            } else {
                let percent = sync_head.height * 100 / highest_height;
                Cow::Owned(format!("Downloading headers: {}%", percent))
            }
        }
        SyncStatus::TxHashsetDownload(stat) => {
            if stat.total_size > 0 {
                let percent = stat.downloaded_size * 100 / stat.total_size;
                Cow::Owned(format!("Downloading chain state: {}%", percent))
            } else {
                Cow::Borrowed("Downloading chain state")
            }
        }
        SyncStatus::TxHashsetSetup => {
            Cow::Borrowed("Preparing state for validation")
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
            Cow::Owned(format!("Validating state - range proofs: {}%", r_percent))
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
            Cow::Owned(format!("Validating state - kernels: {}%", k_percent))
        }
        SyncStatus::TxHashsetSave => {
            Cow::Borrowed("Finalizing chain state")
        }
        SyncStatus::TxHashsetDone => {
            Cow::Borrowed("Finalized chain state")
        }
        SyncStatus::BodySync {
            current_height,
            highest_height,
        } => {
            if highest_height == 0 {
                Cow::Borrowed("Downloading blocks data")
            } else {
                Cow::Owned(format!(
                    "Downloading blocks: {}%",
                    current_height * 100 / highest_height
                ))
            }
        }
        SyncStatus::Shutdown => Cow::Borrowed("Shutting down"),
    }
}


