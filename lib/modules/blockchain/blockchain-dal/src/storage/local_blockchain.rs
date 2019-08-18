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

//! Local blockchain storage

use crate::entities::block::DALBlock;
use super::MapStorage;
use dubp_common_doc::BlockNumber;
use fnv::FnvHashMap;
use rkv::{IntegerStore, Manager, Rkv, StoreOptions, Value};

impl MapStorage<BlockNumber, DALBlock> {

    open(Option<&PathBuf>) -> Result<Self, DALError>;

    get(&self, key: &K) -> Result<Option<V>, DALError>;
    put(&self, key: K, value: V) -> Result<(), DALError>;
    delete(&self, key: &K) -> Result<(), DALError>;

    get_values(&self, keys: Vec<&K>) -> Result<Vec<(&K, Option<V>)>, DALError>;
    put_values(&self, datas: Vec<(K, V)>) -> Result<Vec<()>, DALError>;
    delete_values(&self, keys: Vec<&K>) -> Result<(), DALError>;

    fn save(&self) -> Result<(), DALError> {
        if let Some(file_backend) = self.open_file_backend() {
            file_backend.sync(true)?;
        }
        Ok(())
    }
}