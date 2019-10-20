//  Copyright (C) 2017-2019  The AXIOM TEAM Association.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Common test tools for DURS project.

use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;

/// Initialize stdout logger
pub fn init_logger_stdout(off_targets: Vec<&'static str>) {
    let colors = match std::env::var("DURS_TESTS_LOG_COLOR")
        .unwrap_or_else(|_| String::from("no"))
        .as_str()
    {
        "yes" => true,
        "no" => false,
        v => panic!(
            "Unexpected value '{}' for env var DURS_TESTS_LOG_COLOR !",
            v
        ),
    };

    let level_filter = match std::env::var("DURS_TESTS_LOG_LEVEL")
        .unwrap_or_else(|_| String::from("debug"))
        .as_str()
    {
        "off" => LevelFilter::Off,
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        v => panic!(
            "Unexpected value '{}' for env var DURS_TESTS_LOG_LEVEL !",
            v
        ),
    };

    // Config logger
    let mut logger_config = fern::Dispatch::new()
        .level(level_filter)
        .format(move |out, message, record| {
            if colors {
                let colors_config = ColoredLevelConfig::new()
                    .info(Color::Green)
                    .debug(Color::Cyan);
                out.finish(format_args!(
                    "{}[{}][{}] {}",
                    chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                    record.target(),
                    colors_config.color(record.level()),
                    message
                ))
            } else {
                out.finish(format_args!(
                    "{}[{}][{}] {}",
                    chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                    record.target(),
                    record.level(),
                    message
                ))
            }
        })
        .chain(std::io::stdout());

    for target in off_targets {
        logger_config = logger_config.level_for(target, LevelFilter::Off);
    }

    // Active stdout logger
    logger_config
        .apply()
        .expect("TESTS: fail to init stdout logger !");
}
