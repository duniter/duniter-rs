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

//! Define free structure database

use crate::errors::DALError;
use rustbreak::backend::{FileBackend, MemoryBackend};
use rustbreak::error::RustbreakError;
use rustbreak::{deser::Bincode, Database, FileDatabase, MemoryDatabase};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::default::Default;
use std::fmt::Debug;
use std::fs;
use std::panic::UnwindSafe;
use std::path::PathBuf;

/// Open free structured rustbreak memory database
pub fn open_free_struct_memory_db<
    D: Serialize + DeserializeOwned + Debug + Default + Clone + Send,
>() -> Result<MemoryDatabase<D, Bincode>, DALError> {
    let backend = MemoryBackend::new();
    let db = MemoryDatabase::<D, Bincode>::from_parts(D::default(), backend, Bincode);
    Ok(db)
}

/// Open free structured rustbreak file database
pub fn open_free_struct_file_db<
    D: Serialize + DeserializeOwned + Debug + Default + Clone + Send,
>(
    dbs_folder_path: &PathBuf,
    db_file_name: &str,
) -> Result<FileDatabase<D, Bincode>, DALError> {
    let mut db_path = dbs_folder_path.clone();
    db_path.push(db_file_name);
    let file_path = db_path.as_path();
    if file_path.exists()
        && fs::metadata(file_path)
            .expect("fail to get file size")
            .len()
            > 0
    {
        let backend = FileBackend::open(db_path.as_path())?;
        let db = FileDatabase::<D, Bincode>::from_parts(D::default(), backend, Bincode);
        db.load()?;
        Ok(db)
    } else {
        Ok(FileDatabase::<D, Bincode>::from_path(
            db_path.as_path(),
            D::default(),
        )?)
    }
}

#[derive(Debug)]
/// Database
pub enum BinFreeStructDb<D: Serialize + DeserializeOwned + Debug + Default + Clone + Send> {
    /// File database
    File(Database<D, FileBackend, Bincode>),
    /// Memory database
    Mem(Database<D, MemoryBackend, Bincode>),
}

impl<D: Serialize + DeserializeOwned + Debug + Default + Clone + Send> BinFreeStructDb<D> {
    /// Flush the data structure to the backend
    pub fn save(&self) -> Result<(), RustbreakError> {
        match *self {
            BinFreeStructDb::File(ref file_db) => file_db.save(),
            BinFreeStructDb::Mem(ref mem_db) => mem_db.save(),
        }
    }
    /// Read lock the database and get write access to the Data container
    /// This gives you a read-only lock on the database. You can have as many readers in parallel as you wish.
    pub fn read<T, R>(&self, task: T) -> Result<R, RustbreakError>
    where
        T: FnOnce(&D) -> R,
    {
        match *self {
            BinFreeStructDb::File(ref file_db) => file_db.read(task),
            BinFreeStructDb::Mem(ref mem_db) => mem_db.read(task),
        }
    }
    /// Write lock the database and get write access to the Data container
    /// This gives you an exclusive lock on the memory object. Trying to open the database in writing will block if it is currently being written to.
    pub fn write<T>(&self, task: T) -> Result<(), RustbreakError>
    where
        T: FnOnce(&mut D),
    {
        match *self {
            BinFreeStructDb::File(ref file_db) => file_db.write(task),
            BinFreeStructDb::Mem(ref mem_db) => mem_db.write(task),
        }
    }
    /// Write lock the database and get write access to the Data container in a safe way (clone of the internal data is made).
    pub fn write_safe<T>(&self, task: T) -> Result<(), RustbreakError>
    where
        T: FnOnce(&mut D) + UnwindSafe,
    {
        match *self {
            BinFreeStructDb::File(ref file_db) => file_db.write_safe(task),
            BinFreeStructDb::Mem(ref mem_db) => mem_db.write_safe(task),
        }
    }
    /// Load the Data from the backend
    pub fn load(&self) -> Result<(), RustbreakError> {
        match *self {
            BinFreeStructDb::File(ref file_db) => file_db.load(),
            BinFreeStructDb::Mem(ref mem_db) => mem_db.load(),
        }
    }
}
