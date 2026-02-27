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

use egui::{Align, CornerRadius, Layout, RichText, StrokeKind};

use crate::gui::icons::{CIRCLE_HALF, DOTS_THREE_CIRCLE, PLUGS, PLUGS_CONNECTED, POWER, QR_CODE, SHIELD_CHECKERED, SHIELD_SLASH, WARNING_CIRCLE, WRENCH};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::wallets::wallet::transport::settings::WalletTransportSettingsContent;
use crate::gui::views::wallets::wallet::types::WalletContentContainer;
use crate::gui::views::{Modal, QrCodeContent, View};
use crate::gui::Colors;
use crate::tor::{Tor, TorConfig};
use crate::wallet::Wallet;

/// Wallet transport panel content.
pub struct WalletTransportContent {
    /// QR code address content.
    pub qr_address_content: Option<QrCodeContent>,

    /// Settings content.
    pub settings_content: Option<WalletTransportSettingsContent>,
}

impl WalletContentContainer for WalletTransportContent {
    fn modal_ids(&self) -> Vec<&'static str> { vec![] }

    fn modal_ui(&mut self, _: &mut egui::Ui, _: &Wallet, _: &Modal, _: &dyn PlatformCallbacks) {
    }

    fn container_ui(&mut self, ui: &mut egui::Ui, wallet: &Wallet, cb: &dyn PlatformCallbacks) {
        if let Some(content) = self.qr_address_content.as_mut() {
            ui.add_space(6.0);

            // Draw QR code content.
            content.ui(ui, cb);

            ui.vertical_centered_justified(|ui| {
                View::button(ui, t!("close"), Colors::white_or_black(false), || {
                    self.qr_address_content = None;
                });
            });
            ui.add_space(6.0);
        } else if let Some(content) = self.settings_content.as_mut() {
            let mut closed = false;
            content.ui(ui, wallet, cb, || {
                closed = true;
            });
            if closed {
                self.settings_content = None;
            }
        } else {
            self.tor_header_ui(ui, wallet);
        }
    }
}

impl Default for WalletTransportContent {
    fn default() -> Self {
        Self {
            qr_address_content: None,
            settings_content: None,
        }
    }
}

impl WalletTransportContent {
    /// Check if it's possible to go back at navigation stack.
    pub fn can_back(&self) -> bool {
        self.settings_content.is_some() || self.qr_address_content.is_some()
    }

    /// Navigate back on navigation stack.
    pub fn back(&mut self) {
        if self.settings_content.is_some() {
            self.settings_content = None;
        } else if self.qr_address_content.is_some() {
            self.qr_address_content = None;
        }
    }

    /// Draw Tor transport header content.
    fn tor_header_ui(&mut self, ui: &mut egui::Ui, wallet: &Wallet) {
        let wallet_data = wallet.get_data();
        if wallet_data.is_none() {
            return;
        }
        let addr = wallet.slatepack_address().unwrap();

        // Setup layout size.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(78.0);

        // Draw round background.
        let info = wallet.get_data().unwrap().info;
        let awaiting_balance = info.amount_awaiting_confirmation > 0 ||
            info.amount_awaiting_finalization > 0 || info.amount_locked > 0;
        let rounding = if awaiting_balance {
            View::item_rounding(1, 3, false)
        } else {
            View::item_rounding(1, 2, false)
        };
        ui.painter().rect(rect,
                          rounding,
                          Colors::fill(),
                          View::item_stroke(),
                          StrokeKind::Outside);

        ui.vertical(|ui| {
            ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
                // Show button to show QR code address.
                let r = if awaiting_balance {
                    View::item_rounding(1, 3, true)
                } else {
                    View::item_rounding(1, 2, true)
                };
                View::item_button(ui,  r, QR_CODE, None, || {
                    self.qr_address_content = Some(QrCodeContent::new(addr.clone(), false)
                        .with_max_size(320.0));
                });

                // Draw button to enable/disable Tor listener for current wallet.
                let service_id = &wallet.identifier();
                if  !Tor::is_service_starting(service_id) && wallet.foreign_api_port().is_some() &&
                    wallet.secret_key().is_some() {
                    if !Tor::is_service_running(service_id) {
                        let r = CornerRadius::default();
                        View::item_button(ui, r, POWER, Some(Colors::green()), || {
                            let api_port = wallet.foreign_api_port().unwrap();
                            let key = wallet.secret_key().unwrap();
                            Tor::start_service(api_port, key, service_id);
                        });
                    } else {
                        let r = CornerRadius::default();
                        View::item_button(ui, r, POWER, Some(Colors::red()), || {
                            Tor::stop_service(service_id);
                        });
                    }
                }

                // Draw button to show Tor transport settings.
                let button_rounding = View::item_rounding(1, 3, true);
                View::item_button(ui, button_rounding, WRENCH, None, || {
                    self.settings_content = Some(WalletTransportSettingsContent::default());
                });

                let layout_size = ui.available_size();
                ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                    ui.add_space(6.0);
                    ui.vertical(|ui| {
                        ui.add_space(3.0);

                        let is_running = Tor::is_service_running(service_id);
                        let has_error = Tor::is_service_failed(service_id);
                        let is_starting = Tor::is_service_starting(service_id);
                        let address_color = if is_running && !is_starting {
                            Colors::green()
                        } else if has_error {
                            Colors::red()
                        } else {
                            Colors::inactive_text()
                        };
                        // Show slatepack address text.
                        View::animate_text(ui, addr.clone(), 17.0, address_color, is_starting);
                        ui.add_space(1.0);

                        let (icon, text) = if is_starting {
                            (DOTS_THREE_CIRCLE, t!("transport.connecting"))
                        } else if has_error {
                            (WARNING_CIRCLE, t!("transport.conn_error"))
                        } else if is_running {
                            (PLUGS_CONNECTED, t!("transport.connected"))
                        } else if let Some(_) = TorConfig::get_proxy() {
                            (PLUGS_CONNECTED, t!("app_settings.proxy"))
                        } else {
                            (PLUGS, t!("transport.disconnected"))
                        };
                        let status_text = format!("{} {}", icon, text);
                        // Show connection status text.
                        View::ellipsize_text(ui, status_text, 15.0, Colors::text(false));
                        ui.add_space(1.0);

                        let bridges_text = if is_starting || has_error {
                            match TorConfig::get_bridge() {
                                None => {
                                    format!("{} {}", SHIELD_SLASH, t!("transport.bridges_disabled"))
                                }
                                Some(b) => {
                                    let name = b.protocol_name().to_uppercase();
                                    format!("{} {}",
                                            SHIELD_CHECKERED,
                                            t!("transport.bridge_name", "b" = name))
                                }
                            }
                        } else {
                            format!("{} {}", CIRCLE_HALF, t!("transport.tor_network"))
                        };
                        // Show bridge info text.
                        ui.label(RichText::new(bridges_text).size(15.0).color(Colors::gray()));
                    });
                });
            });
        });
    }
}