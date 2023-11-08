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

use egui::{Align, Id, Layout, Margin, RichText, Rounding, ScrollArea, Widget};
use grin_chain::SyncStatus;
use grin_core::core::amount_to_hr_string;

use crate::AppConfig;
use crate::gui::Colors;
use crate::gui::icons::{CHECK, CHECK_FAT, DOWNLOAD, FILE_ARCHIVE, GEAR_FINE, LIST, PACKAGE, PLUS, POWER, REPEAT, UPLOAD, WALLET};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, Root, View};
use crate::gui::views::types::{ModalPosition, TextEditOptions};
use crate::gui::views::wallets::{WalletInfo, WalletReceive, WalletSend, WalletSettings};
use crate::gui::views::wallets::types::{WalletTab, WalletTabType};
use crate::node::Node;
use crate::wallet::{Wallet, WalletConfig};
use crate::wallet::types::{GRIN, WalletAccount, WalletData};

/// Selected and opened wallet content.
pub struct WalletContent {
    /// List of wallet accounts for [`Modal`].
    accounts: Vec<WalletAccount>,

    /// Account label [`Modal`] value.
    account_label_edit: String,
    /// Flag to check if error occurred during account creation at [`Modal`].
    account_creation_error: bool,

    /// Current tab content to show.
    pub current_tab: Box<dyn WalletTab>,
}

impl Default for WalletContent {
    fn default() -> Self {
        Self {
            accounts: vec![],
            account_label_edit: "".to_string(),
            account_creation_error: false,
            current_tab: Box::new(WalletInfo::default())
        }
    }
}

/// Identifier for account creation [`Modal`].
const CREATE_ACCOUNT_MODAL: &'static str = "create_account_modal";
/// Identifier for account list [`Modal`].
const ACCOUNT_LIST_MODAL: &'static str = "account_list_modal";

impl WalletContent {
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              frame: &mut eframe::Frame,
              wallet: &mut Wallet,
              cb: &dyn PlatformCallbacks) {
        // Show modal content for this ui container.
        self.modal_content_ui(ui, wallet, cb);

        let data = wallet.get_data();
        let data_empty = data.is_none();

        // Show wallet balance panel when data is not empty and current tab is not Settings.
        let show_balance = self.current_tab.get_type() != WalletTabType::Settings && !data_empty;
        egui::TopBottomPanel::top("wallet_balance")
            .frame(egui::Frame {
                fill: Colors::FILL,
                stroke: View::DEFAULT_STROKE,
                inner_margin: Margin {
                    left: View::far_left_inset_margin(ui) + 4.0,
                    right: View::get_right_inset() + 4.0,
                    top: 4.0,
                    bottom: 0.0,
                },
                outer_margin: Margin {
                    left: 0.0,
                    right: 0.0,
                    top: 0.0,
                    bottom: -1.0,
                },
                ..Default::default()
            })
            .show_animated_inside(ui, show_balance, |ui| {
                ui.vertical_centered(|ui| {
                    // Draw wallet tabs.
                    View::max_width_ui(ui, Root::SIDE_PANEL_WIDTH * 1.35, |ui| {
                        self.account_ui(ui, wallet, data.unwrap(), cb);
                    });
                });
            });

        // Show wallet tabs panel.
        egui::TopBottomPanel::bottom("wallet_tabs")
            .frame(egui::Frame {
                stroke: View::DEFAULT_STROKE,
                fill: Colors::FILL,
                inner_margin: Margin {
                    left: View::far_left_inset_margin(ui) + 4.0,
                    right: View::get_right_inset() + 4.0,
                    top: 4.0,
                    bottom: View::get_bottom_inset() + 4.0,
                },
                ..Default::default()
            })
            .show_animated_inside(ui, !Self::block_navigation_on_sync(wallet), |ui| {
                ui.vertical_centered(|ui| {
                    // Draw wallet tabs.
                    View::max_width_ui(ui, Root::SIDE_PANEL_WIDTH * 1.3, |ui| {
                        self.tabs_ui(ui);
                    });
                });
            });

        // Show tab content panel.
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: Colors::WHITE,
                stroke: View::DEFAULT_STROKE,
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                self.current_tab.ui(ui, frame, wallet, cb);
            });

        // Refresh content after 1 second for synced wallet.
        if !data_empty {
            ui.ctx().request_repaint_after(Duration::from_millis(1000));
        } else {
            ui.ctx().request_repaint();
        }
    }

    /// Draw [`Modal`] content for this ui container.
    fn modal_content_ui(&mut self,
                        ui: &mut egui::Ui,
                        wallet: &mut Wallet,
                        cb: &dyn PlatformCallbacks) {
        match Modal::opened() {
            None => {}
            Some(id) => {
                match id {
                    CREATE_ACCOUNT_MODAL => {
                        Modal::ui(ui.ctx(), |ui, modal| {
                            self.create_account_modal_ui(ui, wallet, modal, cb);
                        });
                    },
                    ACCOUNT_LIST_MODAL => {
                        Modal::ui(ui.ctx(), |ui, modal| {
                            self.account_list_modal_ui(ui, wallet, modal);
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    /// Draw wallet account content.
    fn account_ui(&mut self,
                  ui: &mut egui::Ui,
                  wallet: &Wallet,
                  data: WalletData,
                  cb: &dyn PlatformCallbacks) {
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(75.0);
        // Draw round background.
        let rounding = View::item_rounding(0, 2, false);
        ui.painter().rect(rect, rounding, Colors::BUTTON, View::ITEM_STROKE);

        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            // Setup padding for item buttons.
            ui.style_mut().spacing.button_padding = egui::vec2(14.0, 0.0);

            // Draw button to add new account.
            View::item_button(ui, View::item_rounding(0, 2, true), PLUS, None, || {
                // Show account creation modal.
                Modal::new(CREATE_ACCOUNT_MODAL)
                    .position(ModalPosition::CenterTop)
                    .title(t!("wallets.create_account"))
                    .show();
                cb.show_keyboard();
            });

            // Draw button to show list of accounts.
            View::item_button(ui, Rounding::ZERO, LIST, None, || {
                // Load accounts.
                self.accounts = wallet.accounts();
                // Show account list modal.
                Modal::new(ACCOUNT_LIST_MODAL)
                    .position(ModalPosition::Center)
                    .title(t!("wallets.accounts"))
                    .show();
            });

            let layout_size = ui.available_size();
            ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                ui.add_space(6.0);
                ui.vertical(|ui| {
                    ui.add_space(3.0);
                    // Show spendable amount.
                    let amount = amount_to_hr_string(data.info.amount_currently_spendable, false);
                    let amount_text = format!("{} {}", amount, GRIN);
                    ui.label(RichText::new(amount_text).size(18.0).color(Colors::BLACK));
                    ui.add_space(-2.0);

                    // Show account label.
                    let account = wallet.get_config().account;
                    let default_acc_label = WalletConfig::DEFAULT_ACCOUNT_LABEL.to_string();
                    let acc_label = if account == default_acc_label {
                        t!("wallets.default_account")
                    } else {
                        account.to_owned()
                    };
                    let acc_text = format!("{} {}", FILE_ARCHIVE, acc_label);
                    View::ellipsize_text(ui, acc_text, 15.0, Colors::TEXT);

                    // Show confirmed height.
                    let height_text = format!("{} {}", PACKAGE, data.info.last_confirmed_height);
                    ui.label(RichText::new(height_text).size(15.0).color(Colors::GRAY));
                })
            });
        });
    }

    /// Draw account list [`Modal`] content.
    fn account_list_modal_ui(&mut self, ui: &mut egui::Ui, wallet: &mut Wallet, modal: &Modal) {
        ui.add_space(3.0);

        // Show list of accounts.
        let size = self.accounts.len();
        ScrollArea::vertical()
            .max_height(300.0)
            .id_source("account_list_modal_scroll")
            .auto_shrink([true; 2])
            .show_rows(ui, ACCOUNT_ITEM_HEIGHT, size, |ui, row_range| {
                for index in row_range {
                    // Add space before the first item.
                    if index == 0 {
                        ui.add_space(4.0);
                    }
                    let acc = self.accounts.get(index).unwrap();
                    account_item_ui(ui, modal, wallet, acc, index, size);
                    if index == size - 1 {
                        ui.add_space(4.0);
                    }
                }
            });

        ui.add_space(2.0);
        View::horizontal_line(ui, Colors::STROKE);
        ui.add_space(6.0);

        ui.vertical_centered_justified(|ui| {
            View::button(ui, t!("close"), Colors::WHITE, || {
                // Close modal.
                modal.close();
            });
            ui.add_space(6.0);
        });
    }

    /// Draw account creation [`Modal`] content.
    fn create_account_modal_ui(&mut self,
                               ui: &mut egui::Ui,
                               wallet: &mut Wallet,
                               modal: &Modal,
                               cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("wallets.name"))
                .size(17.0)
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Draw account name edit.
            let text_edit_id = Id::from(modal.id).with(wallet.get_config().id);
            let text_edit_opts = TextEditOptions::new(text_edit_id);
            View::text_edit(ui, cb, &mut self.account_label_edit, text_edit_opts);

            // Show error occurred during account creation..
            if self.account_creation_error {
                ui.add_space(12.0);
                ui.label(RichText::new(t!("error"))
                    .size(17.0)
                    .color(Colors::RED));
            }
            ui.add_space(12.0);
        });

        // Show modal buttons.
        ui.scope(|ui| {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::WHITE, || {
                        // Close modal.
                        cb.hide_keyboard();
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    // Create button callback.
                    let mut on_create = || {
                        if !self.account_label_edit.is_empty() {
                            let label = &self.account_label_edit;
                            match wallet.create_account(label) {
                                Ok(_) => {
                                    let _ = wallet.set_active_account(label);
                                    cb.hide_keyboard();
                                    modal.close();
                                },
                                Err(_) => self.account_creation_error = true
                            };
                        }
                    };

                    View::on_enter_key(ui, || {
                        (on_create)();
                    });

                    View::button(ui, t!("create"), Colors::WHITE, on_create);
                });
            });
            ui.add_space(6.0);
        });
    }

    /// Draw tab buttons in the bottom of the screen.
    fn tabs_ui(&mut self, ui: &mut egui::Ui) {
        ui.scope(|ui| {
            // Setup spacing between tabs.
            ui.style_mut().spacing.item_spacing = egui::vec2(4.0, 0.0);
            // Setup vertical padding inside tab button.
            ui.style_mut().spacing.button_padding = egui::vec2(0.0, 4.0);

            // Draw tab buttons.
            let current_type = self.current_tab.get_type();
            ui.columns(4, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::tab_button(ui, WALLET, current_type == WalletTabType::Info, || {
                        self.current_tab = Box::new(WalletInfo::default());
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::tab_button(ui, DOWNLOAD, current_type == WalletTabType::Receive, || {
                        self.current_tab = Box::new(WalletReceive::default());
                    });
                });
                columns[2].vertical_centered_justified(|ui| {
                    View::tab_button(ui, UPLOAD, current_type == WalletTabType::Send, || {
                        self.current_tab = Box::new(WalletSend::default());
                    });
                });
                columns[3].vertical_centered_justified(|ui| {
                    View::tab_button(ui, GEAR_FINE, current_type == WalletTabType::Settings, || {
                        self.current_tab = Box::new(WalletSettings::default());
                    });
                });
            });
        });
    }

    /// Draw content when wallet is syncing and not ready to use, returns `true` at this case.
    pub fn sync_ui(ui: &mut egui::Ui, frame: &mut eframe::Frame, wallet: &Wallet) -> bool {
        if wallet.is_repairing() && !wallet.sync_error() {
            Self::sync_progress_ui(ui, wallet);
            return true;
        } else if wallet.is_closing() {
            Self::sync_progress_ui(ui, wallet);
            return true;
        } else if wallet.get_current_ext_conn_id().is_none() {
            if !Node::is_running() || Node::is_stopping() {
                let dual_panel_root = Root::is_dual_panel_mode(frame);
                View::center_content(ui, 108.0, |ui| {
                    let text = t!("wallets.enable_node", "settings" => GEAR_FINE);
                    ui.label(RichText::new(text).size(16.0).color(Colors::INACTIVE_TEXT));
                    ui.add_space(8.0);
                    // Show button to enable integrated node at non-dual root panel mode
                    // or when network connections are not showing and node is not stopping
                    if (!dual_panel_root || AppConfig::show_connections_network_panel())
                        && !Node::is_stopping() {
                        let enable_node_text = format!("{} {}", POWER, t!("network.enable_node"));
                        View::button(ui, enable_node_text, Colors::GOLD, || {
                            Node::start();
                        });
                    }
                });
                return true
            } else if wallet.sync_error()
                && Node::get_sync_status() == Some(SyncStatus::NoSync) {
                Self::sync_error_ui(ui, wallet);
                return true;
            } else if wallet.get_data().is_none() {
                Self::sync_progress_ui(ui, wallet);
                return true;
            }
        } else if wallet.sync_error() {
            Self::sync_error_ui(ui, wallet);
            return true;
        } else if wallet.get_data().is_none() {
            Self::sync_progress_ui(ui, wallet);
            return true;
        }
        false
    }

    /// Draw wallet sync error content.
    fn sync_error_ui(ui: &mut egui::Ui, wallet: &Wallet) {
        View::center_content(ui, 108.0, |ui| {
            let text = t!("wallets.wallet_loading_err", "settings" => GEAR_FINE);
            ui.label(RichText::new(text).size(16.0).color(Colors::INACTIVE_TEXT));
            ui.add_space(8.0);
            let retry_text = format!("{} {}", REPEAT, t!("retry"));
            View::button(ui, retry_text, Colors::GOLD, || {
                wallet.set_sync_error(false);
            });
        });
    }

    /// Check when to block tabs navigation on sync progress.
    pub fn block_navigation_on_sync(wallet: &Wallet) -> bool {
        let sync_error = wallet.sync_error();
        let integrated_node = wallet.get_current_ext_conn_id().is_none();
        let integrated_node_ready = Node::get_sync_status() == Some(SyncStatus::NoSync);
        let sync_after_opening = wallet.get_data().is_none() && !wallet.sync_error();
        // Block navigation if wallet is repairing and integrated node is not launching
        // and if wallet is closing or syncing after opening when there is no data to show.
        (wallet.is_repairing() && (integrated_node_ready || !integrated_node) && !sync_error)
            || wallet.is_closing() || (sync_after_opening && !integrated_node)
    }

    /// Draw wallet sync progress content.
    pub fn sync_progress_ui(ui: &mut egui::Ui, wallet: &Wallet) {
        View::center_content(ui, 162.0, |ui| {
            View::big_loading_spinner(ui);
            ui.add_space(18.0);
            // Setup sync progress text.
            let text = {
                let integrated_node = wallet.get_current_ext_conn_id().is_none();
                let integrated_node_ready = Node::get_sync_status() == Some(SyncStatus::NoSync);
                let info_progress = wallet.info_sync_progress();

                if wallet.is_closing() {
                    t!("wallets.wallet_closing")
                } else if integrated_node && !integrated_node_ready {
                    t!("wallets.node_loading", "settings" => GEAR_FINE)
                } else if wallet.is_repairing() {
                    let repair_progress = wallet.repairing_progress();
                    if repair_progress == 0 {
                        t!("wallets.wallet_checking")
                    } else {
                        format!("{}: {}%", t!("wallets.wallet_checking"), repair_progress)
                    }
                } else if info_progress != 100 {
                    if info_progress == 0 {
                        t!("wallets.wallet_loading")
                    } else {
                        format!("{}: {}%", t!("wallets.wallet_loading"), info_progress)
                    }
                } else {
                    let tx_progress = wallet.txs_sync_progress();
                    if tx_progress == 0 {
                        t!("wallets.tx_loading")
                    } else {
                        format!("{}: {}%", t!("wallets.tx_loading"), tx_progress)
                    }
                }
            };
            ui.label(RichText::new(text).size(16.0).color(Colors::INACTIVE_TEXT));
        });
    }
}

const ACCOUNT_ITEM_HEIGHT: f32 = 75.0;

/// Draw account item.
fn account_item_ui(ui: &mut egui::Ui,
                   modal: &Modal,
                   wallet: &mut Wallet,
                   acc: &WalletAccount,
                   index: usize,
                   size: usize) {
    // Setup layout size.
    let mut rect = ui.available_rect_before_wrap();
    rect.set_height(ACCOUNT_ITEM_HEIGHT);

    // Draw round background.
    let bg_rect = rect.clone();
    let item_rounding = View::item_rounding(index, size, false);
    ui.painter().rect(bg_rect, item_rounding, Colors::FILL, View::ITEM_STROKE);

    ui.vertical(|ui| {
        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            // Draw button to select account.
            let is_current_account = wallet.get_config().account == acc.label;
            if !is_current_account {
                let button_rounding = View::item_rounding(index, size, true);
                View::item_button(ui, button_rounding, CHECK, None, || {
                    let _ = wallet.set_active_account(&acc.label);
                    modal.close();
                });
            } else {
                ui.add_space(12.0);
                ui.label(RichText::new(CHECK_FAT).size(20.0).color(Colors::GREEN));
            }

            let layout_size = ui.available_size();
            ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                ui.add_space(6.0);
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    // Show spendable amount.
                    let amount = amount_to_hr_string(acc.spendable_amount, false);
                    let amount_text = format!("{} {}", amount, GRIN);
                    ui.label(RichText::new(amount_text).size(18.0).color(Colors::BLACK));
                    ui.add_space(-2.0);

                    // Show account name.
                    let default_acc_label = WalletConfig::DEFAULT_ACCOUNT_LABEL.to_string();
                    let acc_label = if acc.label == default_acc_label {
                        t!("wallets.default_account")
                    } else {
                        acc.label.to_owned()
                    };
                    View::ellipsize_text(ui, acc_label, 15.0, Colors::TEXT);

                    // Show account BIP32 derivation path.
                    ui.label(RichText::new(acc.path.to_owned()).size(15.0).color(Colors::GRAY));
                    ui.add_space(3.0);
                });
            });
        });
    });
}