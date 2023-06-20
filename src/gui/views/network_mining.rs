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

use std::fmt::format;
use std::net::IpAddr;
use std::str::FromStr;
use chrono::{DateTime, NaiveDateTime, Utc};
use egui::{RichText, Rounding, ScrollArea, Stroke};
use grin_chain::SyncStatus;
use grin_servers::WorkerStats;
use pnet::ipnetwork::IpNetwork;

use crate::gui::Colors;
use crate::gui::icons::{BARBELL, CLOCK_AFTERNOON, COMPUTER_TOWER, CPU, CUBE, FADERS, FOLDER_DASHED, FOLDER_NOTCH_MINUS, FOLDER_NOTCH_PLUS, PLUGS, PLUGS_CONNECTED, POLYGON, WRENCH};
use crate::gui::platform::PlatformCallbacks;
use crate::gui::views::{Network, NetworkTab, NetworkTabType, View};
use crate::node::Node;
use crate::Settings;

#[derive(Default)]
pub struct NetworkMining;

impl NetworkTab for NetworkMining {
    fn get_type(&self) -> NetworkTabType {
        NetworkTabType::Mining
    }

    fn ui(&mut self, ui: &mut egui::Ui, cb: &dyn PlatformCallbacks) {
        let server_stats = Node::get_stats();

        // Show message when node is not running or loading spinner when mining are not available.
        if !server_stats.is_some() || Node::get_sync_status().unwrap() != SyncStatus::NoSync {
            if !Node::is_running() {
                Network::disabled_server_content(ui);
            } else {
                View::center_content(ui, 162.0, |ui| {
                    View::big_loading_spinner(ui);
                    ui.add_space(18.0);
                    ui.label(RichText::new(t!("network_mining.loading"))
                        .size(16.0)
                        .color(Colors::INACTIVE_TEXT)
                    );
                });
            }
            return;
        }

        let stratum_stats = &server_stats.as_ref().unwrap().stratum_stats;

        // Stratum server address + port from config.
        let saved_stratum_addr = Settings::node_config_to_read()
            .members.clone()
            .server.stratum_mining_config.unwrap()
            .stratum_server_addr.unwrap();
        let (stratum_addr, stratum_port) = saved_stratum_addr.split_once(":").unwrap();

        // List of available ip addresses.
        let mut addresses = Vec::new();
        for net_if in pnet::datalink::interfaces() {
            for ip in net_if.ips {
                if ip.is_ipv4() {
                    addresses.push(ip.ip());
                }
            }
        }

        // Show error message when IP addresses are not available on the system.
        if addresses.is_empty() {
            View::center_content(ui, 52.0, |ui| {
                ui.label(RichText::new(t!("network_mining.no_ip_addresses"))
                    .size(16.0)
                    .color(Colors::INACTIVE_TEXT)
                );
            });
            return;
        }

        // Show stratum server setup when mining server is not enabled.
        if !stratum_stats.is_running && !Node::is_stratum_server_starting() {
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.add_space(6.0);
                    View::sub_title(ui,
                                    format!("{} {}", WRENCH, t!("network_mining.server_setup")));

                    ui.add_space(4.0);
                    View::horizontal_line(ui, Colors::ITEM_STROKE);
                    ui.add_space(6.0);

                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new(t!("network_mining.choose_ip_address"))
                            .size(16.0)
                            .color(Colors::GRAY)
                        );
                        ui.add_space(10.0);

                        if addresses.len() != 0 {
                            let saved_ip_addr = &IpAddr::from_str(stratum_addr).unwrap();
                            let mut selected_addr = saved_ip_addr;

                            // Set first IP address as current if saved is not present at system.
                            if !addresses.contains(selected_addr) {
                                selected_addr = addresses.get(0).unwrap();
                            }

                            // Show available IP addresses on the system.
                            let _ = addresses.chunks(2).map(|x| {
                                if x.len() == 2 {
                                    ui.columns(2, |columns| {
                                        let addr0 = x.get(0).unwrap();
                                        columns[0].vertical_centered(|ui| {
                                            View::radio_value(ui,
                                                              &mut selected_addr,
                                                              addr0,
                                                              addr0.to_string());
                                        });
                                        let addr1 = x.get(1).unwrap();
                                        columns[1].vertical_centered(|ui| {
                                            View::radio_value(ui,
                                                              &mut selected_addr,
                                                              addr1,
                                                              addr1.to_string());
                                        })
                                    });
                                    ui.add_space(12.0);
                                } else {
                                    let addr = x.get(0).unwrap();
                                    View::radio_value(ui,
                                                      &mut selected_addr,
                                                      addr,
                                                      addr.to_string());
                                    ui.add_space(4.0);
                                }
                            }).collect::<Vec<_>>();

                            // Save stratum server address at config if it was changed.
                            if saved_ip_addr != selected_addr {
                                let addr_to_save = format!("{}:{}", selected_addr, stratum_port);
                                let mut w_node_config = Settings::node_config_to_update();
                                w_node_config.members
                                    .server.stratum_mining_config.as_mut().unwrap()
                                    .stratum_server_addr = Some(addr_to_save);
                                w_node_config.save();
                            }
                        }

                        ui.label(RichText::new(t!("network_mining.change_port"))
                            .size(16.0)
                            .color(Colors::GRAY)
                        );

                        // Show button to choose server port.
                        ui.add_space(6.0);
                        View::button(ui, stratum_port.to_string(), Colors::WHITE, || {
                            //TODO: Open modal to change value
                            cb.show_keyboard();
                        });

                        ui.add_space(14.0);
                        View::horizontal_line(ui, Colors::ITEM_STROKE);
                        ui.add_space(6.0);

                        // Show message about stratum server config.
                        let text = t!(
                            "network_mining.server_setting",
                            "address" => saved_stratum_addr,
                            "settings" => FADERS
                        );
                        ui.label(RichText::new(text)
                            .size(16.0)
                            .color(Colors::INACTIVE_TEXT)
                        );
                        ui.add_space(8.0);

                        // Show button to enable server.
                        View::button(ui, t!("network_mining.enable_server"), Colors::GOLD, || {
                            Node::start_stratum_server();
                        });
                        ui.add_space(2.0);

                        let stratum_enabled = Settings::node_config_to_read()
                            .members.clone()
                            .server.stratum_mining_config.unwrap()
                            .enable_stratum_server.unwrap();

                        // Show stratum server autorun checkbox.
                        View::checkbox(ui, stratum_enabled, t!("network.autorun"), || {
                            let mut w_node_config = Settings::node_config_to_update();
                            w_node_config.members
                                .server.stratum_mining_config.as_mut().unwrap()
                                .enable_stratum_server = Some(!stratum_enabled);
                            w_node_config.save();
                        });
                    });
                    ui.add_space(6.0);
                });
            return;
        } else if Node::is_stratum_server_starting() {
            ui.centered_and_justified(|ui| {
                View::big_loading_spinner(ui);
            });
            return;
        }

        // Show stratum mining server info.
        View::sub_title(ui, format!("{} {}", COMPUTER_TOWER, t!("network_mining.server")));
        ui.columns(2, |columns| {
            columns[0].vertical_centered(|ui| {
                View::rounded_box(ui,
                                  saved_stratum_addr,
                                  t!("network_mining.ip_address"),
                                  [true, false, true, false]);
            });
            columns[1].vertical_centered(|ui| {
                //TODO: Stratum mining wallet listening address. Replace with local wallet name.
                let wallet_address = Settings::node_config_to_read()
                    .members.clone()
                    .server.stratum_mining_config.unwrap()
                    .wallet_listener_url
                    .replace("http://", "");
                View::rounded_box(ui,
                                  wallet_address,
                                  t!("network_mining.rewards_wallet"),
                                  [false, true, false, true]);
            });
        });
        ui.add_space(4.0);

        // Show network info.
        View::sub_title(ui, format!("{} {}", POLYGON, t!("network.self")));
        ui.columns(3, |columns| {
            columns[0].vertical_centered(|ui| {
                let difficulty = if stratum_stats.network_difficulty > 0 {
                    stratum_stats.network_difficulty.to_string()
                } else {
                    "-".into()
                };
                View::rounded_box(ui,
                                  difficulty,
                                  t!("network_node.difficulty"),
                                  [true, false, true, false]);
            });
            columns[1].vertical_centered(|ui| {
                let block_height = if stratum_stats.block_height > 0 {
                    stratum_stats.block_height.to_string()
                } else {
                    "-".into()
                };
                View::rounded_box(ui,
                                  block_height,
                                  t!("network_node.header"),
                                  [false, false, false, false]);
            });
            columns[2].vertical_centered(|ui| {
                let hashrate = if stratum_stats.network_hashrate > 0.0 {
                    format!("{:.*}", 2, stratum_stats.network_hashrate)
                } else {
                    "-".into()
                };
                View::rounded_box(ui,
                                  hashrate,
                                  t!("network_mining.hashrate", "bits" => stratum_stats.edge_bits),
                                  [false, true, false, true]);
            });
        });
        ui.add_space(4.0);

        // Show mining info.
        View::sub_title(ui, format!("{} {}", CPU, t!("network_mining.miners")));
        ui.columns(2, |columns| {
            columns[0].vertical_centered(|ui| {
                View::rounded_box(ui,
                                  stratum_stats.num_workers.to_string(),
                                  t!("network_mining.devices"),
                                  [true, false, true, false]);
            });

            columns[1].vertical_centered(|ui| {
                View::rounded_box(ui,
                                  stratum_stats.blocks_found.to_string(),
                                  t!("network_mining.blocks_found"),
                                  [false, true, false, true]);
            });
        });
        ui.add_space(4.0);

        // Show workers stats or info text when possible.
        let workers_size = stratum_stats.worker_stats.len();
        if workers_size != 0 && stratum_stats.num_workers > 0 {
            ui.add_space(4.0);
            View::horizontal_line(ui, Colors::ITEM_STROKE);
            ui.add_space(4.0);

            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .id_source("stratum_workers_scroll")
                .show_rows(
                    ui,
                    WORKER_UI_HEIGHT,
                    workers_size,
                    |ui, row_range| {
                        for index in row_range {
                            let worker = stratum_stats.worker_stats.get(index).unwrap();
                            let rounding = if workers_size == 1 {
                                [true, true]
                            } else if index == 0 {
                                [true, false]
                            } else if index == workers_size - 1 {
                                [false, true]
                            } else {
                                [false, false]
                            };
                            draw_worker_stats(ui, worker, rounding)
                        }
                    },
                );
        } else if ui.available_height() > 142.0 {
            View::center_content(ui, 142.0, |ui| {
                ui.label(RichText::new(t!("network_mining.info", "settings" => FADERS))
                    .size(16.0)
                    .color(Colors::INACTIVE_TEXT)
                );
            });
        }
    }
}

const WORKER_UI_HEIGHT: f32 = 77.0;

fn draw_worker_stats(ui: &mut egui::Ui, ws: &WorkerStats, rounding: [bool; 2]) {
    // Add space before the first item.
    if rounding[0] {
        ui.add_space(4.0);
    }

    ui.horizontal_wrapped(|ui| {
        ui.vertical_centered_justified(|ui| {
            let mut rect = ui.available_rect_before_wrap();
            rect.set_height(WORKER_UI_HEIGHT);
            ui.painter().rect(
                rect,
                Rounding {
                    nw: if rounding[0] { 8.0 } else { 0.0 },
                    ne: if rounding[0] { 8.0 } else { 0.0 },
                    sw: if rounding[1] { 8.0 } else { 0.0 },
                    se: if rounding[1] { 8.0 } else { 0.0 },
                },
                Colors::WHITE,
                Stroke { width: 1.0, color: Colors::ITEM_STROKE }
            );

            ui.add_space(2.0);
            ui.horizontal_top(|ui| {
                let (status_text, status_icon, status_color) = match ws.is_connected {
                    true => { (t!("network_mining.connected"), PLUGS_CONNECTED, Colors::BLACK) }
                    false => { (t!("network_mining.disconnected"), PLUGS, Colors::INACTIVE_TEXT) }
                };
                ui.add_space(5.0);
                ui.heading(RichText::new(status_icon)
                    .color(status_color)
                    .size(18.0));
                ui.add_space(2.0);

                // Draw worker ID.
                ui.heading(RichText::new(&ws.id)
                    .color(status_color)
                    .size(18.0));
                ui.add_space(3.0);

                // Draw worker status.
                ui.heading(RichText::new(status_text)
                    .color(status_color)
                    .size(18.0));
            });
            ui.horizontal_top(|ui| {
                ui.add_space(6.0);
                ui.heading(RichText::new(BARBELL)
                    .color(Colors::TITLE)
                    .size(16.0));
                ui.add_space(4.0);
                // Draw difficulty.
                ui.heading(RichText::new(ws.pow_difficulty.to_string())
                    .color(Colors::TITLE)
                    .size(16.0));
                ui.add_space(6.0);

                ui.heading(RichText::new(FOLDER_NOTCH_PLUS)
                    .color(Colors::GREEN)
                    .size(16.0));
                ui.add_space(3.0);
                // Draw accepted shares.
                ui.heading(RichText::new(ws.num_accepted.to_string())
                    .color(Colors::GREEN)
                    .size(16.0));
                ui.add_space(6.0);

                ui.heading(RichText::new(FOLDER_NOTCH_MINUS)
                    .color(Colors::RED)
                    .size(16.0));
                ui.add_space(3.0);
                // Draw rejected shares.
                ui.heading(RichText::new(ws.num_rejected.to_string())
                    .color(Colors::RED)
                    .size(16.0));
                ui.add_space(6.0);

                ui.heading(RichText::new(FOLDER_DASHED)
                    .color(Colors::GRAY)
                    .size(16.0));
                ui.add_space(3.0);
                // Draw stale shares.
                ui.heading(RichText::new(ws.num_stale.to_string())
                    .color(Colors::GRAY)
                    .size(16.0));
                ui.add_space(6.0);

                ui.heading(RichText::new(CUBE)
                    .color(Colors::TITLE)
                    .size(16.0));
                ui.add_space(3.0);
                // Draw blocks found.
                ui.heading(RichText::new(ws.num_blocks_found.to_string())
                    .color(Colors::TITLE)
                    .size(16.0));
            });
            ui.horizontal_top(|ui| {
                ui.add_space(6.0);
                ui.heading(RichText::new(CLOCK_AFTERNOON)
                    .color(Colors::TITLE)
                    .size(16.0));
                ui.add_space(4.0);

                // Draw block time
                let seen = ws.last_seen.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                let naive_datetime = NaiveDateTime::from_timestamp_opt(seen as i64, 0).unwrap();
                let datetime: DateTime<Utc> = DateTime::from_utc(naive_datetime, Utc);
                ui.heading(RichText::new(datetime.to_string())
                    .color(Colors::GRAY)
                    .size(16.0));

            });
        });
    });
}