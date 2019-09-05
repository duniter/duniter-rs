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

//! Describe current meta datas

#[derive(Clone, Copy, Debug)]
/// Current meta data key
pub enum CurrentMetaDataKey {
    /// Version of the database structure
    DbVersion,
    /// Currency name
    CurrencyName,
    /// Current blockstamp
    CurrentBlockstamp,
    /// Current "blokchain" time
    CurrentBlockchainTime,
    /// Fork tree
    ForkTree,
}

impl CurrentMetaDataKey {
    /// To u32
    pub fn to_u32(self) -> u32 {
        match self {
            Self::DbVersion => 0,
            Self::CurrencyName => 1,
            Self::CurrentBlockstamp => 2,
            Self::CurrentBlockchainTime => 3,
            Self::ForkTree => 4,
        }
    }
}
