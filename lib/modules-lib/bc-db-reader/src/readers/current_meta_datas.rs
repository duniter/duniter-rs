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

use crate::constants::*;
use crate::entities::current_meta_datas::CurrentMetaDataKey;
use crate::entities::fork_tree::ForkTree;
use crate::{DbReadable, DbValue};
use dubp_common_doc::{Blockstamp, CurrencyName};
use durs_dbs_tools::DbError;

/// Get DB version
pub fn get_db_version<DB: DbReadable>(db: &DB) -> Result<usize, DbError> {
    db.read(|r| {
        if let Some(v) = db
            .get_int_store(CURRENT_METAS_DATAS)
            .get(r, CurrentMetaDataKey::DbVersion.to_u32())?
        {
            if let DbValue::U64(db_version) = v {
                Ok(db_version as usize)
            } else {
                Err(DbError::DBCorrupted)
            }
        } else {
            Err(DbError::DBCorrupted)
        }
    })
}

/// Get currency name
pub fn get_currency_name<DB: DbReadable>(db: &DB) -> Result<Option<CurrencyName>, DbError> {
    db.read(|r| {
        if let Some(v) = db
            .get_int_store(CURRENT_METAS_DATAS)
            .get(r, CurrentMetaDataKey::CurrencyName.to_u32())?
        {
            if let DbValue::Str(curency_name) = v {
                Ok(Some(CurrencyName(curency_name.to_owned())))
            } else {
                Err(DbError::DBCorrupted)
            }
        } else {
            Ok(None)
        }
    })
}

/// Get current blockstamp
pub fn get_current_blockstamp<DB: DbReadable>(db: &DB) -> Result<Option<Blockstamp>, DbError> {
    db.read(|r| {
        if let Some(v) = db
            .get_int_store(CURRENT_METAS_DATAS)
            .get(r, CurrentMetaDataKey::CurrentBlockstamp.to_u32())?
        {
            if let DbValue::Blob(current_blockstamp_bytes) = v {
                Ok(Some(
                    Blockstamp::from_bytes(current_blockstamp_bytes)
                        .map_err(|_| DbError::DBCorrupted)?,
                ))
            } else {
                Err(DbError::DBCorrupted)
            }
        } else {
            Ok(None)
        }
    })
}

/// Get current common time (also named "blockchain time")
pub fn get_current_common_time<DB: DbReadable>(db: &DB) -> Result<u64, DbError> {
    db.read(|r| {
        if let Some(v) = db
            .get_int_store(CURRENT_METAS_DATAS)
            .get(r, CurrentMetaDataKey::CurrentBlockchainTime.to_u32())?
        {
            if let DbValue::U64(current_common_time) = v {
                Ok(current_common_time)
            } else {
                Err(DbError::DBCorrupted)
            }
        } else {
            Ok(0u64)
        }
    })
}

/// Get fork tree root
pub fn get_fork_tree<DB: DbReadable>(db: &DB) -> Result<ForkTree, DbError> {
    db.read(|r| {
        if let Some(v) = db
            .get_int_store(CURRENT_METAS_DATAS)
            .get(r, CurrentMetaDataKey::ForkTree.to_u32())?
        {
            Ok(DB::from_db_value::<ForkTree>(v)?)
        } else {
            Ok(ForkTree::default())
        }
    })
}
