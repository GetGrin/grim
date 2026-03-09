// Copyright 2025 The Grim Developers
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

use egui::{Align, Layout, RichText, ScrollArea, StrokeKind};
use egui::scroll_area::ScrollBarVisibility;
use grin_core::core::amount_to_hr_string;

use crate::gui::icons::{CHECK, FOLDER_USER, PACKAGE, PATH, SCAN, SPINNER, USERS_THREE, USER_PLUS};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::types::{ModalPosition, QrScanResult};
use crate::gui::views::wallets::wallet::account::create::CreateAccountContent;
use crate::gui::views::wallets::wallet::types::{WalletContentContainer, GRIN};
use crate::gui::views::{CameraContent, CameraScanContent, Content, Modal, View};
use crate::gui::Colors;
use crate::gui::views::wallets::wallet::request::SendRequestContent;
use crate::wallet::{Wallet, WalletConfig};
use crate::wallet::types::{WalletAccount, WalletTask};

/// Wallet account panel content.
pub struct WalletAccountContent {
    /// Flag to show account list content.
    pub show_list: bool,
    /// Account creation [`Modal`] content.
    create_account_content: CreateAccountContent,

    /// QR code scan content.
    qr_scan_content: Option<CameraContent>,
    /// QR code scan result
    qr_scan_result: Option<QrScanResult>,
    /// Send request creation [`Modal`] content.
    send_content: Option<SendRequestContent>,
}

/// Account creation [`Modal`] identifier.
const CREATE_MODAL_ID: &'static str = "create_account_modal";
/// Identifier for sending request creation [`Modal`].
const SEND_MODAL_ID: &'static str = "account_send_request_modal";

impl WalletContentContainer for WalletAccountContent {
    fn modal_ids(&self) -> Vec<&'static str> {
        vec![
            CREATE_MODAL_ID,
            SEND_MODAL_ID
        ]
    }

    fn modal_ui(&mut self,
                ui: &mut egui::Ui,
                wallet: &Wallet,
                modal: &Modal,
                cb: &dyn PlatformCallbacks) {
        match modal.id {
            CREATE_MODAL_ID => self.create_account_content.ui(ui, wallet, modal, cb),
            SEND_MODAL_ID => {
                if let Some(c) = self.send_content.as_mut() {
                    c.modal_ui(ui, wallet, modal, cb);
                }
            }
            _ => {}
        }
    }

    fn container_ui(&mut self, ui: &mut egui::Ui, wallet: &Wallet, cb: &dyn PlatformCallbacks) {
        if self.qr_scan_showing() {
            self.qr_scan_ui(ui, wallet, cb);
        } else {
            View::max_width_ui(ui, Content::SIDE_PANEL_WIDTH * 1.3, |ui| {
                if self.show_list {
                    self.list_ui(ui, wallet);
                } else {
                    // Show account content.
                    self.account_ui(ui, wallet, cb);
                }
            });
        }
    }
}

impl Default for WalletAccountContent {
    fn default() -> Self {
        Self {
            show_list: false,
            create_account_content: CreateAccountContent::default(),
            qr_scan_content: None,
            qr_scan_result: None,
            send_content: None,
        }
    }
}

const ACCOUNT_ITEM_HEIGHT: f32 = 75.0;

impl WalletAccountContent {
    /// Check if QR code scanner was opened.
    pub fn qr_scan_showing(&self) -> bool {
        self.qr_scan_content.is_some() || self.qr_scan_result.is_some()
    }

    /// Close QR code scanner.
    pub fn close_qr_scan(&mut self, cb: &dyn PlatformCallbacks) {
        if !self.qr_scan_showing() {
            return;
        }
        cb.stop_camera();
        self.qr_scan_content = None;
        self.qr_scan_result = None;
    }

    /// Check if it's possible to go back at navigation stack.
    pub fn can_back(&self) -> bool {
        self.qr_scan_showing() || self.show_list
    }

    /// Navigate back on navigation stack.
    pub fn back(&mut self, cb: &dyn PlatformCallbacks) {
        if self.qr_scan_showing() {
            self.close_qr_scan(cb);
        } else if self.show_list {
            self.show_list = false;
        }
    }

    /// Draw wallet account content.
    fn account_ui(&mut self,
                  ui: &mut egui::Ui,
                  wallet: &Wallet,
                  cb: &dyn PlatformCallbacks) {
        // Check wallet data.
        if wallet.get_data().is_none() {
            return;
        }

        let data = wallet.get_data().unwrap();

        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(75.0);

        // Draw round background.
        let rounding = View::item_rounding(0, 2, false);
        ui.painter().rect(rect,
                          rounding,
                          Colors::fill(),
                          View::item_stroke(),
                          StrokeKind::Outside);

        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            // Draw button to show QR code scanner.
            let wallet_synced = wallet.synced_from_node();
            if wallet_synced {
                View::item_button(ui, View::item_rounding(0, 2, true), SCAN, None, || {
                    self.qr_scan_content = Some(CameraContent::default());
                    cb.start_camera();
                });
            }

            // Draw button to show list of accounts.
            let accounts = wallet.accounts();
            let accounts_icon = if accounts.len() > 1 {
                USERS_THREE
            } else {
                USER_PLUS
            };
            let rounding = if wallet_synced {
                View::item_rounding(1, 3, true)
            } else {
                View::item_rounding(0, 2, true)
            };
            View::item_button(ui, rounding, accounts_icon, None, || {
                if accounts.len() == 1 {
                    self.create_account_content = CreateAccountContent::default();
                    Modal::new(CREATE_MODAL_ID)
                        .position(ModalPosition::CenterTop)
                        .title(t!("wallets.accounts"))
                        .show();
                } else {
                    self.show_list = true;
                }
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
                        t!("wallets.default_account").into()
                    } else {
                        account.to_owned()
                    };
                    let acc_text = format!("{} {}", FOLDER_USER, acc_label);
                    View::ellipsize_text(ui, acc_text, 15.0, Colors::text(false));

                    // Show confirmed height or sync progress.
                    let status_text = if wallet.message_opening() {
                        format!("{} {}", SPINNER, t!("wallets.loading"))
                    } else if !wallet.syncing() {
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
                    let animate = wallet.syncing() || wallet.message_opening();
                    View::animate_text(ui, status_text, 15.0, Colors::gray(), animate);
                })
            });
        });
    }

    /// Draw account list content.
    fn list_ui(&mut self, ui: &mut egui::Ui, wallet: &Wallet) {
        let accounts = wallet.accounts();
        let size = accounts.len();
        ScrollArea::vertical()
            .id_salt("account_list_scroll")
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .max_height(411.0)
            .auto_shrink([true; 2])
            .show_rows(ui, ACCOUNT_ITEM_HEIGHT, size, |ui, row_range| {
                for index in row_range {
                    let acc = accounts.get(index).unwrap().clone();
                    let current = wallet.get_config().account == acc.label;
                    account_item_ui(ui, &acc, current, index, size, || {
                        let _ = wallet.set_active_account(&acc.label);
                        self.show_list = false;
                    });
                    if index == size - 1 {
                        ui.add_space(4.0);
                    }
                }
            });

        ui.add_space(2.0);
        View::horizontal_line(ui, Colors::item_stroke());
        ui.add_space(6.0);

        // Setup spacing between buttons.
        ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

        // Show modal buttons.
        ui.columns(2, |columns| {
            columns[0].vertical_centered_justified(|ui| {
                View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                    self.show_list = false;
                });
            });
            columns[1].vertical_centered_justified(|ui| {
                View::button(ui, t!("modal.add"), Colors::white_or_black(false), || {
                    self.show_list = false;
                    self.create_account_content = CreateAccountContent::default();
                    Modal::new(CREATE_MODAL_ID)
                        .position(ModalPosition::CenterTop)
                        .title(t!("wallets.accounts"))
                        .show();
                });
            });
        });
        ui.add_space(6.0);
    }

    /// Draw QR code scanner content.
    fn qr_scan_ui(&mut self, ui: &mut egui::Ui, wallet: &Wallet, cb: &dyn PlatformCallbacks) {
        View::max_width_ui(ui, Content::SIDE_PANEL_WIDTH, |ui| {
            if self.qr_scan_content.is_some() {
                if let Some(result) = self.qr_scan_content.as_ref().unwrap().qr_scan_result() {
                    cb.stop_camera();
                    self.qr_scan_content = None;
                    match result {
                        QrScanResult::Address(a) => {
                            if let Some(data) = wallet.get_data() {
                                if data.info.amount_currently_spendable > 0 {
                                    let address = Some(a.to_string());
                                    self.send_content = Some(SendRequestContent::new(address));
                                    Modal::new(SEND_MODAL_ID)
                                        .position(ModalPosition::CenterTop)
                                        .title(t!("wallets.send"))
                                        .show();
                                }
                            }
                        }
                        QrScanResult::Slatepack(m) => {
                            wallet.task(WalletTask::OpenMessage(m));
                        }
                        _ => {
                            self.qr_scan_result = Some(result);
                        }
                    }
                } else {
                    // Draw QR code scan content.
                    self.qr_scan_content.as_mut().unwrap().ui(ui, cb);
                    ui.add_space(6.0);
                    ui.vertical_centered_justified(|ui| {
                        View::button(ui, t!("close"), Colors::white_or_black(false), || {
                            self.close_qr_scan(cb);
                        });
                    });
                }
            } else if let Some(res) = &self.qr_scan_result.clone() {
                CameraScanContent::result_ui(ui, res, cb, || {
                    self.qr_scan_result = None;
                }, || {
                    self.qr_scan_content = Some(CameraContent::default());
                    cb.start_camera();
                });
            }
            ui.add_space(6.0);
        });
    }
}

/// Draw account item.
fn account_item_ui(ui: &mut egui::Ui,
                   acc: &WalletAccount,
                   current: bool,
                   index: usize,
                   size: usize,
                   mut on_select: impl FnMut()) {
    // Setup layout size.
    let mut rect = ui.available_rect_before_wrap();
    rect.set_height(ACCOUNT_ITEM_HEIGHT);

    // Draw round background.
    let bg_rect = rect.clone();
    let item_rounding = View::item_rounding(index, size, false);
    ui.painter().rect(bg_rect,
                      item_rounding,
                      Colors::fill(),
                      View::item_stroke(),
                      StrokeKind::Outside);

    ui.vertical(|ui| {
        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            // Draw button to select account.
            if current {
                View::selected_item_check(ui);
            } else {
                let button_rounding = View::item_rounding(index, size, true);
                View::item_button(ui, button_rounding, CHECK, None, || {
                    on_select();
                });
            }

            let layout_size = ui.available_size();
            ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.add_space(3.0);
                    // Show spendable amount.
                    let amount = amount_to_hr_string(acc.spendable_amount, true);
                    let amount_text = format!("{} {}", amount, GRIN);
                    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                        ui.add_space(1.0);
                        ui.label(RichText::new(amount_text)
                            .size(18.0)
                            .color(Colors::white_or_black(true)));
                    });
                    ui.add_space(-2.0);

                    // Show account name.
                    let default_acc_label = WalletConfig::DEFAULT_ACCOUNT_LABEL.to_string();
                    let acc_label = if acc.label == default_acc_label {
                        t!("wallets.default_account").into()
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