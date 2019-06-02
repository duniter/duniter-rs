//  Copyright (C) 2018  The Durs Project Developers.
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

//! Duniter protocol currency parameters DB

use crate::constants::CURRENCY_PARAMS_DB_NAME;
use crate::genesis_block_params::GenesisBlockParams;
use crate::{CurrencyName, CurrencyParameters};
use durs_common_tools::fns::bin_file::{read_bin_file, write_bin_file};
use failure::Fail;
use std::path::PathBuf;

/// Currency parameters db datas
type CurrencyParamsDbDatas = Option<(CurrencyName, GenesisBlockParams)>;

/// Currency params Db error
#[derive(Debug, Fail)]
pub enum CurrencyParamsDbError {
    /// Serialize/Deserialize error
    #[fail(display = "SerDe error: {}", _0)]
    SerDe(bincode::Error),
    /// I/O Error
    #[fail(display = "I/O error: {}", _0)]
    Io(std::io::Error),
}

/// Get currency name
pub fn get_currency_name(
    datas_path: PathBuf,
) -> Result<Option<CurrencyName>, CurrencyParamsDbError> {
    let db_datas: CurrencyParamsDbDatas = read_currency_params_db(datas_path)?;

    if let Some((currency_name, _genesis_block_params)) = db_datas {
        Ok(Some(currency_name))
    } else {
        Ok(None)
    }
}

/// Get currency parameters
pub fn get_currency_params(
    datas_path: PathBuf,
) -> Result<Option<(CurrencyName, CurrencyParameters)>, CurrencyParamsDbError> {
    let db_datas: CurrencyParamsDbDatas = read_currency_params_db(datas_path)?;

    if let Some((currency_name, genesis_block_params)) = db_datas {
        let currency_params = match genesis_block_params {
            GenesisBlockParams::V10(genesis_block_v10_params) => {
                CurrencyParameters::from((&currency_name, genesis_block_v10_params))
            }
        };
        Ok(Some((currency_name, currency_params)))
    } else {
        Ok(None)
    }
}

fn read_currency_params_db(
    mut datas_path: PathBuf,
) -> Result<CurrencyParamsDbDatas, CurrencyParamsDbError> {
    datas_path.push(CURRENCY_PARAMS_DB_NAME);

    if !datas_path.exists() {
        return Ok(None);
    }

    let bin_vec = read_bin_file(datas_path.as_path()).map_err(CurrencyParamsDbError::Io)?;
    let db_datas: CurrencyParamsDbDatas =
        bincode::deserialize(&bin_vec).map_err(CurrencyParamsDbError::SerDe)?;

    Ok(db_datas)
}

/// Write currency parameters
pub fn write_currency_params(
    mut datas_path: PathBuf,
    currency_name: CurrencyName,
    genesis_block_params: GenesisBlockParams,
) -> Result<(), CurrencyParamsDbError> {
    datas_path.push(CURRENCY_PARAMS_DB_NAME);

    let db_datas: CurrencyParamsDbDatas = Some((currency_name, genesis_block_params));

    Ok(write_bin_file(
        datas_path.as_path(),
        &bincode::serialize(&db_datas).map_err(CurrencyParamsDbError::SerDe)?,
    )
    .map_err(CurrencyParamsDbError::Io)?)
}
