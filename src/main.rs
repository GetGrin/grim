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

#![windows_subsystem = "windows"]

pub fn main() {
    #[allow(dead_code)]
    #[cfg(not(target_os = "android"))]
    real_main();
}

#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn real_main() {
    #[cfg(debug_assertions)]
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    // Handle file path argument passing.
    let args: Vec<_> = std::env::args().collect();
    let mut data = None;
    if args.len() > 1 {
        let path = std::path::PathBuf::from(&args[1]);
        let content = match std::fs::read_to_string(path) {
            Ok(s) => Some(s),
            Err(_) => Some(args[1].clone())
        };
        data = content
    }

    // Setup callback on panic crash.
    std::panic::set_hook(Box::new(|info| {
        // Format error.
        let backtrace = backtrace::Backtrace::new();
        let time = grim::gui::views::View::format_time(chrono::Utc::now().timestamp());
        let os = egui::os::OperatingSystem::from_target_os();
        let ver = grim::VERSION;
        let msg = panic_info_message(info);
        let loc = if let Some(location) = info.location() {
            format!("{}:{}:{}", location.file(), location.line(), location.column())
        } else {
            "no location found.".parse().unwrap()
        };
        let err = format!("{} - {:?} - v{}\n{}\n{}\n\n{:?}", time, os, ver, msg, loc, backtrace);
        // Save backtrace to file.
        let log = grim::Settings::crash_report_path();
        if log.exists() {
            use std::io::{Seek, SeekFrom, Write};
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .append(true)
                .open(log)
                .unwrap();
            if file.seek(SeekFrom::End(0)).is_ok() {
                file.write(err.as_bytes()).unwrap_or_default();
            }
        } else {
            std::fs::write(log, err.as_bytes()).unwrap_or_default();
        }
        // Print message error.
        println!("{}\n{}", msg, loc);
    }));

    // Start GUI.
    let _ = std::panic::catch_unwind(|| {
        if is_app_running(&data) {
            return;
        } else if let Some(data) = data {
            grim::on_data(data);
        }
        let platform = grim::gui::platform::Desktop::new();
        start_app_socket(platform.clone());
        start_desktop_gui(platform);
    });
}

/// Get panic message from crash payload.
#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn panic_info_message<'pi>(panic_info: &'pi std::panic::PanicHookInfo<'_>) -> &'pi str {
    let payload = panic_info.payload();
    // taken from: https://github.com/rust-lang/rust/blob/4b9f4b221b92193c7e95b1beb502c6eb32c3b613/library/std/src/panicking.rs#L194-L200
    match payload.downcast_ref::<&'static str>() {
        Some(msg) => *msg,
        None => match payload.downcast_ref::<String>() {
            Some(msg) => msg.as_str(),
            // Copy what rustc does in the default panic handler
            None => "Box<dyn Any>",
        },
    }
}

/// Start GUI with Desktop related setup passing data from opening.
#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn start_desktop_gui(platform: grim::gui::platform::Desktop) {
    use grim::AppConfig;
    let os = egui::os::OperatingSystem::from_target_os();
    let (width, height) = AppConfig::window_size();
    let mut viewport = egui::ViewportBuilder::default()
        .with_min_inner_size([AppConfig::MIN_WIDTH, AppConfig::MIN_HEIGHT])
        .with_inner_size([width, height]);

    // Setup icon.
    if let Ok(icon) = eframe::icon_data::from_png_bytes(include_bytes!("../img/icon.png")) {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }
    // Setup window position.
    if let Some((x, y)) = AppConfig::window_pos() {
        viewport = viewport.with_position(egui::pos2(x, y));
    }
    // Setup window decorations.
    let is_mac = os == egui::os::OperatingSystem::Mac;
    let is_win = os == egui::os::OperatingSystem::Windows;
    viewport = viewport
        .with_fullsize_content_view(true)
        .with_window_level(egui::WindowLevel::Normal)
        .with_title_shown(is_win)
        .with_titlebar_buttons_shown(is_win)
        .with_titlebar_shown(is_win)
        .with_transparent(true)
        .with_decorations(is_mac || is_win);

    let renderer = if is_win {
        eframe::Renderer::Wgpu
    } else {
        eframe::Renderer::Glow
    };

    let mut options = eframe::NativeOptions {
        renderer,
        viewport,
        ..Default::default()
    };

    // Start GUI.
    let app = grim::gui::App::new(platform.clone());
    match grim::start(options.clone(), grim::app_creator(app)) {
        Ok(_) => {}
        Err(_) => {
            // Start with another renderer on error.
            if is_win {
                options.renderer = eframe::Renderer::Glow;
            } else {
                options.renderer = eframe::Renderer::Wgpu;
            }

            let app = grim::gui::App::new(platform);
            match grim::start(options, grim::app_creator(app)) {
                Ok(_) => {}
                Err(e) => {
                    panic!("{}", e);
                }
            }
        }
    }
}

/// Check if application is already running to pass data.
#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn is_app_running(data: &Option<String>) -> bool {
    let res: Result<(), Box<dyn std::error::Error>> = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            use interprocess::local_socket::{
                tokio::{prelude::*, Stream}
            };
            use tokio::{
                io::AsyncWriteExt,
            };

            let socket_path = grim::Settings::socket_path();
            let name = socket_name(&socket_path)?;

            // Connect to running application socket.
            let conn = Stream::connect(name).await?;
            let data = data.clone().unwrap_or("".to_string());
            if data.is_empty() {
                return Ok(());
            }
            let (rec, mut sen) = conn.split();

            // Send data to socket.
            let _ = sen.write_all(data.as_bytes()).await;

            drop((rec, sen));
            Ok(())
        });
    match res {
        Ok(_) => true,
        Err(_) => false
    }
}

/// Start desktop socket that handles data for single application instance.
#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn start_app_socket(platform: grim::gui::platform::Desktop) {
    std::thread::spawn(move || {
        let _ = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                use interprocess::local_socket::{
                    tokio::{prelude::*, Stream},
                    Listener, ListenerOptions,
                };
                use std::io;
                use tokio::{
                    io::{AsyncBufReadExt, BufReader},
                };
                use grim::gui::platform::PlatformCallbacks;

                // Handle incoming connection.
                async fn handle_conn(conn: Stream)
                                     -> io::Result<String> {
                    let mut read = BufReader::new(&conn);
                    let mut buffer = String::new();
                    // Read data.
                    let _ = read.read_line(&mut buffer).await;
                    Ok(buffer)
                }

                // Setup socket name.
                let socket_path = grim::Settings::socket_path();
                if socket_path.exists() {
                    let _ = std::fs::remove_file(&socket_path);
                }
                let name = socket_name(&socket_path)?;

                // Create listener.
                let opts = ListenerOptions::new().name(name);
                let listener = match opts.create_tokio() {
                    Err(e) if e.kind() == io::ErrorKind::AddrInUse => {
                        eprintln!("Socket file is occupied.");
                        return Err::<Listener, io::Error>(e);
                    }
                    x => x?,
                };

                loop {
                    let conn = match listener.accept().await {
                        Ok(c) => c,
                        Err(e) => {
                            println!("{:?}", e);
                            continue
                        }
                    };
                    // Handle connection.
                    let res = handle_conn(conn).await;
                    match res {
                        Ok(data) => {
                            grim::on_data(data);
                            platform.request_user_attention();
                        },
                        Err(_) => {}
                    }
                }
            });
    });
}

/// Get application socket name from provided path.
#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn socket_name(path: &std::path::PathBuf) -> std::io::Result<interprocess::local_socket::Name<'_>> {
    use interprocess::local_socket::{NameType, ToFsName, ToNsName};
    let name = if egui::os::OperatingSystem::Mac != egui::os::OperatingSystem::from_target_os() &&
        interprocess::local_socket::GenericNamespaced::is_supported() {
        grim::Settings::SOCKET_NAME.to_ns_name::<interprocess::local_socket::GenericNamespaced>()?
    } else {
        path.clone().to_fs_name::<interprocess::local_socket::GenericFilePath>()?
    };
    Ok(name)
}