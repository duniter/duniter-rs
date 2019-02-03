//  Copyright (C) 2018  The Durs Project Developers.
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
use crate::{BinDB, DALError, IdentitiesV10Datas};
use dup_crypto::keys::*;
use durs_wot::NodeId;
use std::collections::HashMap;

/// Get identity in databases
pub fn get_identity(
    db: &BinDB<IdentitiesV10Datas>,
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
    identities_db: &BinDB<IdentitiesV10Datas>,
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
    identities_db: &BinDB<IdentitiesV10Datas>,
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
    identities_db: &BinDB<IdentitiesV10Datas>,
) -> Result<HashMap<PubKey, NodeId>, DALError> {
    Ok(identities_db.read(|db| {
        let mut wot_index: HashMap<PubKey, NodeId> = HashMap::new();
        for (pubkey, member_datas) in db {
            let wot_id = member_datas.wot_id;
            wot_index.insert(*pubkey, wot_id);
        }
        wot_index
    })?)
}
