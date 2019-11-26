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

//! Dunitrust core logger

use crate::commands::DursCoreOptions;
use failure::Fail;
use fern::colors::{Color, ColoredLevelConfig};
use log::{Level, SetLoggerError};
use std::fs::File;
use std::path::PathBuf;

#[derive(Debug, Fail)]
pub enum InitLoggerError {
    #[fail(display = "Fail to create log file: {}", _0)]
    FailCreateLogFile(std::io::Error),
    #[fail(display = "Fail to create term logger")]
    FailCreateTermLogger,
    #[fail(display = "Fail to open log file: {}", _0)]
    FailOpenLogFile(std::io::Error),
    #[fail(display = "Invalid log file path")]
    LogFilePathNotStr,
    #[fail(display = "Set logger error: {}", _0)]
    SetLoggerError(SetLoggerError),
}

impl From<SetLoggerError> for InitLoggerError {
    fn from(e: SetLoggerError) -> Self {
        InitLoggerError::SetLoggerError(e)
    }
}

/// Initialize logger
/// Warning: This function cannot use the macro fatal_error! because the logger is not yet initialized, so it must use panic !
pub fn init(
    profile_path: PathBuf,
    soft_name: &'static str,
    soft_version: &'static str,
    durs_core_opts: &DursCoreOptions,
) -> Result<(), InitLoggerError> {
    let mut log_file_path = profile_path;

    // Get log_file_path
    log_file_path.push(format!("{}.log", soft_name));

    // Get log_file_path_str
    let log_file_path_str = log_file_path
        .to_str()
        .ok_or(InitLoggerError::LogFilePathNotStr)?;

    // Create log file if not exist
    if !log_file_path.as_path().exists() {
        File::create(log_file_path_str).map_err(InitLoggerError::FailCreateLogFile)?;
    }

    // Get log level filter
    let logs_level_filter = durs_core_opts
        .logs_level
        .unwrap_or(Level::Info)
        .to_level_filter();

    // Config loggers
    let loggers_common_config = fern::Dispatch::new()
        .level(logs_level_filter)
        .level_for("ws", log::LevelFilter::Error)
        .format(|out, message, record| {
            let colors = ColoredLevelConfig::new()
                .info(Color::Green)
                .debug(Color::Cyan);
            let level = record.level();
            if level >= Level::Debug {
                out.finish(format_args!(
                    "{}[{}:{}][{}] {}",
                    chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                    record.file_static().unwrap_or("unknown source file"),
                    record.line().unwrap_or(0),
                    colors.color(level),
                    message
                ))
            } else {
                out.finish(format_args!(
                    "{}[{}][{}] {}",
                    chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                    record.target(),
                    colors.color(level),
                    message
                ))
            }
        });
    let file_config = fern::Dispatch::new()
        .chain(fern::log_file(log_file_path_str).map_err(InitLoggerError::FailOpenLogFile)?);
    let term_config = fern::Dispatch::new().chain(std::io::stdout());

    /*let logger_config = Config {
        time: Some(Level::Error),
        level: Some(Level::Error),
        target: Some(Level::Debug),
        location: Some(Level::Debug),
        time_format: Some("%Y-%m-%d %H:%M:%S%:z"),
    };*/

    if durs_core_opts.log_stdout {
        loggers_common_config
            .chain(file_config)
            .chain(term_config)
            .apply()?;
    /*CombinedLogger::init(vec![
        TermLogger::new(logs_level_filter, logger_config)
            .ok_or(InitLoggerError::FailCreateTermLogger)?,
        WriteLogger::new(logs_level_filter, logger_config, file_logger_opts),
    ])?;*/
    } else {
        loggers_common_config.chain(file_config).apply()?;
        //WriteLogger::init(logs_level_filter, logger_config, file_logger_opts)?;
    }

    info!(
        "Launching {}",
        crate::get_software_infos(soft_name, soft_version)
    );
    info!("Successfully init logger");
    Ok(())
}
