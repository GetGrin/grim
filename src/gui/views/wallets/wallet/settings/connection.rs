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

use egui::RichText;

use crate::gui::icons::{GLOBE, PLUS_CIRCLE};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::network::modals::ExternalConnectionModal;
use crate::gui::views::network::ConnectionsContent;
use crate::gui::views::types::{ContentContainer, ModalPosition};
use crate::gui::views::{Modal, View};
use crate::gui::Colors;
use crate::wallet::types::ConnectionMethod;
use crate::wallet::{ConnectionsConfig, ExternalConnection};

/// Wallet connection settings content.
pub struct ConnectionSettings {
    /// Selected connection method.
    pub method: ConnectionMethod,

    /// External connection [`Modal`] content.
    ext_conn_modal: ExternalConnectionModal,
}

impl Default for ConnectionSettings {
    fn default() -> Self {
        Self {
            method: ConnectionMethod::Integrated,
            ext_conn_modal: ExternalConnectionModal::new(None),
        }
    }
}

impl ContentContainer for ConnectionSettings {
    fn modal_ids(&self) -> Vec<&'static str> {
        vec![
            ExternalConnectionModal::WALLET_ID
        ]
    }

    fn modal_ui(&mut self,
                ui: &mut egui::Ui,
                modal: &Modal,
                cb: &dyn PlatformCallbacks) {
        match modal.id {
            ExternalConnectionModal::WALLET_ID => {
                self.ext_conn_modal.ui(ui, cb, modal, |_| {});
            },
            _ => {}
        }
    }

    fn container_ui(&mut self, ui: &mut egui::Ui, _: &dyn PlatformCallbacks) {
        ui.add_space(2.0);
        View::sub_title(ui, format!("{} {}", GLOBE, t!("wallets.conn_method")));
        View::horizontal_line(ui, Colors::stroke());
        ui.add_space(6.0);

        ui.vertical_centered(|ui| {
            ui.add_space(6.0);
            // Show integrated node selection.
            let cur_integrated = self.method == ConnectionMethod::Integrated;
            let bg = if cur_integrated {
                Colors::fill_deep()
            } else {
                Colors::fill_lite()
            };
            ConnectionsContent::integrated_node_item_ui(ui, bg, (!cur_integrated, || {
                self.method = ConnectionMethod::Integrated;
            }), |ui| {
                if cur_integrated {
                    View::selected_item_check(ui);
                }
                cur_integrated
            });

            ui.add_space(8.0);
            ui.label(RichText::new(t!("wallets.ext_conn")).size(16.0).color(Colors::gray()));
            ui.add_space(6.0);

            // Show button to add new external node connection.
            let add_node_text = format!("{} {}", PLUS_CIRCLE, t!("wallets.add_node"));
            View::button(ui, add_node_text, Colors::white_or_black(false), || {
                self.ext_conn_modal = ExternalConnectionModal::new(None);
                Modal::new(ExternalConnectionModal::WALLET_ID)
                    .position(ModalPosition::CenterTop)
                    .title(t!("wallets.add_node"))
                    .show();
            });
            ui.add_space(4.0);

            // Check for removed active connection.
            let cur_method = &self.method.clone();
            let mut ext_conn_list = ConnectionsConfig::ext_conn_list();
            let has_method = !ext_conn_list.iter().filter(|c| {
                match cur_method {
                    ConnectionMethod::Integrated => true,
                    ConnectionMethod::External(id, url) => id == &c.id || url == &c.url
                }
            }).collect::<Vec<&ExternalConnection>>().is_empty();
            if !has_method {
                match cur_method {
                    ConnectionMethod::External(id, url) => {
                        ext_conn_list.push(ExternalConnection {
                            id: *id,
                            url: url.clone(),
                            username: Some("grin".to_string()),
                            secret: None,
                            available: Some(true),
                        })
                    }
                    _ => {}
                }
            }

            let len = ext_conn_list.len();
            if len != 0 {
                ui.add_space(8.0);
                for (i, c) in ext_conn_list.iter().enumerate() {
                    ui.horizontal_wrapped(|ui| {
                        // Draw external connection item.
                        let is_current = match cur_method {
                            ConnectionMethod::External(id, url) => id == &c.id || url == &c.url,
                            _ => false
                        };
                        let bg = if is_current {
                            Colors::fill()
                        } else {
                            Colors::fill_lite()
                        };
                        ConnectionsContent::ext_conn_item_ui(ui, bg, c, i, len, (!is_current, || {
                            self.method = ConnectionMethod::External(c.id, c.url.clone());
                        }), |ui| {
                            if is_current {
                                View::selected_item_check(ui);
                            }
                        });
                    });
                }
            }
        });
    }
}