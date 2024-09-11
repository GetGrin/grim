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
            Err(_) => None
        };
        data = content
    }

    // Check if another app instance already running.
    if is_app_running(data.clone()) {
        return;
    }

    // Setup callback on panic crash.
    std::panic::set_hook(Box::new(|info| {
        let backtrace = backtrace::Backtrace::new();
        // Format error.
        let time = grim::gui::views::View::format_time(chrono::Utc::now().timestamp());
        let target = egui::os::OperatingSystem::from_target_os();
        let ver = grim::VERSION;
        let msg = panic_message::panic_info_message(info);
        let err = format!("{} - {:?} - v{}\n\n{}\n\n{:?}", time, target, ver, msg, backtrace);
        // Save backtrace to file.
        let log = grim::Settings::crash_report_path();
        if log.exists() {
            std::fs::remove_file(log.clone()).unwrap();
        }
        std::fs::write(log, err.as_bytes()).unwrap();
        // Setup flag to show crash after app restart.
        grim::AppConfig::set_show_crash(true);
    }));

    // Start GUI.
    match std::panic::catch_unwind(|| {
        start_desktop_gui(data);
    }) {
        Ok(_) => {}
        Err(e) => println!("{:?}", e)
    }
}

/// Check if application is already running to pass extra data.
#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn is_app_running(data: Option<String>) -> bool {
    use tor_rtcompat::BlockOn;
    let runtime = tor_rtcompat::tokio::TokioNativeTlsRuntime::create().unwrap();
    let res: Result<(), Box<dyn std::error::Error>> = runtime
        .block_on(async {
            use interprocess::local_socket::{
                tokio::{prelude::*, Stream},
                GenericFilePath
            };
            use tokio::{
                io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
                try_join,
            };

            let socket_path = grim::Settings::socket_path();
            let name = socket_path.to_fs_name::<GenericFilePath>()?;
            // Connect to running application socket.
            let conn = Stream::connect(name).await?;

            let (rec, mut sen) = conn.split();
            let mut rec = BufReader::new(rec);
            let data = data.unwrap_or("".to_string());
            let mut buffer = String::with_capacity(data.len());

            // Send extra data to socket.
            let send = sen.write_all(data.as_bytes());
            let recv = rec.read_line(&mut buffer);
            try_join!(send, recv)?;

            drop((rec, sen));
            Ok(())
        });
    return match res {
        Ok(_) => true,
        Err(_) => false
    }
}

/// Start GUI with Desktop related setup passing extra data from opening.
#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn start_desktop_gui(data: Option<String>) {
    use grim::AppConfig;
    use dark_light::Mode;

    // Setup system theme if not set.
    if let None = AppConfig::dark_theme() {
        let dark = match dark_light::detect() {
            Mode::Dark => true,
            Mode::Light => false,
            Mode::Default => false
        };
        AppConfig::set_dark_theme(dark);
    }

    let (width, height) = AppConfig::window_size();
    let mut viewport = egui::ViewportBuilder::default()
        .with_min_inner_size([AppConfig::MIN_WIDTH, AppConfig::MIN_HEIGHT])
        .with_inner_size([width, height]);

    // Setup an icon.
    if let Ok(icon) = eframe::icon_data::from_png_bytes(include_bytes!("../img/icon.png")) {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }
    // Setup window position.
    if let Some((x, y)) = AppConfig::window_pos() {
        viewport = viewport.with_position(egui::pos2(x, y));
    }
    // Setup window decorations.
    let is_mac = egui::os::OperatingSystem::from_target_os() == egui::os::OperatingSystem::Mac;
    viewport = viewport
        .with_window_level(egui::WindowLevel::Normal)
        .with_fullsize_content_view(true)
        .with_title_shown(false)
        .with_titlebar_buttons_shown(false)
        .with_titlebar_shown(false)
        .with_transparent(true)
        .with_decorations(is_mac);

    let mut options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    // Use Glow renderer for Windows.
    let win = egui::os::OperatingSystem::from_target_os() == egui::os::OperatingSystem::Windows;
    options.renderer = if win {
        eframe::Renderer::Glow
    } else {
        eframe::Renderer::Wgpu
    };

    let mut platform = grim::gui::platform::Desktop::new(data);

    // Start app socket at separate thread.
    let socket_pl = platform.clone();
    platform = socket_pl.clone();
    std::thread::spawn(move || {
        start_app_socket(socket_pl);
    });

    // Start GUI.
    let app = grim::gui::App::new(platform.clone());
    match grim::start(options.clone(), grim::app_creator(app)) {
        Ok(_) => {}
        Err(e) => {
            if win {
                panic!("{}", e);
            }
            // Start with another renderer on error.
            options.renderer = eframe::Renderer::Glow;

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

/// Start socket that handles data for single application instance.
#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn start_app_socket(platform: grim::gui::platform::Desktop) {
    use tor_rtcompat::BlockOn;
    let runtime = tor_rtcompat::tokio::TokioNativeTlsRuntime::create().unwrap();
    let _: Result<_, _> = runtime
        .block_on(async {
            use interprocess::local_socket::{
                tokio::{prelude::*, Stream},
                GenericFilePath, Listener, ListenerOptions,
            };
            use std::io;
            use tokio::{
                io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
                try_join,
            };

            // Handle incoming connection.
            async fn handle_conn(conn: Stream)
                -> io::Result<String> {
                let mut rec = BufReader::new(&conn);
                let mut sen = &conn;

                let mut buffer = String::new();
                let send = sen.write_all(b"");
                let recv = rec.read_line(&mut buffer);

                // Read data and send answer.
                try_join!(recv, send)?;

                Ok(buffer)
            }

            let socket_path = grim::Settings::socket_path();
            std::fs::remove_file(socket_path.clone()).unwrap();
            let name = socket_path.to_fs_name::<GenericFilePath>()?;
            let opts = ListenerOptions::new().name(name);

            // Create socket listener.
            let listener = match opts.create_tokio() {
                Err(e) if e.kind() == io::ErrorKind::AddrInUse => {
                    eprintln!("Socket file is occupied.");
                    return Err::<Listener, io::Error>(e);
                }
                x => x?,
            };

            // Handle connections.
            loop {
                let conn = match listener.accept().await {
                    Ok(c) => c,
                    Err(_) => continue
                };
                let res = handle_conn(conn).await;
                match res {
                    Ok(data) => {
                        platform.on_data(data)
                    },
                    Err(_) => {}
                }
            }
        });
}