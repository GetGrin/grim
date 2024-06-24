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
use crate::gui::icons::{ARROW_LEFT, CARET_RIGHT, COMPUTER_TOWER, FOLDER_LOCK, FOLDER_OPEN, GEAR, GLOBE, GLOBE_SIMPLE, LOCK_KEY, PLUS, SIDEBAR_SIMPLE, SPINNER, SUITCASE, WARNING_CIRCLE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, Root, TitlePanel, View};
use crate::gui::views::types::{ModalContainer, ModalPosition, TextEditOptions, TitleContentType, TitleType};
use crate::gui::views::wallets::creation::WalletCreation;
use crate::gui::views::wallets::types::WalletTabType;
use crate::gui::views::wallets::WalletContent;
use crate::wallet::{Wallet, WalletList};

/// Wallets content.
pub struct WalletsContent {
    /// List of wallets.
    wallets: WalletList,

    /// Password to open wallet for [`Modal`].
    pass_edit: String,
    /// Flag to check if wrong password was entered.
    wrong_pass: bool,

    /// Selected [`Wallet`] content.
    wallet_content: WalletContent,
    /// Wallet creation content.
    creation_content: WalletCreation,

    /// Flag to show [`Wallet`] list at dual panel mode.
    show_wallets_at_dual_panel: bool,

    /// [`Modal`] identifiers allowed at this ui container.
    modal_ids: Vec<&'static str>
}

/// Identifier for wallet opening [`Modal`].
const OPEN_WALLET_MODAL: &'static str = "open_wallet_modal";

impl Default for WalletsContent {
    fn default() -> Self {
        Self {
            wallets: WalletList::default(),
            pass_edit: "".to_string(),
            wrong_pass: false,
            wallet_content: WalletContent::default(),
            creation_content: WalletCreation::default(),
            show_wallets_at_dual_panel: AppConfig::show_wallets_at_dual_panel(),
            modal_ids: vec![
                OPEN_WALLET_MODAL,
                WalletCreation::NAME_PASS_MODAL
            ]
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
            OPEN_WALLET_MODAL => self.open_wallet_modal_ui(ui, modal, cb),
            WalletCreation::NAME_PASS_MODAL => {
                self.creation_content.name_pass_modal_ui(ui, modal, cb)
            },
            _ => {}
        }
    }
}

impl WalletsContent {
    pub fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        // Draw modal content for current ui container.
        self.current_modal_ui(ui, cb);

        // Setup wallet content flags.
        let empty_list = self.wallets.is_current_list_empty();
        let create_wallet = self.creation_content.can_go_back();
        let show_wallet = self.wallets.is_selected_open();

        // Setup panels parameters.
        let dual_panel = is_dual_panel_mode(ui);
        let wallet_panel_width = self.wallet_panel_width(ui, empty_list, dual_panel, show_wallet);
        let content_width = ui.available_width();

        // Show title panel.
        View::max_width_ui(ui, ui.available_width(), |ui| {
            self.title_ui(ui, dual_panel, create_wallet, show_wallet);
        });

        // Show wallet panel content.
        egui::SidePanel::right("wallet_panel")
            .resizable(false)
            .exact_width(wallet_panel_width)
            .frame(egui::Frame {
                fill: if empty_list && !create_wallet
                    || (dual_panel && show_wallet && !self.show_wallets_at_dual_panel) {
                    Colors::fill_deep()
                } else {
                    if create_wallet {
                        Colors::white_or_black(false)
                    } else {
                        Colors::button()
                    }
                },
                ..Default::default()
            })
            .show_animated_inside(ui, self.wallet_panel_opened(), |ui| {
                // Do not draw content on zero width.
                if content_width == 0.0 {
                    return;
                }
                if create_wallet || !show_wallet {
                    // Show wallet creation content.
                    self.creation_content.ui(ui, cb, |wallet| {
                        // Add created wallet to list.
                        self.wallets.add(wallet);
                        // Reset wallet content.
                        self.wallet_content = WalletContent::default();
                    });
                } else  {
                    let selected_id = self.wallets.selected_id.clone();
                    let list = self.wallets.mut_list();
                    for wallet in list {
                        // Show content for selected wallet.
                        if selected_id == Some(wallet.get_config().id) {
                            // Setup wallet content width.
                            let mut rect = ui.available_rect_before_wrap();
                            let mut width = ui.available_width();
                            if dual_panel && self.show_wallets_at_dual_panel {
                                width = content_width - Root::SIDE_PANEL_WIDTH;
                            }
                            rect.set_width(width);
                            // Show wallet content.
                            ui.allocate_ui_at_rect(rect, |ui| {
                                self.wallet_content.ui(ui, wallet, cb);
                            });
                            break;
                        }
                    }
                }
            });

        // Flag to check if wallet list is hidden on the screen.
        let list_hidden = content_width == 0.0 || empty_list || create_wallet
            || (dual_panel && show_wallet && !self.show_wallets_at_dual_panel)
            || (!dual_panel && show_wallet);

        // Setup flag to show wallets bottom panel if wallet is not showing
        // at non-dual panel mode and network is no open or showing at dual panel mode.
        let show_bottom_panel = !list_hidden &&
            ((!show_wallet && !dual_panel && !Root::is_network_panel_open()) || dual_panel);

        // Show wallets bottom panel.
        if show_bottom_panel {
            egui::TopBottomPanel::bottom("wallets_bottom_panel")
                .frame(egui::Frame {
                    fill: Colors::fill(),
                    inner_margin: Margin {
                        left: View::get_left_inset() + 4.0,
                        right: View::far_right_inset_margin(ui) + 4.0,
                        top: 6.0,
                        bottom: View::get_bottom_inset() + 5.0,
                    },
                    ..Default::default()
                })
                .show_inside(ui, |ui| {
                    // Setup spacing between tabs.
                    ui.style_mut().spacing.item_spacing = egui::vec2(6.0, 0.0);
                    // Setup vertical padding inside buttons.
                    ui.style_mut().spacing.button_padding = egui::vec2(10.0, 4.0);

                    ui.vertical_centered(|ui| {
                        let pressed = Modal::opened() == Some(WalletCreation::NAME_PASS_MODAL);
                        View::tab_button(ui, PLUS, pressed, || {
                            self.creation_content.show_name_pass_modal(cb);
                        });
                    });
                });
        }

        // Show wallet list.
        egui::CentralPanel::default()
            .frame(if list_hidden {
                egui::Frame::default()
            } else {
                egui::Frame {
                    stroke: View::item_stroke(),
                    fill: Colors::fill_deep(),
                    inner_margin: Margin {
                        left: View::far_left_inset_margin(ui) + 4.0,
                        right: View::far_right_inset_margin(ui) + 4.0,
                        top: 3.0,
                        bottom: 4.0,
                    },
                    ..Default::default()
                }
            })
            .show_inside(ui, |ui| {
                // Update ui after 1 sec at single panel mode.
                if !dual_panel {
                    ui.ctx().request_repaint_after(Duration::from_millis(1000));
                }
                self.wallet_list_ui(ui, cb);
            });
    }

    /// Check if wallet panel is showing.
    pub fn wallet_panel_opened(&self) -> bool {
        let empty_list = self.wallets.is_current_list_empty();
        empty_list || self.creating_wallet() || self.showing_wallet()
    }

    /// Check if opened wallet is showing.
    pub fn showing_wallet(&self) -> bool {
        self.wallets.is_selected_open()
    }

    /// Check if wallet is creating.
    pub fn creating_wallet(&self) -> bool {
        self.creation_content.can_go_back()
    }

    /// Draw [`TitlePanel`] content.
    fn title_ui(&mut self,
                ui: &mut egui::Ui,
                dual_panel: bool,
                create_wallet: bool,
                show_wallet: bool) {
        let show_list = self.show_wallets_at_dual_panel;

        // Setup title.
        let title_content = if self.wallets.is_selected_open() && (!dual_panel
            || (dual_panel && !show_list)) && !create_wallet {
            let title_text = self.wallet_content.current_tab.get_type().name().to_uppercase();
            if self.wallet_content.current_tab.get_type() == WalletTabType::Settings {
                TitleType::Single(TitleContentType::Title(title_text))
            } else {
                let subtitle_text = self.wallets.selected_name();
                TitleType::Single(TitleContentType::WithSubTitle(title_text, subtitle_text, false))
            }
        } else {
            let title_text = if create_wallet {
                t!("wallets.add")
            } else {
                t!("wallets.title")
            }.to_uppercase();
            let dual_title = !create_wallet && show_wallet && dual_panel;
            if dual_title {
                let wallet_tab_type = self.wallet_content.current_tab.get_type();
                let wallet_tab_name = wallet_tab_type.name().to_uppercase();
                let title_content = if wallet_tab_type == WalletTabType::Settings {
                    TitleContentType::Title(wallet_tab_name)
                } else {
                    let subtitle_text = self.wallets.selected_name();
                    TitleContentType::WithSubTitle(wallet_tab_name, subtitle_text, false)
                };
                TitleType::Dual(TitleContentType::Title(title_text), title_content)
            } else {
                TitleType::Single(TitleContentType::Title(title_text))
            }
        };

        // Draw title panel.
        TitlePanel::ui(title_content, |ui| {
            if show_wallet && !dual_panel {
                View::title_button_big(ui, ARROW_LEFT, |_| {
                    self.wallets.select(None);
                });
            } else if create_wallet {
                View::title_button_big(ui, ARROW_LEFT, |_| {
                    self.creation_content.back();
                });
            } else if show_wallet && dual_panel {
                let list_icon = if show_list {
                    SIDEBAR_SIMPLE
                } else {
                    SUITCASE
                };
                View::title_button_big(ui, list_icon, |_| {
                    self.show_wallets_at_dual_panel = !show_list;
                    AppConfig::toggle_show_wallets_at_dual_panel();
                });
            } else if !Root::is_dual_panel_mode(ui) {
                View::title_button_big(ui, GLOBE, |_| {
                    Root::toggle_network_panel();
                });
            };
        }, |ui| {
            View::title_button_big(ui, GEAR, |_| {
                // Show settings modal.
                Modal::new(Root::SETTINGS_MODAL)
                    .position(ModalPosition::CenterTop)
                    .title(t!("settings"))
                    .show();
            });
        }, ui);
    }

    /// Calculate [`WalletContent`] panel width.
    fn wallet_panel_width(
        &self,
        ui:&mut egui::Ui,
        list_empty: bool,
        dual_panel: bool,
        show_wallet: bool
    ) -> f32 {
        let create_wallet = self.creation_content.can_go_back();
        let available_width = if list_empty || create_wallet || (show_wallet && !dual_panel)
            || (show_wallet && !self.show_wallets_at_dual_panel) {
            ui.available_width()
        } else {
            ui.available_width() - Root::SIDE_PANEL_WIDTH
        };
        if dual_panel && show_wallet && self.show_wallets_at_dual_panel {
            let min_width = Root::SIDE_PANEL_WIDTH + View::get_right_inset();
            f32::max(min_width, available_width)
        } else {
            available_width
        }
    }

    /// Draw list of wallets.
    fn wallet_list_ui(&mut self,
                      ui: &mut egui::Ui,
                      cb: &dyn PlatformCallbacks) {
        ScrollArea::vertical()
            .id_source("wallet_list")
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    View::max_width_ui(ui, Root::SIDE_PANEL_WIDTH * 1.3, |ui| {
                        // Show application logo and name.
                        View::app_logo_name_version(ui);
                        ui.add_space(15.0);

                        let mut list = self.wallets.list().clone();
                        // Remove deleted wallet from the list.
                        list.retain(|w| {
                            let deleted = w.is_deleted();
                            if deleted {
                                self.wallets.select(None);
                                self.wallets.remove(w.get_config().id);
                                ui.ctx().request_repaint();
                            }
                            !deleted
                        });
                        for wallet in &list {
                            // Check if wallet reopen is needed.
                            if !wallet.is_open() && wallet.reopen_needed() {
                                wallet.set_reopen(false);
                                self.wallets.select(Some(wallet.get_config().id));
                                self.show_open_wallet_modal(cb);
                            }
                            // Draw wallet list item.
                            self.wallet_item_ui(ui, wallet, cb);
                            ui.add_space(5.0);
                        }
                    });
                });
            });
    }

    /// Draw wallet list item.
    fn wallet_item_ui(&mut self,
                      ui: &mut egui::Ui,
                      wallet: &Wallet,
                      cb: &dyn PlatformCallbacks) {
        let config = wallet.get_config();
        let id = config.id;
        let is_selected = self.wallets.selected_id == Some(id);
        let current = is_selected && wallet.is_open();
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
            // Setup padding for item buttons.
            ui.style_mut().spacing.button_padding = egui::vec2(14.0, 0.0);

            if !wallet.is_open() {
                // Show button to open closed wallet.
                View::item_button(ui, View::item_rounding(0, 1, true), FOLDER_OPEN, None, || {
                    self.wallets.select(Some(id));
                    self.show_open_wallet_modal(cb);
                });
            } else {
                if !is_selected {
                    // Show button to select opened wallet.
                    View::item_button(ui, View::item_rounding(0, 1, true), CARET_RIGHT, None, || {
                        // Reset wallet content.
                        self.wallet_content = WalletContent::default();
                        // Select wallet.
                        self.wallets.select(Some(id));
                    });
                }

                // Show button to close opened wallet.
                if !wallet.is_closing()  {
                    View::item_button(ui, if !is_selected {
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
                    // Setup wallet name text.
                    let name_color = if is_selected {
                        Colors::white_or_black(true)
                    } else {
                        Colors::title(false)
                    };
                    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                            ui.add_space(1.0);
                            View::ellipsize_text(ui, config.name, 18.0, name_color);
                    });

                    // Setup wallet status text.
                    let status_text = if wallet.is_open() {
                        if wallet.sync_error() {
                            format!("{} {}", WARNING_CIRCLE, t!("error"))
                        } else if wallet.is_closing() {
                            format!("{} {}", SPINNER, t!("wallets.closing"))
                        } else if wallet.is_repairing() {
                            let repair_progress = wallet.repairing_progress();
                            if repair_progress == 0 {
                                format!("{} {}", SPINNER, t!("wallets.checking"))
                            } else {
                                format!("{} {}: {}%",
                                        SPINNER,
                                        t!("wallets.checking"),
                                        repair_progress)
                            }
                        } else if wallet.syncing() {
                            let info_progress = wallet.info_sync_progress();
                            if info_progress == 100 || info_progress == 0 {
                                format!("{} {}", SPINNER, t!("wallets.loading"))
                            } else {
                                format!("{} {}: {}%",
                                        SPINNER,
                                        t!("wallets.loading"),
                                        info_progress)
                            }
                        } else {
                            format!("{} {}", FOLDER_OPEN, t!("wallets.unlocked"))
                        }
                    } else {
                        format!("{} {}", FOLDER_LOCK, t!("wallets.locked"))
                    };
                    View::ellipsize_text(ui, status_text, 15.0, Colors::text(false));
                    ui.add_space(1.0);

                    // Setup wallet connection text.
                    let conn_text = if let Some(conn) = wallet.get_current_ext_conn() {
                        format!("{} {}", GLOBE_SIMPLE, conn.url)
                    } else {
                        format!("{} {}", COMPUTER_TOWER, t!("network.node"))
                    };
                    ui.label(RichText::new(conn_text).size(15.0).color(Colors::gray()));
                    ui.add_space(3.0);
                });
            });
        });
    }

    /// Show [`Modal`] to open selected wallet.
    pub fn show_open_wallet_modal(&mut self, cb: &dyn PlatformCallbacks) {
        // Reset modal values.
        self.pass_edit = String::from("");
        self.wrong_pass = false;
        // Show modal.
        Modal::new(OPEN_WALLET_MODAL)
            .position(ModalPosition::CenterTop)
            .title(t!("wallets.open"))
            .show();
        cb.show_keyboard();
    }

    /// Draw wallet opening [`Modal`] content.
    fn open_wallet_modal_ui(&mut self,
                            ui: &mut egui::Ui,
                            modal: &Modal,
                            cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("wallets.pass"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Show password input.
            let mut pass_edit_opts = TextEditOptions::new(Id::from(modal.id)).password();
            View::text_edit(ui, cb, &mut self.pass_edit, &mut pass_edit_opts);

            // Show information when password is empty.
            if self.pass_edit.is_empty() {
                self.wrong_pass = false;
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
                        cb.hide_keyboard();
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    // Callback for button to continue.
                    let mut on_continue = || {
                        if self.pass_edit.is_empty() {
                            return;
                        }
                        match self.wallets.open_selected(self.pass_edit.clone()) {
                            Ok(_) => {
                                // Clear values.
                                self.pass_edit = "".to_string();
                                self.wrong_pass = false;
                                // Close modal.
                                cb.hide_keyboard();
                                modal.close();
                                // Reset wallet content.
                                self.wallet_content = WalletContent::default();
                            }
                            Err(_) => self.wrong_pass = true
                        }
                    };

                    // Continue on Enter key press.
                    View::on_enter_key(ui, || {
                        (on_continue)();
                    });

                    View::button(ui, t!("continue"), Colors::white_or_black(false), on_continue);
                });
            });
            ui.add_space(6.0);
        });
    }

    /// Handle Back key event.
    /// Return `false` when event was handled.
    pub fn on_back(&mut self) -> bool {
        let can_go_back = self.creation_content.can_go_back();
        if can_go_back {
            self.creation_content.back();
            return false
        } else {
            if self.wallets.is_selected_open() {
                self.wallets.select(None);
                return false
            }
        }
        true
    }
}

/// Check if it's possible to show [`WalletsContent`] and [`WalletContent`] panels at same time.
fn is_dual_panel_mode(ui: &mut egui::Ui) -> bool {
    let dual_panel_root = Root::is_dual_panel_mode(ui);
    let max_width = ui.available_width();
    dual_panel_root && max_width >= (Root::SIDE_PANEL_WIDTH * 2.0) + View::get_right_inset()
}