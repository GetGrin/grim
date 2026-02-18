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

use egui::{Align, Id, Layout, RichText, StrokeKind};
use egui::os::OperatingSystem;
use url::Url;

use crate::gui::icons::{CLOUD_CHECK, NOTCHES, PENCIL, SCAN, TERMINAL};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::types::{ContentContainer, ModalPosition};
use crate::gui::views::{CameraScanContent, FilePickContent, FilePickContentType, Modal, TextEdit, View};
use crate::gui::Colors;
use crate::tor::{TorBridge, TorConfig, TorProxy};

/// Transport settings content.
pub struct TorSettingsContent {
    /// Flag to check if settings were changed.
    pub settings_changed: bool,

    /// Proxy URL input value for [`Modal`].
    proxy_url_edit: String,
    /// Flag to check if entered proxy address was correct.
    proxy_url_error: bool,

    /// Tor bridge binary path value for [`Modal`].
    bridge_bin_path_edit: String,
    /// Button to pick binary file for bridge.
    bridge_bin_pick_file: FilePickContent,

    /// Tor bridge connection line value for [`Modal`].
    bridge_conn_line_edit: String,
    /// Bridge line QR code scanner [`Modal`] content.
    bridge_qr_scan_content: Option<CameraScanContent>,
}

/// Identifier for proxy URL edit [`Modal`].
const PROXY_URL_EDIT_MODAL: &'static str = "tor_proxy_edit_modal";
/// Identifier for bridge binary path input [`Modal`].
const BRIDGE_BIN_EDIT_MODAL: &'static str = "bridge_bin_edit_modal";
/// Identifier for bridge connection line input [`Modal`].
const BRIDGE_CONN_LINE_EDIT_MODAL: &'static str = "bridge_conn_line_edit_modal";
/// Identifier for [`Modal`] to scan bridge line from QR code.
const SCAN_BRIDGE_CONN_LINE_MODAL: &'static str = "scan_bridge_conn_line_modal";

impl Default for TorSettingsContent {
    fn default() -> Self {
        // Setup Tor bridge binary path edit text.
        let bridge = TorConfig::get_bridge();
        let (bin_path, conn_line) = if let Some(b) = bridge {
            (b.binary_path(), b.connection_line())
        } else {
            ("".to_string(), "".to_string())
        };
        Self {
            settings_changed: false,
            proxy_url_edit: "".to_string(),
            proxy_url_error: false,
            bridge_bin_path_edit: bin_path,
            bridge_bin_pick_file: FilePickContent::new(
                FilePickContentType::ItemButton(View::item_rounding(0, 1, true))
            ).no_parse(),
            bridge_conn_line_edit: conn_line,
            bridge_qr_scan_content: None,
        }
    }
}

impl ContentContainer for TorSettingsContent {
    fn modal_ids(&self) -> Vec<&'static str> {
        vec![
            PROXY_URL_EDIT_MODAL,
            BRIDGE_BIN_EDIT_MODAL,
            BRIDGE_CONN_LINE_EDIT_MODAL,
            SCAN_BRIDGE_CONN_LINE_MODAL
        ]
    }

    fn modal_ui(&mut self, ui: &mut egui::Ui, modal: &Modal, cb: &dyn PlatformCallbacks) {
        match modal.id {
            PROXY_URL_EDIT_MODAL => self.proxy_modal_ui(ui, cb),
            BRIDGE_BIN_EDIT_MODAL => self.bridge_bin_edit_modal_ui(ui, cb),
            BRIDGE_CONN_LINE_EDIT_MODAL => self.bridge_conn_line_edit_modal_ui(ui, cb),
            SCAN_BRIDGE_CONN_LINE_MODAL => {
                if let Some(content) = self.bridge_qr_scan_content.as_mut() {
                    let mut close = false;
                    content.modal_ui(ui, cb, |res| {
                        // Save connection line after scanning.
                        let line = res.text();
                        let bridge = TorConfig::get_bridge().unwrap();
                        if bridge.connection_line() != line {
                            TorBridge::save_bridge_conn_line(&bridge, line);
                            self.settings_changed = true;
                        }
                        close = true;
                    });
                    if close {
                        self.bridge_qr_scan_content = None;
                        cb.stop_camera();
                        Modal::close();
                    }
                }
            }
            _ => {}
        }
    }

    fn container_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        ui.label(RichText::new(format!("{}:", t!("wallets.conn_method")))
            .size(17.0)
            .color(Colors::inactive_text()));
        ui.add_space(10.0);

        let mut proxy = TorConfig::get_proxy();
        let current_proxy = proxy.clone();
        ui.columns(2, |columns| {
            columns[0].vertical_centered(|ui| {
                let name = t!("network_settings.default");
                View::radio_value(ui, &mut proxy, None, name);
            });
            columns[1].vertical_centered(|ui| {
                let name = t!("app_settings.proxy");
                let val = current_proxy.clone()
                    .unwrap_or(TorProxy::SOCKS5(TorProxy::DEFAULT_SOCKS5_URL.to_string()));
                View::radio_value(ui, &mut proxy, Some(val), name);
            });
        });
        ui.add_space(14.0);
        View::horizontal_line(ui, Colors::item_stroke());
        ui.add_space(6.0);

        if let Some(p) = proxy.as_mut() {
            ui.label(RichText::new(format!("{}:", t!("app_settings.proxy")))
                .size(17.0)
                .color(Colors::inactive_text()));
            ui.add_space(10.0);
            ui.columns(2, |columns| {
                columns[0].vertical_centered(|ui| {
                    let value = TorConfig::get_socks5_proxy();
                    View::radio_value(ui, p, value, "SOCKS5".to_string());
                });
                columns[1].vertical_centered(|ui| {
                    let value = TorConfig::get_http_proxy();
                    View::radio_value(ui, p, value, "HTTP".to_string());
                });
            });
            ui.add_space(14.0);
            // Show proxy settings.
            self.proxy_item_ui(p.url(), ui);
            ui.add_space(8.0);
        }

        // Check if proxy type was changed to save.
        if current_proxy != proxy {
            TorConfig::save_proxy(proxy.clone());
            self.settings_changed = true;
        }
        if proxy.is_some() {
            return;
        }

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
                    let default_bridge = TorConfig::get_webtunnel();
                    self.bridge_bin_path_edit = default_bridge.binary_path();
                    self.bridge_conn_line_edit = default_bridge.connection_line();
                    Some(default_bridge)
                };
                TorConfig::save_bridge(value);
                self.settings_changed = true;
            });
        });

        if bridge.is_some() {
            ui.add_space(6.0);
            // Show bridge selection for non-Android.
            let is_android = OperatingSystem::from_target_os() == OperatingSystem::Android;
            if !is_android {
                let current_bridge = bridge.unwrap();
                let mut bridge = current_bridge.clone();

                ui.columns(2, |columns| {
                    columns[0].vertical_centered(|ui| {
                        // Show Webtunnel bridge selector.
                        let webtunnel = TorConfig::get_webtunnel();
                        let name = webtunnel.protocol_name().to_uppercase();
                        View::radio_value(ui, &mut bridge, webtunnel, name);

                    });
                    columns[1].vertical_centered(|ui| {
                        // Show Obfs4 bridge selector.
                        let obfs4 = TorConfig::get_obfs4();
                        let name = obfs4.protocol_name().to_uppercase();
                        View::radio_value(ui, &mut bridge, obfs4, name);
                    });
                });
                ui.add_space(10.0);
                ui.vertical_centered(|ui| {
                    // Show Snowflake bridge selector.
                    let snowflake = TorConfig::get_snowflake();
                    let name = snowflake.protocol_name().to_uppercase();
                    View::radio_value(ui, &mut bridge, snowflake, name);
                });
                ui.add_space(16.0);

                // Check if bridge type was changed to save.
                if current_bridge != bridge {
                    TorConfig::save_bridge(Some(bridge.clone()));
                    self.bridge_bin_path_edit = bridge.binary_path();
                    self.bridge_conn_line_edit = bridge.connection_line();
                    self.settings_changed = true;
                }
            }

            if let Some(br) = TorConfig::get_bridge().as_ref() {
                // Show bridge binary setup for non-Android.
                if !is_android {
                    self.bridge_bin_ui(ui, br, cb);
                    ui.add_space(10.0);
                }
                // Show bridge connection line setup.
                self.bridge_conn_line_ui(ui, br, cb);
            }

            ui.add_space(8.0);
        }
    }
}

impl TorSettingsContent {
    /// Draw proxy edit modal content.
    fn proxy_modal_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        let on_save = |c: &mut TorSettingsContent| {
            let http = "http://";
            let socks = "socks5://";
            let url = c.proxy_url_edit.trim().to_string();
            c.proxy_url_error = Url::parse(url.as_str()).is_err();
            if !c.proxy_url_error {
                let proxy = TorConfig::get_proxy().unwrap();
                if url.contains(socks) {
                    TorConfig::save_proxy(Some(TorProxy::SOCKS5(url)));
                } else if url.contains(http) {
                    TorConfig::save_proxy(Some(TorProxy::HTTP(url)));
                } else {
                    match proxy {
                        TorProxy::SOCKS5(_) => {
                            TorConfig::save_proxy(Some(TorProxy::SOCKS5(url)));
                        }
                        TorProxy::HTTP(_) => {
                            TorConfig::save_proxy(Some(TorProxy::HTTP(url)));
                        }
                    }
                }
                Modal::close();
            }
        };

        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            let label = format!("{}:", t!("enter_url"));
            ui.label(RichText::new(label).size(17.0).color(Colors::gray()));
            ui.add_space(8.0);

            // Draw proxy URL text edit.
            let mut edit = TextEdit::new(Id::from("proxy_url_edit").with(PROXY_URL_EDIT_MODAL))
                .paste();
            edit.ui(ui, &mut self.proxy_url_edit, cb);
            if edit.enter_pressed {
                on_save(self);
            }

            // Show error when specified address is incorrect.
            if self.proxy_url_error {
                ui.add_space(10.0);
                ui.label(RichText::new(t!("wallets.invalid_url"))
                    .size(16.0)
                    .color(Colors::red()));
            }
            ui.add_space(12.0);

            // Show modal buttons.
            ui.scope(|ui| {
                // Setup spacing between buttons.
                ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

                ui.columns(2, |columns| {
                    columns[0].vertical_centered_justified(|ui| {
                        View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                            Modal::close();
                        });
                    });
                    columns[1].vertical_centered_justified(|ui| {
                        View::button(ui, t!("modal.save"), Colors::white_or_black(false), || {
                            on_save(self);
                        });
                    });
                });
                ui.add_space(6.0);
            });
        });
    }

    /// Draw proxy item content.
    fn proxy_item_ui(&mut self, url: String, ui: &mut egui::Ui) {
        // Setup layout size.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(56.0);

        // Draw round background.
        let bg_rect = rect.clone();
        let item_rounding = View::item_rounding(0, 1, false);
        ui.painter().rect(bg_rect,
                          item_rounding,
                          Colors::fill(),
                          View::item_stroke(),
                          StrokeKind::Middle);

        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            View::item_button(ui, View::item_rounding(0, 1, true), PENCIL, None, || {
                self.proxy_url_edit = url.clone();
                // Show proxy URL edit modal.
                Modal::new(PROXY_URL_EDIT_MODAL)
                    .position(ModalPosition::CenterTop)
                    .title(t!("app_settings.proxy"))
                    .show();
            });
            let layout_size = ui.available_size();
            ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    View::ellipsize_text(ui, url, 18.0, Colors::title(false));
                    ui.add_space(1.0);

                    let value = format!("{} {}", CLOUD_CHECK, t!("network_settings.enabled"));
                    ui.label(RichText::new(value).size(15.0).color(Colors::gray()));
                    ui.add_space(3.0);
                });
            });
        });
    }

    /// Draw bridge binary setup content.
    fn bridge_bin_ui(&mut self, ui: &mut egui::Ui, bridge: &TorBridge, cb: &dyn PlatformCallbacks) {
        // Setup layout size.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(56.0);

        // Draw round background.
        let bg_rect = rect.clone();
        let item_rounding = View::item_rounding(0, 1, false);
        ui.painter().rect(bg_rect,
                          item_rounding,
                          Colors::fill(),
                          View::item_stroke(),
                          StrokeKind::Middle);

        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            self.bridge_bin_pick_file.ui(ui, cb, |path| {
                if bridge.binary_path() != path {
                    TorBridge::save_bridge_bin_path(bridge, path);
                    self.settings_changed = true;
                }
            });
            View::item_button(ui, View::item_rounding(1, 3, true), PENCIL, None, || {
                self.bridge_bin_path_edit = bridge.binary_path();
                // Show binary path edit modal.
                let title = bridge.protocol_name();
                Modal::new(BRIDGE_BIN_EDIT_MODAL)
                    .position(ModalPosition::CenterTop)
                    .title(title)
                    .show();
            });
            let layout_size = ui.available_size();
            ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    View::ellipsize_text(ui, bridge.binary_path(), 18.0, Colors::title(false));
                    ui.add_space(1.0);
                    let value = format!("{} {}",
                                        TERMINAL,
                                        t!("transport.bin_file").replace(":", ""));
                    ui.label(RichText::new(value).size(15.0).color(Colors::gray()));
                    ui.add_space(3.0);
                });
            });
        });
    }

    /// Draw bridge binary input [`Modal`] content.
    fn bridge_bin_edit_modal_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        let on_save = |c: &mut TorSettingsContent| {
            let bridge = TorConfig::get_bridge().unwrap();
            if bridge.binary_path() != c.bridge_bin_path_edit {
                TorBridge::save_bridge_bin_path(&bridge, c.bridge_bin_path_edit.clone());
                c.settings_changed = true;
            }
            Modal::close();
        };

        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("transport.bin_file"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Draw p2p port text edit.
            let mut edit = TextEdit::new(Id::from(BRIDGE_BIN_EDIT_MODAL)).paste();
            edit.ui(ui, &mut self.bridge_bin_path_edit, cb);
            if edit.enter_pressed {
                on_save(self);
            }
            ui.add_space(12.0);

            // Show modal buttons.
            ui.scope(|ui| {
                // Setup spacing between buttons.
                ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

                ui.columns(2, |columns| {
                    columns[0].vertical_centered_justified(|ui| {
                        View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                            // Close modal.
                            Modal::close();
                        });
                    });
                    columns[1].vertical_centered_justified(|ui| {
                        View::button(ui, t!("modal.save"), Colors::white_or_black(false), || {
                            on_save(self);
                        });
                    });
                });
                ui.add_space(6.0);
            });
        });
    }

    /// Draw bridge connection line setup content.
    fn bridge_conn_line_ui(&mut self,
                           ui: &mut egui::Ui,
                           bridge: &TorBridge,
                           cb: &dyn PlatformCallbacks) {
        // Setup layout size.
        let mut rect = ui.available_rect_before_wrap();
        rect.set_height(56.0);

        // Draw round background.
        let bg_rect = rect.clone();
        let item_rounding = View::item_rounding(0, 1, false);
        ui.painter().rect(bg_rect,
                          item_rounding,
                          Colors::fill(),
                          View::item_stroke(),
                          StrokeKind::Middle);

        ui.allocate_ui_with_layout(rect.size(), Layout::right_to_left(Align::Center), |ui| {
            View::item_button(ui, View::item_rounding(0, 1, true), SCAN, None, || {
                self.show_qr_scan_bridge_modal(cb);
            });
            View::item_button(ui, View::item_rounding(1, 3 , true), PENCIL, None, || {
                self.bridge_conn_line_edit = bridge.connection_line();
                // Show connection line edit modal.
                let title = bridge.protocol_name();
                Modal::new(BRIDGE_CONN_LINE_EDIT_MODAL)
                    .position(ModalPosition::CenterTop)
                    .title(title)
                    .show();
            });
            let layout_size = ui.available_size();
            ui.allocate_ui_with_layout(layout_size, Layout::left_to_right(Align::Center), |ui| {
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    View::ellipsize_text(ui, bridge.connection_line(), 18.0, Colors::title(false));
                    ui.add_space(1.0);
                    let value = format!("{} {}",
                                        NOTCHES,
                                        t!("transport.conn_line").replace(":", ""));
                    ui.label(RichText::new(value).size(15.0).color(Colors::gray()));
                    ui.add_space(3.0);
                });
            });
        });
    }

    /// Show bridge connection line QR code scanner.
    fn show_qr_scan_bridge_modal(&mut self, cb: &dyn PlatformCallbacks) {
        self.bridge_qr_scan_content = Some(CameraScanContent::default());
        // Show QR code scan modal.
        Modal::new(SCAN_BRIDGE_CONN_LINE_MODAL)
            .position(ModalPosition::CenterTop)
            .title(t!("scan_qr"))
            .closeable(false)
            .show();
        cb.start_camera();
    }

    /// Draw bridge connection line input [`Modal`] content.
    fn bridge_conn_line_edit_modal_ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        let on_save = |c: &mut TorSettingsContent| {
            let bridge = TorConfig::get_bridge().unwrap();
            if bridge.connection_line() != c.bridge_conn_line_edit {
                TorBridge::save_bridge_conn_line(&bridge, c.bridge_conn_line_edit.clone());
                c.settings_changed = true;
            }
            Modal::close();
        };

        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t!("transport.conn_line"))
                .size(17.0)
                .color(Colors::gray()));
            ui.add_space(8.0);

            // Draw connection line text edit.
            let mut edit = TextEdit::new(Id::from(BRIDGE_CONN_LINE_EDIT_MODAL)).paste();
            edit.ui(ui, &mut self.bridge_conn_line_edit, cb);
            if edit.enter_pressed {
                on_save(self);
            }
            ui.add_space(12.0);

            // Show modal buttons.
            ui.scope(|ui| {
                // Setup spacing between buttons.
                ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

                ui.columns(2, |columns| {
                    columns[0].vertical_centered_justified(|ui| {
                        View::button(ui, t!("modal.cancel"), Colors::white_or_black(false), || {
                            // Close modal.
                            Modal::close();
                        });
                    });
                    columns[1].vertical_centered_justified(|ui| {
                        View::button(ui, t!("modal.save"), Colors::white_or_black(false), || {
                            on_save(self);
                        });
                    });
                });
                ui.add_space(6.0);
            });
        });
    }
}