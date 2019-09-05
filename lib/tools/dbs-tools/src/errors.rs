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

//! Common Datas Access Layer for Dunitrust project
//! Errors manadgment

use rustbreak::error::{RustbreakError, RustbreakErrorKind};

#[derive(Debug)]
/// Data Access Layer Error
pub enum DbError {
    /// A database is corrupted, you have to reset the data completely
    DBCorrupted,
    ///  Database not exist
    DBNotExist,
    /// Error in read operation
    ReadError,
    /// Error with the file system
    FileSystemError(std::io::Error),
    /// Serialization/Deserialization error
    SerdeError(String),
    /// Rkv store error
    StoreError(rkv::error::StoreError),
    /// Unknown error
    UnknowError,
    /// Abort write transaction
    WriteAbort {
        /// Reason of transaction abort
        reason: String,
    },
    /// Error in write operation
    WriteError,
    /// Capturing a panic signal during a write operation
    WritePanic,
}

impl From<bincode::Error> for DbError {
    fn from(e: bincode::Error) -> DbError {
        DbError::SerdeError(format!("{}", e))
    }
}

impl From<rkv::error::StoreError> for DbError {
    fn from(e: rkv::error::StoreError) -> DbError {
        DbError::StoreError(e)
    }
}

impl<T> From<std::sync::PoisonError<T>> for DbError {
    fn from(_: std::sync::PoisonError<T>) -> DbError {
        DbError::DBCorrupted
    }
}

impl From<RustbreakError> for DbError {
    fn from(rust_break_error: RustbreakError) -> DbError {
        match rust_break_error.kind() {
            RustbreakErrorKind::Serialization => DbError::WriteError,
            RustbreakErrorKind::Deserialization => DbError::ReadError,
            RustbreakErrorKind::Poison => DbError::DBCorrupted,
            RustbreakErrorKind::Backend => DbError::DBCorrupted,
            RustbreakErrorKind::WritePanic => DbError::WritePanic,
            _ => DbError::UnknowError,
        }
    }
}
