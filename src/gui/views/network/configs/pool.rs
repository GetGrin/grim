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

use crate::gui::{Colors, Navigator};
use crate::gui::icons::{BEZIER_CURVE, BOUNDING_BOX, CHART_SCATTER, CIRCLES_THREE, CLOCK_COUNTDOWN, HAND_COINS};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, ModalPosition, View};
use crate::gui::views::network::settings::NetworkSettings;
use crate::node::NodeConfig;

/// Memory pool setup ui section.
pub struct PoolSetup {
    /// Base fee value that's accepted into the pool.
    fee_base_edit: String,

    /// Reorg cache retention period value in minutes.
    reorg_period_edit: String,

    /// Maximum number of transactions allowed in the pool.
    pool_size_edit: String,

    /// Maximum number of transactions allowed in the stempool.
    stempool_size_edit: String,

    /// Maximum total weight of transactions to build a block.
    max_weight_edit: String,
}

impl Default for PoolSetup {
    fn default() -> Self {
        Self {
            fee_base_edit: NodeConfig::get_base_fee(),
            reorg_period_edit: NodeConfig::get_reorg_cache_period(),
            pool_size_edit: NodeConfig::get_max_pool_size(),
            stempool_size_edit: NodeConfig::get_max_stempool_size(),
            max_weight_edit: NodeConfig::get_mineable_max_weight(),
        }
    }
}

impl PoolSetup {
    /// Identifier for base fee value [`Modal`].
    pub const FEE_BASE_MODAL: &'static str = "fee_base";
    /// Identifier for reorg cache retention period value [`Modal`].
    pub const REORG_PERIOD_MODAL: &'static str = "reorg_period";
    /// Identifier for maximum number of transactions in the pool [`Modal`].
    pub const POOL_SIZE_MODAL: &'static str = "pool_size";
    /// Identifier for maximum number of transactions in the stempool [`Modal`].
    pub const STEMPOOL_SIZE_MODAL: &'static str = "stempool_size";
    /// Identifier for maximum total weight of transactions [`Modal`].
    pub const MAX_WEIGHT_MODAL: &'static str = "max_weight";

    pub fn ui(&mut self, ui: &mut Ui, cb: &dyn PlatformCallbacks) {
        View::sub_title(ui, format!("{} {}", CHART_SCATTER, t!("network_settings.tx_pool")));
        View::horizontal_line(ui, Colors::STROKE);
        ui.add_space(6.0);

        ui.vertical_centered(|ui| {
            // Show base fee setup.
            self.fee_base_ui(ui, cb);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);

            // Show reorg cache retention period setup.
            self.reorg_period_ui(ui, cb);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);

            // Show pool size setup.
            self.pool_size_ui(ui, cb);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);

            // Show stem pool size setup.
            self.stempool_size_ui(ui, cb);

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(6.0);

            // Show max weight of transactions setup.
            self.max_weight_ui(ui, cb);
        });
    }

    /// Draw fee base setup content.
    fn fee_base_ui(&mut self, ui: &mut Ui, cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("network_settings.pool_fee"))
            .size(16.0)
            .color(Colors::GRAY)
        );
        ui.add_space(6.0);

        let fee = NodeConfig::get_base_fee();
        View::button(ui, format!("{} {}", HAND_COINS, fee.clone()), Colors::BUTTON, || {
            // Setup values for modal.
            self.fee_base_edit = fee;
            // Show fee setup modal.
            let fee_modal = Modal::new(Self::FEE_BASE_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"));
            Navigator::show_modal(fee_modal);
            cb.show_keyboard();
        });
        ui.add_space(6.0);
    }

    /// Draw fee base [`Modal`] content.
    pub fn fee_base_modal(&mut self, ui: &mut Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.pool_fee"))
                .size(18.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw fee base text edit.
            let text_edit_resp = egui::TextEdit::singleline(&mut self.fee_base_edit)
                .id(Id::from(modal.id))
                .font(TextStyle::Heading)
                .desired_width(84.0)
                .cursor_at_end(true)
                .ui(ui);
            text_edit_resp.request_focus();
            if text_edit_resp.clicked() {
                cb.show_keyboard();
            }

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.fee_base_edit.parse::<u64>().is_err() {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.not_valid_value"))
                    .size(18.0)
                    .color(Colors::RED));
            } else {
                NetworkSettings::node_restart_required_ui(ui);
            }
            ui.add_space(12.0);

            // Show modal buttons.
            ui.scope(|ui| {
                // Setup spacing between buttons.
                ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

                // Save button callback.
                let on_save = || {
                    if let Ok(fee) = self.fee_base_edit.parse::<u64>() {
                        NodeConfig::save_base_fee(fee);
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
        });
    }

    /// Draw reorg cache retention period setup content.
    fn reorg_period_ui(&mut self, ui: &mut Ui, cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("network_settings.reorg_period"))
            .size(16.0)
            .color(Colors::GRAY)
        );
        ui.add_space(6.0);

        let period = NodeConfig::get_reorg_cache_period();
        View::button(ui, format!("{} {}", CLOCK_COUNTDOWN, period.clone()), Colors::BUTTON, || {
            // Setup values for modal.
            self.reorg_period_edit = period;
            // Show reorg period setup modal.
            let reorg_modal = Modal::new(Self::REORG_PERIOD_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"));
            Navigator::show_modal(reorg_modal);
            cb.show_keyboard();
        });
        ui.add_space(6.0);
    }

    /// Draw reorg cache retention period [`Modal`] content.
    pub fn reorg_period_modal(&mut self, ui: &mut Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.reorg_period"))
                .size(18.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw reorg period text edit.
            let text_edit_resp = egui::TextEdit::singleline(&mut self.reorg_period_edit)
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
            if self.reorg_period_edit.parse::<u32>().is_err() {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.reorg_period"))
                    .size(18.0)
                    .color(Colors::RED));
            } else {
                NetworkSettings::node_restart_required_ui(ui);
            }
            ui.add_space(12.0);

            // Show modal buttons.
            ui.scope(|ui| {
                // Setup spacing between buttons.
                ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

                // Save button callback.
                let on_save = || {
                    if let Ok(period) = self.reorg_period_edit.parse::<u32>() {
                        NodeConfig::save_reorg_cache_period(period);
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
        });
    }

    /// Draw maximum number of transactions in the pool setup content.
    fn pool_size_ui(&mut self, ui: &mut Ui, cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("network_settings.max_tx_pool"))
            .size(16.0)
            .color(Colors::GRAY)
        );
        ui.add_space(6.0);

        let size = NodeConfig::get_max_pool_size();
        View::button(ui, format!("{} {}", CIRCLES_THREE, size.clone()), Colors::BUTTON, || {
            // Setup values for modal.
            self.pool_size_edit = size;
            // Show pool size setup modal.
            let size_modal = Modal::new(Self::POOL_SIZE_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"));
            Navigator::show_modal(size_modal);
            cb.show_keyboard();
        });
        ui.add_space(6.0);
    }

    /// Draw maximum number of transactions in the pool [`Modal`] content.
    pub fn pool_size_modal(&mut self, ui: &mut Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.max_tx_pool"))
                .size(18.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw pool size text edit.
            let text_edit_resp = egui::TextEdit::singleline(&mut self.pool_size_edit)
                .id(Id::from(modal.id))
                .font(TextStyle::Heading)
                .desired_width(72.0)
                .cursor_at_end(true)
                .ui(ui);
            text_edit_resp.request_focus();
            if text_edit_resp.clicked() {
                cb.show_keyboard();
            }

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.pool_size_edit.parse::<usize>().is_err() {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.not_valid_value"))
                    .size(18.0)
                    .color(Colors::RED));
            } else {
                NetworkSettings::node_restart_required_ui(ui);
            }
            ui.add_space(12.0);

            // Show modal buttons.
            ui.scope(|ui| {
                // Setup spacing between buttons.
                ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

                // Save button callback.
                let on_save = || {
                    if let Ok(size) = self.pool_size_edit.parse::<usize>() {
                        NodeConfig::save_max_pool_size(size);
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
        });
    }

    /// Draw maximum number of transactions in the stempool setup content.
    fn stempool_size_ui(&mut self, ui: &mut Ui, cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("network_settings.max_tx_stempool"))
            .size(16.0)
            .color(Colors::GRAY)
        );
        ui.add_space(6.0);

        let size = NodeConfig::get_max_stempool_size();
        View::button(ui, format!("{} {}", BEZIER_CURVE, size.clone()), Colors::BUTTON, || {
            // Setup values for modal.
            self.stempool_size_edit = size;
            // Show stempool size setup modal.
            let stem_modal = Modal::new(Self::STEMPOOL_SIZE_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"));
            Navigator::show_modal(stem_modal);
            cb.show_keyboard();
        });
        ui.add_space(6.0);
    }

    /// Draw maximum number of transactions in the stempool [`Modal`] content.
    pub fn stempool_size_modal(&mut self, ui: &mut Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.max_tx_stempool"))
                .size(18.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw stempool size text edit.
            let text_edit_resp = egui::TextEdit::singleline(&mut self.stempool_size_edit)
                .id(Id::from(modal.id))
                .font(TextStyle::Heading)
                .desired_width(72.0)
                .cursor_at_end(true)
                .ui(ui);
            text_edit_resp.request_focus();
            if text_edit_resp.clicked() {
                cb.show_keyboard();
            }

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.stempool_size_edit.parse::<usize>().is_err() {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.not_valid_value"))
                    .size(18.0)
                    .color(Colors::RED));
            } else {
                NetworkSettings::node_restart_required_ui(ui);
            }
            ui.add_space(12.0);

            // Show modal buttons.
            ui.scope(|ui| {
                // Setup spacing between buttons.
                ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

                // Save button callback.
                let on_save = || {
                    if let Ok(size) = self.stempool_size_edit.parse::<usize>() {
                        NodeConfig::save_max_stempool_size(size);
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
        });
    }

    /// Draw maximum total weight of transactions setup content.
    fn max_weight_ui(&mut self, ui: &mut Ui, cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(t!("network_settings.max_tx_weight"))
            .size(16.0)
            .color(Colors::GRAY)
        );
        ui.add_space(6.0);

        let weight = NodeConfig::get_mineable_max_weight();
        View::button(ui, format!("{} {}", BOUNDING_BOX, weight.clone()), Colors::BUTTON, || {
            // Setup values for modal.
            self.max_weight_edit = weight;
            // Show total tx weight setup modal.
            let weight_modal = Modal::new(Self::MAX_WEIGHT_MODAL)
                .position(ModalPosition::CenterTop)
                .title(t!("network_settings.change_value"));
            Navigator::show_modal(weight_modal);
            cb.show_keyboard();
        });
        ui.add_space(6.0);
    }

    /// Draw maximum total weight of transactions [`Modal`] content.
    pub fn max_weight_modal(&mut self, ui: &mut Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("network_settings.max_tx_weight"))
                .size(18.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw tx weight text edit.
            let text_edit_resp = egui::TextEdit::singleline(&mut self.max_weight_edit)
                .id(Id::from(modal.id))
                .font(TextStyle::Heading)
                .desired_width(72.0)
                .cursor_at_end(true)
                .ui(ui);
            text_edit_resp.request_focus();
            if text_edit_resp.clicked() {
                cb.show_keyboard();
            }

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.max_weight_edit.parse::<u64>().is_err() {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.not_valid_value"))
                    .size(18.0)
                    .color(Colors::RED));
            } else {
                NetworkSettings::node_restart_required_ui(ui);
            }
            ui.add_space(12.0);

            // Show modal buttons.
            ui.scope(|ui| {
                // Setup spacing between buttons.
                ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

                // Save button callback.
                let on_save = || {
                    if let Ok(weight) = self.max_weight_edit.parse::<u64>() {
                        NodeConfig::save_mineable_max_weight(weight);
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
        });
    }
}