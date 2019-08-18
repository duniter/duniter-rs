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

use crate::DALError;

/// Storage type
pub enum StorageType {
    Single,
    SingleInteger,
    Multi,
    MultiInteger,
}

pub enum DB {
    File(Arc<RwLock<Rkv>>),
    Mem(),
}

impl DB {
    /// Open database
    pub fn open(db_path: Option<&PathBuf>) -> DB {
        let bc_backend = if let Some(db_path) = db_path {
            let mut manager = Manager::singleton()
                .write()
                .expect("fail to get rkb manager !");
            let db = manager
                .get_or_create(db_path.as_path(), Rkv::new)
                .expect("Fail to open LMDB blockchain database !");
            DB::File(db)
        } else {
            DB::Mem()
        };
    }
    /// Open integer storage (astorage is like a table or collection)
    pub fn open_integer_storage(&self, storage_name: &str) -> Result<IntegerStore<u32>, DalError> {
        let rkv = self.clone().read().expect("Fail to read lock Rkv");
        rkv.open_integer(storage_name, StoreOptions::create())?;
    }
}

/*pub trait MapStorage<K, V> {

    open(Option<&PathBuf>) -> Result<Self, DALError>;

    get(&self, key: &K) -> Result<Option<V>, DALError>;
    put(&self, key: K, value: V) -> Result<(), DALError>;
    delete(&self, key: &K) -> Result<(), DALError>;

    get_values(&self, keys: Vec<&K>) -> Result<Vec<(&K, Option<V>)>, DALError>;
    put_values(&self, datas: Vec<(K, V)>) -> Result<Vec<()>, DALError>;
    delete_values(&self, keys: Vec<&K>) -> Result<(), DALError>;

    fn save(&self) -> Result<(), DALError>;
}*/
