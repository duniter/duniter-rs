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

//! Define Key-Value database

mod file;

pub use file::{
    KvFileDbHandler, KvFileDbRead, KvFileDbReader, KvFileDbRoHandler, KvFileDbSchema,
    KvFileDbStoreType, KvFileDbWriter,
};
pub use rkv::{
    IntegerStore, MultiIntegerStore, MultiStore, OwnedValue as KvFileDbOwnedValue, SingleStore,
    Value as KvFileDbValue,
};
