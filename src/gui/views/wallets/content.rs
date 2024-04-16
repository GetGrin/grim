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

use egui::{Align, Id, Layout, Margin, RichText, Rounding, ScrollArea, Widget};

use crate::AppConfig;
use crate::gui::Colors;
use crate::gui::icons::{ARROW_LEFT, CARET_RIGHT, COMPUTER_TOWER, FOLDER_LOCK, FOLDER_OPEN, GEAR, GLOBE, GLOBE_SIMPLE, LOCK_KEY, PLUS, SIDEBAR_SIMPLE, SPINNER, SUITCASE, WARNING_CIRCLE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, Root, TitlePanel, View};
use crate::gui::views::types::{ModalContainer, ModalPosition, TextEditOptions, TitleContentType, TitleType};
use crate::gui::views::wallets::creation::WalletCreation;
use crate::gui::views::wallets::types::WalletTabType;
use crate::gui::views::wallets::WalletContent;
use crate::wallet::{ConnectionsConfig, ExternalConnection, Wallet, WalletList};

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
                _: &mut eframe::Frame,
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
    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, cb: &dyn PlatformCallbacks) {
        // Draw modal content for current ui container.
        self.current_modal_ui(ui, frame, cb);

        // Setup wallet content flags.
        let empty_list = self.wallets.is_current_list_empty();
        let create_wallet = self.creation_content.can_go_back();
        let show_wallet = self.wallets.is_selected_open();

        // Setup panels parameters.
        let dual_panel = is_dual_panel_mode(ui);
        let open_wallet_panel = dual_panel || show_wallet || create_wallet || empty_list;
        let wallet_panel_width = self.wallet_panel_width(ui, empty_list, dual_panel, show_wallet);
        let content_width = ui.available_width();

        // Show title panel.
        self.title_ui(ui, frame, dual_panel, create_wallet, show_wallet);

        // Show wallet panel content.
        egui::SidePanel::right("wallet_panel")
            .resizable(false)
            .exact_width(wallet_panel_width)
            .frame(egui::Frame {
                fill: if empty_list && !create_wallet
                    || (dual_panel && show_wallet && !self.show_wallets_at_dual_panel) {
                    Colors::FILL_DARK
                } else {
                    if create_wallet {
                        Colors::WHITE
                    } else {
                        Colors::BUTTON
                    }
                },
                ..Default::default()
            })
            .show_animated_inside(ui, open_wallet_panel, |ui| {
                // Do not draw content on zero width.
                if content_width == 0.0 {
                    return;
                }
                if create_wallet || !show_wallet {
                    // Show wallet creation content.
                    self.creation_content.ui(ui, frame, cb, |wallet| {
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
                                self.wallet_content.ui(ui, frame, wallet, cb);
                            });
                            break;
                        }
                    }
                }
            });

        // Flag to check if wallet list is hidden on the screen.
        let list_hidden = content_width == 0.0
            || (dual_panel && show_wallet && !self.show_wallets_at_dual_panel)
            || (!dual_panel && show_wallet);

        // Setup flag to show wallets bottom panel if wallet is not showing
        // at non-dual panel mode and network is no open or showing at dual panel mode.
        let show_bottom_panel =
            (!show_wallet && !dual_panel && !Root::is_network_panel_open()) ||
                (dual_panel && show_wallet);

        // Show wallets bottom panel.
        egui::TopBottomPanel::bottom("wallets_bottom_panel")
            .frame(egui::Frame {
                fill: Colors::FILL,
                stroke: View::DEFAULT_STROKE,
                inner_margin: Margin {
                    left: View::get_left_inset() + 4.0,
                    right: View::far_right_inset_margin(ui) + 4.0,
                    top: 4.0,
                    bottom: View::get_bottom_inset() + 4.0,
                },
                ..Default::default()
            })
            .show_animated_inside(ui, !create_wallet && !list_hidden && show_bottom_panel, |ui| {
                // Setup vertical padding inside buttons.
                ui.style_mut().spacing.button_padding = egui::vec2(10.0, 4.0);

                ui.vertical_centered(|ui| {
                    View::tab_button(ui, PLUS, false, || {
                        self.creation_content.show_name_pass_modal(cb);
                    });
                });
            });

        // Show non-empty list if wallet is not creating.
        if !empty_list && !create_wallet {
            // Show wallet list panel.
            egui::CentralPanel::default()
                .frame(egui::Frame {
                    stroke: View::DEFAULT_STROKE,
                    fill: Colors::FILL_DARK,
                    inner_margin: Margin {
                        left: if list_hidden {
                            0.0
                        } else {
                            View::far_left_inset_margin(ui) + 4.0
                        },
                        right: if list_hidden {
                            0.0
                        } else {
                            View::far_right_inset_margin(ui) + 4.0
                        },
                        top: 4.0,
                        bottom: View::get_bottom_inset() + 4.0,
                    },
                    ..Default::default()
                })
                .show_inside(ui, |ui| {
                    // Do not draw content when list is hidden.
                    if list_hidden {
                        return;
                    }
                    // Show list of wallets.
                    self.wallet_list_ui(ui, dual_panel, cb);
                });
        }
    }

    /// Draw [`TitlePanel`] content.
    fn title_ui(&mut self,
                ui: &mut egui::Ui,
                frame: &mut eframe::Frame,
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
        TitlePanel::ui(title_content, |ui, frame| {
            if show_wallet && !dual_panel {
                View::title_button(ui, ARROW_LEFT, || {
                    self.wallets.select(None);
                });
            } else if create_wallet {
                View::title_button(ui, ARROW_LEFT, || {
                    self.creation_content.back();
                });
            } else if show_wallet && dual_panel {
                let list_icon = if show_list {
                    SIDEBAR_SIMPLE
                } else {
                    SUITCASE
                };
                View::title_button(ui, list_icon, || {
                    self.show_wallets_at_dual_panel = !show_list;
                    AppConfig::toggle_show_wallets_at_dual_panel();
                });
            } else if !Root::is_dual_panel_mode(ui) {
                View::title_button(ui, GLOBE, || {
                    Root::toggle_network_panel();
                });
            };
        }, |ui, frame| {
            View::title_button(ui, GEAR, || {
                //TODO: show settings.
            });
        }, ui, frame);
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
                      dual_panel: bool,
                      cb: &dyn PlatformCallbacks) {
        ui.scope(|ui| {
            // Setup scroll bar color.
            ui.style_mut().visuals.widgets.inactive.bg_fill = Colors::ITEM_HOVER;
            ui.style_mut().visuals.widgets.hovered.bg_fill = Colors::STROKE;

            // Draw list of wallets.
            let scroll = ScrollArea::vertical()
                .id_source("wallet_list")
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        // Setup wallet list width.
                        let max_width = if !dual_panel {
                            Root::SIDE_PANEL_WIDTH * 1.3
                        } else {
                            ui.available_width()
                        };
                        View::max_width_ui(ui, max_width, |ui| {
                            let mut list = self.wallets.list().clone();
                            // Remove deleted wallet from the list.
                            list.retain(|w| !w.is_deleted());
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
        let is_current = wallet.is_open() && is_selected;

        // Draw round background.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(78.0);
        let rounding = View::item_rounding(0, 1, false);
        let bg_color = if is_current { Colors::ITEM_CURRENT } else { Colors::FILL };
        ui.painter().rect(rect, rounding, bg_color, View::HOVER_STROKE);

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
                ui.add_space(7.0);
                ui.vertical(|ui| {
                    ui.add_space(3.0);
                    // Setup wallet name text.
                    let name_color = if is_selected { Colors::BLACK } else { Colors::TITLE };
                    View::ellipsize_text(ui, config.name, 18.0, name_color);

                    // Setup wallet connection text.
                    let conn_text = if let Some(id) = wallet.get_current_ext_conn_id() {
                        let ext_conn_url = match ConnectionsConfig::ext_conn(id) {
                            None => ExternalConnection::DEFAULT_MAIN_URL.to_string(),
                            Some(ext_conn) => ext_conn.url
                        };
                        format!("{} {}", GLOBE_SIMPLE, ext_conn_url)
                    } else {
                        format!("{} {}", COMPUTER_TOWER, t!("network.node"))
                    };
                    View::ellipsize_text(ui, conn_text, 15.0, Colors::TEXT);
                    ui.add_space(1.0);

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
                        } else if wallet.get_data().is_none() {
                            let info_progress = wallet.info_sync_progress();
                            if info_progress != 100 {
                                if info_progress == 0 {
                                    format!("{} {}", SPINNER, t!("wallets.loading"))
                                } else {
                                    format!("{} {}: {}%",
                                            SPINNER,
                                            t!("wallets.loading"),
                                            info_progress)
                                }
                            } else {
                                let tx_progress = wallet.txs_sync_progress();
                                if tx_progress == 0 {
                                    format!("{} {}", SPINNER, t!("wallets.tx_loading"))
                                } else {
                                    format!("{} {}: {}%",
                                            SPINNER,
                                            t!("wallets.tx_loading"),
                                            tx_progress)
                                }
                            }
                        } else {
                            format!("{} {}", FOLDER_OPEN, t!("wallets.unlocked"))
                        }
                    } else {
                        format!("{} {}", FOLDER_LOCK, t!("wallets.locked"))
                    };
                    ui.label(RichText::new(status_text).size(15.0).color(Colors::GRAY));
                    ui.add_space(4.0);
                })
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
                .color(Colors::GRAY));
            ui.add_space(8.0);

            // Show password input.
            let pass_edit_opts = TextEditOptions::new(Id::from(modal.id)).password();
            View::text_edit(ui, cb, &mut self.pass_edit, pass_edit_opts);

            // Show information when password is empty.
            if self.pass_edit.is_empty() {
                ui.add_space(10.0);
                ui.label(RichText::new(t!("wallets.pass_empty"))
                    .size(17.0)
                    .color(Colors::INACTIVE_TEXT));
            } else if self.wrong_pass {
                ui.add_space(10.0);
                ui.label(RichText::new(t!("wallets.wrong_pass"))
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

                    View::button(ui, t!("continue"), Colors::WHITE, on_continue);
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