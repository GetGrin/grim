// Copyright 2024 The Grim Developers
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

use eframe::epaint::RectShape;
use egui::scroll_area::ScrollBarVisibility;
use egui::{Align, CursorIcon, Layout, RichText, ScrollArea, Sense, StrokeKind, UiBuilder};

use crate::gui::Colors;
use crate::gui::icons::{COMPUTER_TOWER, GLOBE_SIMPLE, PLUGS_CONNECTED};
use crate::gui::views::wallets::wallet::types::wallet_status_text;
use crate::gui::views::{Modal, View};
use crate::wallet::types::ConnectionMethod;
use crate::wallet::{Wallet, WalletList};

/// Wallet list [`Modal`] content
pub struct WalletListModal {
	/// Selected wallet API URL.
	selected_url: Option<String>,

	/// Optional data to pass after wallet selection.
	data: Option<String>,

	/// Flag to check if wallet can be opened from the list.
	can_open: bool,
}

impl WalletListModal {
	/// Create new content instance.
	pub fn new(wallet_url: Option<String>, data: Option<String>, can_open: bool) -> Self {
		Self {
			selected_url: wallet_url,
			data,
			can_open,
		}
	}

	/// Draw content.
	pub fn ui(
		&mut self,
		ui: &mut egui::Ui,
		wallets: &WalletList,
		mut on_select: impl FnMut(Wallet, Option<String>),
	) {
		ui.add_space(4.0);
		ScrollArea::vertical()
			.max_height(373.0)
			.id_salt("select_wallet_list_scroll")
			.scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
			.auto_shrink([true; 2])
			.show(ui, |ui| {
				ui.add_space(2.0);
				ui.vertical_centered(|ui| {
					let data = self.data.clone();
					for wallet in wallets.list() {
						// Draw wallet list item.
						let current = self.selected_url == Some(wallet.get_config().api_url());
						self.wallet_item_ui(ui, wallet, current, || {
							Modal::close();
							on_select(wallet.clone(), data.clone());
						});
						ui.add_space(5.0);
					}
				});
			});

		ui.add_space(2.0);
		View::horizontal_line(ui, Colors::item_stroke());
		ui.add_space(6.0);

		// Show button to close modal.
		ui.vertical_centered_justified(|ui| {
			View::button(
				ui,
				t!("modal.cancel"),
				Colors::white_or_black(false),
				|| {
					self.data = None;
					Modal::close();
				},
			);
		});
		ui.add_space(6.0);
	}

	/// Draw wallet list item with provided callback on select.
	fn wallet_item_ui(
		&mut self,
		ui: &mut egui::Ui,
		wallet: &Wallet,
		current: bool,
		on_select: impl FnOnce(),
	) {
		let config = wallet.get_config();

		// Draw round background.
		let mut rect = ui.available_rect_before_wrap();
		rect.set_height(78.0);
		let r = View::item_rounding(0, 1, false);
		let bg = if current {
			Colors::fill()
		} else {
			Colors::fill_deep()
		};
		let mut bg_shape = RectShape::new(rect, r, bg, View::item_stroke(), StrokeKind::Outside);
		let bg_idx = ui.painter().add(bg_shape.clone());

		let res = ui
			.scope_builder(
				UiBuilder::new()
					.sense(Sense::click())
					.layout(Layout::right_to_left(Align::Center))
					.max_rect(rect),
				|ui| {
					if !self.can_open && current {
						View::selected_item_check(ui);
					}

					let layout_size = ui.available_size();
					ui.allocate_ui_with_layout(
						layout_size,
						Layout::left_to_right(Align::Center),
						|ui| {
							ui.add_space(6.0);
							ui.vertical(|ui| {
								ui.add_space(3.0);
								// Show wallet name text.
								ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
									ui.add_space(1.0);
									View::ellipsize_text(
										ui,
										config.name.clone(),
										18.0,
										Colors::title(false),
									);
								});

								// Show wallet connection text.
								let connection = wallet.get_current_connection();
								let conn_text = match connection {
									ConnectionMethod::Integrated => {
										format!("{} {}", COMPUTER_TOWER, t!("network.node"))
									}
									ConnectionMethod::External(_, url) => {
										format!("{} {}", GLOBE_SIMPLE, url)
									}
								};
								ui.label(
									RichText::new(conn_text)
										.size(15.0)
										.color(Colors::text(false)),
								);
								ui.add_space(1.0);

								// Show wallet API text or open status.
								if self.can_open {
									ui.label(
										RichText::new(wallet_status_text(wallet))
											.size(15.0)
											.color(Colors::gray()),
									);
								} else {
									let api_text =
										format!("{} {}", PLUGS_CONNECTED, config.api_url());
									ui.label(
										RichText::new(api_text).size(15.0).color(Colors::gray()),
									);
								}
								if current {
									ui.add_space(3.0);
								} else {
									ui.add_space(5.0);
								}
							});
						},
					);
				},
			)
			.response;
		let clicked = res.clicked() || res.long_touched();
		// Setup background and cursor.
		if res.hovered() && !current {
			res.on_hover_cursor(CursorIcon::PointingHand);
			bg_shape.fill = Colors::fill();
		}
		ui.painter().set(bg_idx, bg_shape);
		// Handle clicks on layout.
		if clicked {
			on_select();
		}
	}
}
