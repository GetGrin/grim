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

use egui::{Align, Id, Layout, Margin, RichText, Rounding, ScrollArea};
use egui::scroll_area::ScrollBarVisibility;

use crate::gui::Colors;
use crate::gui::icons::{CHECK_CIRCLE, COPY, DOTS_THREE_CIRCLE, EXPORT, GEAR_SIX, GLOBE_SIMPLE, POWER, QR_CODE, SHIELD_CHECKERED, SHIELD_SLASH, WARNING_CIRCLE, X_CIRCLE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Modal, QrCodeContent, Content, View};
use crate::gui::views::types::ModalPosition;
use crate::gui::views::wallets::wallet::transport::send::TransportSendModal;
use crate::gui::views::wallets::wallet::transport::settings::TransportSettingsModal;
use crate::gui::views::wallets::wallet::types::{WalletTab, WalletTabType};
use crate::gui::views::wallets::wallet::WalletContent;
use crate::tor::{Tor, TorConfig};
use crate::wallet::types::WalletData;
use crate::wallet::Wallet;

/// Wallet transport tab content.
pub struct WalletTransport {
    /// Sending [`Modal`] content.
    send_modal_content: Option<TransportSendModal>,

    /// QR code address image [`Modal`] content.
    qr_address_content: Option<QrCodeContent>,

    /// Tor settings [`Modal`] content.
    settings_modal_content: Option<TransportSettingsModal>,
}

impl WalletTab for WalletTransport {
    fn get_type(&self) -> WalletTabType {
        WalletTabType::Transport
    }

    fn ui(&mut self,
          ui: &mut egui::Ui,
          wallet: &Wallet,
          cb: &dyn PlatformCallbacks) {
        if WalletContent::sync_ui(ui, wallet) {
            return;
        }

        // Show modal content for this ui container.
        self.modal_content_ui(ui, wallet, cb);

        // Show transport content panel.
        egui::CentralPanel::default()
            .frame(egui::Frame {
                stroke: View::item_stroke(),
                fill: Colors::white_or_black(false),
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
                    .id_source(Id::from("wallet_transport").with(wallet.get_config().id))
                    .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            View::max_width_ui(ui, Content::SIDE_PANEL_WIDTH * 1.3, |ui| {
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

/// Identifier for [`Modal`] to show QR code address image.
const QR_ADDRESS_MODAL: &'static str = "qr_address_modal";

impl Default for WalletTransport {
    fn default() -> Self {
        Self {
            send_modal_content: None,
            qr_address_content: None,
            settings_modal_content: None,
        }
    }
}

impl WalletTransport {
    /// Draw wallet transport content.
    pub fn ui(&mut self, ui: &mut egui::Ui, wallet: &Wallet, cb: &dyn PlatformCallbacks) {
        ui.add_space(3.0);
        ui.label(RichText::new(t!("transport.desc"))
            .size(16.0)
            .color(Colors::inactive_text()));
        ui.add_space(7.0);

        // Draw Tor transport content.
        self.tor_ui(ui, wallet, cb);
    }

    /// Draw [`Modal`] content for this ui container.
    fn modal_content_ui(&mut self,
                        ui: &mut egui::Ui,
                        wallet: &Wallet,
                        cb: &dyn PlatformCallbacks) {
        match Modal::opened() {
            None => {}
            Some(id) => {
                match id {
                    SEND_TOR_MODAL => {
                        if let Some(content) = self.send_modal_content.as_mut() {
                            Modal::ui(ui.ctx(), |ui, modal| {
                                content.ui(ui, wallet, modal, cb);
                            });
                        }
                    }
                    TOR_SETTINGS_MODAL => {
                        if let Some(content) = self.settings_modal_content.as_mut() {
                            Modal::ui(ui.ctx(), |ui, modal| {
                                content.ui(ui, wallet, modal, cb);
                            });
                        }
                    }
                    QR_ADDRESS_MODAL => {
                        Modal::ui(ui.ctx(), |ui, modal| {
                            self.qr_address_modal_ui(ui, modal, cb);
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    /// Draw Tor transport content.
    fn tor_ui(&mut self, ui: &mut egui::Ui, wallet: &Wallet, cb: &dyn PlatformCallbacks) {
        let data = wallet.get_data().unwrap();

        // Draw header content.
        self.tor_header_ui(ui, wallet);

        // Draw receive info content.
        if wallet.slatepack_address().is_some() {
            self.tor_receive_ui(ui, wallet, &data, cb);
        }

        // Draw send content.
        let service_id = &wallet.identifier();
        if data.info.amount_currently_spendable > 0 && wallet.foreign_api_port().is_some() &&
            !Tor::is_service_starting(service_id) {
            self.tor_send_ui(ui, cb);
        }
    }

    /// Draw Tor transport header content.
    fn tor_header_ui(&mut self, ui: &mut egui::Ui, wallet: &Wallet) {
        // Setup layout size.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(78.0);

        // Draw round background.
        let bg_rect = rect.clone();
        let item_rounding = View::item_rounding(0, 2, false);
        ui.painter().rect(bg_rect, item_rounding, Colors::button(), View::item_stroke());

        ui.vertical(|ui| {
            ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
                // Draw button to setup Tor transport.
                let button_rounding = View::item_rounding(0, 2, true);
                View::item_button(ui, button_rounding, GEAR_SIX, None, || {
                    self.settings_modal_content = Some(TransportSettingsModal::default());
                    // Show Tor settings modal.
                    Modal::new(TOR_SETTINGS_MODAL)
                        .position(ModalPosition::CenterTop)
                        .title(t!("transport.tor_settings"))
                        .closeable(false)
                        .show();
                });

                // Draw button to enable/disable Tor listener for current wallet.
                let service_id = &wallet.identifier();
                if  !Tor::is_service_starting(service_id) && wallet.foreign_api_port().is_some() {
                    if !Tor::is_service_running(service_id) {
                        View::item_button(ui, Rounding::default(), POWER, Some(Colors::green()), || {
                            if let Ok(key) = wallet.secret_key() {
                                let api_port = wallet.foreign_api_port().unwrap();
                                Tor::start_service(api_port, key, service_id);
                            }
                        });
                    } else {
                        View::item_button(ui, Rounding::default(), POWER, Some(Colors::red()), || {
                            Tor::stop_service(service_id);
                        });
                    }
                }

                let layout_size = ui.available_size();
                ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                    ui.add_space(6.0);
                    ui.vertical(|ui| {
                        ui.add_space(3.0);
                        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                            ui.add_space(1.0);
                            ui.label(RichText::new(t!("transport.tor_network"))
                                .size(18.0)
                                .color(Colors::title(false)));
                        });

                        // Setup Tor status text.
                        let is_running = Tor::is_service_running(service_id);
                        let is_starting = Tor::is_service_starting(service_id);
                        let has_error = Tor::is_service_failed(service_id);
                        let (icon, text) = if wallet.foreign_api_port().is_none() {
                            (DOTS_THREE_CIRCLE, t!("wallets.loading"))
                        } else if is_starting {
                            (DOTS_THREE_CIRCLE, t!("transport.connecting"))
                        } else if has_error {
                            (WARNING_CIRCLE, t!("transport.conn_error"))
                        } else if is_running {
                            (CHECK_CIRCLE, t!("transport.connected"))
                        } else {
                            (X_CIRCLE, t!("transport.disconnected"))
                        };
                        let status_text = format!("{} {}", icon, text);
                        ui.label(RichText::new(status_text).size(15.0).color(Colors::text(false)));
                        ui.add_space(1.0);

                        // Setup bridges status text.
                        let bridge = TorConfig::get_bridge();
                        let bridges_text = match &bridge {
                            None => {
                                format!("{} {}", SHIELD_SLASH, t!("transport.bridges_disabled"))
                            }
                            Some(b) => {
                                let name = b.protocol_name().to_uppercase();
                                format!("{} {}",
                                        SHIELD_CHECKERED,
                                        t!("transport.bridge_name", "b" = name))
                            }
                        };

                        ui.label(RichText::new(bridges_text).size(15.0).color(Colors::gray()));
                    });
                });
            });
        });
    }

    /// Draw Tor receive content.
    fn tor_receive_ui(&mut self,
                      ui: &mut egui::Ui,
                      wallet: &Wallet,
                      data: &WalletData,
                      cb: &dyn PlatformCallbacks) {
        let addr = wallet.slatepack_address().unwrap();
        let service_id = &wallet.identifier();
        let can_send = data.info.amount_currently_spendable > 0;

        // Setup layout size.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(52.0);

        // Draw round background.
        let bg_rect = rect.clone();
        let item_rounding = if can_send {
            View::item_rounding(1, 3, false)
        } else {
            View::item_rounding(1, 2, false)
        };
        ui.painter().rect(bg_rect, item_rounding, Colors::button(), View::item_stroke());

        ui.vertical(|ui| {
            ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
                // Draw button to setup Tor transport.
                let button_rounding = if can_send {
                    View::item_rounding(1, 3, true)
                } else {
                    View::item_rounding(1, 2, true)
                };
                View::item_button(ui, button_rounding, QR_CODE, None, || {
                    // Show QR code image address modal.
                    self.qr_address_content = Some(QrCodeContent::new(addr.clone(), false));
                    Modal::new(QR_ADDRESS_MODAL)
                        .position(ModalPosition::CenterTop)
                        .title(t!("network_mining.address"))
                        .show();
                });

                // Show button to enable/disable Tor listener for current wallet.
                View::item_button(ui, Rounding::default(), COPY, None, || {
                    cb.copy_string_to_buffer(addr.clone());
                });

                let layout_size = ui.available_size();
                ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                    ui.add_space(6.0);
                    ui.vertical(|ui| {
                        ui.add_space(3.0);

                        // Show wallet Slatepack address.
                        let address_color = if Tor::is_service_starting(service_id) ||
                            wallet.foreign_api_port().is_none() {
                            Colors::inactive_text()
                        } else if Tor::is_service_running(service_id) {
                            Colors::green()
                        } else {
                            Colors::red()
                        };
                        View::ellipsize_text(ui, addr, 15.0, address_color);

                        let address_label = format!("{} {}",
                                                    GLOBE_SIMPLE,
                                                    t!("network_mining.address"));
                        ui.label(RichText::new(address_label).size(15.0).color(Colors::gray()));
                    });
                });
            });
        });
    }

    /// Draw QR code image address [`Modal`] content.
    fn qr_address_modal_ui(&mut self,
                           ui: &mut egui::Ui,
                           modal: &Modal,
                           cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);

        // Draw QR code content.
        if let Some(content) = self.qr_address_content.as_mut() {
            content.ui(ui, cb);
        } else {
            modal.close();
            return;
        }

        ui.vertical_centered_justified(|ui| {
            View::button(ui, t!("close"), Colors::white_or_black(false), || {
                self.qr_address_content = None;
                modal.close();
            });
        });
        ui.add_space(6.0);
    }

    /// Draw Tor send content.
    fn tor_send_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        // Setup layout size.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(55.0);

        // Draw round background.
        let bg_rect = rect.clone();
        let item_rounding = View::item_rounding(1, 2, false);
        ui.painter().rect(bg_rect, item_rounding, Colors::fill(), View::item_stroke());

        ui.vertical(|ui| {
            ui.allocate_ui_with_layout(rect.size(), Layout::top_down(Align::Center), |ui| {
                ui.add_space(7.0);
                // Draw button to open sending modal.
                let send_text = format!("{} {}", EXPORT, t!("wallets.send"));
                View::button(ui, send_text, Colors::white_or_black(false), || {
                    self.show_send_tor_modal(cb, None);
                });
            });
        });
    }

    /// Show [`Modal`] to send over Tor.
    pub fn show_send_tor_modal(&mut self, cb: &dyn PlatformCallbacks, address: Option<String>) {
        self.send_modal_content = Some(TransportSendModal::new(address));
        // Show modal.
        Modal::new(SEND_TOR_MODAL)
            .position(ModalPosition::CenterTop)
            .title(t!("wallets.send"))
            .show();
        cb.show_keyboard();
    }
}