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

use egui::{Margin, RichText};
use grin_core::core::amount_to_hr_string;

use crate::gui::Colors;
use crate::gui::icons::{DOWNLOAD, GEAR_FINE, UPLOAD};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Root, View};
use crate::gui::views::wallets::types::WalletTab;
use crate::gui::views::wallets::wallet::types::WalletTabType;
use crate::gui::views::wallets::wallet::WalletContent;
use crate::wallet::types::WalletData;
use crate::wallet::Wallet;

/// Wallet info tab content.
#[derive(Default)]
pub struct WalletInfo;

impl WalletTab for WalletInfo {
    fn get_type(&self) -> WalletTabType {
        WalletTabType::Info
    }

    fn ui(&mut self,
          ui: &mut egui::Ui,
          frame: &mut eframe::Frame,
          wallet: &mut Wallet,
          cb: &dyn PlatformCallbacks) {
        if WalletContent::sync_ui(ui, frame, wallet) {
            return;
        }

        let data = wallet.get_data().unwrap();

        // Show wallet transactions panel.
        egui::CentralPanel::default()
            .frame(egui::Frame {
                stroke: View::ITEM_STROKE,
                fill: Colors::BUTTON,
                inner_margin: Margin {
                    left: View::far_left_inset_margin(ui) + 4.0,
                    right: View::get_right_inset() + 4.0,
                    top: 0.0,
                    bottom: 4.0,
                },
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    View::max_width_ui(ui, Root::SIDE_PANEL_WIDTH * 1.3, |ui| {
                        self.txs_ui(ui, &data);
                    });
                });
                if data.txs.is_empty() {
                    View::center_content(ui, 96.0, |ui| {
                        let empty_text = t!(
                            "wallets.txs_empty",
                            "receive" => DOWNLOAD,
                            "send" => UPLOAD,
                            "settings" => GEAR_FINE
                        );
                        ui.label(RichText::new(empty_text).size(16.0).color(Colors::INACTIVE_TEXT));
                    });
                } else {

                }
            });
    }
}

impl WalletInfo {
    /// Draw transactions content.
    fn txs_ui(&self, ui: &mut egui::Ui, data: &WalletData) {
        // Show awaiting confirmation amount.
        let awaiting_conf = amount_to_hr_string(data.info.amount_awaiting_confirmation, false);
        View::rounded_box(ui,
                          format!("{} ツ", awaiting_conf),
                          t!("wallets.await_conf_amount"),
                          [false, false, false, false]);
        // Show awaiting finalization amount.
        let awaiting_conf = amount_to_hr_string(data.info.amount_awaiting_finalization, false);
        View::rounded_box(ui,
                          format!("{} ツ", awaiting_conf),
                          t!("wallets.await_fin_amount"),
                          [false, false, false, false]);
        // Show locked amount.
        let awaiting_conf = amount_to_hr_string(data.info.amount_locked, false);
        View::rounded_box(ui,
                          format!("{} ツ", awaiting_conf),
                          t!("wallets.locked_amount"),
                          [false, false, true, true]);
    }
}