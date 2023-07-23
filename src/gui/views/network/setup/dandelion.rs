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

use egui::{Id, RichText, TextStyle, Ui, Widget};

use crate::gui::Colors;
use crate::gui::icons::{CLOCK_COUNTDOWN, GRAPH, TIMER, WATCH};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, ModalPosition, View};
use crate::gui::views::network::settings::NetworkSettings;
use crate::node::NodeConfig;

/// Dandelion setup ui section.
pub struct DandelionSetup {
    /// Epoch duration value in seconds.
    epoch_edit: String,

    /// Embargo expiration time value in seconds to fluff and broadcast if tx not seen on network.
    embargo_edit: String,

    /// Aggregation period value in seconds.
    aggregation_edit: String,

    /// Stem phase probability value (stem 90% of the time, fluff 10% of the time by default).
    stem_prob_edit: String,
}

impl Default for DandelionSetup {
    fn default() -> Self {
        Self {
            epoch_edit: NodeConfig::get_dandelion_epoch(),
            embargo_edit: NodeConfig::get_reorg_cache_period(),
            aggregation_edit: NodeConfig::get_dandelion_aggregation(),
            stem_prob_edit: NodeConfig::get_stem_probability()
        }
    }
}

impl DandelionSetup {
    /// Identifier epoch duration value [`Modal`].
    pub const EPOCH_MODAL: &'static str = "epoch_secs";
    /// Identifier for embargo expiration time value [`Modal`].
    pub const EMBARGO_MODAL: &'static str = "embargo_secs";
    /// Identifier for aggregation period value [`Modal`].
    pub const AGGREGATION_MODAL: &'static str = "aggregation_secs";
    /// Identifier for Stem phase probability value [`Modal`].
    pub const STEM_PROBABILITY_MODAL: &'static str = "stem_probability";

    pub fn ui(&mut self, ui: &mut Ui, cb: &dyn PlatformCallbacks) {
        View::sub_title(ui, format!("{} {}", GRAPH, "Dandelion"));
        View::horizontal_line(ui, Colors::STROKE);
        ui.add_space(6.0);

        ui.vertical_centered(|ui| {
            // Show epoch duration setup.
            self.epoch_ui(ui, cb);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);

            // Show embargo expiration time setup.
            self.embargo_ui(ui, cb);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);

            // Show aggregation period setup.
            self.aggregation_ui(ui, cb);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);

            // Show Stem phase probability setup.
            self.stem_prob_ui(ui, cb);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
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
    fn epoch_ui(&mut self, ui: &mut Ui, cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("network_settings.epoch_duration"))
            .size(16.0)
            .color(Colors::GRAY)
        );
        ui.add_space(6.0);

        let epoch = NodeConfig::get_dandelion_epoch();
        View::button(ui, format!("{} {}", WATCH, epoch.clone()), Colors::BUTTON, || {
            // Setup values for modal.
            self.epoch_edit = epoch;
            // Show epoch setup modal.
            Modal::new(Self::EPOCH_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"))
                .show();
            cb.show_keyboard();
        });
        ui.add_space(6.0);
    }

    /// Draw epoch duration [`Modal`] content.
    pub fn epoch_modal(&mut self, ui: &mut Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.epoch_duration"))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw epoch text edit.
            let text_edit_resp = egui::TextEdit::singleline(&mut self.epoch_edit)
                .id(Id::from(modal.id))
                .font(TextStyle::Heading)
                .desired_width(52.0)
                .cursor_at_end(true)
                .ui(ui);
            text_edit_resp.request_focus();
            if text_edit_resp.clicked() {
                cb.show_keyboard();
            }

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.epoch_edit.parse::<u16>().is_err() {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.not_valid_value"))
                    .size(17.0)
                    .color(Colors::RED));
            } else {
                NetworkSettings::node_restart_required_ui(ui);
            }
            ui.add_space(12.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            // Save button callback.
            let on_save = || {
                if let Ok(epoch) = self.epoch_edit.parse::<u16>() {
                    NodeConfig::save_dandelion_epoch(epoch);
                    cb.hide_keyboard();
                    modal.close();
                }
            };

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::WHITE, || {
                        // Close modal.
                        cb.hide_keyboard();
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.save"), Colors::WHITE, on_save);
                });
            });
            ui.add_space(6.0);
        });
    }

    /// Draw embargo expiration time setup content.
    fn embargo_ui(&mut self, ui: &mut Ui, cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("network_settings.embargo_timer"))
            .size(16.0)
            .color(Colors::GRAY)
        );
        ui.add_space(6.0);

        let embargo = NodeConfig::get_dandelion_embargo();
        View::button(ui, format!("{} {}", TIMER, embargo.clone()), Colors::BUTTON, || {
            // Setup values for modal.
            self.embargo_edit = embargo;
            // Show embargo setup modal.
            Modal::new(Self::EMBARGO_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"))
                .show();
        });
        ui.add_space(6.0);
    }

    /// Draw epoch duration [`Modal`] content.
    pub fn embargo_modal(&mut self, ui: &mut Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.embargo_timer"))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw embargo text edit.
            let text_edit_resp = egui::TextEdit::singleline(&mut self.embargo_edit)
                .id(Id::from(modal.id))
                .font(TextStyle::Heading)
                .desired_width(52.0)
                .cursor_at_end(true)
                .ui(ui);
            text_edit_resp.request_focus();
            if text_edit_resp.clicked() {
                cb.show_keyboard();
            }

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.embargo_edit.parse::<u16>().is_err() {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.not_valid_value"))
                    .size(17.0)
                    .color(Colors::RED));
            } else {
                NetworkSettings::node_restart_required_ui(ui);
            }
            ui.add_space(12.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            // Save button callback.
            let on_save = || {
                if let Ok(embargo) = self.embargo_edit.parse::<u16>() {
                    NodeConfig::save_dandelion_embargo(embargo);
                    cb.hide_keyboard();
                    modal.close();
                }
            };

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::WHITE, || {
                        // Close modal.
                        cb.hide_keyboard();
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.save"), Colors::WHITE, on_save);
                });
            });
            ui.add_space(6.0);
        });
    }

    /// Draw aggregation period setup content.
    fn aggregation_ui(&mut self, ui: &mut Ui, cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("network_settings.aggregation_period"))
            .size(16.0)
            .color(Colors::GRAY)
        );
        ui.add_space(6.0);

        let agg = NodeConfig::get_dandelion_aggregation();
        View::button(ui, format!("{} {}", CLOCK_COUNTDOWN, agg.clone()), Colors::BUTTON, || {
            // Setup values for modal.
            self.aggregation_edit = agg;
            // Show aggregation setup modal.
            Modal::new(Self::AGGREGATION_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"))
                .show();
            cb.show_keyboard();
        });
        ui.add_space(6.0);
    }

    /// Draw aggregation period [`Modal`] content.
    pub fn aggregation_modal(&mut self, ui: &mut Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.aggregation_period"))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw aggregation period text edit.
            let text_edit_resp = egui::TextEdit::singleline(&mut self.aggregation_edit)
                .id(Id::from(modal.id))
                .font(TextStyle::Heading)
                .desired_width(42.0)
                .cursor_at_end(true)
                .ui(ui);
            text_edit_resp.request_focus();
            if text_edit_resp.clicked() {
                cb.show_keyboard();
            }

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.aggregation_edit.parse::<u16>().is_err() {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.not_valid_value"))
                    .size(17.0)
                    .color(Colors::RED));
            } else {
                NetworkSettings::node_restart_required_ui(ui);
            }
            ui.add_space(12.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            // Save button callback.
            let on_save = || {
                if let Ok(embargo) = self.aggregation_edit.parse::<u16>() {
                    NodeConfig::save_dandelion_aggregation(embargo);
                    cb.hide_keyboard();
                    modal.close();
                }
            };

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::WHITE, || {
                        // Close modal.
                        cb.hide_keyboard();
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.save"), Colors::WHITE, on_save);
                });
            });
            ui.add_space(6.0);
        });
    }

    /// Draw stem phase probability setup content.
    fn stem_prob_ui(&mut self, ui: &mut Ui, cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("network_settings.stem_probability"))
            .size(16.0)
            .color(Colors::GRAY)
        );
        ui.add_space(6.0);

        let stem_prob = NodeConfig::get_stem_probability();
        View::button(ui, format!("{}%", stem_prob.clone()), Colors::BUTTON, || {
            // Setup values for modal.
            self.stem_prob_edit = stem_prob;
            // Show stem probability setup modal.
            Modal::new(Self::STEM_PROBABILITY_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"))
                .show();
            cb.show_keyboard();
        });
        ui.add_space(6.0);
    }

    /// Draw stem phase probability [`Modal`] content.
    pub fn stem_prob_modal(&mut self, ui: &mut Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.stem_probability"))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw stem phase probability text edit.
            let text_edit_resp = egui::TextEdit::singleline(&mut self.stem_prob_edit)
                .id(Id::from(modal.id))
                .font(TextStyle::Heading)
                .desired_width(42.0)
                .cursor_at_end(true)
                .ui(ui);
            text_edit_resp.request_focus();
            if text_edit_resp.clicked() {
                cb.show_keyboard();
            }

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.stem_prob_edit.parse::<u8>().is_err() {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.not_valid_value"))
                    .size(17.0)
                    .color(Colors::RED));
            } else {
                NetworkSettings::node_restart_required_ui(ui);
            }
            ui.add_space(12.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            // Save button callback.
            let on_save = || {
                if let Ok(prob) = self.stem_prob_edit.parse::<u8>() {
                    NodeConfig::save_stem_probability(prob);
                    cb.hide_keyboard();
                    modal.close();
                }
            };

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::WHITE, || {
                        // Close modal.
                        cb.hide_keyboard();
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.save"), Colors::WHITE, on_save);
                });
            });
            ui.add_space(6.0);
        });
    }
}