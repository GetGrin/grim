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

use std::sync::{Arc, RwLock};
use std::thread;
use egui::{Align, Id, Layout, Margin, RichText, Rounding, ScrollArea};
use egui::scroll_area::ScrollBarVisibility;
use grin_core::core::{amount_from_hr_string, amount_to_hr_string};
use grin_wallet_libwallet::SlatepackAddress;

use crate::gui::Colors;
use crate::gui::icons::{CHECK_CIRCLE, COMPUTER_TOWER, COPY, DOTS_THREE_CIRCLE, EXPORT, GEAR_SIX, POWER, QR_CODE, WARNING_CIRCLE, X_CIRCLE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, Root, View};
use crate::gui::views::types::{ModalPosition, TextEditOptions};
use crate::gui::views::wallets::wallet::types::{WalletTab, WalletTabType};
use crate::gui::views::wallets::wallet::WalletContent;
use crate::tor::Tor;
use crate::wallet::Wallet;

/// Wallet transport tab content.
pub struct WalletTransport {
    /// Flag to check if transaction is sending over Tor to show progress at [`Modal`].
    tor_sending: Arc<RwLock<bool>>,
    /// Flag to check if error occurred during sending of transaction over Tor at [`Modal`].
    tor_send_error: Arc<RwLock<bool>>,
    /// Flag to check if transaction sent successfully over Tor [`Modal`].
    tor_success: Arc<RwLock<bool>>,
    /// Entered amount value for [`Modal`].
    amount_edit: String,
    /// Entered address value for [`Modal`].
    address_edit: String,
    /// Flag to check if entered address is incorrect at [`Modal`].
    address_error: bool,
    /// Flag to check if [`Modal`] was just opened to focus on first field.
    modal_just_opened: bool,
}

impl Default for WalletTransport {
    fn default() -> Self {
        Self {
            tor_sending: Arc::new(RwLock::new(false)),
            tor_send_error: Arc::new(RwLock::new(false)),
            tor_success: Arc::new(RwLock::new(false)),
            amount_edit: "".to_string(),
            address_edit: "".to_string(),
            address_error: false,
            modal_just_opened: false,
        }
    }
}

impl WalletTab for WalletTransport {
    fn get_type(&self) -> WalletTabType {
        WalletTabType::Transport
    }

    fn ui(&mut self,
          ui: &mut egui::Ui,
          _: &mut eframe::Frame,
          wallet: &mut Wallet,
          cb: &dyn PlatformCallbacks) {
        if WalletContent::sync_ui(ui, wallet) {
            return;
        }

        // Show modal content for this ui container.
        self.modal_content_ui(ui, wallet, cb);

        // Show transport content panel.
        egui::CentralPanel::default()
            .frame(egui::Frame {
                stroke: View::ITEM_STROKE,
                fill: Colors::WHITE,
                inner_margin: Margin {
                    left: View::far_left_inset_margin(ui) + 4.0,
                    right: View::get_right_inset() + 4.0,
                    top: 3.0,
                    bottom: 4.0,
                },
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                ScrollArea::vertical()
                    .scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
                    .id_source(Id::from("wallet_transport").with(wallet.get_config().id))
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            View::max_width_ui(ui, Root::SIDE_PANEL_WIDTH * 1.3, |ui| {
                                self.ui(ui, wallet, cb);
                            });
                        });
                    });
            });
    }
}

/// Identifier for [`Modal`] to send amount over Tor.
const SEND_TOR_MODAL: &'static str = "send_tor_modal";

/// Identifier for [`Modal`] to setup Tor service.
const TOR_SETTINGS_MODAL: &'static str = "tor_settings_modal";

impl WalletTransport {
    /// Draw wallet transport content.
    pub fn ui(&mut self, ui: &mut egui::Ui, wallet: &mut Wallet, cb: &dyn PlatformCallbacks) {
        ui.add_space(3.0);
        ui.label(RichText::new(t!("transport.desc"))
            .size(16.0)
            .color(Colors::INACTIVE_TEXT));
        ui.add_space(7.0);

        // Draw Tor content.
        self.tor_ui(ui, wallet, cb);
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
                    SEND_TOR_MODAL => {
                        Modal::ui(ui.ctx(), |ui, modal| {
                            self.send_tor_modal_ui(ui, wallet, modal, cb);
                        });
                    }
                    TOR_SETTINGS_MODAL => {
                        Modal::ui(ui.ctx(), |ui, modal| {
                            self.tor_settings_modal_ui(ui, wallet, modal);
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    /// Draw Tor transport content.
    fn tor_ui(&mut self, ui: &mut egui::Ui, wallet: &mut Wallet, cb: &dyn PlatformCallbacks) {
        // Draw header content.
        self.tor_header_ui(ui, wallet);

        // Draw receive info content.
        if wallet.slatepack_address().is_some() {
            self.tor_receive_ui(ui, wallet, cb);
        }

        // Draw send content.
        self.tor_send_ui(ui, cb);
    }

    /// Draw Tor transport header content.
    fn tor_header_ui(&self, ui: &mut egui::Ui, wallet: &mut Wallet) {
        // Setup layout size.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(78.0);

        // Draw round background.
        let bg_rect = rect.clone();
        let item_rounding = View::item_rounding(0, 2, false);
        ui.painter().rect(bg_rect, item_rounding, Colors::BUTTON, View::ITEM_STROKE);

        ui.vertical(|ui| {
            ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
                // Draw button to setup Tor transport.
                let button_rounding = View::item_rounding(0, 2, true);
                View::item_button(ui, button_rounding, GEAR_SIX, None, || {
                    // Show modal.
                    Modal::new(TOR_SETTINGS_MODAL)
                        .position(ModalPosition::CenterTop)
                        .title(t!("transport.tor_settings"))
                        .show();
                });

                // Draw button to enable/disable Tor listener for current wallet.
                let service_id = &wallet.identifier();
                if !Tor::is_service_running(service_id) &&
                    wallet.foreign_api_port().is_some() {
                    View::item_button(ui, Rounding::default(), POWER, Some(Colors::GREEN), || {
                        if let Ok(key) = wallet.secret_key() {
                            Tor::start_service(wallet.foreign_api_port().unwrap(), key, service_id);
                        }
                    });
                } else if !Tor::is_service_starting(service_id) {
                    View::item_button(ui, Rounding::default(), POWER, Some(Colors::RED), || {
                        Tor::stop_service(service_id);
                    });
                }

                let layout_size = ui.available_size();
                ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                    ui.add_space(6.0);
                    ui.vertical(|ui| {
                        ui.add_space(3.0);
                        ui.label(RichText::new(t!("transport.tor_network"))
                            .size(18.0)
                            .color(Colors::TITLE));

                        // Setup wallet API address text.
                        let port = wallet.foreign_api_port().unwrap();
                        let address_text = format!("{} http://127.0.0.1:{}",
                                                   COMPUTER_TOWER,
                                                   port);
                        ui.label(RichText::new(address_text).size(15.0).color(Colors::TEXT));
                        ui.add_space(1.0);

                        // Setup Tor status text.
                        let is_running = Tor::is_service_running(service_id);
                        let is_starting = Tor::is_service_starting(service_id);
                        let has_error = Tor::is_service_failed(service_id);
                        let (icon, text) = if is_starting {
                            (DOTS_THREE_CIRCLE, t!("transport.connecting"))
                        } else if has_error {
                            (WARNING_CIRCLE, t!("transport.conn_error"))
                        } else if is_running {
                            (CHECK_CIRCLE, t!("transport.connected"))
                        } else {
                            (X_CIRCLE, t!("transport.disconnected"))
                        };
                        let status_text = format!("{} {}", icon, text);
                        ui.label(RichText::new(status_text).size(15.0).color(Colors::GRAY));
                    });
                });
            });
        });
    }

    /// Draw tor transport settings [`Modal`] content.
    fn tor_settings_modal_ui(&self, ui: &mut egui::Ui, wallet: &mut Wallet, modal: &Modal) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("transport.tor_autorun_desc"))
                .size(17.0)
                .color(Colors::INACTIVE_TEXT));

            // Show Tor service autorun checkbox.
            let autorun = wallet.auto_start_tor_listener();
            View::checkbox(ui, autorun, t!("network.autorun"), || {
                wallet.update_auto_start_tor_listener(!autorun);
            });
        });
        ui.add_space(6.0);
        ui.vertical_centered_justified(|ui| {
            View::button(ui, t!("close"), Colors::WHITE, || {
                modal.close();
            });
            ui.add_space(6.0);
        });
    }

    /// Draw Tor send content.
    fn tor_receive_ui(&self, ui: &mut egui::Ui, wallet: &mut Wallet, cb: &dyn PlatformCallbacks) {
        let slatepack_addr = wallet.slatepack_address().unwrap();
        let service_id = &wallet.identifier();

        // Setup layout size.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(52.0);

        // Draw round background.
        let bg_rect = rect.clone();
        let item_rounding = View::item_rounding(1, 3, false);
        ui.painter().rect(bg_rect, item_rounding, Colors::BUTTON, View::ITEM_STROKE);

        ui.vertical(|ui| {
            ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
                // Draw button to setup Tor transport.
                let button_rounding = View::item_rounding(1, 3, true);
                View::item_button(ui, button_rounding, QR_CODE, None, || {
                    //TODO: qr for address
                });

                // Show button to enable/disable Tor listener for current wallet.
                View::item_button(ui, Rounding::default(), COPY, None, || {
                    cb.copy_string_to_buffer(slatepack_addr.clone());
                });

                let layout_size = ui.available_size();
                ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                    ui.add_space(6.0);
                    ui.vertical(|ui| {
                        ui.add_space(3.0);

                        // Show wallet Slatepack address.
                        let address_color = if Tor::is_service_starting(service_id) {
                            Colors::INACTIVE_TEXT
                        } else if Tor::is_service_running(service_id) {
                            Colors::GREEN
                        } else {
                            Colors::RED
                        };
                        View::ellipsize_text(ui, slatepack_addr, 15.0, address_color);

                        let address_label = format!("{} {}",
                                                    COMPUTER_TOWER,
                                                    t!("network_mining.address"));
                        ui.label(RichText::new(address_label).size(15.0).color(Colors::GRAY));
                    });
                });
            });
        });
    }

    /// Draw Tor receive content.
    fn tor_send_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        // Setup layout size.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(55.0);

        // Draw round background.
        let bg_rect = rect.clone();
        let item_rounding = View::item_rounding(1, 2, false);
        ui.painter().rect(bg_rect, item_rounding, Colors::FILL, View::ITEM_STROKE);

        ui.vertical(|ui| {
            ui.allocate_ui_with_layout(rect.size(), Layout::top_down(Align::Center), |ui| {
                ui.add_space(7.0);
                // Draw button to open sending modal.
                let send_text = format!("{} {}", EXPORT, t!("wallets.send"));
                View::button(ui, send_text, Colors::WHITE, || {
                    self.show_send_tor_modal(cb);
                });
            });
        });
    }

    /// Show [`Modal`] to send over Tor.
    fn show_send_tor_modal(&mut self, cb: &dyn PlatformCallbacks) {
        // Setup modal values.
        let mut w_send_err = self.tor_send_error.write().unwrap();
        *w_send_err = false;
        let mut w_sending = self.tor_sending.write().unwrap();
        *w_sending = false;
        let mut w_success = self.tor_success.write().unwrap();
        *w_success = false;
        self.modal_just_opened = true;
        self.amount_edit = "".to_string();
        self.address_edit = "".to_string();
        self.address_error = false;
        // Show modal.
        Modal::new(SEND_TOR_MODAL)
            .position(ModalPosition::CenterTop)
            .title(t!("wallets.send"))
            .show();
        cb.show_keyboard();
    }

    /// Check if error occurred during sending over Tor at [`Modal`].
    fn has_tor_send_error(&self) -> bool {
        let r_send_err = self.tor_send_error.read().unwrap();
        r_send_err.clone()
    }

    /// Check if transaction is sending over Tor to show progress at [`Modal`].
    fn tor_sending(&self) -> bool {
        let r_sending = self.tor_sending.read().unwrap();
        r_sending.clone()
    }

    /// Check if transaction sent over Tor with success at [`Modal`].
    fn tor_success(&self) -> bool {
        let r_success = self.tor_success.read().unwrap();
        r_success.clone()
    }

    /// Draw amount input [`Modal`] content to send over Tor.
    /// Draw amount input [`Modal`] content to send over Tor.
    fn send_tor_modal_ui(&mut self,
                       ui: &mut egui::Ui,
                       wallet: &mut Wallet,
                       modal: &Modal,
                       cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);
        let has_send_err = self.has_tor_send_error();
        let sending = self.tor_sending();
        if !has_send_err && !sending {
            ui.vertical_centered(|ui| {
                let data = wallet.get_data().unwrap();
                let amount = amount_to_hr_string(data.info.amount_currently_spendable, true);
                let enter_text = t!("wallets.enter_amount_send","amount" => amount);
                ui.label(RichText::new(enter_text)
                    .size(17.0)
                    .color(Colors::GRAY));
            });
            ui.add_space(8.0);

            // Draw amount text edit.
            let amount_edit_id = Id::from(modal.id).with("amount").with(wallet.get_config().id);
            let mut amount_edit_opts = TextEditOptions::new(amount_edit_id).h_center().no_focus();
            let amount_edit_before = self.amount_edit.clone();
            if self.modal_just_opened {
                self.modal_just_opened = false;
                amount_edit_opts.focus = true;
            }
            View::text_edit(ui, cb, &mut self.amount_edit, amount_edit_opts);
            ui.add_space(8.0);

            // Check value if input was changed.
            if amount_edit_before != self.amount_edit {
                if !self.amount_edit.is_empty() {
                    match amount_from_hr_string(self.amount_edit.as_str()) {
                        Ok(a) => {
                            if !self.amount_edit.contains(".") {
                                // To avoid input of several "0".
                                if a == 0 {
                                    self.amount_edit = "0".to_string();
                                    return;
                                }
                            } else {
                                // Check input after ".".
                                let parts = self.amount_edit.split(".").collect::<Vec<&str>>();
                                if parts.len() == 2 && parts[1].len() > 9 {
                                    self.amount_edit = amount_edit_before;
                                    return;
                                }
                            }

                            // Do not input amount more than balance in sending.
                            let b = wallet.get_data().unwrap().info.amount_currently_spendable;
                            if b < a {
                                self.amount_edit = amount_edit_before;
                            }
                        }
                        Err(_) => {
                            self.amount_edit = amount_edit_before;
                        }
                    }
                }
            }

            // Show address error or input description.
            ui.vertical_centered(|ui| {
                if self.address_error {
                    ui.label(RichText::new(t!("transport.incorrect_addr_err"))
                        .size(17.0)
                        .color(Colors::RED));
                } else {
                    ui.label(RichText::new(t!("transport.receiver_address"))
                        .size(17.0)
                        .color(Colors::GRAY));
                }
            });
            ui.add_space(8.0);

            // Draw address text edit.
            let addr_edit_before = self.address_edit.clone();
            let address_edit_id = Id::from(modal.id).with("address").with(wallet.get_config().id);
            let address_edit_opts = TextEditOptions::new(address_edit_id)
                .paste()
                .scan_qr()
                .no_focus();
            View::text_edit(ui, cb, &mut self.address_edit, address_edit_opts);
            ui.add_space(12.0);

            // Check value if input was changed.
            if addr_edit_before != self.address_edit {
                self.address_error = false;
            }

            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::WHITE, || {
                        self.amount_edit = "".to_string();
                        self.address_edit = "".to_string();
                        cb.hide_keyboard();
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("continue"), Colors::WHITE, || {
                        if self.amount_edit.is_empty() {
                            return;
                        }

                        // Check entered address.
                        let addr_str = self.address_edit.as_str();
                        if let Ok(addr) = SlatepackAddress::try_from(addr_str) {
                            // Parse amount and send over Tor.
                            if let Ok(a) = amount_from_hr_string(self.amount_edit.as_str()) {
                                cb.hide_keyboard();
                                let mut w_sending = self.tor_sending.write().unwrap();
                                *w_sending = true;
                                {
                                    let send_error = self.tor_send_error.clone();
                                    let send_success = self.tor_success.clone();
                                    let mut wallet = wallet.clone();
                                    thread::spawn(move || {
                                        tokio::runtime::Builder::new_multi_thread()
                                            .enable_all()
                                            .build()
                                            .unwrap()
                                            .block_on(async {
                                                if wallet.send_tor(a, &addr).await.is_some() {
                                                    let mut w_send_success
                                                        = send_success.write().unwrap();
                                                    *w_send_success = true;
                                                } else {
                                                    let mut w_send_error
                                                        = send_error.write().unwrap();
                                                    *w_send_error = true;
                                                }
                                            });
                                    });
                                }
                            }
                        } else {
                            self.address_error = true;
                        }
                    });
                });
            });
            ui.add_space(6.0);
        } else if has_send_err {
            ui.add_space(6.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(t!("transport.tor_send_error"))
                    .size(17.0)
                    .color(Colors::RED));
            });
            ui.add_space(12.0);

            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(6.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::WHITE, || {
                        self.amount_edit = "".to_string();
                        self.address_edit = "".to_string();
                        cb.hide_keyboard();
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("repeat"), Colors::WHITE, || {
                        // Parse amount and send over Tor.
                        if let Ok(a) = amount_from_hr_string(self.amount_edit.as_str()) {
                            let mut w_send_error = self.tor_send_error.write().unwrap();
                            *w_send_error = false;
                            let mut w_sending = self.tor_sending.write().unwrap();
                            *w_sending = true;
                            {
                                let addr_text = self.address_edit.clone();
                                let send_error = self.tor_send_error.clone();
                                let send_success = self.tor_success.clone();
                                let mut wallet = wallet.clone();
                                thread::spawn(move || {
                                    tokio::runtime::Builder::new_multi_thread()
                                        .enable_all()
                                        .build()
                                        .unwrap()
                                        .block_on(async {
                                            let addr_str = addr_text.as_str();
                                            let addr = &SlatepackAddress::try_from(addr_str)
                                                .unwrap();
                                            if wallet.send_tor(a, &addr).await.is_some() {
                                                let mut w_send_success
                                                    = send_success.write().unwrap();
                                                *w_send_success = true;
                                            } else {
                                                let mut w_send_error
                                                    = send_error.write().unwrap();
                                                *w_send_error = true;
                                            }
                                        });
                                });
                            }
                        }
                    });
                });
            });
            ui.add_space(6.0);
        } else {
            ui.add_space(16.0);
            ui.vertical_centered(|ui| {
                View::small_loading_spinner(ui);
                ui.add_space(12.0);
                ui.label(RichText::new(t!("transport.tor_sending"))
                    .size(17.0)
                    .color(Colors::GRAY));
            });
            ui.add_space(10.0);

            // Close modal on success sending.
            if self.tor_success() {
                modal.close();
            }
        }
    }
}