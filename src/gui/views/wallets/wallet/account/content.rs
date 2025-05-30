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

use eframe::emath::Align;
use eframe::epaint::StrokeKind;
use egui::{Layout, RichText};
use grin_core::core::amount_to_hr_string;

use crate::gui::icons::{FOLDER_USER, PACKAGE, SCAN, SPINNER, USERS_THREE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::types::{ContentContainer, ModalPosition};
use crate::gui::views::wallets::wallet::modals::WalletAccountsModal;
use crate::gui::views::wallets::wallet::types::GRIN;
use crate::gui::views::{CameraContent, Content, Modal, View};
use crate::gui::Colors;
use crate::wallet::{Wallet, WalletConfig};

/// Wallet account panel content.
pub struct AccountContent {
    /// Wallet instance.
    wallet: Wallet,

    /// Wallet accounts [`Modal`] content.
    accounts_modal_content: WalletAccountsModal,

    /// QR code scan content.
    qr_scan_content: Option<CameraContent>,
}

/// Identifier for account list [`Modal`].
const ACCOUNT_LIST_MODAL: &'static str = "account_list_modal";

impl ContentContainer for AccountContent {
    fn modal_ids(&self) -> Vec<&'static str> {
        vec![
            ACCOUNT_LIST_MODAL,
        ]
    }

    fn modal_ui(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        match modal.id {
            ACCOUNT_LIST_MODAL => {
                self.accounts_modal_content.ui(ui, &self.wallet, modal, cb);
            }
            _ => {}
        }
    }

    fn on_back(&mut self, _: &dyn PlatformCallbacks) -> bool {
        true
    }

    fn container_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        if self.qr_scan_content.is_some() {
            if let Some(result) = self.qr_scan_content.as_ref().unwrap().qr_scan_result() {
                // match result {
                //     QrScanResult::Address(address) => {
                //         self.current_tab =
                //             Box::new(WalletTransport::new(Some(address.to_string())));
                //     }
                //     _ => {
                //         self.current_tab =
                //             Box::new(WalletMessages::new(Some(result.text())))
                //     }
                // }
                // Stop camera and close scanning.
                cb.stop_camera();
                self.qr_scan_content = None;
            } else {
                View::max_width_ui(ui, Content::SIDE_PANEL_WIDTH, |ui| {
                    self.qr_scan_content.as_mut().unwrap().ui(ui, cb);
                    ui.add_space(6.0);
                    ui.vertical_centered_justified(|ui| {
                        View::button(ui, t!("close"), Colors::white_or_black(false), || {
                            cb.stop_camera();
                            self.qr_scan_content = None;
                        });
                    });
                    ui.add_space(6.0);
                });
            }
        } else {
            View::max_width_ui(ui, Content::SIDE_PANEL_WIDTH * 1.3, |ui| {
                self.account_ui(ui, cb);
            });
        }
    }
}

impl AccountContent {
    /// Create new wallet account content.
    pub fn new(wallet: Wallet) -> Self {
        let accounts_modal = WalletAccountsModal::new(wallet.accounts());
        Self {
            wallet,
            accounts_modal_content: accounts_modal,
            qr_scan_content: None,
        }
    }

    /// Check if QR code scanner is showing.
    pub fn qr_scan_showing(&self) -> bool {
        self.qr_scan_content.is_some()
    }

    /// Close QR code scanner.
    pub fn close_qr_scan(&mut self) {
        self.qr_scan_content = None;
    }

    /// Draw wallet account content.
    fn account_ui(&mut self,
                  ui: &mut egui::Ui,
                  cb: &dyn PlatformCallbacks) {
        // Check wallet data.
        if self.wallet.get_data().is_none() {
            return;
        }
        let data = self.wallet.get_data().unwrap();

        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(75.0);
        // Draw round background.
        let rounding = View::item_rounding(0, 2, false);
        ui.painter().rect(rect,
                          rounding,
                          Colors::fill_lite(),
                          View::item_stroke(),
                          StrokeKind::Middle);

        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            // Draw button to show QR code scanner.
            View::item_button(ui, View::item_rounding(0, 2, true), SCAN, None, || {
                self.qr_scan_content = Some(CameraContent::default());
                cb.start_camera();
            });

            // Draw button to show list of accounts.
            View::item_button(ui, View::item_rounding(1, 3, true), USERS_THREE, None, || {
                self.accounts_modal_content = WalletAccountsModal::new(self.wallet.accounts());
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
                    let account = self.wallet.get_config().account;
                    let default_acc_label = WalletConfig::DEFAULT_ACCOUNT_LABEL.to_string();
                    let acc_label = if account == default_acc_label {
                        t!("wallets.default_account")
                    } else {
                        account.to_owned()
                    };
                    let acc_text = format!("{} {}", FOLDER_USER, acc_label);
                    View::ellipsize_text(ui, acc_text, 15.0, Colors::text(false));

                    // Show confirmed height or sync progress.
                    let status_text = if !self.wallet.syncing() {
                        format!("{} {}", PACKAGE, data.info.last_confirmed_height)
                    } else {
                        let info_progress = self.wallet.info_sync_progress();
                        if info_progress == 100 || info_progress == 0 {
                            format!("{} {}", SPINNER, t!("wallets.wallet_loading"))
                        } else {
                            if self.wallet.is_repairing() {
                                let rep_progress = self.wallet.repairing_progress();
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
                    View::animate_text(ui,
                                       status_text,
                                       15.0,
                                       Colors::gray(),
                                       self.wallet.syncing());
                })
            });
        });
    }
}