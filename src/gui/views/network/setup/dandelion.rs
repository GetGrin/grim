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

use egui::{Id, RichText};

use crate::gui::icons::{CLOCK_COUNTDOWN, GRAPH, TIMER, WATCH};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::types::{ModalContainer, ModalPosition};
use crate::gui::views::{Modal, TextEdit, View};
use crate::gui::Colors;
use crate::gui::views::network::NetworkSettings;
use crate::node::NodeConfig;

/// Dandelion server setup section content.
pub struct DandelionSetup {
    /// Epoch duration value in seconds.
    epoch_edit: String,

    /// Embargo expiration time value in seconds to fluff and broadcast if tx not seen on network.
    embargo_edit: String,

    /// Aggregation period value in seconds.
    aggregation_edit: String,

    /// Stem phase probability value (stem 90% of the time, fluff 10% of the time by default).
    stem_prob_edit: String,

    /// [`Modal`] identifiers allowed at this ui container.
    modal_ids: Vec<&'static str>,
}

/// Identifier epoch duration value [`Modal`].
pub const EPOCH_MODAL: &'static str = "epoch_secs";
/// Identifier for embargo expiration time value [`Modal`].
pub const EMBARGO_MODAL: &'static str = "embargo_secs";
/// Identifier for aggregation period value [`Modal`].
pub const AGGREGATION_MODAL: &'static str = "aggregation_secs";
/// Identifier for Stem phase probability value [`Modal`].
pub const STEM_PROBABILITY_MODAL: &'static str = "stem_probability";

impl Default for DandelionSetup {
    fn default() -> Self {
        Self {
            epoch_edit: NodeConfig::get_dandelion_epoch(),
            embargo_edit: NodeConfig::get_reorg_cache_period(),
            aggregation_edit: NodeConfig::get_dandelion_aggregation(),
            stem_prob_edit: NodeConfig::get_stem_probability(),
            modal_ids: vec![
                EPOCH_MODAL,
                EMBARGO_MODAL,
                AGGREGATION_MODAL,
                STEM_PROBABILITY_MODAL
            ]
        }
    }
}

impl ModalContainer for DandelionSetup {
    fn modal_ids(&self) -> &Vec<&'static str> {
        &self.modal_ids
    }

    fn modal_ui(&mut self,
                ui: &mut egui::Ui,
                modal: &Modal,
                cb: &dyn PlatformCallbacks) {
        match modal.id {
            EPOCH_MODAL => self.epoch_modal(ui, modal, cb),
            EMBARGO_MODAL => self.embargo_modal(ui, modal, cb),
            AGGREGATION_MODAL => self.aggregation_modal(ui, modal, cb),
            STEM_PROBABILITY_MODAL => self.stem_prob_modal(ui, modal, cb),
            _ => {}
        }
    }
}

impl DandelionSetup {
    pub fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        // Draw modal content for current ui container.
        self.current_modal_ui(ui, cb);

        View::sub_title(ui, format!("{} {}", GRAPH, "Dandelion"));
        View::horizontal_line(ui, Colors::stroke());
        ui.add_space(6.0);

        ui.vertical_centered(|ui| {
            // Show epoch duration setup.
            self.epoch_ui(ui);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(6.0);

            // Show embargo expiration time setup.
            self.embargo_ui(ui);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(6.0);

            // Show aggregation period setup.
            self.aggregation_ui(ui);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(6.0);

            // Show Stem phase probability setup.
            self.stem_prob_ui(ui);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(8.0);

            // Show setup to always stem our txs.
            let always_stem = NodeConfig::always_stem_our_txs();
            View::checkbox(ui, always_stem, t!("network_settings.stem_txs"), || {
                NodeConfig::toggle_always_stem_our_txs();
            });
            ui.add_space(6.0);
        });
    }

    /// Draw epoch duration setup content.
    fn epoch_ui(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(t!("network_settings.epoch_duration"))
            .size(16.0)
            .color(Colors::gray())
        );
        ui.add_space(6.0);

        let epoch = NodeConfig::get_dandelion_epoch();
        View::button(ui, format!("{} {}", WATCH, &epoch), Colors::white_or_black(false), || {
            // Setup values for modal.
            self.epoch_edit = epoch;
            // Show epoch setup modal.
            Modal::new(EPOCH_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"))
                .show();
        });
        ui.add_space(6.0);
    }

    /// Draw epoch duration [`Modal`] content.
    fn epoch_modal(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        // Save button callback.
        let on_save = |c: &mut DandelionSetup| {
            if let Ok(epoch) = c.epoch_edit.parse::<u16>() {
                NodeConfig::save_dandelion_epoch(epoch);
                Modal::close();
            }
        };

        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.epoch_duration"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Draw epoch text edit.
            let mut epoch_edit = TextEdit::new(Id::from(modal.id)).h_center().numeric();
            epoch_edit.ui(ui, &mut self.epoch_edit, cb);
            if epoch_edit.enter_pressed {
                on_save(self);
            }

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.epoch_edit.parse::<u16>().is_err() {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.not_valid_value"))
                    .size(17.0)
                    .color(Colors::red()));
            } else {
                NetworkSettings::node_restart_required_ui(ui);
            }
            ui.add_space(12.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                        // Close modal.
                        Modal::close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.save"), Colors::white_or_black(false), || {
                        on_save(self);
                    });
                });
            });
            ui.add_space(6.0);
        });
    }

    /// Draw embargo expiration time setup content.
    fn embargo_ui(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(t!("network_settings.embargo_timer"))
            .size(16.0)
            .color(Colors::gray())
        );
        ui.add_space(6.0);

        let embargo = NodeConfig::get_dandelion_embargo();
        View::button(ui, format!("{} {}", TIMER, &embargo), Colors::white_or_black(false), || {
            self.embargo_edit = embargo;
            // Show embargo setup modal.
            Modal::new(EMBARGO_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"))
                .show();
        });
        ui.add_space(6.0);
    }

    /// Draw epoch duration [`Modal`] content.
    fn embargo_modal(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        // Save button callback.
        let on_save = |c: &mut DandelionSetup| {
            if let Ok(embargo) = c.embargo_edit.parse::<u16>() {
                NodeConfig::save_dandelion_embargo(embargo);
                Modal::close();
            }
        };

        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.embargo_timer"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Draw embargo text edit.
            let mut embargo_edit = TextEdit::new(Id::from(modal.id)).h_center().numeric();
            embargo_edit.ui(ui, &mut self.embargo_edit, cb);
            if embargo_edit.enter_pressed {
                on_save(self);
            }

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.embargo_edit.parse::<u16>().is_err() {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.not_valid_value"))
                    .size(17.0)
                    .color(Colors::red()));
            } else {
                NetworkSettings::node_restart_required_ui(ui);
            }
            ui.add_space(12.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                        // Close modal.
                        Modal::close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.save"), Colors::white_or_black(false), || {
                        on_save(self);
                    });
                });
            });
            ui.add_space(6.0);
        });
    }

    /// Draw aggregation period setup content.
    fn aggregation_ui(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(t!("network_settings.aggregation_period"))
            .size(16.0)
            .color(Colors::gray())
        );
        ui.add_space(6.0);

        let ag = NodeConfig::get_dandelion_aggregation();
        View::button(ui, format!("{} {}", CLOCK_COUNTDOWN, &ag), Colors::white_or_black(false), || {
            // Setup values for modal.
            self.aggregation_edit = ag;
            // Show aggregation setup modal.
            Modal::new(AGGREGATION_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"))
                .show();
        });
        ui.add_space(6.0);
    }

    /// Draw aggregation period [`Modal`] content.
    fn aggregation_modal(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        // Save button callback.
        let on_save = |c: &mut DandelionSetup| {
            if let Ok(embargo) = c.aggregation_edit.parse::<u16>() {
                NodeConfig::save_dandelion_aggregation(embargo);
                Modal::close();
            }
        };

        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.aggregation_period"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Draw aggregation period text edit.
            let mut aggregation_edit = TextEdit::new(Id::from(modal.id)).h_center().numeric();
            aggregation_edit.ui(ui, &mut self.aggregation_edit, cb);
            if aggregation_edit.enter_pressed {
                on_save(self);
            }

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.aggregation_edit.parse::<u16>().is_err() {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.not_valid_value"))
                    .size(17.0)
                    .color(Colors::red()));
            } else {
                NetworkSettings::node_restart_required_ui(ui);
            }
            ui.add_space(12.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                        // Close modal.
                        Modal::close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.save"), Colors::white_or_black(false), || {
                        on_save(self);
                    });
                });
            });
            ui.add_space(6.0);
        });
    }

    /// Draw stem phase probability setup content.
    fn stem_prob_ui(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(t!("network_settings.stem_probability"))
            .size(16.0)
            .color(Colors::gray())
        );
        ui.add_space(6.0);

        let stem_prob = NodeConfig::get_stem_probability();
        View::button(ui, format!("{}%", &stem_prob), Colors::white_or_black(false), || {
            // Setup values for modal.
            self.stem_prob_edit = stem_prob;
            // Show stem probability setup modal.
            Modal::new(STEM_PROBABILITY_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"))
                .show();
        });
        ui.add_space(6.0);
    }

    /// Draw stem phase probability [`Modal`] content.
    fn stem_prob_modal(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        // Save button callback.
        let on_save = |c: &mut DandelionSetup| {
            if let Ok(prob) = c.stem_prob_edit.parse::<u8>() {
                NodeConfig::save_stem_probability(prob);
                Modal::close();
            }
        };

        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.stem_probability"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Draw stem phase probability text edit.
            let mut stem_prob_edit = TextEdit::new(Id::from(modal.id)).h_center().numeric();
            stem_prob_edit.ui(ui, &mut self.stem_prob_edit, cb);
            if stem_prob_edit.enter_pressed {
                on_save(self);
            }

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.stem_prob_edit.parse::<u8>().is_err() {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.not_valid_value"))
                    .size(17.0)
                    .color(Colors::red()));
            } else {
                NetworkSettings::node_restart_required_ui(ui);
            }
            ui.add_space(12.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                        // Close modal.
                        Modal::close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.save"), Colors::white_or_black(false), || {
                        on_save(self);
                    });
                });
            });
            ui.add_space(6.0);
        });
    }
}