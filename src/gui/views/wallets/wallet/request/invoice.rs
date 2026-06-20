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

use crate::gui::Colors;
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{CameraContent, Modal, TextEdit, View};
use crate::wallet::Wallet;
use crate::wallet::types::WalletTask;
use egui::{Id, RichText};
use grin_core::core::{amount_from_hr_string, amount_to_hr_string};
use grin_wallet_libwallet::SlatepackAddress;

/// Invoice request creation content.
pub struct InvoiceRequestContent {
	/// Amount to receive.
	amount_edit: String,

	/// Sender address.
	address_edit: String,
	/// Flag to check if entered address is incorrect.
	address_error: bool,

	/// Address QR code scanner content.
	address_scan_content: Option<CameraContent>,
}

impl Default for InvoiceRequestContent {
	fn default() -> Self {
		Self {
			amount_edit: "".to_string(),
			address_edit: "".to_string(),
			address_error: false,
			address_scan_content: None,
		}
	}
}

impl InvoiceRequestContent {
	/// Draw [`Modal`] content.
	pub fn ui(
		&mut self,
		ui: &mut egui::Ui,
		wallet: &Wallet,
		modal: &Modal,
		cb: &dyn PlatformCallbacks,
	) {
		// Setup callback on continue.
		let on_continue = |m: &mut InvoiceRequestContent| {
			if m.amount_edit.is_empty() {
				return;
			}
			if let Ok(a) = amount_from_hr_string(m.amount_edit.as_str()) {
				let addr_str = m.address_edit.as_str();
				let addr = if let Ok(r) = SlatepackAddress::try_from(addr_str.trim()) {
					Some(r)
				} else {
					None
				};
				wallet.task(WalletTask::Receive(a, addr));
				Modal::close();
			}
		};

		ui.add_space(6.0);

		// Draw QR code scanner content if requested.
		if let Some(content) = self.address_scan_content.as_mut() {
			let mut close_scan = true;
			content.modal_ui(ui, cb, |result| {
				if let Some(result) = result {
					self.address_edit = result.text();
				} else {
					modal.enable_closing();
					close_scan = true;
				}
			});
			if close_scan {
				self.address_scan_content = None;
			}
			return;
		}

		// Draw amount input content.
		ui.vertical_centered(|ui| {
			ui.label(
				RichText::new(t!("wallets.enter_amount_receive"))
					.size(17.0)
					.color(Colors::gray()),
			);
		});
		ui.add_space(8.0);

		// Draw request amount text input.
		let amount_edit_before = self.amount_edit.clone();
		let mut amount_edit = TextEdit::new(Id::from(modal.id).with(wallet.get_config().id))
			.h_center()
			.numeric()
			.focus(Modal::first_draw());
		amount_edit.ui(ui, &mut self.amount_edit, cb);

		// Check value if input was changed.
		if amount_edit_before != self.amount_edit {
			if !self.amount_edit.is_empty() {
				self.amount_edit = self.amount_edit.trim().replace(",", ".");
				match amount_from_hr_string(self.amount_edit.as_str()) {
					Ok(amount) => {
						if !self.amount_edit.contains(".") {
							// To avoid input of several `0` before `.` and put `.` after first `0`.
							if self.amount_edit.len() != 1 && self.amount_edit.starts_with("0") {
								let amount_text = amount_to_hr_string(amount, true);
								let amount_parts = amount_text.split(".").collect::<Vec<&str>>();
								self.amount_edit = format!("0.{}", amount_parts[0]);
								amount_edit.cursor_to_end(self.amount_edit.len(), ui);
							}
						} else {
							// Check input after `.`.
							let parts = self.amount_edit.split(".").collect::<Vec<&str>>();
							if parts.len() == 2
								&& (parts[1].len() > 9 || (amount == 0 && parts[1].len() > 8))
							{
								self.amount_edit = amount_edit_before;
							}
						}
					}
					Err(_) => {
						self.amount_edit = amount_edit_before;
					}
				}
			}
		}

		ui.add_space(8.0);

		// Show address error or input description.
		ui.vertical_centered(|ui| {
			if self.address_error {
				ui.label(
					RichText::new(t!("transport.incorrect_addr_err"))
						.size(17.0)
						.color(Colors::red()),
				);
			} else {
				ui.label(
					RichText::new(t!("transport.sender_address"))
						.size(17.0)
						.color(Colors::gray()),
				);
			}
		});
		ui.add_space(6.0);

		// Show address text edit.
		let addr_edit_before = self.address_edit.clone();
		let address_edit_id = Id::from(modal.id)
			.with("_address")
			.with(wallet.get_config().id);
		let mut address_edit = TextEdit::new(address_edit_id)
			.paste()
			.focus(false)
			.scan_qr();
		if amount_edit.enter_pressed {
			address_edit.focus_request();
		}
		address_edit.ui(ui, &mut self.address_edit, cb);
		// Check if scan button was pressed.
		if address_edit.scan_pressed {
			modal.disable_closing();
			self.address_scan_content = Some(CameraContent::default());
		}

		ui.add_space(12.0);
		// Check value if input was changed.
		if addr_edit_before != self.address_edit {
			self.address_error = false;
		}
		// Continue on Enter press.
		if address_edit.enter_pressed {
			on_continue(self);
		}

		// Setup spacing between buttons.
		ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 0.0);

		ui.columns(2, |columns| {
			columns[0].vertical_centered_justified(|ui| {
				View::button(
					ui,
					t!("modal.cancel"),
					Colors::white_or_black(false),
					|| {
						Modal::close();
					},
				);
			});
			columns[1].vertical_centered_justified(|ui| {
				// Button to create Slatepack message request.
				View::button(ui, t!("continue"), Colors::white_or_black(false), || {
					on_continue(self);
				});
			});
		});
		ui.add_space(6.0);
	}
}
