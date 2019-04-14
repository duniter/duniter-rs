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

use crate::entities::currency_params::CurrencyParameters;
use crate::*;

/// Get currency parameters
pub fn get_currency_params(
    blockchain_db: &BinDB<LocalBlockchainV10Datas>,
) -> Result<Option<CurrencyParameters>, DALError> {
    Ok(blockchain_db.read(|db| {
        if let Some(genesis_block) = db.get(&BlockNumber(0)) {
            if genesis_block.block.parameters.is_some() {
                Some(CurrencyParameters::from((
                    genesis_block.block.currency.clone(),
                    genesis_block.block.parameters.expect("safe unwrap"),
                )))
            } else {
                panic!("The genesis block are None parameters !");
            }
        } else {
            None
        }
    })?)
}
