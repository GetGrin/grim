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

use egui::scroll_area::ScrollBarVisibility;
use egui::{Id, Margin, RichText, ScrollArea};
use grin_chain::SyncStatus;
use std::time::Duration;
use grin_wallet_libwallet::Error;
use crate::gui::icons::{ARROWS_CLOCKWISE, FILE_ARROW_DOWN, FILE_ARROW_UP, GEAR_FINE, POWER, STACK};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::types::{LinePosition, ModalPosition};
use crate::gui::views::wallets::types::{WalletTab, WalletTabType};
use crate::gui::views::wallets::wallet::account::AccountContent;
use crate::gui::views::wallets::wallet::request::{InvoiceRequestContent, SendRequestContent};
use crate::gui::views::wallets::wallet::transport::WalletTransportContent;
use crate::gui::views::wallets::wallet::types::WalletContentContainer;
use crate::gui::views::wallets::wallet::WalletSettings;
use crate::gui::views::wallets::WalletTransactions;
use crate::gui::views::{Content, FilePickContent, FilePickContentType, Modal, View};
use crate::gui::Colors;
use crate::node::Node;
use crate::wallet::types::{ConnectionMethod, WalletTransaction};
use crate::wallet::{ExternalConnection, Wallet};
use crate::AppConfig;

/// Wallet content.
pub struct WalletContent {
    /// Current tab content to show.
    current_tab: Box<dyn WalletTab>,

    /// Account panel content.
    pub account_content: AccountContent,
    /// Transport panel content.
    pub transport_content: WalletTransportContent,

    /// Invoice request creation [`Modal`] content.
    invoice_request_content: Option<InvoiceRequestContent>,
    /// Send request creation [`Modal`] content.
    send_request_content: Option<SendRequestContent>,

    /// Tab button to pick file for parsing.
    file_pick_tab_button: FilePickContent,
}

/// Identifier for invoice creation [`Modal`].
const INVOICE_MODAL_ID: &'static str = "invoice_request_modal";
/// Identifier for sending request creation [`Modal`].
const SEND_MODAL_ID: &'static str = "send_request_modal";

impl WalletContentContainer for WalletContent {
    fn modal_ids(&self) -> Vec<&'static str> {
        vec![
            INVOICE_MODAL_ID,
            SEND_MODAL_ID
        ]
    }

    fn modal_ui(&mut self, ui: &mut egui::Ui, w: &Wallet, m: &Modal, cb: &dyn PlatformCallbacks) {
        match m.id {
            INVOICE_MODAL_ID => {
                if let Some(c) = self.invoice_request_content.as_mut() {
                    c.modal_ui(ui, w, m, cb);
                }
            }
            SEND_MODAL_ID => {
                if let Some(c) = self.send_request_content.as_mut() {
                    c.modal_ui(ui, w, m, cb);
                }
            }
            _ => {}
        }
    }

    fn container_ui(&mut self, ui: &mut egui::Ui, wallet: &Wallet, cb: &dyn PlatformCallbacks) {
        if self.account_content.can_back() {
            ui.ctx().request_repaint();
        } else {
            ui.ctx().request_repaint_after(Duration::from_millis(1000));
        }

        let dual_panel = Content::is_dual_panel_mode(ui.ctx());
        let show_wallets_dual = AppConfig::show_wallets_at_dual_panel();

        let wallet_id = wallet.identifier();
        let data = wallet.get_data();
        let block_nav = self.block_navigation_on_sync(wallet);

        // Show wallet account panel not on settings tab when navigation is not blocked and QR code
        // scanner is not showing and wallet data is not empty.
        let mut show_account = self.current_tab.get_type() != WalletTabType::Settings && !block_nav
            && !wallet.sync_error() && data.is_some();
        if wallet.get_current_connection() == ConnectionMethod::Integrated &&
            !Node::is_running() {
            show_account = false;
        }

        // Consume inserted message.
        if let Some(res) = wallet.consume_message_result() {
            self.on_transaction(res);
        }

        // Show wallet tabs.
        egui::TopBottomPanel::bottom("wallet_tabs")
            .frame(egui::Frame {
                inner_margin: Margin {
                    left: (View::far_left_inset_margin(ui) + View::TAB_ITEMS_PADDING) as i8,
                    right: (View::get_right_inset() + View::TAB_ITEMS_PADDING) as i8,
                    top: View::TAB_ITEMS_PADDING as i8,
                    bottom: (View::get_bottom_inset() + View::TAB_ITEMS_PADDING) as i8,
                },
                fill: Colors::fill(),
                ..Default::default()
            })
            .show_animated_inside(ui, !block_nav, |ui| {
                let r = ui.available_rect_before_wrap();
                View::max_width_ui(ui, Content::SIDE_PANEL_WIDTH * 1.3, |ui| {
                    self.tabs_ui(wallet, ui, cb);
                });
                let rect = {
                    let mut r = r.clone();
                    r.min.x -= View::far_left_inset_margin(ui) + View::TAB_ITEMS_PADDING;
                    r.min.y -= View::TAB_ITEMS_PADDING;
                    r.max.x += View::get_right_inset() + View::TAB_ITEMS_PADDING;
                    r.max.y += View::get_bottom_inset() + View::TAB_ITEMS_PADDING;
                    r
                };
                // Draw cover for content below opened panel.
                if self.can_back() && show_account {
                    View::content_cover_ui(ui, rect, "wallet_tabs_content_cover", || {
                        self.back(cb);
                    });
                } else {
                    // Draw content divider line.
                    View::line(ui, LinePosition::TOP, &rect, Colors::stroke());
                }
            });

        // Close scanner when account panel got hidden.
        if !show_account && self.account_content.qr_scan_showing() {
            self.account_content.close_qr_scan(cb);
        }

        // Flag to check if account panel is opened.
        let top_panel_expanded = self.account_content.can_back() ||
            self.transport_content.can_back();

        // Show wallet account content.
        if !self.transport_content.can_back() && show_account {
            egui::TopBottomPanel::top(Id::from("wallet_account").with(wallet.identifier()))
                .frame(egui::Frame {
                    inner_margin: Margin {
                        left: (View::far_left_inset_margin(ui) + 4.0) as i8,
                        right: (View::get_right_inset() + 4.0) as i8,
                        top: 4.0 as i8,
                        bottom: 0.0 as i8,
                    },
                    fill: if top_panel_expanded {
                        Colors::fill_lite()
                    } else {
                        Colors::TRANSPARENT
                    },
                    ..Default::default()
                })
                .show_inside(ui, |ui| {
                    let rect = ui.available_rect_before_wrap();
                    self.account_content.ui(ui, &wallet, cb);
                    // Draw content divider lines.
                    let r = {
                        let mut r = rect.clone();
                        r.min.x -= 4.0 + View::far_left_inset_margin(ui);
                        r.min.y -= 4.0;
                        r.max.x += 4.0 + View::get_right_inset();
                        r
                    };
                    if dual_panel && show_wallets_dual {
                        View::line(ui, LinePosition::LEFT, &r, Colors::item_stroke());
                    }
                });
        }

        // Show wallet transport content.
        if !self.account_content.can_back() && show_account {
            egui::TopBottomPanel::top(Id::from("wallet_transport").with(wallet.identifier()))
                .frame(egui::Frame {
                    inner_margin: Margin {
                        left: (View::far_left_inset_margin(ui) + 4.0) as i8,
                        right: (View::get_right_inset() + 4.0) as i8,
                        top: 1.0 as i8,
                        bottom: 1.0 as i8,
                    },
                    fill: if top_panel_expanded {
                        Colors::fill_lite()
                    } else {
                        Colors::TRANSPARENT
                    },
                    ..Default::default()
                })
                .show_inside(ui, |ui| {
                    let rect = ui.available_rect_before_wrap();
                    View::max_width_ui(ui, Content::SIDE_PANEL_WIDTH * 1.3, |ui| {
                        self.transport_content.ui(ui, &wallet, cb);
                    });
                    // Draw content divider lines.
                    let r = {
                        let mut r = rect.clone();
                        r.min.x -= 4.0 + View::far_left_inset_margin(ui);
                        r.min.y -= 1.0;
                        r.max.x += 4.0 + View::get_right_inset();
                        r
                    };
                    if dual_panel && show_wallets_dual {
                        View::line(ui, LinePosition::LEFT, &r, Colors::item_stroke());
                    }
                });
        }

        // Show tab content.
        egui::CentralPanel::default()
            .frame(egui::Frame {
                inner_margin: Margin {
                    left: (View::far_left_inset_margin(ui) + 4.0) as i8,
                    right: (View::get_right_inset() + 4.0) as i8,
                    top: 0.0 as i8,
                    bottom: 4.0 as i8,
                },
                fill: if self.current_tab.get_type() == WalletTabType::Settings {
                    Colors::fill_lite()
                } else {
                    Colors::TRANSPARENT
                },
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                let rect = ui.available_rect_before_wrap();
                let tab_type = self.current_tab.get_type();
                let show_sync = (tab_type != WalletTabType::Settings || block_nav) &&
                    sync_ui(ui, &wallet);
                if !show_sync {
                    if tab_type != WalletTabType::Txs {
                        ui.add_space(3.0);
                        ScrollArea::vertical()
                            .id_salt(Id::from("wallet_tab_content_scroll")
                                .with(tab_type.name())
                                .with(wallet_id))
                            .auto_shrink([false; 2])
                            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                            .show(ui, |ui| {
                                View::max_width_ui(ui, Content::SIDE_PANEL_WIDTH * 1.3, |ui| {
                                    self.current_tab.ui(ui, &wallet, cb);
                                });
                            });
                    } else {
                        self.current_tab.ui(ui, &wallet, cb);
                    }
                }
                let rect = {
                    let mut r = rect.clone();
                    r.min.x -= View::far_left_inset_margin(ui) + 4.0;
                    r.max.x += View::get_right_inset() + 4.0;
                    r.max.y += 4.0;
                    r
                };
                // Draw cover for content below opened panel.
                if !show_sync && self.can_back() {
                    View::content_cover_ui(ui, rect, "wallet_panel_content_cover", || {
                        self.back(cb);
                    });
                }
                // Draw content divider line.
                if dual_panel && show_wallets_dual {
                    View::line(ui, LinePosition::LEFT, &rect, Colors::item_stroke());
                }
            });
    }
}

impl Default for WalletContent {
    fn default() -> Self {
        Self {
            current_tab: Box::new(WalletTransactions::new(None)),
            account_content: AccountContent::default(),
            transport_content: WalletTransportContent::default(),
            invoice_request_content: None,
            send_request_content: None,
            file_pick_tab_button: FilePickContent::new(FilePickContentType::Tab),
        }
    }
}

impl WalletContent {
    /// Get title based on current navigation state.
    pub fn title(&self) -> String {
        if self.account_content.qr_scan_showing() {
            t!("scan_qr")
        } else if self.account_content.list_content.is_some() {
            t!("wallets.accounts")
        } else if self.transport_content.settings_content.is_some() {
            t!("wallets.transport")
        } else if self.transport_content.qr_address_content.is_some() {
            t!("network_mining.address")
        } else {
            self.current_tab.get_type().name()
        }
    }

    /// Callback on incoming transaction for user to take action.
    fn on_transaction(&mut self, tx_result: Result<WalletTransaction, Error>) {
        if let Ok(tx) = tx_result {
            self.current_tab = Box::new(WalletTransactions::new(Some(tx)));
        }
    }

    /// Check if it's possible to go back at navigation stack.
    pub fn can_back(&self) -> bool {
        self.account_content.can_back() || self.transport_content.can_back()
    }

    /// Navigate back on navigation stack.
    pub fn back(&mut self, cb: &dyn PlatformCallbacks) {
        if self.account_content.can_back() {
            self.account_content.back(cb);
        } else if self.transport_content.can_back() {
            self.transport_content.back();
        }
    }

    /// Check when to block tabs navigation on sync progress.
    fn block_navigation_on_sync(&self, wallet: &Wallet) -> bool {
        let sync_error = wallet.sync_error();
        let integrated_node = wallet.get_current_connection() == ConnectionMethod::Integrated;
        let integrated_node_ready = Node::get_sync_status() == Some(SyncStatus::NoSync);
        let sync_after_opening = wallet.get_data().is_none() && !wallet.sync_error();
        // Block navigation if wallet is repairing and integrated node is not launching
        // and if wallet is closing or syncing after opening when there is no data to show.
        (wallet.is_repairing() && (integrated_node_ready || !integrated_node) && !sync_error)
            || wallet.is_closing() || (sync_after_opening &&
            (!integrated_node || integrated_node_ready))
    }

    /// Draw tab buttons at the bottom of the screen.
    fn tabs_ui(&mut self, wallet: &Wallet, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        ui.scope(|ui| {
            // Setup spacing between tabs.
            ui.style_mut().spacing.item_spacing = egui::vec2(View::TAB_ITEMS_PADDING, 0.0);

            // Setup vertical padding inside buttons.
            ui.style_mut().spacing.button_padding = egui::vec2(0.0, 4.0);

            let has_wallet_data = wallet.get_data().is_some();
            let can_send = has_wallet_data &&
                wallet.get_data().unwrap().info.amount_currently_spendable > 0;

            let current_type = self.current_tab.get_type();
            let tabs_amount = if can_send { 5 } else { 4 };
            ui.columns(tabs_amount, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::tab_button(ui, STACK, None, Some(current_type == WalletTabType::Txs), |_| {
                        self.current_tab = Box::new(WalletTransactions::new(None));
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    let active = if has_wallet_data {
                        Some(false)
                    } else {
                        None
                    };
                    View::tab_button(ui, FILE_ARROW_DOWN, Some(Colors::green()), active, |_| {
                        self.invoice_request_content = Some(InvoiceRequestContent::default());
                        Modal::new(INVOICE_MODAL_ID)
                            .position(ModalPosition::CenterTop)
                            .title(t!("wallets.receive"))
                            .show();
                    });
                });
                columns[2].vertical_centered_justified(|ui| {
                    if wallet.message_opening() {
                        View::small_loading_spinner(ui);
                    } else {
                        let mut message = "".to_string();
                        self.file_pick_tab_button.ui(ui, cb, |m| {
                            message = m;
                        });
                        if !message.is_empty() {
                            wallet.open_slatepack(message);
                        }
                    }
                });
                if can_send {
                    columns[3].vertical_centered_justified(|ui| {
                        View::tab_button(ui, FILE_ARROW_UP, Some(Colors::red()), Some(false), |_| {
                            self.send_request_content = Some(SendRequestContent::new(None));
                            Modal::new(SEND_MODAL_ID)
                                .position(ModalPosition::CenterTop)
                                .title(t!("wallets.send"))
                                .show();
                        });
                    });
                }
                let settings_index = if tabs_amount == 5 { 4 } else { 3 };
                columns[settings_index].vertical_centered_justified(|ui| {
                    let active = Some(current_type == WalletTabType::Settings);
                    View::tab_button(ui, GEAR_FINE, None, active, |ui| {
                        ExternalConnection::check(None, ui.ctx());
                        self.current_tab = Box::new(WalletSettings::default());
                    });
                });
            });
        });
    }
}

/// Draw content when wallet is syncing and not ready to use, returns `true` at this case.
fn sync_ui(ui: &mut egui::Ui, wallet: &Wallet) -> bool {
    if wallet.is_repairing() && !wallet.sync_error() {
        sync_progress_ui(ui, wallet);
        return true;
    } else if wallet.is_closing() {
        sync_progress_ui(ui, wallet);
        return true;
    } else if wallet.get_current_connection() == ConnectionMethod::Integrated {
        if !Node::is_running() || Node::is_stopping() {
            View::center_content(ui, 108.0, |ui| {
                View::max_width_ui(ui, Content::SIDE_PANEL_WIDTH * 1.3, |ui| {
                    let text = t!("wallets.enable_node", "settings" => GEAR_FINE);
                    ui.label(RichText::new(text).size(16.0).color(Colors::inactive_text()));
                    ui.add_space(8.0);
                    // Show button to enable integrated node at non-dual root panel mode
                    // or when network connections are not showing and node is not stopping
                    let dual_panel = Content::is_dual_panel_mode(ui.ctx());
                    if (!dual_panel || AppConfig::show_connections_network_panel())
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
            sync_error_ui(ui, wallet);
            return true;
        } else if wallet.get_data().is_none() {
            sync_progress_ui(ui, wallet);
            return true;
        }
    } else if wallet.sync_error() {
        sync_error_ui(ui, wallet);
        return true;
    } else if wallet.get_data().is_none() {
        sync_progress_ui(ui, wallet);
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

/// Draw wallet sync progress content.
fn sync_progress_ui(ui: &mut egui::Ui, wallet: &Wallet) {
    View::center_content(ui, 162.0, |ui| {
        View::max_width_ui(ui, Content::SIDE_PANEL_WIDTH * 1.3, |ui| {
            View::big_loading_spinner(ui);
            ui.add_space(18.0);
            // Setup sync progress text.
            let text = {
                let int_node = wallet.get_current_connection() == ConnectionMethod::Integrated;
                let int_ready = Node::get_sync_status() == Some(SyncStatus::NoSync);
                let info_progress = wallet.info_sync_progress();

                if wallet.is_closing() {
                    t!("wallets.wallet_closing")
                } else if int_node && !int_ready {
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