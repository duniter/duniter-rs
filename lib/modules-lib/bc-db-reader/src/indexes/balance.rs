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

//! Balances stored index.

use super::sources::*;
use crate::BalancesV10Datas;
use dubp_user_docs::documents::transaction::UTXOConditionsGroup;
use durs_dbs_tools::{BinFreeStructDb, DbError};

/// Get address balance
pub fn get_address_balance(
    balances_db: &BinFreeStructDb<BalancesV10Datas>,
    address: &UTXOConditionsGroup,
) -> Result<Option<SourceAmount>, DbError> {
    Ok(balances_db.read(|db| {
        if let Some(balance_and_utxos) = db.get(address) {
            Some(balance_and_utxos.0)
        } else {
            None
        }
    })?)
}
