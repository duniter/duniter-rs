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

//! Define Key-Value file database

use crate::errors::DbError;
use durs_common_tools::fatal_error;
use log::error;
use rkv::{DatabaseFlags, EnvironmentFlags, Manager, OwnedValue, Rkv, StoreOptions, Value};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Key-value database reader
pub struct KvFileDbReader<'r>(&'r rkv::Reader<'r>);

impl<'r> rkv::Readable for KvFileDbReader<'r> {
    fn get<K: AsRef<[u8]>>(
        &self,
        db: rkv::Database,
        k: &K,
    ) -> Result<Option<Value>, rkv::StoreError> {
        self.0.get(db, k)
    }
    fn open_ro_cursor(&self, db: rkv::Database) -> Result<rkv::RoCursor, rkv::StoreError> {
        self.0.open_ro_cursor(db)
    }
}

#[cfg(feature = "mock")]
#[derive(Clone, Copy, Debug)]
/// Mock key-value database reader
pub struct MockKvFileDbReader;

#[cfg(feature = "mock")]
impl rkv::Readable for MockKvFileDbReader {
    fn get<K: AsRef<[u8]>>(
        &self,
        _: rkv::Database,
        _: &K,
    ) -> Result<Option<Value>, rkv::StoreError> {
        unreachable!()
    }
    fn open_ro_cursor(&self, _: rkv::Database) -> Result<rkv::RoCursor, rkv::StoreError> {
        unreachable!()
    }
}

/// Key-value database writer
pub struct KvFileDbWriter<'w> {
    buffer: Vec<u8>,
    writer: rkv::Writer<'w>,
}

impl<'w> rkv::Readable for KvFileDbWriter<'w> {
    fn get<K: AsRef<[u8]>>(
        &self,
        db: rkv::Database,
        k: &K,
    ) -> Result<Option<Value>, rkv::StoreError> {
        self.writer.get(db, k)
    }
    fn open_ro_cursor(&self, db: rkv::Database) -> Result<rkv::RoCursor, rkv::StoreError> {
        self.writer.open_ro_cursor(db)
    }
}

impl<'a> AsRef<rkv::Writer<'a>> for KvFileDbWriter<'a> {
    fn as_ref(&self) -> &rkv::Writer<'a> {
        &self.writer
    }
}

impl<'a> AsMut<rkv::Writer<'a>> for KvFileDbWriter<'a> {
    fn as_mut(&mut self) -> &mut rkv::Writer<'a> {
        &mut self.writer
    }
}

#[inline]
/// Convert DB value to a rust type
pub fn from_db_value<T: DeserializeOwned>(v: Value) -> Result<T, DbError> {
    if let Value::Blob(bytes) = v {
        Ok(bincode::deserialize::<T>(bytes)?)
    } else {
        Err(DbError::DBCorrupted)
    }
}

/// Key-value file Database handler
#[derive(Debug)]
pub struct KvFileDbHandler {
    arc: Arc<RwLock<Rkv>>,
    path: PathBuf,
    schema: KvFileDbSchema,
    stores: HashMap<String, KvFileDbStore>,
}

/// Key-value file Database read-only handler
#[derive(Debug)]
pub struct KvFileDbRoHandler(KvFileDbHandler);

impl KvFileDbRoHandler {
    /// Open Key-value file Database in read-only mode
    pub fn open_db_ro(path: &Path, schema: &KvFileDbSchema) -> Result<KvFileDbRoHandler, DbError> {
        let mut db_main_file = path.to_owned();
        db_main_file.push("data.mdb");
        if !db_main_file.as_path().is_file() {
            return Err(DbError::DBNotExist);
        }

        let mut manager = Manager::singleton().write()?;
        let mut env = Rkv::environment_builder();
        env.set_flags(EnvironmentFlags::READ_ONLY)
            .set_max_dbs(64)
            .set_map_size(std::u32::MAX as usize);
        let arc = manager.get_or_create(path, |path| Rkv::from_env(path, env))?;

        let mut stores = HashMap::new();
        for (store_name, store_type) in &schema.stores {
            let store = match store_type {
                KvFileDbStoreType::Single => {
                    KvFileDbStore::Single(arc.clone().read()?.open_single(
                        store_name.as_str(),
                        StoreOptions {
                            create: false,
                            flags: DatabaseFlags::empty(),
                        },
                    )?)
                }
                KvFileDbStoreType::SingleIntKey => {
                    KvFileDbStore::SingleIntKey(arc.clone().read()?.open_integer(
                        store_name.as_str(),
                        StoreOptions {
                            create: false,
                            flags: DatabaseFlags::INTEGER_KEY,
                        },
                    )?)
                }
                KvFileDbStoreType::Multi => KvFileDbStore::Multi(arc.clone().read()?.open_multi(
                    store_name.as_str(),
                    StoreOptions {
                        create: false,
                        flags: DatabaseFlags::empty(),
                    },
                )?),
                KvFileDbStoreType::MultiIntKey => {
                    KvFileDbStore::MultiIntKey(arc.clone().read()?.open_multi_integer(
                        store_name.as_str(),
                        StoreOptions {
                            create: false,
                            flags: DatabaseFlags::INTEGER_KEY,
                        },
                    )?)
                }
            };
            stores.insert(store_name.to_owned(), store);
        }

        Ok(KvFileDbRoHandler(KvFileDbHandler {
            arc,
            path: path.to_owned(),
            schema: schema.clone(),
            stores,
        }))
    }
}

/// Key-value file Database read operations
pub trait KvFileDbRead: Sized {
    /// get a single store
    fn get_store(&self, store_name: &str) -> &super::SingleStore;

    /// Get an integer store
    fn get_int_store(&self, store_name: &str) -> &super::IntegerStore<u32>;

    /// get a multi store
    fn get_multi_store(&self, store_name: &str) -> &super::MultiStore;

    /// get a multi integer store
    fn get_multi_int_store(&self, store_name: &str) -> &super::MultiIntegerStore<u32>;

    /// Read datas in transaction database
    fn read<F, R>(&self, f: F) -> Result<R, DbError>
    where
        F: Fn(KvFileDbReader) -> Result<R, DbError>;
}

impl KvFileDbRead for KvFileDbRoHandler {
    #[inline]
    fn get_store(&self, store_name: &str) -> &super::SingleStore {
        self.0.get_store(store_name)
    }
    #[inline]
    fn get_int_store(&self, store_name: &str) -> &super::IntegerStore<u32> {
        self.0.get_int_store(store_name)
    }
    #[inline]
    fn get_multi_store(&self, store_name: &str) -> &super::MultiStore {
        self.0.get_multi_store(store_name)
    }
    #[inline]
    fn get_multi_int_store(&self, store_name: &str) -> &super::MultiIntegerStore<u32> {
        self.0.get_multi_int_store(store_name)
    }
    #[inline]
    fn read<F, R>(&self, f: F) -> Result<R, DbError>
    where
        F: Fn(KvFileDbReader) -> Result<R, DbError>,
    {
        self.0.read(f)
    }
}

/// Describe Key-Value database schema
#[derive(Debug, Clone)]
pub struct KvFileDbSchema {
    /// Database collections
    pub stores: HashMap<String, KvFileDbStoreType>,
}

/// Key-value store type (store is like "table" in SGBD)
#[derive(Debug, Clone, Copy)]
pub enum KvFileDbStoreType {
    /// Single valued map
    Single,
    /// Single valued map with integer key
    SingleIntKey,
    /// Multi valued map
    Multi,
    /// Multi valued map with integer key
    MultiIntKey,
}

/// Key-value file DB store (store is like "table" in SGBD)
pub enum KvFileDbStore {
    /// Single valued map
    Single(super::SingleStore),
    /// Single valued map with integer key
    SingleIntKey(super::IntegerStore<u32>),
    /// Multi valued map
    Multi(super::MultiStore),
    /// Multi valued map with integer key
    MultiIntKey(super::MultiIntegerStore<u32>),
}

impl Debug for KvFileDbStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single(_) => write!(f, "KvFileDbStore::Single ()"),
            Self::SingleIntKey(_) => write!(f, "KvFileDbStore::SingleIntKey ()"),
            Self::Multi(_) => write!(f, "KvFileDbStore::Multi ()"),
            Self::MultiIntKey(_) => write!(f, "KvFileDbStore::MultiIntKey ()"),
        }
    }
}

impl KvFileDbRead for KvFileDbHandler {
    fn get_int_store(&self, store_name: &str) -> &super::IntegerStore<u32> {
        if let Some(store_enum) = self.stores.get(store_name) {
            if let KvFileDbStore::SingleIntKey(store) = store_enum {
                store
            } else {
                fatal_error!("Dev error: store '{}' is not an integer store.", store_name);
            }
        } else {
            fatal_error!("Dev error: store '{}' don't exist in DB.", store_name);
        }
    }
    fn get_store(&self, store_name: &str) -> &super::SingleStore {
        if let Some(store_enum) = self.stores.get(store_name) {
            if let KvFileDbStore::Single(store) = store_enum {
                store
            } else {
                fatal_error!("Dev error: store '{}' is not a single store.", store_name);
            }
        } else {
            fatal_error!("Dev error: store '{}' don't exist in DB.", store_name);
        }
    }
    fn get_multi_store(&self, store_name: &str) -> &super::MultiStore {
        if let Some(store_enum) = self.stores.get(store_name) {
            if let KvFileDbStore::Multi(store) = store_enum {
                store
            } else {
                fatal_error!("Dev error: store '{}' is not a multi store.", store_name);
            }
        } else {
            fatal_error!("Dev error: store '{}' don't exist in DB.", store_name);
        }
    }
    fn get_multi_int_store(&self, store_name: &str) -> &super::MultiIntegerStore<u32> {
        if let Some(store_enum) = self.stores.get(store_name) {
            if let KvFileDbStore::MultiIntKey(store) = store_enum {
                store
            } else {
                fatal_error!(
                    "Dev error: store '{}' is not a multi integer store.",
                    store_name
                );
            }
        } else {
            fatal_error!("Dev error: store '{}' don't exist in DB.", store_name);
        }
    }
    fn read<F, R>(&self, f: F) -> Result<R, DbError>
    where
        F: Fn(KvFileDbReader) -> Result<R, DbError>,
    {
        Ok(f(KvFileDbReader(&self.arc_clone().read()?.read()?))?)
    }
}

impl KvFileDbHandler {
    fn arc(&self) -> &Arc<RwLock<Rkv>> {
        &self.arc
    }
    fn arc_clone(&self) -> Arc<RwLock<Rkv>> {
        self.arc().clone()
    }
    /// Convert bytes to DB value
    pub fn db_value(bytes: &[u8]) -> Result<Value, DbError> {
        Ok(Value::Blob(bytes))
    }
    /// Open Key-value file Database
    #[inline]
    pub fn open_db(path: &Path, schema: &KvFileDbSchema) -> Result<KvFileDbHandler, DbError> {
        KvFileDbHandler::open_db_inner(path, schema, true)
    }
    fn open_db_inner(
        path: &Path,
        schema: &KvFileDbSchema,
        first_open: bool,
    ) -> Result<KvFileDbHandler, DbError> {
        let mut env_flags = EnvironmentFlags::NO_MEM_INIT;
        env_flags.insert(EnvironmentFlags::NO_SYNC);
        let mut manager = Manager::singleton().write()?;
        let mut env = Rkv::environment_builder();
        env.set_flags(env_flags)
            .set_max_dbs(64)
            .set_map_size(std::u32::MAX as usize);
        let arc = manager.get_or_create(path, |path| Rkv::from_env(path, env))?;

        let mut stores = HashMap::new();
        for (store_name, store_type) in &schema.stores {
            let store = match store_type {
                KvFileDbStoreType::Single => {
                    KvFileDbStore::Single(arc.clone().read()?.open_single(
                        store_name.as_str(),
                        StoreOptions {
                            create: first_open,
                            flags: DatabaseFlags::empty(),
                        },
                    )?)
                }
                KvFileDbStoreType::SingleIntKey => {
                    KvFileDbStore::SingleIntKey(arc.clone().read()?.open_integer(
                        store_name.as_str(),
                        StoreOptions {
                            create: first_open,
                            flags: DatabaseFlags::INTEGER_KEY,
                        },
                    )?)
                }
                KvFileDbStoreType::Multi => KvFileDbStore::Multi(arc.clone().read()?.open_multi(
                    store_name.as_str(),
                    StoreOptions {
                        create: first_open,
                        flags: DatabaseFlags::empty(),
                    },
                )?),
                KvFileDbStoreType::MultiIntKey => {
                    KvFileDbStore::MultiIntKey(arc.clone().read()?.open_multi_integer(
                        store_name.as_str(),
                        StoreOptions {
                            create: first_open,
                            flags: DatabaseFlags::INTEGER_KEY,
                        },
                    )?)
                }
            };
            stores.insert(store_name.to_owned(), store);
        }

        Ok(KvFileDbHandler {
            arc,
            path: path.to_owned(),
            schema: schema.clone(),
            stores,
        })
    }
    /// Persist DB datas on disk
    pub fn save(&self) -> Result<(), DbError> {
        Ok(self.arc_clone().read()?.sync(true)?)
    }
    /// Write datas in database
    /// /!\ The written data are visible to readers but not persisted on the disk until a save() is performed.
    pub fn write<F>(&self, f: F) -> Result<(), DbError>
    where
        F: FnOnce(KvFileDbWriter) -> Result<KvFileDbWriter, DbError>,
    {
        f(KvFileDbWriter {
            buffer: Vec::with_capacity(0),
            writer: self.arc().read()?.write()?,
        })?
        .writer
        .commit()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use tempfile::tempdir;

    fn get_int_store_str_val(
        ro_db: &KvFileDbRoHandler,
        store_name: &str,
        key: u32,
    ) -> Result<Option<String>, DbError> {
        ro_db.read(|r| {
            if let Some(Value::Str(v)) = ro_db.get_int_store(store_name).get(&r, key)? {
                Ok(Some(v.to_owned()))
            } else {
                Ok(None)
            }
        })
    }

    #[test]
    fn test_open_db_wr_and_ro() -> Result<(), DbError> {
        let tmp_dir = tempdir().map_err(DbError::FileSystemError)?;
        let mut stores = HashMap::new();
        stores.insert("test1".to_owned(), KvFileDbStoreType::SingleIntKey);
        let schema = KvFileDbSchema { stores };
        let db = KvFileDbHandler::open_db(tmp_dir.path(), &schema)?;
        let store_test1 = db.get_int_store("test1");

        db.write(|mut w| {
            store_test1.put(w.as_mut(), 3, &Value::Str("toto"))?;
            Ok(w)
        })?;

        let ro_db = KvFileDbRoHandler::open_db_ro(tmp_dir.path(), &schema)?;

        assert_eq!(
            Some("toto".to_owned()),
            get_int_store_str_val(&ro_db, "test1", 3)?
        );

        db.write(|mut w| {
            store_test1.put(w.as_mut(), 3, &Value::Str("titi"))?;
            Ok(w)
        })?;

        assert_eq!(
            Some("titi".to_owned()),
            get_int_store_str_val(&ro_db, "test1", 3)?
        );

        db.write(|mut w| {
            store_test1.put(w.as_mut(), 3, &Value::Str("tutu"))?;
            assert_eq!(
                Some("titi".to_owned()),
                get_int_store_str_val(&ro_db, "test1", 3)?
            );
            Ok(w)
        })?;

        let db_path = tmp_dir.path().to_owned();
        let thread = std::thread::spawn(move || {
            let ro_db =
                KvFileDbRoHandler::open_db_ro(db_path.as_path(), &schema).expect("Fail to open DB");
            assert_eq!(
                Some("tutu".to_owned()),
                get_int_store_str_val(&ro_db, "test1", 3).expect("Fail to read DB")
            );
        });

        assert_eq!(
            Some("tutu".to_owned()),
            get_int_store_str_val(&ro_db, "test1", 3).expect("Fail to read DB")
        );

        let _ = thread.join();

        Ok(())
    }
}
