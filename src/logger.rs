// Copyright 2026 The Grim Developers
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

use std::{panic, thread};
use std::fs::File;
use backtrace::Backtrace;
use log4rs::append::Append;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::Config;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::filter::threshold::ThresholdFilter;
use log::{error, LevelFilter};

use crate::Settings;

const LOGGING_PATTERN: &str = "{d(%Y%m%d %H:%M:%S%.3f)} {h({l})} {M} - {m}{n}";

/// 32 log files to rotate over by default.
const ROTATE_LOG_FILES: u32 = 32;
/// Size of the log in bytes to rotate over (6 megabytes).
const MAX_FILE_SIZE: u64 = 1024 * 1024 * 6;

/// Include build information.
pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

/// Initialize the logger.
pub fn init_logger() {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(&LOGGING_PATTERN)))
        .build();

    let mut root = Root::builder();

    let mut app = vec![];
    app.push(
        Appender::builder()
            .filter(Box::new(ThresholdFilter::new(LevelFilter::Info)))
            .build("stdout", Box::new(stdout)),
    );
    root = root.appender("stdout");

    // Setup file logging.
    let filter = Box::new(ThresholdFilter::new(LevelFilter::Info));
    let file: Box<dyn Append> = {
        let path = Settings::log_path();
        let roller = FixedWindowRoller::builder()
            .build(&format!("{}.{{}}.gz", path), ROTATE_LOG_FILES)
            .unwrap();
        let trigger = SizeTrigger::new(MAX_FILE_SIZE);
        let policy = CompoundPolicy::new(Box::new(trigger), Box::new(roller));
        Box::new(
            RollingFileAppender::builder()
                .append(true)
                .encoder(Box::new(PatternEncoder::new(&LOGGING_PATTERN)))
                .build(path, Box::new(policy))
                .expect("Failed to create logfile"),
        )
    };
    app.push(
        Appender::builder()
            .filter(filter)
            .build("file", file),
    );
    root = root.appender("file");

    let config = Config::builder()
        .appenders(app)
        .build(root.build(LevelFilter::Info))
        .unwrap();
    let _ = log4rs::init_config(config).unwrap();

    log::info!("{}", build_info());

    send_panic_to_log();
}

/// Get information about application build.
fn build_info() -> String {
    format!(
        "This is Grim version {}, built for {} by {}.",
        built_info::PKG_VERSION,
        built_info::TARGET,
        built_info::RUSTC_VERSION,
    )
}

/// Hook to send panics to logs as well as stderr.
fn send_panic_to_log() {
    panic::set_hook(Box::new(|info| {
        let backtrace = Backtrace::new();

        let thread = thread::current();
        let thread = thread.name().unwrap_or("unnamed");

        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &**s,
                None => "Box<Any>",
            },
        };

        match info.location() {
            Some(location) => {
                error!(
					"{}\nThread '{}' panicked at '{}': {}:{}{:?}\n\n",
                    build_info(),
					thread,
					msg,
					location.file(),
					location.line(),
					backtrace
				);
            }
            None => error!("Thread '{}' panicked at '{}'{:?}", thread, msg, backtrace),
        }
        // Also print to stderr.
        eprintln!(
            "Thread '{}' panicked with message:\n\"{}\"\nSee {} for further details.",
            thread, msg, Settings::log_path()
        );
        // Create file to show report send on launch.
        let log = Settings::crash_check_path();
        let _ = File::create(log);
    }));
}