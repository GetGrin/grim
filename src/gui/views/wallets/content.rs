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
use egui::{Align, Id, Layout, Margin, RichText, Rounding, ScrollArea};
use egui::scroll_area::ScrollBarVisibility;

use crate::AppConfig;
use crate::gui::Colors;
use crate::gui::icons::{ARROW_LEFT, CARET_RIGHT, COMPUTER_TOWER, FOLDER_OPEN, FOLDER_PLUS, GEAR, GLOBE, GLOBE_SIMPLE, LOCK_KEY, PLUS, SIDEBAR_SIMPLE, SUITCASE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, Content, TitlePanel, View};
use crate::gui::views::types::{ModalContainer, ModalPosition, LinePosition, TitleContentType, TitleType};
use crate::gui::views::wallets::creation::WalletCreation;
use crate::gui::views::wallets::modals::{AddWalletModal, OpenWalletModal, WalletConnectionModal, WalletsModal};
use crate::gui::views::wallets::types::WalletTabType;
use crate::gui::views::wallets::wallet::types::wallet_status_text;
use crate::gui::views::wallets::WalletContent;
use crate::wallet::{ExternalConnection, Wallet, WalletList};
use crate::wallet::types::ConnectionMethod;

/// Wallets content.
pub struct WalletsContent {
    /// List of wallets.
    wallets: WalletList,

    /// Initial wallet creation [`Modal`] content.
    add_wallet_modal_content: Option<AddWalletModal>,
    /// Wallet opening [`Modal`] content.
    open_wallet_content: Option<OpenWalletModal>,
    /// Wallet connection selection content.
    conn_selection_content: Option<WalletConnectionModal>,
    /// Wallet selection [`Modal`] content.
    wallet_selection_content: Option<WalletsModal>,

    /// Selected [`Wallet`] content.
    wallet_content: Option<WalletContent>,
    /// Wallet creation content.
    creation_content: Option<WalletCreation>,

    /// Flag to show [`Wallet`] list at dual panel mode.
    show_wallets_at_dual_panel: bool,

    /// [`Modal`] identifiers allowed at this ui container.
    modal_ids: Vec<&'static str>
}

const ADD_WALLET_MODAL: &'static str = "wallets_add_modal";
const OPEN_WALLET_MODAL: &'static str = "wallets_open_wallet";
const SELECT_CONNECTION_MODAL: &'static str = "wallets_select_conn_modal";
const SELECT_WALLET_MODAL: &'static str = "wallets_select_modal";

impl Default for WalletsContent {
    fn default() -> Self {
        Self {
            wallets: WalletList::default(),
            wallet_selection_content: None,
            open_wallet_content: None,
            conn_selection_content: None,
            wallet_content: None,
            creation_content: None,
            show_wallets_at_dual_panel: AppConfig::show_wallets_at_dual_panel(),
            modal_ids: vec![
                ADD_WALLET_MODAL,
                OPEN_WALLET_MODAL,
                SELECT_CONNECTION_MODAL,
                SELECT_WALLET_MODAL,
            ],
            add_wallet_modal_content: None,
        }
    }
}

impl ModalContainer for WalletsContent {
    fn modal_ids(&self) -> &Vec<&'static str> {
        &self.modal_ids
    }

    fn modal_ui(&mut self,
                ui: &mut egui::Ui,
                modal: &Modal,
                cb: &dyn PlatformCallbacks) {
        match modal.id {
            ADD_WALLET_MODAL => {
                if let Some(content) = self.add_wallet_modal_content.as_mut() {
                    content.ui(ui, modal, cb, |name, pass| {
                        self.creation_content = Some(
                            WalletCreation::new(name.clone(), pass.clone())
                        );
                    });
                }
                if self.creation_content.is_some() {
                    self.add_wallet_modal_content = None;
                }
            },
            OPEN_WALLET_MODAL => {
                let mut open = false;
                if let Some(open_content) = self.open_wallet_content.as_mut() {
                    open_content.ui(ui, modal, cb, |wallet, data| {
                        self.wallet_content = Some(WalletContent::new(wallet, data));
                        open = true;
                    });
                }
                if open {
                    self.open_wallet_content = None;
                }
            },
            SELECT_CONNECTION_MODAL => {
                if let Some(content) = self.conn_selection_content.as_mut() {
                    content.ui(ui, modal, cb, |conn| {
                        if let Some(wallet_content) = &self.wallet_content {
                            wallet_content.wallet.update_connection(&conn);
                        }
                    });
                }
            }
            SELECT_WALLET_MODAL => {
                let mut select = false;
                if let Some(content) = self.wallet_selection_content.as_mut() {
                    content.ui(ui, modal, &mut self.wallets, cb, |wallet, data| {
                        self.wallet_content = Some(WalletContent::new(wallet, data));
                        select = true;
                    });
                }
                if select {
                    self.wallet_selection_content = None;
                }
            }
            _ => {}
        }
    }
}

impl WalletsContent {
    /// Draw wallets content.
    pub fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        self.current_modal_ui(ui, cb);

        let creating_wallet = self.creating_wallet();
        let showing_wallet = self.showing_wallet() && !creating_wallet;
        let dual_panel = Self::is_dual_panel_mode(ui);
        let content_width = ui.available_width();
        let list_hidden = creating_wallet || self.wallets.list().is_empty()
            || (showing_wallet && self.wallet_content.as_ref().unwrap().qr_scan_content.is_some())
            || (dual_panel && showing_wallet && !self.show_wallets_at_dual_panel)
            || (!dual_panel && showing_wallet);

        // Show title panel.
        self.title_ui(ui, dual_panel, showing_wallet, cb);

        if showing_wallet {
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
                .show_inside(ui, |ui| {
                    // Show opened wallet content.
                    if let Some(content) = self.wallet_content.as_mut() {
                        content.ui(ui, cb);
                    }
                });
        }

        if !list_hidden {
            egui::TopBottomPanel::bottom("wallets_bottom_panel")
                .frame(egui::Frame {
                    inner_margin: Margin {
                        left: View::far_left_inset_margin(ui) + View::TAB_ITEMS_PADDING,
                        right: View::far_right_inset_margin(ui) + View::TAB_ITEMS_PADDING,
                        top: View::TAB_ITEMS_PADDING,
                        bottom: View::get_bottom_inset() + View::TAB_ITEMS_PADDING,
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
                            self.show_add_wallet_modal(cb);
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

            egui::SidePanel::left("wallet_list_panel")
                .exact_width(if dual_panel && showing_wallet {
                    Content::SIDE_PANEL_WIDTH
                } else {
                    content_width
                })
                .resizable(false)
                .frame(egui::Frame {
                    inner_margin: Margin {
                        left: View::far_left_inset_margin(ui) + 4.0,
                        right: View::far_right_inset_margin(ui) + 4.0,
                        top: 3.0,
                        bottom: 4.0,
                    },
                    fill: Colors::fill_deep(),
                    ..Default::default()
                })
                .show_inside(ui, |ui| {
                    if !dual_panel && !showing_wallet {
                        ui.ctx().request_repaint_after(Duration::from_millis(1000));
                    }
                    // Show wallet list.
                    self.wallet_list_ui(ui, cb);
                });
        }

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
                if self.creation_content.is_some() {
                    let creation = self.creation_content.as_mut().unwrap();
                    let pass = creation.pass.clone();
                    let mut created = false;
                    // Show wallet creation content.
                    creation.ui(ui, cb, |wallet| {
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
                            self.show_add_wallet_modal(cb);
                        });
                    });
                } else {
                    return;
                }
            });
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

    /// Handle data from deeplink or opened file.
    pub fn on_data(&mut self, ui: &mut egui::Ui, data: Option<String>, cb: &dyn PlatformCallbacks) {
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
                self.show_opening_modal(w, data, cb);
            }
        } else {
            self.show_wallet_selection_modal(data);
        }
    }

    /// Show initial wallet creation [`Modal`].
    pub fn show_add_wallet_modal(&mut self, cb: &dyn PlatformCallbacks) {
        self.add_wallet_modal_content = Some(AddWalletModal::default());
        Modal::new(ADD_WALLET_MODAL)
            .position(ModalPosition::CenterTop)
            .title(t!("wallets.add"))
            .show();
        cb.show_keyboard();
    }

    /// Show wallet selection with provided optional data.
    fn show_wallet_selection_modal(&mut self, data: Option<String>) {
        self.wallet_selection_content = Some(WalletsModal::new(None, data, true));
        Modal::new(SELECT_WALLET_MODAL)
            .position(ModalPosition::Center)
            .title(t!("network_settings.choose_wallet"))
            .show();
    }

    /// Handle Back key event returning `false` when event was handled.
    pub fn on_back(&mut self, cb: &dyn PlatformCallbacks) -> bool {
        if self.creation_content.is_some() {
            // Close wallet creation.
            let creation = self.creation_content.as_mut().unwrap();
            if creation.on_back() {
                self.creation_content = None;
            }
            return false;
        } else {
            if self.showing_wallet() {
                let content = self.wallet_content.as_mut().unwrap();
                // Close opened QR code scanner.
                if content.qr_scan_content.is_some() {
                    cb.stop_camera();
                    content.qr_scan_content = None;
                    return false;
                }
                // Close opened wallet.
                self.wallet_content = None;
                return false;
            }
        }
        true
    }

    /// Draw [`TitlePanel`] content.
    fn title_ui(&mut self,
                ui: &mut egui::Ui,
                dual_panel: bool,
                show_wallet: bool,
                cb: &dyn PlatformCallbacks) {
        let show_list = self.show_wallets_at_dual_panel;
        let creating_wallet = self.creating_wallet();
        let qr_scan = {
            let mut scan = false;
            if show_wallet {
                scan = self.wallet_content.as_mut().unwrap().qr_scan_content.is_some();
            }
            scan
        };
        // Setup title.
        let title_content = if show_wallet && (!dual_panel
            || (dual_panel && !show_list)) && !creating_wallet {
            let wallet_content = self.wallet_content.as_ref().unwrap();
            let wallet_tab_type = wallet_content.current_tab.get_type();
            let title_text = if qr_scan {
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
            let title_text = if qr_scan {
                t!("scan_qr")
            } else if creating_wallet {
                t!("wallets.add")
            } else {
                t!("wallets.title")
            };
            let dual_title = !qr_scan && !creating_wallet && show_wallet && dual_panel;
            if dual_title {
                let wallet_content = self.wallet_content.as_ref().unwrap();
                let wallet_tab_type = wallet_content.current_tab.get_type();
                let wallet_title_text = wallet_tab_type.name();
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
        TitlePanel::new(Id::new("wallets_title_panel")).ui(title_content, |ui| {
            if show_wallet && !dual_panel {
                View::title_button_big(ui, ARROW_LEFT, |_| {
                    let wallet_qr_scan = self.wallet_content
                        .as_ref()
                        .unwrap()
                        .qr_scan_content
                        .is_some();
                    if wallet_qr_scan {
                        cb.stop_camera();
                        self.wallet_content.as_mut().unwrap().qr_scan_content = None;
                        return;
                    }
                    self.wallet_content = None;
                });
            } else if self.creation_content.is_some() {
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
                        self.wallet_content.as_mut().unwrap().qr_scan_content = None;
                    });
                } else {
                    let list_icon = if show_list {
                        SIDEBAR_SIMPLE
                    } else {
                        SUITCASE
                    };
                    View::title_button_big(ui, list_icon, |_| {
                        self.show_wallets_at_dual_panel = !show_list;
                        AppConfig::toggle_show_wallets_at_dual_panel();
                    });
                }
            } else if !Content::is_dual_panel_mode(ui.ctx()) {
                View::title_button_big(ui, GLOBE, |_| {
                    Content::toggle_network_panel();
                });
            };
        }, |ui| {
            View::title_button_big(ui, GEAR, |_| {
                // Show settings modal.
                Modal::new(Content::SETTINGS_MODAL)
                    .position(ModalPosition::CenterTop)
                    .title(t!("settings"))
                    .show();
            });
        }, ui);
    }

    /// Draw list of wallets.
    fn wallet_list_ui(&mut self,
                      ui: &mut egui::Ui,
                      cb: &dyn PlatformCallbacks) {
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
                            self.show_opening_modal(w.clone(), None, cb);
                        }
                        self.wallet_item_ui(ui, w, cb);
                        ui.add_space(5.0);
                    }
                });
            });
    }

    /// Draw wallet list item.
    fn wallet_item_ui(&mut self,
                      ui: &mut egui::Ui,
                      wallet: &Wallet,
                      cb: &dyn PlatformCallbacks) {
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
        let (bg, stroke) = if current {
            (Colors::fill_deep(), View::item_stroke())
        } else {
            (Colors::fill(), View::hover_stroke())
        };
        ui.painter().rect(rect, rounding, bg, stroke);

        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            if !wallet.is_open() {
                // Show button to open closed wallet.
                View::item_button(ui, View::item_rounding(0, 1, true), FOLDER_OPEN, None, || {
                    self.show_opening_modal(wallet.clone(), None, cb);
                });
                if !wallet.syncing() {
                    let mut show_selection = false;
                    View::item_button(ui, Rounding::default(), GLOBE, None, || {
                        self.wallet_content = Some(WalletContent::new(wallet.clone(), None));
                        self.conn_selection_content = Some(
                            WalletConnectionModal::new(wallet.get_current_connection())
                        );
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
                        Rounding::default()
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
    fn show_opening_modal(&mut self,
                          wallet: Wallet,
                          data: Option<String>,
                          cb: &dyn PlatformCallbacks) {
        self.wallet_content = Some(WalletContent::new(wallet.clone(), None));
        self.open_wallet_content = Some(OpenWalletModal::new(wallet, data));
        Modal::new(OPEN_WALLET_MODAL)
            .position(ModalPosition::CenterTop)
            .title(t!("wallets.open"))
            .show();
        cb.show_keyboard();
    }

    /// Check if it's possible to show [`WalletsContent`] and [`WalletContent`] panels at same time.
    fn is_dual_panel_mode(ui: &mut egui::Ui) -> bool {
        let dual_panel_root = Content::is_dual_panel_mode(ui.ctx());
        let max_width = ui.available_width();
        dual_panel_root && max_width >= (Content::SIDE_PANEL_WIDTH * 2.0) + View::get_right_inset()
    }
}