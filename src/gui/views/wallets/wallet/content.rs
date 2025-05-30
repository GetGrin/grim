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

use crate::gui::icons::{ARROWS_CLOCKWISE, BRIDGE, CHAT_CIRCLE_TEXT, GEAR_FINE, GRAPH, POWER};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::types::{ContentContainer, LinePosition};
use crate::gui::views::wallets::types::{WalletTab, WalletTabType};
use crate::gui::views::wallets::wallet::account::AccountContent;
use crate::gui::views::wallets::wallet::WalletSettings;
use crate::gui::views::wallets::{WalletMessages, WalletTransactions, WalletTransport};
use crate::gui::views::{Content, Modal, View};
use crate::gui::Colors;
use crate::node::Node;
use crate::wallet::types::ConnectionMethod;
use crate::wallet::{ExternalConnection, Wallet};
use crate::AppConfig;

/// Wallet content.
pub struct WalletContent {
    /// Selected and opened wallet.
    pub wallet: Wallet,
    /// Current tab content to show.
    pub current_tab: Box<dyn WalletTab>,

    account_content: AccountContent,
}

impl ContentContainer for WalletContent {
    fn modal_ids(&self) -> Vec<&'static str> {
        vec![]
    }

    fn modal_ui(&mut self, _: &mut egui::Ui, _: &Modal, _: &dyn PlatformCallbacks) {}

    fn on_back(&mut self, _: &dyn PlatformCallbacks) -> bool {
        true
    }

    fn container_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        ui.ctx().request_repaint_after(Duration::from_millis(1000));

        let dual_panel = Content::is_dual_panel_mode(ui.ctx());
        let show_wallets_dual = AppConfig::show_wallets_at_dual_panel();

        let wallet = &self.wallet;
        let wallet_id = wallet.identifier();
        let data = wallet.get_data();
        let show_qr_scan = self.account_content.qr_scan_showing();
        let hide_tabs = Self::block_navigation_on_sync(wallet);

        // Show wallet account panel not on settings tab when navigation is not blocked and QR code
        // scanner is not showing and wallet data is not empty.
        let mut show_account = self.current_tab.get_type() != WalletTabType::Settings && !hide_tabs
            && !wallet.sync_error() && data.is_some();
        if wallet.get_current_connection() == ConnectionMethod::Integrated && !Node::is_running() {
            show_account = false;
        }
        // Close scanner when balance got hidden.
        if !show_account && show_qr_scan {
            cb.stop_camera();
            self.account_content.close_qr_scan();
        }
        egui::TopBottomPanel::top(Id::from("wallet_account").with(wallet.identifier()))
            .frame(egui::Frame {
                inner_margin: Margin {
                    left: (View::far_left_inset_margin(ui) + 4.0) as i8,
                    right: (View::get_right_inset() + 4.0) as i8,
                    top: 4.0 as i8,
                    bottom: 0.0 as i8,
                },
                fill: Colors::fill(),
                ..Default::default()
            })
            .show_animated_inside(ui, show_account, |ui| {
                let rect = ui.available_rect_before_wrap();
                self.account_content.ui(ui, cb);
                // Draw content divider lines.
                let r = {
                    let mut r = rect.clone();
                    r.min.x -= 4.0 + View::far_left_inset_margin(ui);
                    r.min.y -= 4.0;
                    r.max.x += 4.0 + View::get_right_inset();
                    r
                };
                View::line(ui, LinePosition::BOTTOM, &r, Colors::item_stroke());
                if dual_panel && show_wallets_dual && !show_qr_scan {
                    View::line(ui, LinePosition::LEFT, &r, Colors::item_stroke());
                }
            });

        // Show wallet tabs.
        let show_tabs = !hide_tabs && !self.account_content.qr_scan_showing();
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
            .show_animated_inside(ui, show_tabs, |ui| {
                let rect = ui.available_rect_before_wrap();
                View::max_width_ui(ui, Content::SIDE_PANEL_WIDTH * 1.3, |ui| {
                    self.tabs_ui(ui);
                });
                let rect = {
                    let mut r = rect.clone();
                    r.min.x -= View::far_left_inset_margin(ui) + View::TAB_ITEMS_PADDING;
                    r.min.y -= View::TAB_ITEMS_PADDING;
                    r.max.x += View::get_right_inset() + View::TAB_ITEMS_PADDING;
                    r.max.y += View::get_bottom_inset() + View::TAB_ITEMS_PADDING;
                    r
                };
                // Draw content divider line.
                View::line(ui, LinePosition::TOP, &rect, Colors::stroke());
            });

        // Show tab content.
        egui::CentralPanel::default()
            .frame(egui::Frame {
                inner_margin:  Margin {
                    left: (View::far_left_inset_margin(ui) + 4.0) as i8,
                    right: (View::get_right_inset() + 4.0) as i8,
                    top: 0.0 as i8,
                    bottom: 4.0 as i8,
                },
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                let rect = ui.available_rect_before_wrap();
                let tab_type = self.current_tab.get_type();
                let show_sync = (tab_type != WalletTabType::Settings || hide_tabs) &&
                    sync_ui(ui, &self.wallet);
                if !show_sync {
                    if tab_type != WalletTabType::Txs {
                        ui.add_space(3.0);
                        ScrollArea::vertical()
                            .id_salt(Id::from("wallet_scroll")
                                .with(tab_type.name())
                                .with(wallet_id))
                            .auto_shrink([false; 2])
                            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                            .show(ui, |ui| {
                                View::max_width_ui(ui, Content::SIDE_PANEL_WIDTH * 1.3, |ui| {
                                    self.current_tab.ui(ui, &self.wallet, cb);
                                });
                            });
                    } else {
                        self.current_tab.ui(ui, &self.wallet, cb);
                    }
                }
                let rect = {
                    let mut r = rect.clone();
                    r.min.x -= View::far_left_inset_margin(ui) + 4.0;
                    r.max.x += View::get_right_inset() + 4.0;
                    r.max.y += 4.0;
                    r
                };
                // Draw cover when QR code scanner is active.
                if show_qr_scan {
                    View::content_cover_ui(ui, rect, "wallet_tab", || {
                        cb.stop_camera();
                        self.account_content.close_qr_scan();
                    });
                }
                // Draw content divider line.
                if dual_panel && show_wallets_dual {
                    View::line(ui, LinePosition::LEFT, &rect, Colors::item_stroke());
                }
            });
    }
}

impl WalletContent {
    /// Create new instance with optional data.
    pub fn new(wallet: Wallet, data: Option<String>) -> Self {
        let account_content =  AccountContent::new(wallet.clone());
        let mut content = Self {
            wallet,
            current_tab: Box::new(WalletTransactions::default()),
            account_content,
        };
        if data.is_some() {
            content.on_data(data);
        }
        content
    }

    /// Check if QR code scanner is opened.
    pub fn qr_scan_showing(&self) -> bool {
        self.account_content.qr_scan_showing()
    }

    /// Close QR code scanner.
    pub fn close_qr_scan(&mut self) {
        self.account_content.close_qr_scan();
    }

    /// Handle data from deeplink or opened file.
    pub fn on_data(&mut self, data: Option<String>) {
        self.current_tab = Box::new(WalletMessages::new(data));
    }

    /// Check when to block tabs navigation on sync progress.
    pub fn block_navigation_on_sync(wallet: &Wallet) -> bool {
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
    fn tabs_ui(&mut self, ui: &mut egui::Ui) {
        ui.scope(|ui| {
            // Setup spacing between tabs.
            ui.style_mut().spacing.item_spacing = egui::vec2(View::TAB_ITEMS_PADDING, 0.0);

            // Setup vertical padding inside buttons.
            ui.style_mut().spacing.button_padding = egui::vec2(0.0, 4.0);

            let current_type = self.current_tab.get_type();
            ui.columns(4, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::tab_button(ui, GRAPH, current_type == WalletTabType::Txs, |_| {
                        self.current_tab = Box::new(WalletTransactions::default());
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    let is_messages = current_type == WalletTabType::Messages;
                    View::tab_button(ui, CHAT_CIRCLE_TEXT, is_messages, |_| {
                        self.current_tab = Box::new(
                            WalletMessages::new(None)
                        );
                    });
                });
                columns[2].vertical_centered_justified(|ui| {
                    View::tab_button(ui, BRIDGE, current_type == WalletTabType::Transport, |_| {
                        self.current_tab = Box::new(WalletTransport::new(None));
                    });
                });
                columns[3].vertical_centered_justified(|ui| {
                    View::tab_button(ui, GEAR_FINE, current_type == WalletTabType::Settings, |ui| {
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
