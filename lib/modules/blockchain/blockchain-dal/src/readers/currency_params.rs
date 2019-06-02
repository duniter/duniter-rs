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

use crate::*;
use dubp_documents::documents::block::BlockDocument;
use dup_currency_params::db::write_currency_params;
use dup_currency_params::genesis_block_params::GenesisBlockParams;
use dup_currency_params::CurrencyParameters;
use unwrap::unwrap;

/// Get and write currency params
pub fn get_and_write_currency_params(
    db_path: &PathBuf,
    genesis_block: &BlockDocument,
) -> CurrencyParameters {
    if genesis_block.number.0 != 0 {
        fatal_error!("The genesis block must have number equal to zero !");
    } else if genesis_block.parameters.is_none() {
        fatal_error!("The genesis block must have parameters !");
    } else if let Err(e) = write_currency_params(
        db_path.clone(),
        genesis_block.currency.clone(),
        GenesisBlockParams::V10(unwrap!(genesis_block.parameters)),
    ) {
        fatal_error!("Fail to write currency parameters: {}", e);
    } else {
        CurrencyParameters::from((&genesis_block.currency, unwrap!(genesis_block.parameters)))
    }
}
