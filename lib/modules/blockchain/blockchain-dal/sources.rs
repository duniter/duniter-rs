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

use dubp_documents::v10::transaction::*;
use dubp_documents::BlockId;
use dup_crypto::hashs::Hash;
use dup_crypto::keys::PubKey;
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
        if self.1 == s2.1 {
            SourceAmount(self.0 + s2.0, self.1)
        } else {
            panic!("Source change base not yet supported !")
        }
    }
}

impl Sub for SourceAmount {
    type Output = SourceAmount;
    fn sub(self, s2: SourceAmount) -> Self::Output {
        if self.1 == s2.1 {
            SourceAmount(self.0 - s2.0, self.1)
        } else {
            panic!("Source change base not yet supported !")
        }
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
            _ => panic!("UTXO version not supported !"),
        }
    }
    /// UTXO amount
    pub fn get_amount(&self) -> SourceAmount {
        match *self {
            UTXO::V10(ref utxo_v10) => utxo_v10.get_amount(),
            _ => panic!("UTXO version not supported !"),
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
/// Index of a V10 source
pub enum SourceIndexV10 {
    /// unused Transaction Output
    UTXO(UTXOIndexV10),
    /// universal Dividend
    UD(PubKey, BlockId),
}
