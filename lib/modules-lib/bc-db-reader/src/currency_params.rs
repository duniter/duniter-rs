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

//! Currency parameters storage.

use dubp_block_doc::block::{BlockDocument, BlockDocumentTrait};
use dubp_common_doc::traits::Document;
use dubp_currency_params::db::write_currency_params;
use dubp_currency_params::genesis_block_params::GenesisBlockParams;
use dubp_currency_params::CurrencyParameters;
use durs_common_tools::fatal_error;
use log::error;
use std::path::PathBuf;
use unwrap::unwrap;

/// Get and write currency params
pub fn get_and_write_currency_params(
    db_path: &PathBuf,
    genesis_block: &BlockDocument,
) -> CurrencyParameters {
    if genesis_block.number().0 != 0 {
        fatal_error!("The genesis block must have number equal to zero !");
    }

    match genesis_block {
        BlockDocument::V10(genesis_block_v10) => {
            if genesis_block_v10.parameters.is_none() {
                fatal_error!("The genesis block must have parameters !");
            } else if let Err(e) = write_currency_params(
                db_path.clone(),
                genesis_block_v10.currency().into(),
                GenesisBlockParams::V10(unwrap!(genesis_block_v10.parameters)),
            ) {
                fatal_error!("Fail to write currency parameters: {}", e);
            } else {
                CurrencyParameters::from((
                    &genesis_block_v10.currency().into(),
                    unwrap!(genesis_block_v10.parameters),
                ))
            }
        }
    }
}
