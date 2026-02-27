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

use eframe::emath::Align;
use eframe::epaint::StrokeKind;
use egui::{Id, Layout, RichText};

use crate::gui::icons::{CLOCK_COUNTDOWN, FOLDERS, FOLDER_USER, PASSWORD, PENCIL};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::types::ModalPosition;
use crate::gui::views::wallets::wallet::types::WalletContentContainer;
use crate::gui::views::{FilePickContent, FilePickContentType, Modal, TextEdit, View};
use crate::gui::Colors;
use crate::wallet::Wallet;

/// Common wallet settings content.
pub struct CommonSettings {
    /// Wallet name [`Modal`] value.
    name_edit: String,

    /// Flag to check if wrong password was entered.
    wrong_pass: bool,
    /// Current wallet password [`Modal`] value.
    old_pass_edit: String,
    /// New wallet password [`Modal`] value.
    new_pass_edit: String,

    /// Data path value value for [`Modal`].
    data_path_edit: String,
    /// Button to pick directory for wallet data.
    pick_data_dir: FilePickContent,

    /// Minimum confirmations number value.
    min_confirmations_edit: String,
}

/// Identifier for wallet name [`Modal`].
const NAME_EDIT_MODAL: &'static str = "wallet_name_edit_modal";
/// Identifier for wallet password [`Modal`].
const PASS_EDIT_MODAL: &'static str = "wallet_pass_edit_modal";
/// Identifier for wallet data path value [`Modal`].
const DATA_PATH_MODAL: &'static str = "wallet_data_path";
/// Identifier for minimum confirmations [`Modal`].
const MIN_CONFIRMATIONS_EDIT_MODAL: &'static str = "wallet_min_conf_edit_modal";

impl WalletContentContainer for CommonSettings {
    fn modal_ids(&self) -> Vec<&'static str> {
        vec![
            NAME_EDIT_MODAL,
            PASS_EDIT_MODAL,
            DATA_PATH_MODAL,
            MIN_CONFIRMATIONS_EDIT_MODAL
        ]
    }

    fn modal_ui(&mut self,
                ui: &mut egui::Ui,
                wallet: &Wallet,
                modal: &Modal,
                cb: &dyn PlatformCallbacks) {
        match modal.id {
            NAME_EDIT_MODAL => self.name_modal_ui(ui, wallet, modal, cb),
            PASS_EDIT_MODAL => self.pass_modal_ui(ui, wallet, modal, cb),
            DATA_PATH_MODAL => self.data_path_modal_ui(ui, wallet, cb),
            MIN_CONFIRMATIONS_EDIT_MODAL => self.min_conf_modal_ui(ui, wallet, modal, cb),
            _ => {}
        }
    }

    fn container_ui(&mut self, ui: &mut egui::Ui, wallet: &Wallet, cb: &dyn PlatformCallbacks) {
        ui.add_space(8.0);
        ui.vertical_centered(|ui| {
            let config = wallet.get_config();
            // Show wallet name.
            self.name_ui(ui, config.name);
            // Show data dir for desktop.
            if View::is_desktop() {
                ui.add_space(-4.0);
                self.data_dir_ui(ui, wallet, cb);
            }
            ui.add_space(6.0);
            ui.label(RichText::new(t!("wallets.min_tx_conf_count")).size(16.0).color(Colors::gray()));
            ui.add_space(6.0);

            // Show minimum amount of confirmations value setup.
            let min_conf_text = format!("{} {}", CLOCK_COUNTDOWN, config.min_confirmations);
            View::button(ui, min_conf_text, Colors::white_or_black(false), || {
                self.min_confirmations_edit = config.min_confirmations.to_string();
                // Show minimum amount of confirmations value modal.
                Modal::new(MIN_CONFIRMATIONS_EDIT_MODAL)
                    .position(ModalPosition::CenterTop)
                    .title(t!("network_settings.change_value"))
                    .show();
            });

            ui.add_space(8.0);

            // Setup ability to post wallet transactions with Dandelion.
            View::checkbox(ui, wallet.can_use_dandelion(), t!("wallets.use_dandelion"), || {
                wallet.update_use_dandelion(!wallet.can_use_dandelion());
            });

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::stroke());
            ui.add_space(4.0);
        });
    }
}

impl Default for CommonSettings {
    fn default() -> Self {
        Self {
            name_edit: "".to_string(),
            wrong_pass: false,
            old_pass_edit: "".to_string(),
            new_pass_edit: "".to_string(),
            data_path_edit: "".to_string(),
            pick_data_dir: FilePickContent::new(
                FilePickContentType::ItemButton(View::item_rounding(1, 2, true))
            ).no_parse().pick_folder(),
            min_confirmations_edit: "".to_string(),
        }
    }
}

impl CommonSettings {
    /// Draw content to change wallet name and password.
    fn name_ui(&mut self, ui: &mut egui::Ui, name: String) {
        // Setup layout size.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(56.0);

        // Draw round background.
        let bg_rect = rect.clone();
        let item_rounding = if View::is_desktop() {
            View::item_rounding(0, 2, false)
        } else {
            View::item_rounding(0, 1, false)
        };
        ui.painter().rect(bg_rect,
                          item_rounding,
                          Colors::fill(),
                          View::item_stroke(),
                          StrokeKind::Outside);

        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            View::item_button(ui, View::item_rounding(0, 2, true), PASSWORD, None, || {
                self.old_pass_edit = "".to_string();
                self.new_pass_edit = "".to_string();
                self.wrong_pass = false;
                // Show wallet password modal.
                Modal::new(PASS_EDIT_MODAL)
                    .position(ModalPosition::CenterTop)
                    .title(t!("wallets.wallet"))
                    .show();
            });
            View::item_button(ui, View::item_rounding(1, 3, true), PENCIL, None, || {
                self.name_edit = name.clone();
                // Show wallet name modal.
                Modal::new(NAME_EDIT_MODAL)
                    .position(ModalPosition::CenterTop)
                    .title(t!("wallets.wallet"))
                    .show();
            });
            let layout_size = ui.available_size();
            ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    View::ellipsize_text(ui, name, 18.0, Colors::title(false));
                    ui.add_space(1.0);
                    let desc = format!("{} {}", FOLDER_USER, t!("wallets.name").replace(":", ""));
                    ui.label(RichText::new(desc).size(15.0).color(Colors::gray()));
                    ui.add_space(8.0);
                });
            });
        });
    }

    /// Draw wallet name [`Modal`] content.
    fn name_modal_ui(&mut self,
                     ui: &mut egui::Ui,
                     wallet: &Wallet,
                     modal: &Modal,
                     cb: &dyn PlatformCallbacks) {
        let on_save = |c: &mut CommonSettings| {
            if !c.name_edit.is_empty() {
                wallet.change_name(c.name_edit.clone());
                Modal::close();
            }
        };

        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("wallets.name"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);
            // Show wallet name text edit.
            let mut name_edit = TextEdit::new(Id::from(modal.id).with(wallet.get_config().id));
            name_edit.ui(ui, &mut self.name_edit, cb);
            if name_edit.enter_pressed {
                on_save(self);
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

    /// Draw wallet pass [`Modal`] content.
    fn pass_modal_ui(&mut self,
                     ui: &mut egui::Ui,
                     wallet: &Wallet,
                     modal: &Modal,
                     cb: &dyn PlatformCallbacks) {
        let wallet_id = wallet.get_config().id;
        let on_continue = |c: &mut CommonSettings| {
            if c.new_pass_edit.is_empty() {
                return;
            }
            let old_pass = c.old_pass_edit.clone();
            let new_pass = c.new_pass_edit.clone();
            match wallet.change_password(old_pass, new_pass) {
                Ok(_) => {
                    // Clear password values.
                    c.old_pass_edit = "".to_string();
                    c.new_pass_edit = "".to_string();
                    // Close modal.
                    Modal::close();
                }
                Err(_) => c.wrong_pass = true
            }
        };

        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("wallets.current_pass"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Draw old password text edit.
            let pass_edit_id = Id::from(modal.id).with(wallet_id).with("old_pass");
            let mut pass_edit = TextEdit::new(pass_edit_id)
                .password()
                .focus(Modal::first_draw());
            pass_edit.ui(ui, &mut self.old_pass_edit, cb);
            ui.add_space(8.0);

            ui.label(RichText::new(t!("wallets.new_pass"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Draw new password text edit.
            let new_pass_edit_id = Id::from(modal.id).with(wallet_id).with("new_pass");
            let mut new_pass_edit = TextEdit::new(new_pass_edit_id)
                .password()
                .focus(false);
            if pass_edit.enter_pressed {
                new_pass_edit.focus_request();
            }
            new_pass_edit.ui(ui, &mut self.new_pass_edit, cb);
            if new_pass_edit.enter_pressed {
                on_continue(self);
            }

            // Show information when password is empty.
            if self.old_pass_edit.is_empty() || self.new_pass_edit.is_empty() {
                ui.add_space(10.0);
                ui.label(RichText::new(t!("wallets.pass_empty"))
                    .size(17.0)
                    .color(Colors::inactive_text()));
            } else if self.wrong_pass {
                ui.add_space(10.0);
                ui.label(RichText::new(t!("wallets.wrong_pass"))
                    .size(17.0)
                    .color(Colors::red()));
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
                    View::button(ui, t!("change"), Colors::white_or_black(false), || {
                        on_continue(self);
                    });
                });
            });
            ui.add_space(6.0);
        });
    }

    /// Draw content to change wallet data directory.
    fn data_dir_ui(&mut self, ui: &mut egui::Ui, wallet: &Wallet, cb: &dyn PlatformCallbacks) {
        // Setup layout size.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(56.0);

        // Draw round background.
        let bg_rect = rect.clone();
        let item_rounding = View::item_rounding(1, 2, false);
        ui.painter().rect(bg_rect,
                          item_rounding,
                          Colors::fill(),
                          View::item_stroke(),
                          StrokeKind::Outside);

        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            self.pick_data_dir.ui(ui, cb, |path| {
                wallet.change_data_path(path);
            });
            View::item_button(ui, View::item_rounding(1, 3, true), PENCIL, None, || {
                self.data_path_edit = wallet.get_config().data_path.unwrap_or_default();
                // Show chain data path edit modal.
                Modal::new(DATA_PATH_MODAL)
                    .position(ModalPosition::CenterTop)
                    .title(t!("wallets.wallet"))
                    .show();
            });
            let layout_size = ui.available_size();
            ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    let path = wallet.get_config().data_path.unwrap_or_default();
                    View::ellipsize_text(ui, path, 18.0, Colors::title(false));
                    ui.add_space(1.0);
                    let desc = format!("{} {}", FOLDERS, t!("files_location"));
                    ui.label(RichText::new(desc).size(15.0).color(Colors::gray()));
                    ui.add_space(8.0);
                });
            });
        });
    }

    /// Draw data path input [`Modal`] content.
    fn data_path_modal_ui(&mut self,
                          ui: &mut egui::Ui,
                          wallet: &Wallet,
                          cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            let on_save = |path: &String| {
                wallet.change_data_path(path.clone());
                Modal::close();
            };
            ui.label(RichText::new(format!("{}:", t!("files_location")))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Draw chain data path text edit.
            let mut edit = TextEdit::new(Id::from(DATA_PATH_MODAL)).paste();
            edit.ui(ui, &mut self.data_path_edit, cb);
            if edit.enter_pressed {
                on_save(&self.data_path_edit);
            }
            ui.add_space(12.0);

            // Show modal buttons.
            ui.scope(|ui| {
                // Setup spacing between buttons.
                ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

                ui.columns(2, |columns| {
                    columns[0].vertical_centered_justified(|ui| {
                        View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                            Modal::close();
                        });
                    });
                    columns[1].vertical_centered_justified(|ui| {
                        View::button(ui, t!("modal.save"), Colors::white_or_black(false), || {
                            on_save(&self.data_path_edit);
                        });
                    });
                });
                ui.add_space(6.0);
            });
        });
    }

    /// Draw wallet name [`Modal`] content.
    fn min_conf_modal_ui(&mut self,
                         ui: &mut egui::Ui,
                         wallet: &Wallet,
                         modal: &Modal,
                         cb: &dyn PlatformCallbacks) {
        let on_save = |c: &mut CommonSettings| {
            if let Ok(min_conf) = c.min_confirmations_edit.parse::<u64>() {
                wallet.update_min_confirmations(min_conf);
                Modal::close();
            }
        };

        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("wallets.min_tx_conf_count"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Minimum amount of confirmations text edit.
            let mut min_confirmations_edit = TextEdit::new(Id::from(modal.id)).h_center().numeric();
            min_confirmations_edit.ui(ui, &mut self.min_confirmations_edit, cb);
            if min_confirmations_edit.enter_pressed {
                on_save(self);
            }

            // Show error when specified value is not valid or reminder to restart enabled node.
            if self.min_confirmations_edit.parse::<u64>().is_err() {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("network_settings.not_valid_value"))
                    .size(17.0)
                    .color(Colors::red()));
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