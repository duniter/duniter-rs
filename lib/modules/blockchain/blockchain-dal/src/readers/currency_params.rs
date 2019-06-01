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

use crate::constants::CURRENCY_PARAMS_DB_NAME;
use crate::*;
use dubp_documents::documents::block::BlockDocument;
use dup_currency_params::CurrencyParameters;
use durs_conf::constants::DEFAULT_CURRENCY;
use unwrap::unwrap;

/// Get currency parameters
pub fn get_currency_params(db_path: &PathBuf) -> Result<Option<CurrencyParameters>, DALError> {
    let currency_params_db =
        open_file_db::<CurrencyParamsV10Datas>(db_path, CURRENCY_PARAMS_DB_NAME)
            .expect("Fail to open params db");
    Ok(currency_params_db.read(|db| {
        db.as_ref().map(|(currency_name, block_genesis_params)| {
            CurrencyParameters::from((currency_name.clone(), *block_genesis_params))
        })
    })?)
}

/// Get and write currency params
pub fn get_and_write_currency_params(
    db_path: &PathBuf,
    genesis_block: &BlockDocument,
) -> CurrencyParameters {
    if genesis_block.number.0 != 0 {
        fatal_error!("The genesis block must have number equal to zero !");
    } else if genesis_block.parameters.is_none() {
        fatal_error!("The genesis block must have parameters !");
    } else {
        let currency_params_db = BinDB::File(
            open_file_db::<CurrencyParamsV10Datas>(&db_path, CURRENCY_PARAMS_DB_NAME)
                .expect("Fail to open params db"),
        );
        if genesis_block.currency.0 != DEFAULT_CURRENCY {
            let mut default_currency_path = db_path.clone();
            default_currency_path.push(DEFAULT_CURRENCY);
            let _ = std::fs::remove_file(default_currency_path.as_path());
        }
        currency_params_db
            .write(|db| {
                db.replace((
                    genesis_block.currency.clone(),
                    unwrap!(genesis_block.parameters),
                ));
            })
            .expect("fail to write in params DB");
        currency_params_db.save().expect("Fail to save params db");
        CurrencyParameters::from((
            genesis_block.currency.clone(),
            unwrap!(genesis_block.parameters),
        ))
    }
}
