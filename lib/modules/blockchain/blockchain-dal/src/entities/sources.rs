//  Copyright (C) 2018  The Duniter Project Developers.
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

use dubp_documents::documents::transaction::*;
use dubp_documents::BlockNumber;
use dup_crypto::hashs::Hash;
use dup_crypto::keys::PubKey;
use durs_common_tools::fatal_error;
use std::cmp::Ordering;
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

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
/// UTXOIndexV10
pub struct UTXOIndexV10(pub Hash, pub TxIndex);

/// UTXO content V10
pub type UTXOContentV10 = TransactionOutput;

#[derive(Debug, Clone, Deserialize, Serialize)]
/// V10 Unused Transaction Output
pub struct UTXOV10(pub UTXOIndexV10, pub UTXOContentV10);

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

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
/// Index of a V10 source
pub enum SourceIndexV10 {
    /// unused Transaction Output
    UTXO(UTXOIndexV10),
    /// universal Dividend
    UD(PubKey, BlockNumber),
}
