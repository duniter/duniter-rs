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

//! Sources stored index.

use crate::constants::UTXOS;
use crate::*;
use dubp_common_doc::BlockNumber;
use dubp_indexes::sindex::UniqueIdUTXOv10;
use dubp_user_docs::documents::transaction::*;
use durs_common_tools::fatal_error;
use durs_dbs_tools::DbError;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::{Add, Sub};

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Serialize)]
/// Source amount
pub struct SourceAmount(pub TxAmount, pub TxBase);

impl Default for SourceAmount {
    fn default() -> SourceAmount {
        SourceAmount(TxAmount(0), TxBase(0))
    }
}

impl Ord for SourceAmount {
    fn cmp(&self, other: &SourceAmount) -> Ordering {
        if self.1 == other.1 {
            self.0.cmp(&other.0)
        } else {
            self.1.cmp(&other.1)
        }
    }
}

impl Add for SourceAmount {
    type Output = SourceAmount;
    fn add(self, s2: SourceAmount) -> Self::Output {
        let (mut s_min, s_max) = if self.1 > s2.1 {
            (s2, self)
        } else {
            (self, s2)
        };

        while s_min.1 < s_max.1 {
            (s_min.0).0 /= 10;
            (s_min.1).0 += 1;
        }

        SourceAmount(s_min.0 + s_max.0, s_max.1)
    }
}

impl Sub for SourceAmount {
    type Output = SourceAmount;
    fn sub(self, s2: SourceAmount) -> Self::Output {
        let (mut s_min, s_max) = if self.1 > s2.1 {
            (s2, self)
        } else {
            (self, s2)
        };

        while s_min.1 < s_max.1 {
            (s_min.0).0 /= 10;
            (s_min.1).0 += 1;
        }

        SourceAmount(s_min.0 - s_max.0, s_max.1)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// V10 Unused Transaction Output
pub struct UTXOV10(pub UniqueIdUTXOv10, pub TransactionOutputV10);

impl UTXOV10 {
    /// UTXO conditions
    pub fn get_conditions(&self) -> UTXOConditionsGroup {
        self.1.conditions.conditions.clone()
    }
    /// UTXO amount
    pub fn get_amount(&self) -> SourceAmount {
        SourceAmount(self.1.amount, self.1.base)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Unused Transaction Output
pub enum UTXO {
    /// V10
    V10(UTXOV10),
    /// V11
    V11(),
}

impl UTXO {
    /// UTXO conditions
    pub fn get_conditions(&self) -> UTXOConditionsGroup {
        match *self {
            UTXO::V10(ref utxo_v10) => utxo_v10.get_conditions(),
            _ => fatal_error!("UTXO version not supported !"),
        }
    }
    /// UTXO amount
    pub fn get_amount(&self) -> SourceAmount {
        match *self {
            UTXO::V10(ref utxo_v10) => utxo_v10.get_amount(),
            _ => fatal_error!("UTXO version not supported !"),
        }
    }
}

/// Get utxo v10
pub fn get_utxo_v10<DB: BcDbInReadTx>(
    db: &DB,
    utxo_id: UniqueIdUTXOv10,
) -> Result<Option<TransactionOutputV10>, DbError> {
    let utxo_id_bytes: Vec<u8> = utxo_id.into();
    db.db()
        .get_store(UTXOS)
        .get(db.r(), &utxo_id_bytes)?
        .map(from_db_value)
        .transpose()
}

/// Get block consumed sources
pub fn get_block_consumed_sources_<DB: BcDbInReadTx>(
    db: &DB,
    block_number: BlockNumber,
) -> Result<Option<HashMap<UniqueIdUTXOv10, TransactionOutputV10>>, DbError> {
    db.db()
        .get_int_store(CONSUMED_UTXOS)
        .get(db.r(), block_number.0)?
        .map(from_db_value)
        .transpose()
}
