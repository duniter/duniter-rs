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

use crate::entities::identity::DALIdentity;
use crate::filters::identities::IdentitiesFilter;
use crate::{BinFreeStructDb, DALError, IdentitiesV10Datas};
use dubp_common_doc::traits::Document;
use dubp_common_doc::BlockNumber;
use dup_crypto::keys::*;
use durs_wot::WotId;
use std::collections::HashMap;

/// Get identities in databases
pub fn get_identities(
    db: &BinFreeStructDb<IdentitiesV10Datas>,
    filters: IdentitiesFilter,
    current_block_id: BlockNumber,
) -> Result<Vec<DALIdentity>, DALError> {
    if let Some(pubkey) = filters.by_pubkey {
        if let Some(idty) = db.read(|db| db.get(&pubkey).cloned())? {
            Ok(vec![idty])
        } else {
            Ok(vec![])
        }
    } else {
        Ok(db.read(|db| {
            let mut identities: Vec<&DALIdentity> = db
                .values()
                .filter(|idty| {
                    filters
                        .paging
                        .check_created_on(idty.idty_doc.blockstamp().id, current_block_id)
                })
                .collect();
            identities.sort_by(|i1, i2| {
                i1.idty_doc
                    .blockstamp()
                    .id
                    .cmp(&i2.idty_doc.blockstamp().id)
            });
            identities
                .into_iter()
                .skip(filters.paging.page_size * filters.paging.page_number)
                .take(filters.paging.page_size)
                .cloned()
                .collect()
        })?)
    }
}

/// Get identity in databases
pub fn get_identity(
    db: &BinFreeStructDb<IdentitiesV10Datas>,
    pubkey: &PubKey,
) -> Result<Option<DALIdentity>, DALError> {
    Ok(db.read(|db| {
        if let Some(member_datas) = db.get(&pubkey) {
            Some(member_datas.clone())
        } else {
            None
        }
    })?)
}

/// Get uid from pubkey
pub fn get_uid(
    identities_db: &BinFreeStructDb<IdentitiesV10Datas>,
    pubkey: PubKey,
) -> Result<Option<String>, DALError> {
    Ok(identities_db.read(|db| {
        if let Some(dal_idty) = db.get(&pubkey) {
            Some(String::from(dal_idty.idty_doc.username()))
        } else {
            None
        }
    })?)
}

/// Get pubkey from uid
pub fn get_pubkey_from_uid(
    identities_db: &BinFreeStructDb<IdentitiesV10Datas>,
    uid: &str,
) -> Result<Option<PubKey>, DALError> {
    Ok(identities_db.read(|db| {
        for (pubkey, dal_idty) in db {
            if uid == dal_idty.idty_doc.username() {
                return Some(*pubkey);
            }
        }
        None
    })?)
}

/// Get wot_id index
pub fn get_wot_index(
    identities_db: &BinFreeStructDb<IdentitiesV10Datas>,
) -> Result<HashMap<PubKey, WotId>, DALError> {
    Ok(identities_db.read(|db| {
        db.iter()
            .map(|(pubkey, member_datas)| (*pubkey, member_datas.wot_id))
            .collect()
    })?)
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::entities::identity::*;
    use crate::filters::PagingFilter;
    use crate::*;
    use dubp_common_doc::Blockstamp;
    use dup_crypto_tests_tools::mocks::pubkey;
    use durs_common_tests_tools::collections::slice_same_elems;

    fn gen_mock_dal_idty(pubkey: PubKey, created_block_id: BlockNumber) -> DALIdentity {
        DALIdentity {
            hash: "".to_owned(),
            state: DALIdentityState::Member(vec![]),
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
    fn test_get_identities() -> Result<(), DALError> {
        // Create mock identities
        let mock_identities = vec![
            gen_mock_dal_idty(pubkey('A'), BlockNumber(0)),
            gen_mock_dal_idty(pubkey('B'), BlockNumber(1)),
            gen_mock_dal_idty(pubkey('C'), BlockNumber(3)),
            gen_mock_dal_idty(pubkey('D'), BlockNumber(4)),
            gen_mock_dal_idty(pubkey('E'), BlockNumber(5)),
        ];

        // Write mock identities in DB
        let identities_db = BinFreeStructDb::Mem(
            open_free_struct_memory_db::<IdentitiesV10Datas>().expect("Fail to create memory DB !"),
        );
        for idty in &mock_identities {
            identities_db.write(|db| {
                db.insert(idty.idty_doc.issuers()[0], idty.clone());
            })?;
        }

        // Test default filters
        let mut filters = IdentitiesFilter::default();
        assert!(slice_same_elems(
            &mock_identities,
            &get_identities(&identities_db, filters, BlockNumber(5))?
        ));
        // Test by pubkey filter
        filters = IdentitiesFilter::by_pubkey(pubkey('A'));
        assert_eq!(
            vec![mock_identities[0].clone()],
            get_identities(&identities_db, filters, BlockNumber(5))?
        );
        filters = IdentitiesFilter::by_pubkey(pubkey('C'));
        assert_eq!(
            vec![mock_identities[2].clone()],
            get_identities(&identities_db, filters, BlockNumber(5))?
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
            &get_identities(&identities_db, filters, BlockNumber(5))?
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
            get_identities(&identities_db, filters, BlockNumber(5))?
        );

        Ok(())
    }
}
