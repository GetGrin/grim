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
use egui::{Align, CornerRadius, Id, Layout, Margin, RichText, ScrollArea, StrokeKind};
use std::time::Duration;

use crate::gui::icons::{ARROW_LEFT, CARET_RIGHT, COMPUTER_TOWER, FOLDER_OPEN, FOLDER_PLUS, GEAR, GLOBE, GLOBE_SIMPLE, LOCK_KEY, PLUS, SIDEBAR_SIMPLE, SUITCASE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::settings::SettingsContent;
use crate::gui::views::types::{ContentContainer, LinePosition, ModalPosition, TitleContentType, TitleType};
use crate::gui::views::wallets::creation::WalletCreationContent;
use crate::gui::views::wallets::modals::{AddWalletModal, OpenWalletModal, WalletConnectionModal, WalletsModal};
use crate::gui::views::wallets::types::WalletTabType;
use crate::gui::views::wallets::wallet::types::wallet_status_text;
use crate::gui::views::wallets::WalletContent;
use crate::gui::views::{Content, Modal, TitlePanel, View};
use crate::gui::Colors;
use crate::wallet::types::ConnectionMethod;
use crate::wallet::{ExternalConnection, Wallet, WalletList};
use crate::AppConfig;

/// Wallets content.
pub struct WalletsContent {
    /// List of wallets.
    wallets: WalletList,

    /// Initial wallet creation [`Modal`] content.
    add_wallet_modal_content: AddWalletModal,
    /// Wallet opening [`Modal`] content.
    open_wallet_content: OpenWalletModal,
    /// Wallet connection selection [`Modal`] content.
    conn_selection_content: WalletConnectionModal,
    /// Wallet selection [`Modal`] content.
    wallet_selection_content: WalletsModal,

    /// Selected [`Wallet`] content.
    wallet_content: Option<WalletContent>,
    /// Wallet creation content.
    creation_content: Option<WalletCreationContent>,

    /// Settings content.
    settings_content: Option<SettingsContent>,
}

const ADD_WALLET_MODAL: &'static str = "wallets_add_modal";
const OPEN_WALLET_MODAL: &'static str = "wallets_open_wallet";
const SELECT_CONNECTION_MODAL: &'static str = "wallets_select_conn_modal";
const SELECT_WALLET_MODAL: &'static str = "wallets_select_modal";

impl Default for WalletsContent {
    fn default() -> Self {
        Self {
            wallets: WalletList::default(),
            wallet_selection_content: WalletsModal::new(None, None, true),
            open_wallet_content: OpenWalletModal::new(None),
            add_wallet_modal_content: AddWalletModal::default(),
            conn_selection_content: WalletConnectionModal::new(ConnectionMethod::Integrated),
            wallet_content: None,
            creation_content: None,
            settings_content: None,
        }
    }
}

impl ContentContainer for WalletsContent {
    fn modal_ids(&self) -> Vec<&'static str> {
        vec![
            ADD_WALLET_MODAL,
            OPEN_WALLET_MODAL,
            SELECT_CONNECTION_MODAL,
            SELECT_WALLET_MODAL,
        ]
    }

    fn modal_ui(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        match modal.id {
            ADD_WALLET_MODAL => {
                self.add_wallet_modal_content.ui(ui, modal, cb, |name, pass| {
                    self.creation_content = Some(
                        WalletCreationContent::new(name.clone(), pass.clone())
                    );
                });
            },
            OPEN_WALLET_MODAL => {
                self.open_wallet_content.ui(ui, modal, cb, |pass, _| {
                    if let Some(content) = &self.wallet_content {
                        return match content.wallet.open(pass) {
                            Ok(_) => true,
                            Err(_) => false
                        };
                    }
                    true
                });
            },
            SELECT_CONNECTION_MODAL => {
                self.conn_selection_content.ui(ui, modal, cb, |conn| {
                    if let Some(wallet_content) = &self.wallet_content {
                        wallet_content.wallet.update_connection(&conn);
                    }
                });
            }
            SELECT_WALLET_MODAL => {
                self.wallet_selection_content.ui(ui, &mut self.wallets, |wallet, data| {
                    if !wallet.is_open() {
                        self.open_wallet_content = OpenWalletModal::new(data.clone());
                        Modal::new(OPEN_WALLET_MODAL)
                            .position(ModalPosition::CenterTop)
                            .title(t!("wallets.open"))
                            .show();
                    }
                    self.wallet_content = Some(WalletContent::new(wallet, data));
                });
            }
            _ => {}
        }
    }

    fn container_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        if let Some(data) = crate::consume_incoming_data() {
            if !data.is_empty() {
                self.on_data(ui, Some(data));
            }
        }

        let showing_settings = self.showing_settings();
        let creating_wallet = self.creating_wallet();
        let showing_wallet = self.showing_wallet() && !creating_wallet && !showing_settings;
        let dual_panel = is_dual_panel_mode(ui);
        let content_width = ui.available_width();
        let list_hidden = showing_settings || creating_wallet || self.wallets.list().is_empty()
            || (showing_wallet && self.wallet_content.as_ref().unwrap().qr_scan_showing())
            || (dual_panel && showing_wallet && !AppConfig::show_wallets_at_dual_panel())
            || (!dual_panel && showing_wallet);

        // Show title panel.
        self.title_ui(ui, dual_panel, showing_wallet, cb);

        egui::SidePanel::right("wallet_panel")
            .resizable(false)
            .exact_width(if list_hidden {
                content_width
            } else {
                content_width - Content::SIDE_PANEL_WIDTH
            })
            .frame(egui::Frame {
                ..Default::default()
            })
            .show_animated_inside(ui, showing_wallet, |ui| {
                // Show opened wallet content.
                if let Some(content) = self.wallet_content.as_mut() {
                    content.ui(ui, cb);
                }
            });

        if !list_hidden {
            egui::TopBottomPanel::bottom("wallets_bottom_panel")
                .frame(egui::Frame {
                    inner_margin: Margin {
                        left: (View::far_left_inset_margin(ui) + View::TAB_ITEMS_PADDING) as i8,
                        right: (View::far_right_inset_margin(ui) + View::TAB_ITEMS_PADDING) as i8,
                        top: View::TAB_ITEMS_PADDING as i8,
                        bottom: (View::get_bottom_inset() + View::TAB_ITEMS_PADDING) as i8,
                    },
                    fill: Colors::fill(),
                    ..Default::default()
                })
                .resizable(false)
                .show_inside(ui, |ui| {
                    let rect = ui.available_rect_before_wrap();

                    // Setup spacing between tabs.
                    ui.style_mut().spacing.item_spacing = egui::vec2(View::TAB_ITEMS_PADDING, 0.0);
                    // Setup vertical padding inside buttons.
                    ui.style_mut().spacing.button_padding = egui::vec2(10.0, 4.0);

                    ui.vertical_centered(|ui| {
                        let pressed = Modal::opened() == Some(ADD_WALLET_MODAL);
                        View::tab_button(ui, PLUS, pressed, |_| {
                            self.show_add_wallet_modal();
                        });
                    });

                    // Draw content divider line.
                    let r = {
                        let mut r = rect.clone();
                        r.min.y -= View::TAB_ITEMS_PADDING;
                        r.min.x -= View::TAB_ITEMS_PADDING;
                        r.max.x += View::TAB_ITEMS_PADDING;
                        r
                    };
                    View::line(ui, LinePosition::TOP, &r, Colors::stroke());
                });
        }

        egui::SidePanel::left("wallet_list_panel")
            .exact_width(if dual_panel && showing_wallet {
                Content::SIDE_PANEL_WIDTH
            } else {
                content_width
            })
            .resizable(false)
            .frame(egui::Frame {
                inner_margin: Margin {
                    left: (View::far_left_inset_margin(ui) + 4.0) as i8,
                    right: (View::far_right_inset_margin(ui) + 4.0) as i8,
                    top: 3.0 as i8,
                    bottom: 4.0 as i8,
                },
                fill: Colors::fill_deep(),
                ..Default::default()
            })
            .show_animated_inside(ui, !list_hidden, |ui| {
                if !dual_panel && !showing_wallet {
                    ui.ctx().request_repaint_after(Duration::from_millis(1000));
                }
                // Show wallet list.
                self.wallet_list_ui(ui);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: if creating_wallet {
                    Colors::TRANSPARENT
                } else {
                    Colors::fill_deep()
                },
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                if self.showing_settings() {
                    if let Some(c) = &mut self.settings_content {
                        ScrollArea::vertical()
                            .id_salt("app_settings_wallets")
                            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                ui.add_space(1.0);
                                ui.vertical_centered(|ui| {
                                    // Show application settings content.
                                    View::max_width_ui(ui,
                                                       Content::SIDE_PANEL_WIDTH * 1.3,
                                                       |ui| {
                                                           c.ui(ui, cb);
                                                       });
                                });
                            });
                    }
                } else if self.creating_wallet() {
                    let creation = self.creation_content.as_mut().unwrap();
                    let pass = creation.pass.clone();
                    let mut created = false;
                    // Show wallet creation content.
                    creation.content_ui(ui, cb, |wallet| {
                        self.wallets.add(wallet.clone());
                        if let Ok(_) = wallet.open(pass.clone()) {
                            self.wallet_content = Some(WalletContent::new(wallet, None));
                        }
                        created = true;
                    });
                    if created {
                        self.creation_content = None;
                    }
                } else if self.wallets.list().is_empty() {
                    View::center_content(ui, 350.0 + View::get_bottom_inset(), |ui| {
                        View::app_logo_name_version(ui);
                        ui.add_space(4.0);

                        let text = t!("wallets.create_desc");
                        ui.label(RichText::new(text)
                            .size(16.0)
                            .color(Colors::gray())
                        );
                        ui.add_space(8.0);
                        // Show wallet creation button.
                        let add_text = format!("{} {}", FOLDER_PLUS, t!("wallets.add"));
                        View::button(ui, add_text, Colors::white_or_black(false), || {
                            self.show_add_wallet_modal();
                        });
                    });
                } else {
                    return;
                }
            });
    }
}

impl WalletsContent {
    /// Called to navigate back, return `true` if action was not consumed.
    pub fn on_back(&mut self, cb: &dyn PlatformCallbacks) -> bool {
        if self.showing_settings() {
            // Close settings.
            self.settings_content = None;
            return false;
        } else if self.creating_wallet() {
            // Close wallet creation.
            let creation = self.creation_content.as_mut().unwrap();
            if creation.on_back() {
                self.creation_content = None;
            }
            return false;
        } else if self.showing_wallet() {
            let content = self.wallet_content.as_mut().unwrap();
            // Close opened QR code scanner.
            if content.qr_scan_showing() {
                cb.stop_camera();
                content.close_qr_scan();
                return false;
            }
            // Close account list.
            if content.account_list_showing() {
                content.close_qr_scan();
                return false;
            }
            // Close opened wallet.
            self.wallet_content = None;
            return false;
        }
        true
    }

    /// Check if opened wallet is showing.
    pub fn showing_wallet(&self) -> bool {
        if let Some(wallet_content) = &self.wallet_content {
            let w = &wallet_content.wallet;
            return w.is_open() && !w.is_deleted() &&
                w.get_config().chain_type == AppConfig::chain_type();
        }
        false
    }

    /// Check if wallet is creating.
    pub fn creating_wallet(&self) -> bool {
        self.creation_content.is_some()
    }

    /// Check if application settings are showing.
    pub fn showing_settings(&self) -> bool {
        self.settings_content.is_some()
    }

    /// Handle data from deeplink or opened file.
    fn on_data(&mut self, ui: &mut egui::Ui, data: Option<String>) {
        let wallets_size = self.wallets.list().len();
        if wallets_size == 0 {
            return;
        }
        // Close network panel on single panel mode.
        if !Content::is_dual_panel_mode(ui.ctx()) && Content::is_network_panel_open() {
            Content::toggle_network_panel();
        }
        // Pass data to single wallet or show wallets selection.
        if wallets_size == 1 {
            let w = self.wallets.list()[0].clone();
            if w.is_open() {
                self.wallet_content = Some(WalletContent::new(w, data));
            } else {
                self.show_opening_modal(w, data);
            }
        } else {
            self.wallet_selection_content = WalletsModal::new(None, data, true);
            Modal::new(SELECT_WALLET_MODAL)
                .position(ModalPosition::Center)
                .title(t!("network_settings.choose_wallet"))
                .show();
        }
    }

    /// Show initial wallet creation [`Modal`].
    pub fn show_add_wallet_modal(&mut self) {
        self.add_wallet_modal_content = AddWalletModal::default();
        Modal::new(ADD_WALLET_MODAL)
            .position(ModalPosition::CenterTop)
            .title(t!("wallets.add"))
            .show();
    }

    /// Draw [`TitlePanel`] content.
    fn title_ui(&mut self,
                ui: &mut egui::Ui,
                dual_panel: bool,
                show_wallet: bool,
                cb: &dyn PlatformCallbacks) {
        let show_list = AppConfig::show_wallets_at_dual_panel();
        let showing_settings = self.showing_settings();
        let creating_wallet = self.creating_wallet();
        let account_list_showing = show_wallet && self.wallet_content
            .as_ref()
            .unwrap()
            .account_list_showing();
        let qr_scan = {
            let mut scan = false;
            if show_wallet {
                scan = self.wallet_content.as_mut().unwrap().qr_scan_showing();
            }
            scan
        };
        // Setup title.
        let title_content = if show_wallet && (!dual_panel
            || (dual_panel && !show_list)) && !creating_wallet && !showing_settings {
            let wallet_content = self.wallet_content.as_ref().unwrap();
            let wallet_tab_type = wallet_content.current_tab.get_type();
            let title_text = if account_list_showing {
                t!("wallets.accounts")
            } else if qr_scan {
                t!("scan_qr")
            } else {
                wallet_tab_type.name()
            };
            if wallet_tab_type == WalletTabType::Settings {
                TitleType::Single(TitleContentType::Title(title_text))
            } else {
                let subtitle_text = wallet_content.wallet.get_config().name;
                TitleType::Single(TitleContentType::WithSubTitle(title_text, subtitle_text, false))
            }
        } else {
            let title_text = if showing_settings {
                t!("settings")
            } else if qr_scan {
                t!("scan_qr")
            } else if creating_wallet {
                t!("wallets.add")
            } else {
                t!("wallets.title")
            };
            let dual_title = !showing_settings && !qr_scan && !creating_wallet &&
                show_wallet && dual_panel;
            if dual_title {
                let wallet_content = self.wallet_content.as_ref().unwrap();
                let wallet_tab_type = wallet_content.current_tab.get_type();
                let wallet_title_text = if account_list_showing {
                    t!("wallets.accounts")
                } else {
                    wallet_tab_type.name()
                };
                let wallet_title_content = if wallet_tab_type == WalletTabType::Settings {
                    TitleContentType::Title(wallet_title_text)
                } else {
                    let subtitle_text = wallet_content.wallet.get_config().name;
                    TitleContentType::WithSubTitle(wallet_title_text, subtitle_text, false)
                };
                TitleType::Dual(TitleContentType::Title(title_text), wallet_title_content)
            } else {
                TitleType::Single(TitleContentType::Title(title_text))
            }
        };

        // Draw title panel.
        let mut show_settings = false;
        let showing_settings = self.showing_settings();
        TitlePanel::new(Id::new("wallets_title_panel")).ui(title_content, |ui| {
            if self.showing_settings() {
                View::title_button_big(ui, ARROW_LEFT, |_| {
                    self.settings_content = None;
                });
            } else if show_wallet && !dual_panel {
                View::title_button_big(ui, ARROW_LEFT, |_| {
                    // Close QR code scanner.
                    let wallet_qr_scan = self.wallet_content
                        .as_ref()
                        .unwrap()
                        .qr_scan_showing();
                    if wallet_qr_scan {
                        cb.stop_camera();
                        self.wallet_content.as_mut().unwrap().close_qr_scan();
                        return;
                    }
                    // Close account list.
                    if account_list_showing {
                        self.wallet_content.as_mut().unwrap().close_account_list();
                        return;
                    }
                    self.wallet_content = None;
                });
            } else if self.creating_wallet() {
                let mut close = false;
                if let Some(creation) = self.creation_content.as_mut() {
                    View::title_button_big(ui, ARROW_LEFT, |_| {
                        if creation.on_back() {
                            close = true;
                        }
                    });
                }
                if close {
                    self.creation_content = None;
                }
            } else if show_wallet && dual_panel {
                if qr_scan {
                    View::title_button_big(ui, ARROW_LEFT, |_| {
                        cb.stop_camera();
                        self.wallet_content.as_mut().unwrap().close_qr_scan();
                    });
                } else {
                    let list_icon = if show_list {
                        SIDEBAR_SIMPLE
                    } else {
                        SUITCASE
                    };
                    View::title_button_big(ui, list_icon, |_| {
                        AppConfig::toggle_show_wallets_at_dual_panel();
                    });
                }
            } else if !Content::is_dual_panel_mode(ui.ctx()) {
                View::title_button_big(ui, GLOBE, |_| {
                    Content::toggle_network_panel();
                });
            }
        }, |ui| {
            if !showing_settings {
                View::title_button_big(ui, GEAR, |_| {
                    // Show application settings.
                    show_settings = true;
                });
            }
        }, ui);
        if show_settings {
            self.settings_content = Some(SettingsContent::default());
        }
    }

    /// Draw list of wallets.
    fn wallet_list_ui(&mut self, ui: &mut egui::Ui) {
        ScrollArea::vertical()
            .id_salt("wallet_list_scroll")
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                View::max_width_ui(ui, Content::SIDE_PANEL_WIDTH * 1.3, |ui| {
                    // Show application logo and name.
                    View::app_logo_name_version(ui);
                    ui.add_space(15.0);

                    let list = self.wallets.list().clone();
                    for w in &list {
                        // Remove deleted.
                        if w.is_deleted() {
                            self.wallet_content = None;
                            self.wallets.remove(w.get_config().id);
                            ui.ctx().request_repaint();
                            continue;
                        }
                        // Check if wallet reopen is needed.
                        if w.reopen_needed() && !w.is_open() {
                            w.set_reopen(false);
                            self.show_opening_modal(w.clone(), None);
                        }
                        self.wallet_item_ui(ui, w);
                        ui.add_space(5.0);
                    }
                });
            });
    }

    /// Draw wallet list item.
    fn wallet_item_ui(&mut self, ui: &mut egui::Ui, wallet: &Wallet) {
        let config = wallet.get_config();
        let current = if let Some(content) = &self.wallet_content {
            content.wallet.get_config().id == config.id && wallet.is_open()
        } else {
            false
        };

        // Draw round background.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(78.0);
        let rounding = View::item_rounding(0, 1, false);
        let bg = if current {
            Colors::fill_deep()
        } else {
            Colors::fill()
        };
        ui.painter().rect(rect, rounding, bg, View::item_stroke(), StrokeKind::Middle);

        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            if !wallet.is_open() {
                // Show button to open closed wallet.
                View::item_button(ui, View::item_rounding(0, 1, true), FOLDER_OPEN, None, || {
                    self.show_opening_modal(wallet.clone(), None);
                });
                if !wallet.syncing() {
                    let mut show_selection = false;
                    View::item_button(ui, CornerRadius::default(), GLOBE, None, || {
                        self.wallet_content = Some(WalletContent::new(wallet.clone(), None));
                        self.conn_selection_content =
                            WalletConnectionModal::new(wallet.get_current_connection());
                        // Show connection selection modal.
                        Modal::new(SELECT_CONNECTION_MODAL)
                            .position(ModalPosition::CenterTop)
                            .title(t!("wallets.conn_method"))
                            .show();
                        show_selection = true;
                    });
                    if show_selection {
                        ExternalConnection::check(None, ui.ctx());
                    }
                }
            } else {
                if !current {
                    // Show button to select opened wallet.
                    View::item_button(ui, View::item_rounding(0, 1, true), CARET_RIGHT, None, || {
                        self.wallet_content = Some(WalletContent::new(wallet.clone(), None));
                    });
                }
                // Show button to close opened wallet.
                if !wallet.is_closing()  {
                    View::item_button(ui, if !current {
                        CornerRadius::default()
                    } else {
                        View::item_rounding(0, 1, true)
                    }, LOCK_KEY, None, || {
                        wallet.close();
                    });
                }
            }

            let layout_size = ui.available_size();
            ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                ui.add_space(6.0);
                ui.vertical(|ui| {
                    ui.add_space(3.0);
                    // Show wallet name text.
                    let name_color = if current {
                        Colors::white_or_black(true)
                    } else {
                        Colors::title(false)
                    };
                    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                            ui.add_space(1.0);
                            View::ellipsize_text(ui, config.name, 18.0, name_color);
                    });

                    // Show wallet status text.
                    View::ellipsize_text(ui, wallet_status_text(wallet), 15.0, Colors::text(false));
                    ui.add_space(1.0);

                    // Show wallet connection text.
                    let connection = wallet.get_current_connection();
                    let conn_text = match connection {
                        ConnectionMethod::Integrated => {
                            format!("{} {}", COMPUTER_TOWER, t!("network.node"))
                        }
                        ConnectionMethod::External(_, url) => format!("{} {}", GLOBE_SIMPLE, url)
                    };
                    ui.label(RichText::new(conn_text).size(15.0).color(Colors::gray()));
                    ui.add_space(3.0);
                });
            });
        });
    }

    /// Show [`Modal`] to select and open wallet.
    fn show_opening_modal(&mut self, wallet: Wallet, data: Option<String>) {
        self.wallet_content = Some(WalletContent::new(wallet.clone(), None));
        self.open_wallet_content = OpenWalletModal::new(data);
        Modal::new(OPEN_WALLET_MODAL)
            .position(ModalPosition::CenterTop)
            .title(t!("wallets.open"))
            .show();
    }
}

/// Check if it's possible to show [`WalletsContent`] and [`WalletContent`] panels at same time.
fn is_dual_panel_mode(ui: &mut egui::Ui) -> bool {
    let dual_panel_root = Content::is_dual_panel_mode(ui.ctx());
    let max_width = ui.available_width();
    dual_panel_root && max_width >= (Content::SIDE_PANEL_WIDTH * 2.0) + View::get_right_inset()
}