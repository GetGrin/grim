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

use std::sync::Arc;
use std::thread;
use egui::{Align, Id, Layout, Margin, RichText, Rounding, ScrollArea};
use egui::os::OperatingSystem;
use egui::scroll_area::ScrollBarVisibility;
use parking_lot::RwLock;
use tor_rtcompat::BlockOn;
use tor_rtcompat::tokio::TokioNativeTlsRuntime;
use grin_core::core::{amount_from_hr_string, amount_to_hr_string};
use grin_wallet_libwallet::SlatepackAddress;

use crate::gui::Colors;
use crate::gui::icons::{CHECK_CIRCLE, COPY, DOTS_THREE_CIRCLE, EXPORT, GEAR_SIX, GLOBE_SIMPLE, POWER, QR_CODE, SHIELD_CHECKERED, SHIELD_SLASH, WARNING_CIRCLE, X_CIRCLE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{CameraContent, Modal, QrCodeContent, Content, View};
use crate::gui::views::types::{ModalPosition, TextEditOptions};
use crate::gui::views::wallets::wallet::types::{WalletTab, WalletTabType};
use crate::gui::views::wallets::wallet::WalletContent;
use crate::tor::{Tor, TorBridge, TorConfig};
use crate::wallet::types::WalletData;
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
    /// Flag to check if QR code scanner is opened at address [`Modal`].
    show_address_scan: bool,
    /// Address QR code scanner [`Modal`] content.
    address_scan_content: CameraContent,
    /// Flag to check if [`Modal`] was just opened to focus on first field.
    modal_just_opened: bool,

    /// QR code address image [`Modal`] content.
    qr_address_content: QrCodeContent,

    /// Flag to check if Tor settings were changed.
    tor_settings_changed: bool,
    /// Tor bridge binary path edit text.
    bridge_bin_path_edit: String,
    /// Tor bridge connection line edit text.
    bridge_conn_line_edit: String,
    /// Flag to check if QR code scanner is opened at bridge [`Modal`].
    show_bridge_scan: bool,
    /// Address QR code scanner [`Modal`] content.
    bridge_qr_scan_content: CameraContent,
}

impl WalletTab for WalletTransport {
    fn get_type(&self) -> WalletTabType {
        WalletTabType::Transport
    }

    fn ui(&mut self,
          ui: &mut egui::Ui,
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

impl WalletTransport {
    /// Create new content instance from provided Slatepack address text.
    pub fn new(addr: String) -> Self {
        // Setup Tor bridge binary path edit text.
        let bridge = TorConfig::get_bridge();
        let (bin_path, conn_line) = if let Some(b) = bridge {
            (b.binary_path(), b.connection_line())
        } else {
            ("".to_string(), "".to_string())
        };
        Self {
            tor_sending: Arc::new(RwLock::new(false)),
            tor_send_error: Arc::new(RwLock::new(false)),
            tor_success: Arc::new(RwLock::new(false)),
            amount_edit: "".to_string(),
            address_edit: "".to_string(),
            address_error: false,
            show_address_scan: false,
            address_scan_content: CameraContent::default(),
            modal_just_opened: false,
            qr_address_content: QrCodeContent::new(addr, false),
            tor_settings_changed: false,
            bridge_bin_path_edit: bin_path,
            bridge_conn_line_edit: conn_line,
            show_bridge_scan: false,
            bridge_qr_scan_content: CameraContent::default(),
        }
    }

    /// Draw wallet transport content.
    pub fn ui(&mut self, ui: &mut egui::Ui, wallet: &mut Wallet, cb: &dyn PlatformCallbacks) {
        ui.add_space(3.0);
        ui.label(RichText::new(t!("transport.desc"))
            .size(16.0)
            .color(Colors::inactive_text()));
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
                            self.tor_settings_modal_ui(ui, wallet, modal, cb);
                        });
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
    fn tor_ui(&mut self, ui: &mut egui::Ui, wallet: &mut Wallet, cb: &dyn PlatformCallbacks) {
        let data = wallet.get_data().unwrap();

        // Draw header content.
        self.tor_header_ui(ui, wallet);

        // Draw receive info content.
        if wallet.slatepack_address().is_some() {
            self.tor_receive_ui(ui, wallet, &data, cb);
        }

        // Draw send content.
        if data.info.amount_currently_spendable > 0 && wallet.foreign_api_port().is_some() {
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
                    self.show_tor_settings_modal();
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

    /// Show Tor transport settings [`Modal`].
    fn show_tor_settings_modal(&mut self) {
        self.tor_settings_changed = false;
        // Show Tor settings modal.
        Modal::new(TOR_SETTINGS_MODAL)
            .position(ModalPosition::CenterTop)
            .title(t!("transport.tor_settings"))
            .closeable(false)
            .show();
    }

    /// Draw Tor transport settings [`Modal`] content.
    fn tor_settings_modal_ui(&mut self,
                             ui: &mut egui::Ui,
                             wallet: &Wallet,
                             modal: &Modal,
                             cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);

        // Draw QR code scanner content if requested.
        if self.show_bridge_scan {
            let mut on_stop = |content: &mut CameraContent| {
                cb.stop_camera();
                content.clear_state();
                modal.enable_closing();
                self.show_bridge_scan = false;
            };

            if let Some(result) = self.bridge_qr_scan_content.qr_scan_result() {
                self.bridge_conn_line_edit = result.text();
                on_stop(&mut self.bridge_qr_scan_content);
                cb.show_keyboard();
            } else {
                self.bridge_qr_scan_content.ui(ui, cb);
                ui.add_space(12.0);

                // Setup spacing between buttons.
                ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

                // Show buttons to close modal or come back to sending input.
                ui.columns(2, |cols| {
                    cols[0].vertical_centered_justified(|ui| {
                        View::button(ui, t!("close"), Colors::white_or_black(false), || {
                            on_stop(&mut self.bridge_qr_scan_content);
                            modal.close();
                        });
                    });
                    cols[1].vertical_centered_justified(|ui| {
                        View::button(ui, t!("back"), Colors::white_or_black(false), || {
                            on_stop(&mut self.bridge_qr_scan_content);
                        });
                    });
                });
                ui.add_space(6.0);
            }
            return;
        }

        // Do not show bridges setup on Android.
        let os = OperatingSystem::from_target_os();
        let show_bridges = os != OperatingSystem::Android;
        if show_bridges {
            let bridge = TorConfig::get_bridge();
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(t!("transport.bridges_desc"))
                    .size(17.0)
                    .color(Colors::inactive_text()));

                // Draw checkbox to enable/disable bridges.
                View::checkbox(ui, bridge.is_some(), t!("transport.bridges"), || {
                    // Save value.
                    let value = if bridge.is_some() {
                        None
                    } else {
                        let default_bridge = TorConfig::get_obfs4();
                        self.bridge_bin_path_edit = default_bridge.binary_path();
                        self.bridge_conn_line_edit = default_bridge.connection_line();
                        Some(default_bridge)
                    };
                    TorConfig::save_bridge(value);
                    self.tor_settings_changed = true;
                });
            });

            // Draw bridges selection and path.
            if bridge.is_some() {
                let current_bridge = bridge.unwrap();
                let mut bridge = current_bridge.clone();

                ui.add_space(6.0);
                ui.columns(2, |columns| {
                    columns[0].vertical_centered(|ui| {
                        // Draw Obfs4 bridge selector.
                        let obfs4 = TorConfig::get_obfs4();
                        let name = obfs4.protocol_name().to_uppercase();
                        View::radio_value(ui, &mut bridge, obfs4, name);
                    });
                    columns[1].vertical_centered(|ui| {
                        // Draw Snowflake bridge selector.
                        let snowflake = TorConfig::get_snowflake();
                        let name = snowflake.protocol_name().to_uppercase();
                        View::radio_value(ui, &mut bridge, snowflake, name);
                    });
                });
                ui.add_space(12.0);

                // Check if bridge type was changed to save.
                if current_bridge != bridge {
                    self.tor_settings_changed = true;
                    TorConfig::save_bridge(Some(bridge.clone()));
                    self.bridge_bin_path_edit = bridge.binary_path();
                    self.bridge_conn_line_edit = bridge.connection_line();
                }

                // Draw binary path text edit.
                let bin_edit_id = Id::from(modal.id)
                    .with(wallet.get_config().id)
                    .with("_bin_edit");
                let mut bin_edit_opts = TextEditOptions::new(bin_edit_id)
                    .paste()
                    .no_focus();
                let bin_edit_before = self.bridge_bin_path_edit.clone();
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(t!("transport.bin_file"))
                        .size(17.0)
                        .color(Colors::inactive_text()));
                    ui.add_space(6.0);
                    View::text_edit(ui, cb, &mut self.bridge_bin_path_edit, &mut bin_edit_opts);
                    ui.add_space(6.0);
                });

                // Draw connection line text edit.
                let conn_edit_before = self.bridge_conn_line_edit.clone();
                let conn_edit_id = Id::from(modal.id)
                    .with(wallet.get_config().id)
                    .with("_conn_edit");
                let mut conn_edit_opts = TextEditOptions::new(conn_edit_id)
                    .paste()
                    .no_focus()
                    .scan_qr();
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(t!("transport.conn_line"))
                        .size(17.0)
                        .color(Colors::inactive_text()));
                    ui.add_space(6.0);
                    View::text_edit(ui, cb, &mut self.bridge_conn_line_edit, &mut conn_edit_opts);
                    // Check if scan button was pressed.
                    if conn_edit_opts.scan_pressed {
                        cb.hide_keyboard();
                        modal.disable_closing();
                        conn_edit_opts.scan_pressed = false;
                        self.show_bridge_scan = true;
                    }
                });

                // Check if bin path or connection line text was changed to save bridge.
                if conn_edit_before != self.bridge_conn_line_edit ||
                    bin_edit_before != self.bridge_bin_path_edit {
                    let bin_path = self.bridge_bin_path_edit.trim().to_string();
                    let conn_line = self.bridge_conn_line_edit.trim().to_string();
                    let b = match bridge {
                        TorBridge::Snowflake(_, _) => {
                            TorBridge::Snowflake(bin_path, conn_line)
                        },
                        TorBridge::Obfs4(_, _) => {
                            TorBridge::Obfs4(bin_path, conn_line)
                        }
                    };
                    TorConfig::save_bridge(Some(b));
                    self.tor_settings_changed = true;
                }

                ui.add_space(2.0);
            }

            ui.add_space(6.0);
            View::horizontal_line(ui, Colors::item_stroke());
            ui.add_space(6.0);
        }

        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("transport.tor_autorun_desc"))
                .size(17.0)
                .color(Colors::inactive_text()));

            // Show Tor service autorun checkbox.
            let autorun = wallet.auto_start_tor_listener();
            View::checkbox(ui, autorun, t!("network.autorun"), || {
                wallet.update_auto_start_tor_listener(!autorun);
            });
        });
        ui.add_space(6.0);
        ui.vertical_centered_justified(|ui| {
            View::button(ui, t!("close"), Colors::white_or_black(false), || {
                if self.tor_settings_changed {
                    self.tor_settings_changed = false;
                    // Restart running service or rebuild client.
                    let service_id = &wallet.identifier();
                    if Tor::is_service_running(service_id) {
                        if let Ok(key) = wallet.secret_key() {
                            let api_port = wallet.foreign_api_port().unwrap();
                            Tor::restart_service(api_port, key, service_id);
                        }
                    } else {
                        Tor::rebuild_client();
                    }
                }
                modal.close();
            });
        });
        ui.add_space(6.0);
    }

    /// Draw Tor receive content.
    fn tor_receive_ui(&mut self,
                      ui: &mut egui::Ui,
                      wallet: &Wallet,
                      data: &WalletData,
                      cb: &dyn PlatformCallbacks) {
        let slatepack_addr = wallet.slatepack_address().unwrap();
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
                    self.qr_address_content.clear_state();
                    Modal::new(QR_ADDRESS_MODAL)
                        .position(ModalPosition::CenterTop)
                        .title(t!("network_mining.address"))
                        .show();
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
                        let address_color = if Tor::is_service_starting(service_id) ||
                            wallet.foreign_api_port().is_none() {
                            Colors::inactive_text()
                        } else if Tor::is_service_running(service_id) {
                            Colors::green()
                        } else {
                            Colors::red()
                        };
                        View::ellipsize_text(ui, slatepack_addr, 15.0, address_color);

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
    fn qr_address_modal_ui(&mut self, ui: &mut egui::Ui, m: &Modal, cb: &dyn PlatformCallbacks) {
        ui.add_space(6.0);

        // Draw QR code content.
        let text = self.qr_address_content.text.clone();
        self.qr_address_content.ui(ui, text.clone(), cb);

        ui.vertical_centered_justified(|ui| {
            View::button(ui, t!("close"), Colors::white_or_black(false), || {
                self.qr_address_content.clear_state();
                m.close();
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
        {
            let mut w_send_err = self.tor_send_error.write();
            *w_send_err = false;
            let mut w_sending = self.tor_sending.write();
            *w_sending = false;
            let mut w_success = self.tor_success.write();
            *w_success = false;
        }
        self.modal_just_opened = true;
        self.amount_edit = "".to_string();
        self.address_edit = address.unwrap_or("".to_string());
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
        let r_send_err = self.tor_send_error.read();
        r_send_err.clone()
    }

    /// Check if transaction is sending over Tor to show progress at [`Modal`].
    fn tor_sending(&self) -> bool {
        let r_sending = self.tor_sending.read();
        r_sending.clone()
    }

    /// Check if transaction sent over Tor with success at [`Modal`].
    fn tor_success(&self) -> bool {
        let r_success = self.tor_success.read();
        r_success.clone()
    }

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
            // Draw QR code scanner content if requested.
            if self.show_address_scan {
                let mut on_stop = |content: &mut CameraContent| {
                    cb.stop_camera();
                    content.clear_state();
                    modal.enable_closing();
                    self.show_address_scan = false;
                };

                if let Some(result) = self.address_scan_content.qr_scan_result() {
                    self.address_edit = result.text();
                    self.modal_just_opened = true;
                    on_stop(&mut self.address_scan_content);
                    cb.show_keyboard();
                } else {
                    self.address_scan_content.ui(ui, cb);
                    ui.add_space(6.0);

                    // Setup spacing between buttons.
                    ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

                    // Show buttons to close modal or come back to sending input.
                    ui.columns(2, |cols| {
                        cols[0].vertical_centered_justified(|ui| {
                            View::button(ui, t!("close"), Colors::white_or_black(false), || {
                                on_stop(&mut self.address_scan_content);
                                modal.close();
                            });
                        });
                        cols[1].vertical_centered_justified(|ui| {
                            View::button(ui, t!("back"), Colors::white_or_black(false), || {
                                self.modal_just_opened = true;
                                on_stop(&mut self.address_scan_content);
                                cb.show_keyboard();
                            });
                        });
                    });
                    ui.add_space(6.0);
                }
                return;
            }

            ui.vertical_centered(|ui| {
                let data = wallet.get_data().unwrap();
                let amount = amount_to_hr_string(data.info.amount_currently_spendable, true);
                let enter_text = t!("wallets.enter_amount_send","amount" => amount);
                ui.label(RichText::new(enter_text)
                    .size(17.0)
                    .color(Colors::gray()));
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
            View::text_edit(ui, cb, &mut self.amount_edit, &mut amount_edit_opts);
            ui.add_space(8.0);

            // Check value if input was changed.
            if amount_edit_before != self.amount_edit {
                if !self.amount_edit.is_empty() {
                    // Trim text, replace "," by "." and parse amount.
                    self.amount_edit = self.amount_edit.trim().replace(",", ".");
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
                        .color(Colors::red()));
                } else {
                    ui.label(RichText::new(t!("transport.receiver_address"))
                        .size(17.0)
                        .color(Colors::gray()));
                }
            });
            ui.add_space(6.0);

            // Draw address text edit.
            let addr_edit_before = self.address_edit.clone();
            let address_edit_id = Id::from(modal.id).with("address").with(wallet.get_config().id);
            let mut address_edit_opts = TextEditOptions::new(address_edit_id)
                .paste()
                .no_focus()
                .scan_qr();
            View::text_edit(ui, cb, &mut self.address_edit, &mut address_edit_opts);
            // Check if scan button was pressed.
            if address_edit_opts.scan_pressed {
                cb.hide_keyboard();
                modal.disable_closing();
                address_edit_opts.scan_pressed = false;
                self.show_address_scan = true;
            }
            ui.add_space(12.0);

            // Check value if input was changed.
            if addr_edit_before != self.address_edit {
                self.address_error = false;
            }

            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                        self.amount_edit = "".to_string();
                        self.address_edit = "".to_string();
                        cb.hide_keyboard();
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("continue"), Colors::white_or_black(false), || {
                        if self.amount_edit.is_empty() {
                            return;
                        }

                        // Check entered address.
                        let addr_str = self.address_edit.as_str();
                        if let Ok(addr) = SlatepackAddress::try_from(addr_str) {
                            // Parse amount and send over Tor.
                            if let Ok(a) = amount_from_hr_string(self.amount_edit.as_str()) {
                                cb.hide_keyboard();
                                modal.disable_closing();
                                let mut w_sending = self.tor_sending.write();
                                *w_sending = true;
                                {
                                    let send_error = self.tor_send_error.clone();
                                    let send_success = self.tor_success.clone();
                                    let mut wallet = wallet.clone();
                                    thread::spawn(move || {
                                        let runtime = TokioNativeTlsRuntime::create().unwrap();
                                        runtime
                                            .block_on(async {
                                                if wallet.send_tor(a, &addr)
                                                    .await
                                                    .is_some() {
                                                    let mut w_send_success = send_success.write();
                                                    *w_send_success = true;
                                                } else {
                                                    let mut w_send_error = send_error.write();
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
                    .color(Colors::red()));
            });
            ui.add_space(12.0);

            // Setup spacing between buttons.
            ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

            ui.columns(2, |columns| {
                columns[0].vertical_centered_justified(|ui| {
                    View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                        self.amount_edit = "".to_string();
                        self.address_edit = "".to_string();
                        cb.hide_keyboard();
                        modal.close();
                    });
                });
                columns[1].vertical_centered_justified(|ui| {
                    View::button(ui, t!("repeat"), Colors::white_or_black(false), || {
                        // Parse amount and send over Tor.
                        if let Ok(a) = amount_from_hr_string(self.amount_edit.as_str()) {
                            let mut w_send_error = self.tor_send_error.write();
                            *w_send_error = false;
                            let mut w_sending = self.tor_sending.write();
                            *w_sending = true;
                            {
                                let addr_text = self.address_edit.clone();
                                let send_error = self.tor_send_error.clone();
                                let send_success = self.tor_success.clone();
                                let mut wallet = wallet.clone();
                                thread::spawn(move || {
                                    let runtime = TokioNativeTlsRuntime::create().unwrap();
                                    runtime
                                        .block_on(async {
                                            let addr_str = addr_text.as_str();
                                            let addr = &SlatepackAddress::try_from(addr_str)
                                                .unwrap();
                                            if wallet.send_tor(a, &addr)
                                                .await
                                                .is_some() {
                                                let mut w_send_success = send_success.write();
                                                *w_send_success = true;
                                            } else {
                                                let mut w_send_error = send_error.write();
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
                ui.label(RichText::new(t!("transport.tor_sending", "amount" => self.amount_edit))
                    .size(17.0)
                    .color(Colors::gray()));
            });
            ui.add_space(10.0);

            // Close modal on success sending.
            if self.tor_success() {
                modal.close();
            }
        }
    }
}