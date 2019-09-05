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
use crate::entities::identity::DbIdentity;
use crate::filters::identities::IdentitiesFilter;
use crate::*;
use dubp_common_doc::traits::Document;
use dubp_common_doc::BlockNumber;
use dup_crypto::keys::*;
use durs_dbs_tools::DbError;
use durs_wot::WotId;
use std::collections::HashMap;

/// Get identities in databases
pub fn get_identities<DB: DbReadable>(
    db: &DB,
    filters: IdentitiesFilter,
    current_block_id: BlockNumber,
) -> Result<Vec<DbIdentity>, DbError> {
    if let Some(pubkey) = filters.by_pubkey {
        if let Some(idty) = get_identity(db, &pubkey)? {
            Ok(vec![idty])
        } else {
            Ok(vec![])
        }
    } else {
        db.read(|r| {
            let mut identities: Vec<DbIdentity> = Vec::new();
            for entry in db.get_store(IDENTITIES).iter_start(r)? {
                if let Some(v) = entry?.1 {
                    let db_idty = DB::from_db_value::<DbIdentity>(v)?;
                    if filters
                        .paging
                        .check_created_on(db_idty.idty_doc.blockstamp().id, current_block_id)
                    {
                        identities.push(db_idty);
                    }
                }
            }
            identities.sort_by(|i1, i2| {
                i1.idty_doc
                    .blockstamp()
                    .id
                    .cmp(&i2.idty_doc.blockstamp().id)
            });
            Ok(identities
                .into_iter()
                .skip(filters.paging.page_size * filters.paging.page_number)
                .take(filters.paging.page_size)
                .collect())
        })
    }
}

/// Get identity in databases
pub fn get_identity<DB: DbReadable>(
    db: &DB,
    pubkey: &PubKey,
) -> Result<Option<DbIdentity>, DbError> {
    db.read(|r| {
        if let Some(v) = db.get_store(IDENTITIES).get(r, &pubkey.to_bytes_vector())? {
            Ok(Some(DB::from_db_value(v)?))
        } else {
            Ok(None)
        }
    })
}

/// Get uid from pubkey
#[inline]
pub fn get_uid<DB: DbReadable>(db: &DB, pubkey: &PubKey) -> Result<Option<String>, DbError> {
    Ok(get_identity(db, pubkey)?.map(|db_idty| db_idty.idty_doc.username().to_owned()))
}

/// Get pubkey from uid
pub fn get_pubkey_from_uid<DB: DbReadable>(db: &DB, uid: &str) -> Result<Option<PubKey>, DbError> {
    db.read(|r| {
        for entry in db.get_store(IDENTITIES).iter_start(r)? {
            if let Some(v) = entry?.1 {
                let idty_doc = DB::from_db_value::<DbIdentity>(v)?.idty_doc;
                if idty_doc.username() == uid {
                    return Ok(Some(idty_doc.issuers()[0]));
                }
            }
        }
        Ok(None)
    })
}

/// Get wot_id index
pub fn get_wot_index<DB: DbReadable>(db: &DB) -> Result<HashMap<PubKey, WotId>, DbError> {
    db.read(|r| {
        let mut wot_index = HashMap::new();
        for entry in db.get_store(IDENTITIES).iter_start(r)? {
            if let Some(v) = entry?.1 {
                let db_idty = DB::from_db_value::<DbIdentity>(v)?;
                wot_index.insert(db_idty.idty_doc.issuers()[0], db_idty.wot_id);
            }
        }
        Ok(wot_index)
    })
}

/// Get wot_uid index
pub fn get_wot_uid_index<DB: DbReadable>(db: &DB) -> Result<HashMap<WotId, String>, DbError> {
    db.read(|r| {
        let mut wot_uid_index = HashMap::new();
        for entry in db.get_store(IDENTITIES).iter_start(r)? {
            if let Some(v) = entry?.1 {
                let db_idty = DB::from_db_value::<DbIdentity>(v)?;
                wot_uid_index.insert(db_idty.wot_id, db_idty.idty_doc.username().to_owned());
            }
        }
        Ok(wot_uid_index)
    })
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::entities::identity::*;
    use crate::filters::PagingFilter;
    use dubp_common_doc::Blockstamp;
    use dup_crypto_tests_tools::mocks::pubkey;
    use durs_common_tests_tools::collections::slice_same_elems;
    use durs_dbs_tools::kv_db::KvFileDbHandler;

    fn gen_mock_dal_idty(pubkey: PubKey, created_block_id: BlockNumber) -> DbIdentity {
        DbIdentity {
            hash: "".to_owned(),
            state: DbIdentityState::Member(vec![]),
            joined_on: Blockstamp::default(),
            expired_on: None,
            revoked_on: None,
            idty_doc: dubp_user_docs_tests_tools::mocks::identity::gen_mock_idty(
                pubkey,
                created_block_id,
            ),
            wot_id: WotId(0),
            ms_created_block_id: BlockNumber(0),
            ms_chainable_on: vec![],
            cert_chainable_on: vec![],
        }
    }

    #[test]
    fn test_get_identities() -> Result<(), DbError> {
        // Create mock identities
        let mock_identities = vec![
            gen_mock_dal_idty(pubkey('A'), BlockNumber(0)),
            gen_mock_dal_idty(pubkey('B'), BlockNumber(1)),
            gen_mock_dal_idty(pubkey('C'), BlockNumber(3)),
            gen_mock_dal_idty(pubkey('D'), BlockNumber(4)),
            gen_mock_dal_idty(pubkey('E'), BlockNumber(5)),
        ];

        // Write mock identities in DB
        let db = crate::tests::open_tmp_db()?;
        for idty in &mock_identities {
            let idty_bin = durs_dbs_tools::to_bytes(idty)?;
            db.write(|mut w| {
                db.get_store(IDENTITIES).put(
                    w.as_mut(),
                    &idty.idty_doc.issuers()[0].to_bytes_vector(),
                    &KvFileDbHandler::db_value(&idty_bin)?,
                )?;
                Ok(w)
            })?;
        }

        // Test default filters
        let mut filters = IdentitiesFilter::default();
        assert!(slice_same_elems(
            &mock_identities,
            &get_identities(&db, filters, BlockNumber(5))?
        ));
        // Test by pubkey filter
        filters = IdentitiesFilter::by_pubkey(pubkey('A'));
        assert_eq!(
            vec![mock_identities[0].clone()],
            get_identities(&db, filters, BlockNumber(5))?
        );
        filters = IdentitiesFilter::by_pubkey(pubkey('C'));
        assert_eq!(
            vec![mock_identities[2].clone()],
            get_identities(&db, filters, BlockNumber(5))?
        );

        // Test paging filter with little page size
        filters = IdentitiesFilter {
            paging: PagingFilter {
                from: BlockNumber(0),
                to: None,
                page_size: 2,
                page_number: 1,
            },
            by_pubkey: None,
        };
        assert!(slice_same_elems(
            &vec![mock_identities[2].clone(), mock_identities[3].clone()],
            &get_identities(&db, filters, BlockNumber(5))?
        ));

        // Test paging filter with limited interval
        filters = IdentitiesFilter {
            paging: PagingFilter {
                from: BlockNumber(2),
                to: Some(BlockNumber(3)),
                page_size: 50,
                page_number: 0,
            },
            by_pubkey: None,
        };
        assert_eq!(
            vec![mock_identities[2].clone()],
            get_identities(&db, filters, BlockNumber(5))?
        );

        Ok(())
    }
}
