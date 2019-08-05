//  Copyright (C) 2018  The Dunitrust Project Developers.
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

use log::Level;
use simplelog::{Config, LevelFilter, SimpleLogger, TermLogger};

/// Initialize simple stdout logger
pub fn init_logger_stdout() {
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
    let logger_config = Config {
        time: Some(Level::Error),
        level: Some(Level::Error),
        target: Some(Level::Debug),
        location: Some(Level::Debug),
        time_format: Some("%Y-%m-%d %H:%M:%S%:z"),
    };

    // Active stdout logger
    if colors {
        TermLogger::init(level_filter, logger_config).expect("TESTS: fail to init stdout logger !");
    } else {
        SimpleLogger::init(level_filter, logger_config)
            .expect("TESTS: fail to init stdout logger !");
    }
}
