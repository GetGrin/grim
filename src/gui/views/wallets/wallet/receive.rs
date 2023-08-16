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

use egui::Margin;

use crate::gui::Colors;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::View;
use crate::gui::views::wallets::wallet::types::{WalletTab, WalletTabType};
use crate::gui::views::wallets::wallet::WalletContent;
use crate::wallet::Wallet;

/// Receiving tab content.
#[derive(Default)]
pub struct WalletReceive;

impl WalletTab for WalletReceive {
    fn get_type(&self) -> WalletTabType {
        WalletTabType::Receive
    }

    fn ui(&mut self,
          ui: &mut egui::Ui,
          frame: &mut eframe::Frame,
          wallet: &mut Wallet,
          cb: &dyn PlatformCallbacks) {
        if WalletContent::sync_ui(ui, frame, wallet) {
            return;
        }

        // Show receiving content panel.
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
                self.receive_ui(ui, wallet);
            });
    }
}

impl WalletReceive {
    /// Draw receiving content.
    pub fn receive_ui(&self, ui: &mut egui::Ui, wallet: &mut Wallet) {

    }
}