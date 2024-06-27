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
use egui::{Align, Id, Layout, Margin, RichText, ScrollArea};
use egui::scroll_area::ScrollBarVisibility;
use grin_chain::SyncStatus;
use grin_core::core::amount_to_hr_string;

use crate::AppConfig;
use crate::gui::Colors;
use crate::gui::icons::{ARROWS_CLOCKWISE, BRIDGE, CHAT_CIRCLE_TEXT, CHECK, CHECK_FAT, COPY, FOLDER_USER, GEAR_FINE, GRAPH, PACKAGE, PATH, POWER, SCAN, SPINNER, USERS_THREE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{CameraContent, Modal, Root, View};
use crate::gui::views::types::{ModalPosition, QrScanResult, TextEditOptions};
use crate::gui::views::wallets::{WalletTransactions, WalletMessages, WalletTransport, WalletSettings};
use crate::gui::views::wallets::types::{GRIN, WalletTab, WalletTabType};
use crate::node::Node;
use crate::wallet::{Wallet, WalletConfig};
use crate::wallet::types::{WalletAccount, WalletData};

/// Selected and opened wallet content.
pub struct WalletContent {
    /// List of wallet accounts for [`Modal`].
    accounts: Vec<WalletAccount>,

    /// Flag to check if account is creating.
    account_creating: bool,
    /// Account label [`Modal`] value.
    account_label_edit: String,
    /// Flag to check if error occurred during account creation at [`Modal`].
    account_creation_error: bool,

    /// Camera content for QR scan [`Modal`].
    camera_content: CameraContent,
    /// QR code scan result
    qr_scan_result: Option<QrScanResult>,

    /// Current tab content to show.
    pub current_tab: Box<dyn WalletTab>
}

impl Default for WalletContent {
    fn default() -> Self {
        Self {
            accounts: vec![],
            account_creating: false,
            account_label_edit: "".to_string(),
            account_creation_error: false,
            camera_content: CameraContent::default(),
            qr_scan_result: None,
            current_tab: Box::new(WalletTransactions::default())
        }
    }
}

/// Identifier for account list [`Modal`].
const ACCOUNT_LIST_MODAL: &'static str = "account_list_modal";

/// Identifier for QR code scan [`Modal`].
const QR_CODE_SCAN_MODAL: &'static str = "qr_code_scan_modal";

impl WalletContent {
    /// Draw wallet content.
    pub fn ui(&mut self,
              ui: &mut egui::Ui,
              wallet: &mut Wallet,
              cb: &dyn PlatformCallbacks) {
        // Show modal content for this ui container.
        self.modal_content_ui(ui, wallet, cb);

        let dual_panel = Root::is_dual_panel_mode(ui);

        let data = wallet.get_data();
        let data_empty = data.is_none();

        // Show wallet balance panel not on Settings tab with selected non-repairing
        // wallet, when there is no error and data is not empty.
        let mut show_balance = self.current_tab.get_type() != WalletTabType::Settings && !data_empty
            && !wallet.sync_error() && !wallet.is_repairing() && !wallet.is_closing();
        if wallet.get_current_ext_conn().is_none() && !Node::is_running() {
            show_balance = false;
        }
        egui::TopBottomPanel::top(Id::from("wallet_balance").with(wallet.identifier()))
            .frame(egui::Frame {
                fill: Colors::fill(),
                stroke: View::item_stroke(),
                inner_margin: Margin {
                    left: View::far_left_inset_margin(ui) + 4.0,
                    right: View::get_right_inset() + 4.0,
                    top: 4.0,
                    bottom: 0.0,
                },
                outer_margin: Margin {
                    left: if !dual_panel {
                        -0.5
                    } else {
                        0.0
                    },
                    right: -0.5,
                    top: 0.0,
                    bottom: if dual_panel {
                        -1.0
                    } else {
                        -0.5
                    },
                },
                ..Default::default()
            })
            .show_animated_inside(ui, show_balance, |ui| {
                ui.vertical_centered(|ui| {
                    if !dual_panel {
                        ui.add_space(1.0);
                    }
                    // Draw account info.
                    View::max_width_ui(ui, Root::SIDE_PANEL_WIDTH * 1.3, |ui| {
                        self.account_ui(ui, wallet, data.unwrap(), cb);
                    });
                });
            });

        // Show wallet tabs panel.
        let show_tabs = !Self::block_navigation_on_sync(wallet);
        egui::TopBottomPanel::bottom("wallet_tabs")
            .frame(egui::Frame {
                fill: Colors::fill(),
                inner_margin: Margin {
                    left: View::far_left_inset_margin(ui) + View::TAB_ITEMS_PADDING,
                    right: View::get_right_inset() + View::TAB_ITEMS_PADDING,
                    top: View::TAB_ITEMS_PADDING,
                    bottom: View::get_bottom_inset() + View::TAB_ITEMS_PADDING,
                },
                ..Default::default()
            })
            .show_animated_inside(ui, show_tabs, |ui| {
                ui.vertical_centered(|ui| {
                    // Draw wallet tabs.
                    View::max_width_ui(ui, Root::SIDE_PANEL_WIDTH * 1.3, |ui| {
                        self.tabs_ui(ui, wallet);
                    });
                });
            });

        // Show tab content panel.
        egui::CentralPanel::default()
            .frame(egui::Frame {
                outer_margin: Margin {
                    left: if !dual_panel {
                      -0.5
                    } else {
                        0.0
                    },
                    right: -0.5,
                    top: 0.0,
                    bottom: 0.0,
                },
                stroke: View::item_stroke(),
                fill: Colors::white_or_black(false),
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                self.current_tab.ui(ui, wallet, cb);
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
                    ACCOUNT_LIST_MODAL => {
                        Modal::ui(ui.ctx(), |ui, modal| {
                            self.account_list_modal_ui(ui, wallet, modal, cb);
                        });
                    }
                    QR_CODE_SCAN_MODAL => {
                        Modal::ui(ui.ctx(), |ui, modal| {
                            self.scan_qr_modal_ui(ui, wallet, modal, cb);
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
        ui.painter().rect(rect, rounding, Colors::button(), View::hover_stroke());

        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            // Draw button to scan QR code.
            View::item_button(ui, View::item_rounding(0, 2, true), SCAN, None, || {
                self.qr_scan_result = None;
                self.camera_content.clear_state();
                // Show QR code scan modal.
                Modal::new(QR_CODE_SCAN_MODAL)
                    .position(ModalPosition::CenterTop)
                    .title(t!("scan_qr"))
                    .closeable(false)
                    .show();
                cb.start_camera();
            });

            // Draw button to show list of accounts.
            View::item_button(ui, View::item_rounding(1, 3, true), USERS_THREE, None, || {
                // Load accounts.
                self.account_label_edit = "".to_string();
                self.accounts = wallet.accounts();
                self.account_creating = false;
                // Show account list modal.
                Modal::new(ACCOUNT_LIST_MODAL)
                    .position(ModalPosition::CenterTop)
                    .title(t!("wallets.accounts"))
                    .show();
            });

            let layout_size = ui.available_size();
            ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.add_space(3.0);
                    // Show spendable amount.
                    let amount = amount_to_hr_string(data.info.amount_currently_spendable, true);
                    let amount_text = format!("{} {}", amount, GRIN);
                    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                        ui.add_space(1.0);
                        ui.label(RichText::new(amount_text)
                            .size(18.0)
                            .color(Colors::white_or_black(true)));
                    });
                    ui.add_space(-2.0);

                    // Show account label.
                    let account = wallet.get_config().account;
                    let default_acc_label = WalletConfig::DEFAULT_ACCOUNT_LABEL.to_string();
                    let acc_label = if account == default_acc_label {
                        t!("wallets.default_account")
                    } else {
                        account.to_owned()
                    };
                    let acc_text = format!("{} {}", FOLDER_USER, acc_label);
                    View::ellipsize_text(ui, acc_text, 15.0, Colors::text(false));

                    // Show confirmed height or sync progress.
                    let status_text = if !wallet.syncing() {
                        format!("{} {}", PACKAGE, data.info.last_confirmed_height)
                    } else {
                        let info_progress = wallet.info_sync_progress();
                        if info_progress == 100 || info_progress == 0 {
                            format!("{} {}", SPINNER, t!("wallets.wallet_loading"))
                        } else {
                            if wallet.is_repairing() {
                                let rep_progress = wallet.repairing_progress();
                                if rep_progress == 0 {
                                    format!("{} {}", SPINNER, t!("wallets.wallet_checking"))
                                } else {
                                    format!("{} {}: {}%",
                                            SPINNER,
                                            t!("wallets.wallet_checking"),
                                            rep_progress)
                                }
                            } else {
                                format!("{} {}: {}%",
                                        SPINNER,
                                        t!("wallets.wallet_loading"),
                                        info_progress)
                            }
                        }
                    };
                    View::animate_text(ui, status_text, 15.0, Colors::gray(), wallet.syncing());
                })
            });
        });
    }

    /// Draw account list [`Modal`] content.
    fn account_list_modal_ui(&mut self,
                             ui: &mut egui::Ui,
                             wallet: &mut Wallet,
                             modal: &Modal,
                             cb: &dyn PlatformCallbacks) {
        if self.account_creating {
            ui.add_space(6.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(t!("wallets.new_account_desc"))
                    .size(17.0)
                    .color(Colors::gray()));
                ui.add_space(8.0);

                // Draw account name edit.
                let text_edit_id = Id::from(modal.id).with(wallet.get_config().id);
                let mut text_edit_opts = TextEditOptions::new(text_edit_id);
                View::text_edit(ui, cb, &mut self.account_label_edit, &mut text_edit_opts);

                // Show error occurred during account creation..
                if self.account_creation_error {
                    ui.add_space(12.0);
                    ui.label(RichText::new(t!("error"))
                        .size(17.0)
                        .color(Colors::red()));
                }
                ui.add_space(12.0);
            });

            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            // Show modal buttons.
            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
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

                    View::button(ui, t!("create"), Colors::white_or_black(false), on_create);
                });
            });
            ui.add_space(6.0);
        } else {
            ui.add_space(3.0);

            // Show list of accounts.
            let size = self.accounts.len();
            ScrollArea::vertical()
                .id_source("account_list_modal_scroll")
                .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                .max_height(266.0)
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
            View::horizontal_line(ui, Colors::stroke());
            ui.add_space(6.0);

            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            // Show modal buttons.
            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("create"), Colors::white_or_black(false), || {
                        self.account_creating = true;
                        cb.show_keyboard();
                    });
                });
            });
            ui.add_space(6.0);
        }
    }

    /// Draw QR code scan [`Modal`] content.
    fn scan_qr_modal_ui(&mut self,
                             ui: &mut egui::Ui,
                             wallet: &mut Wallet,
                             modal: &Modal,
                             cb: &dyn PlatformCallbacks) {
        // Show scan result if exists or show camera content while scanning.
        if let Some(result) = &self.qr_scan_result {
            let mut result_text = result.text();
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(3.0);
            ScrollArea::vertical()
                .id_source(Id::from("qr_scan_result_input").with(wallet.get_config().id))
                .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                .max_height(128.0)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.add_space(7.0);
                    egui::TextEdit::multiline(&mut result_text)
                        .font(egui::TextStyle::Small)
                        .desired_rows(5)
                        .interactive(false)
                        .desired_width(f32::INFINITY)
                        .show(ui);
                    ui.add_space(6.0);
                });
            ui.add_space(2.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(10.0);

            // Show copy button.
            ui.vertical_centered(|ui| {
                let copy_text = format!("{} {}", COPY, t!("copy"));
                View::button(ui, copy_text, Colors::button(), || {
                    cb.copy_string_to_buffer(result_text.to_string());
                    self.qr_scan_result = None;
                    modal.close();
                });
            });
            ui.add_space(10.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(6.0);
        } else if let Some(result) = self.camera_content.qr_scan_result() {
            cb.stop_camera();
            self.camera_content.clear_state();
            match &result {
                QrScanResult::Slatepack(message) => {
                    // Redirect to messages to handle parsed message.
                    let mut messages =
                        WalletMessages::new(wallet.can_use_dandelion(), Some(message.to_string()));
                    messages.parse_message(wallet);
                    modal.close();
                    self.current_tab = Box::new(messages);
                    return;
                }
                QrScanResult::Address(receiver) => {
                    if wallet.get_data().unwrap().info.amount_currently_spendable > 0 {
                        // Redirect to send amount with Tor.
                        let addr = wallet.slatepack_address().unwrap();
                        let mut transport = WalletTransport::new(addr.clone());
                        modal.close();
                        transport.show_send_tor_modal(cb, Some(receiver.to_string()));
                        self.current_tab = Box::new(transport);
                        return;
                    }
                }
                _ => {}
            }

            // Set result and rename modal title.
            self.qr_scan_result = Some(result);
            Modal::set_title(t!("scan_result"));
        } else {
            ui.add_space(6.0);
            self.camera_content.ui(ui, cb);
            ui.add_space(6.0);
        }

        if self.qr_scan_result.is_some() {
            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("close"), Colors::white_or_black(false), || {
                        self.qr_scan_result = None;
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("repeat"), Colors::white_or_black(false), || {
                        Modal::set_title(t!("scan_qr"));
                        self.qr_scan_result = None;
                        cb.start_camera();
                    });
                });
            });
        } else {
            ui.vertical_centered_justified(|ui| {
                View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                    cb.stop_camera();
                    modal.close();
                });
            });
        }
        ui.add_space(6.0);
    }

    /// Draw tab buttons in the bottom of the screen.
    fn tabs_ui(&mut self, ui: &mut egui::Ui, wallet: &Wallet) {
        ui.scope(|ui| {
            // Setup spacing between tabs.
            ui.style_mut().spacing.item_spacing = egui::vec2(View::TAB_ITEMS_PADDING, 0.0);
            // Setup vertical padding inside tab button.
            ui.style_mut().spacing.button_padding = egui::vec2(0.0, 4.0);

            // Draw tab buttons.
            let current_type = self.current_tab.get_type();
            ui.columns(4, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::tab_button(ui, GRAPH, current_type == WalletTabType::Txs, || {
                        self.current_tab = Box::new(WalletTransactions::default());
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    let is_messages = current_type == WalletTabType::Messages;
                    View::tab_button(ui, CHAT_CIRCLE_TEXT, is_messages, || {
                        self.current_tab = Box::new(
                            WalletMessages::new(wallet.can_use_dandelion(), None)
                        );
                    });
                });
                columns[2].vertical_centered_justified(|ui| {
                    View::tab_button(ui, BRIDGE, current_type == WalletTabType::Transport, || {
                        let addr = wallet.slatepack_address().unwrap();
                        self.current_tab = Box::new(WalletTransport::new(addr));
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
    pub fn sync_ui(ui: &mut egui::Ui, wallet: &Wallet) -> bool {
        if wallet.is_repairing() && !wallet.sync_error() {
            Self::sync_progress_ui(ui, wallet);
            return true;
        } else if wallet.is_closing() {
            Self::sync_progress_ui(ui, wallet);
            return true;
        } else if wallet.get_current_ext_conn().is_none() {
            if !Node::is_running() || Node::is_stopping() {
                View::center_content(ui, 108.0, |ui| {
                    View::max_width_ui(ui, Root::SIDE_PANEL_WIDTH * 1.5, |ui| {
                        let text = t!("wallets.enable_node", "settings" => GEAR_FINE);
                        ui.label(RichText::new(text).size(16.0).color(Colors::inactive_text()));
                        ui.add_space(8.0);
                        // Show button to enable integrated node at non-dual root panel mode
                        // or when network connections are not showing and node is not stopping
                        let dual_panel_root = Root::is_dual_panel_mode(ui);
                        if (!dual_panel_root || AppConfig::show_connections_network_panel())
                            && !Node::is_stopping() {
                            let enable_text = format!("{} {}", POWER, t!("network.enable_node"));
                            View::action_button(ui, enable_text, || {
                                Node::start();
                            });
                        }
                    });
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
            ui.label(RichText::new(text).size(16.0).color(Colors::inactive_text()));
            ui.add_space(8.0);
            let retry_text = format!("{} {}", ARROWS_CLOCKWISE, t!("retry"));
            View::action_button(ui, retry_text, || {
                wallet.set_sync_error(false);
            });
        });
    }

    /// Check when to block tabs navigation on sync progress.
    pub fn block_navigation_on_sync(wallet: &Wallet) -> bool {
        let sync_error = wallet.sync_error();
        let integrated_node = wallet.get_current_ext_conn().is_none();
        let integrated_node_ready = Node::get_sync_status() == Some(SyncStatus::NoSync);
        let sync_after_opening = wallet.get_data().is_none() && !wallet.sync_error();
        // Block navigation if wallet is repairing and integrated node is not launching
        // and if wallet is closing or syncing after opening when there is no data to show.
        (wallet.is_repairing() && (integrated_node_ready || !integrated_node) && !sync_error)
            || wallet.is_closing() || (sync_after_opening &&
            (!integrated_node || integrated_node_ready))
    }

    /// Draw wallet sync progress content.
    pub fn sync_progress_ui(ui: &mut egui::Ui, wallet: &Wallet) {
        View::center_content(ui, 162.0, |ui| {
            View::max_width_ui(ui, Root::SIDE_PANEL_WIDTH * 1.5, |ui| {
                View::big_loading_spinner(ui);
                ui.add_space(18.0);
                // Setup sync progress text.
                let text = {
                    let integrated_node = wallet.get_current_ext_conn().is_none();
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
                        t!("wallets.tx_loading")
                    }
                };
                ui.label(RichText::new(text).size(16.0).color(Colors::inactive_text()));
            });
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
    ui.painter().rect(bg_rect, item_rounding, Colors::fill(), View::item_stroke());

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
                ui.label(RichText::new(CHECK_FAT).size(20.0).color(Colors::green()));
            }

            let layout_size = ui.available_size();
            ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                ui.add_space(6.0);
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    // Show spendable amount.
                    let amount = amount_to_hr_string(acc.spendable_amount, true);
                    let amount_text = format!("{} {}", amount, GRIN);
                    ui.label(RichText::new(amount_text).size(18.0).color(Colors::white_or_black(true)));
                    ui.add_space(-2.0);

                    // Show account name.
                    let default_acc_label = WalletConfig::DEFAULT_ACCOUNT_LABEL.to_string();
                    let acc_label = if acc.label == default_acc_label {
                        t!("wallets.default_account")
                    } else {
                        acc.label.to_owned()
                    };
                    let acc_name = format!("{} {}", FOLDER_USER, acc_label);
                    View::ellipsize_text(ui, acc_name, 15.0, Colors::text(false));

                    // Show account BIP32 derivation path.
                    let acc_path = format!("{} {}", PATH, acc.path);
                    ui.label(RichText::new(acc_path).size(15.0).color(Colors::gray()));
                    ui.add_space(3.0);
                });
            });
        });
    });
}