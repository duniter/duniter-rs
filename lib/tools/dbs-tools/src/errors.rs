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
pub enum DALError {
    /// Abort write transaction
    WriteAbort {
        /// Reason of transaction abort
        reason: String,
    },
    /// Error in write operation
    WriteError,
    /// Error in read operation
    ReadError,
    /// A database is corrupted, you have to reset the data completely
    DBCorrupted,
    /// Error with the file system
    FileSystemError(std::io::Error),
    /// Serialization/Deserialization error
    SerdeError(String),
    /// Rkv store error
    StoreError(rkv::error::StoreError),
    /// Capturing a panic signal during a write operation
    WritePanic,
    /// Unknown error
    UnknowError,
}

impl From<bincode::Error> for DALError {
    fn from(e: bincode::Error) -> DALError {
        DALError::SerdeError(format!("{}", e))
    }
}

impl From<rkv::error::StoreError> for DALError {
    fn from(e: rkv::error::StoreError) -> DALError {
        DALError::StoreError(e)
    }
}

impl<T> From<std::sync::PoisonError<T>> for DALError {
    fn from(_: std::sync::PoisonError<T>) -> DALError {
        DALError::DBCorrupted
    }
}

impl From<RustbreakError> for DALError {
    fn from(rust_break_error: RustbreakError) -> DALError {
        match rust_break_error.kind() {
            RustbreakErrorKind::Serialization => DALError::WriteError,
            RustbreakErrorKind::Deserialization => DALError::ReadError,
            RustbreakErrorKind::Poison => DALError::DBCorrupted,
            RustbreakErrorKind::Backend => DALError::DBCorrupted,
            RustbreakErrorKind::WritePanic => DALError::WritePanic,
            _ => DALError::UnknowError,
        }
    }
}
