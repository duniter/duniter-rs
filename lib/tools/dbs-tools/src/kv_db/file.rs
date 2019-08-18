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

use crate::errors::DALError;
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
pub type KvFileDbReader<'a> = &'a rkv::Reader<'a>;

/// Key-value database writer
pub struct KvFileDbWriter<'a> {
    buffer: Vec<u8>,
    writer: rkv::Writer<'a>,
}

impl<'a> AsMut<rkv::Writer<'a>> for KvFileDbWriter<'a> {
    fn as_mut(&mut self) -> &mut rkv::Writer<'a> {
        &mut self.writer
    }
}

/// Key-value file Database handler
pub struct KvFileDbHandler {
    arc: Arc<RwLock<Rkv>>,
    path: PathBuf,
    schema: KvFileDbSchema,
    stores: HashMap<String, KvFileDbStore>,
}

/// Key-value file Database read-only handler
pub struct KvFileDbRoHandler(KvFileDbHandler);

/// Key-value file Database read operations
pub trait KvFileDbRead: Sized {
    /// Convert DB value to a rust type
    fn from_db_value<T: DeserializeOwned>(v: Value) -> Result<T, DALError>;

    /// get a single store
    fn get_store(&self, store_name: &str) -> &super::SingleStore;

    /// Get an integer store
    fn get_int_store(&self, store_name: &str) -> &super::IntegerStore<u32>;

    /// Read datas in transaction database
    fn read<F, R>(&self, f: F) -> Result<R, DALError>
    where
        F: FnOnce(KvFileDbReader) -> Result<R, DALError>;

    /// Try to clone database handler
    fn try_clone(&self) -> Result<Self, DALError>;
}

impl KvFileDbRead for KvFileDbRoHandler {
    #[inline]
    fn from_db_value<T: DeserializeOwned>(v: Value) -> Result<T, DALError> {
        KvFileDbHandler::from_db_value(v)
    }
    #[inline]
    fn get_store(&self, store_name: &str) -> &super::SingleStore {
        self.0.get_store(store_name)
    }
    #[inline]
    fn get_int_store(&self, store_name: &str) -> &super::IntegerStore<u32> {
        self.0.get_int_store(store_name)
    }
    #[inline]
    fn read<F, R>(&self, f: F) -> Result<R, DALError>
    where
        F: FnOnce(KvFileDbReader) -> Result<R, DALError>,
    {
        self.0.read(f)
    }
    #[inline]
    fn try_clone(&self) -> Result<Self, DALError> {
        Ok(KvFileDbRoHandler(self.0.try_clone()?))
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

impl KvFileDbRead for KvFileDbHandler {
    #[inline]
    fn from_db_value<T: DeserializeOwned>(v: Value) -> Result<T, DALError> {
        if let Value::Blob(bytes) = v {
            Ok(bincode::deserialize::<T>(bytes)?)
        } else {
            Err(DALError::DBCorrupted)
        }
    }
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
    fn read<F, R>(&self, f: F) -> Result<R, DALError>
    where
        F: FnOnce(KvFileDbReader) -> Result<R, DALError>,
    {
        Ok(f(&self.arc_clone().read()?.read()?)?)
    }
    fn try_clone(&self) -> Result<KvFileDbHandler, DALError> {
        KvFileDbHandler::open_db_inner(&self.path, &self.schema, false)
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
    pub fn db_value(bytes: &[u8]) -> Result<Value, DALError> {
        Ok(Value::Blob(bytes))
    }
    /// Get read_only handler
    pub fn get_ro_handler(&self) -> Result<KvFileDbRoHandler, DALError> {
        Ok(KvFileDbRoHandler(self.try_clone()?))
    }
    /// Open Key-value file Database
    #[inline]
    pub fn open_db(path: &Path, schema: &KvFileDbSchema) -> Result<KvFileDbHandler, DALError> {
        KvFileDbHandler::open_db_inner(path, schema, true)
    }
    fn open_db_inner(
        path: &Path,
        schema: &KvFileDbSchema,
        first_open: bool,
    ) -> Result<KvFileDbHandler, DALError> {
        let mut manager = Manager::singleton().write()?;
        let mut env = Rkv::environment_builder();
        env.set_flags(EnvironmentFlags::NO_SYNC)
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
    pub fn save(&self) -> Result<(), DALError> {
        Ok(self.arc_clone().read()?.sync(true)?)
    }
    /// Write datas in database
    /// /!\ The written data are visible to readers but not persisted on the disk until a save() is performed.
    pub fn write<F>(&self, f: F) -> Result<(), DALError>
    where
        F: FnOnce(KvFileDbWriter) -> Result<KvFileDbWriter, DALError>,
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
