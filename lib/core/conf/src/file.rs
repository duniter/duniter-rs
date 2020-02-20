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

//! Dunitrust configuration file

use crate::constants;
use crate::errors::DursConfFileError;
use crate::DuRsConf;
use durs_module::DursConfTrait;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[inline]
/// Return path to configuration file
pub fn get_conf_path(profile_path: &PathBuf) -> PathBuf {
    let mut conf_path = profile_path.clone();
    conf_path.push(constants::CONF_FILENAME);
    conf_path
}

/// Load configuration from file
pub fn load_conf_from_file(mut conf_file_path: PathBuf) -> Result<DuRsConf, DursConfFileError> {
    // Open conf file
    conf_file_path.push(constants::CONF_FILENAME);
    if conf_file_path.as_path().exists() {
        match File::open(conf_file_path.as_path()) {
            Ok(mut f) => {
                let mut contents = String::new();
                f.read_to_string(&mut contents)
                    .map_err(DursConfFileError::ReadError)?;
                // Parse conf file
                let conf: DuRsConf =
                    serde_json::from_str(&contents).map_err(DursConfFileError::ParseError)?;
                // Upgrade conf to latest version
                let (conf, upgraded) = conf.upgrade();
                // If conf is upgraded, rewrite conf file
                if upgraded {
                    write_conf_file(conf_file_path.as_path(), &conf)
                        .map_err(DursConfFileError::WriteError)?;
                }
                Ok(conf)
            }
            Err(e) => Err(DursConfFileError::ReadError(e)),
        }
    } else {
        // Create conf file with default conf
        let conf = DuRsConf::default();
        write_conf_file(conf_file_path.as_path(), &conf)
            .unwrap_or_else(|_| panic!(dbg!("Fatal error : fail to write default conf file!")));
        Ok(conf)
    }
}

/// Save configuration in profile folder
pub fn write_conf_file<DC: DursConfTrait>(
    conf_path: &Path,
    conf: &DC,
) -> Result<(), std::io::Error> {
    let mut f = File::create(conf_path)?;
    f.write_all(
        serde_json::to_string_pretty(conf)
            .expect("Fatal error : fail to write default conf file !")
            .as_bytes(),
    )?;
    f.sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[inline]
    fn save_old_conf(profile_path: PathBuf) -> std::io::Result<()> {
        let mut conf_path = profile_path.clone();
        conf_path.push(constants::CONF_FILENAME);
        let mut conf_sav_path = profile_path;
        conf_sav_path.push("conf-sav.json");
        std::fs::copy(conf_path.as_path(), conf_sav_path.as_path())?;
        Ok(())
    }

    fn restore_old_conf_and_save_upgraded_conf(profile_path: PathBuf) -> std::io::Result<()> {
        let mut conf_path = profile_path.clone();
        conf_path.push(constants::CONF_FILENAME);
        let mut conf_sav_path = profile_path.clone();
        conf_sav_path.push("conf-sav.json");
        let mut conf_upgraded_path = profile_path;
        conf_upgraded_path.push("conf-upgraded.json");
        std::fs::copy(conf_path.as_path(), &conf_upgraded_path.as_path())?;
        std::fs::copy(conf_sav_path.as_path(), &conf_path.as_path())?;
        std::fs::remove_file(conf_sav_path.as_path())?;
        Ok(())
    }

    #[test]
    fn load_conf_file_v1() -> Result<(), DursConfFileError> {
        let profile_path = PathBuf::from("./test/v1/");
        save_old_conf(profile_path.clone()).map_err(DursConfFileError::WriteError)?;
        let conf = load_conf_from_file(profile_path.clone())?;
        assert_eq!(
            conf.modules()
                .get("ws2p")
                .expect("Not found ws2p conf")
                .clone(),
            json!({
                "sync_endpoints": [
                {
                    "endpoint": "WS2P c1c39a0a i3.ifee.fr 80 /ws2p",
                    "pubkey": "D9D2zaJoWYWveii1JRYLVK3J4Z7ZH3QczoKrnQeiM6mx"
                },
                {
                    "endpoint": "WS2P 15af24db g1.ifee.fr 80 /ws2p",
                    "pubkey": "BoZP6aqtErHjiKLosLrQxBafi4ATciyDZQ6XRQkNefqG"
                },
                {
                    "endpoint": "WS2P b48824f0 g1.monnaielibreoccitanie.org 80 /ws2p",
                    "pubkey": "7v2J4badvfWQ6qwRdCwhhJfAsmKwoxRUNpJHiJHj7zef"
                }
                ]
            })
        );
        restore_old_conf_and_save_upgraded_conf(profile_path)
            .map_err(DursConfFileError::WriteError)?;

        Ok(())
    }

    #[test]
    fn load_conf_file_v2() -> Result<(), DursConfFileError> {
        let profile_path = PathBuf::from("./test/v2/");
        let conf = load_conf_from_file(profile_path)?;
        assert_eq!(
            conf.modules()
                .get("ws2p")
                .expect("Not found ws2p conf")
                .clone(),
            json!({
                "sync_endpoints": [
                {
                    "endpoint": "WS2P c1c39a0a i3.ifee.fr 80 /ws2p",
                    "pubkey": "D9D2zaJoWYWveii1JRYLVK3J4Z7ZH3QczoKrnQeiM6mx"
                },
                {
                    "endpoint": "WS2P 15af24db g1.ifee.fr 80 /ws2p",
                    "pubkey": "BoZP6aqtErHjiKLosLrQxBafi4ATciyDZQ6XRQkNefqG"
                },
                {
                    "endpoint": "WS2P b48824f0 g1.monnaielibreoccitanie.org 80 /ws2p",
                    "pubkey": "7v2J4badvfWQ6qwRdCwhhJfAsmKwoxRUNpJHiJHj7zef"
                }
                ]
            })
        );
        Ok(())
    }
}
