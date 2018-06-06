use currency_params::CurrencyParameters;
use duniter_crypto::keys::*;
use duniter_documents::blockchain::v10::documents::IdentityDocument;
use duniter_documents::{BlockId, Blockstamp};
use duniter_wotb::NodeId;
use rustbreak::backend::Backend;
use std::collections::HashMap;
use std::fmt::Debug;
use {BinDB, DALError, IdentitiesV10Datas, MsExpirV10Datas};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum DALIdentityState {
    Member(Vec<usize>),
    ExpireMember(Vec<usize>),
    ExplicitRevoked(Vec<usize>),
    ExplicitExpireRevoked(Vec<usize>),
    ImplicitRevoked(Vec<usize>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DALIdentity {
    pub hash: String,
    pub state: DALIdentityState,
    pub joined_on: Blockstamp,
    pub expired_on: Option<Blockstamp>,
    pub revoked_on: Option<Blockstamp>,
    pub idty_doc: IdentityDocument,
    pub wotb_id: NodeId,
    pub ms_chainable_on: Vec<u64>,
    pub cert_chainable_on: Vec<u64>,
}

pub fn get_uid<B: Backend + Debug>(
    identities_db: &BinDB<IdentitiesV10Datas, B>,
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

pub fn get_pubkey_from_uid<B: Backend + Debug>(
    identities_db: &BinDB<IdentitiesV10Datas, B>,
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

impl DALIdentity {
    pub fn exclude_identity<B: Backend + Debug>(
        identities_db: &BinDB<IdentitiesV10Datas, B>,
        pubkey: &PubKey,
        exclusion_blockstamp: &Blockstamp,
        revert: bool,
    ) -> Result<(), DALError> {
        let mut idty_datas = identities_db
            .read(|db| db.get(pubkey).cloned())?
            .expect("Fatal error : try to renewal unknow identity !");
        idty_datas.state = if revert {
            match idty_datas.state {
                DALIdentityState::ExpireMember(renewed_counts) => {
                    DALIdentityState::Member(renewed_counts)
                }
                _ => panic!("Try to revert exclusion for a no excluded identity !"),
            }
        } else {
            match idty_datas.state {
                DALIdentityState::Member(renewed_counts) => {
                    DALIdentityState::ExpireMember(renewed_counts)
                }
                _ => panic!("Try to exclude for an already excluded/revoked identity !"),
            }
        };
        idty_datas.expired_on = if revert {
            None
        } else {
            Some(*exclusion_blockstamp)
        };
        // Write new identity datas
        identities_db.write(|db| {
            db.insert(*pubkey, idty_datas);
        })?;
        Ok(())
    }

    pub fn get_wotb_index<B: Backend + Debug>(
        identities_db: &BinDB<IdentitiesV10Datas, B>,
    ) -> Result<HashMap<PubKey, NodeId>, DALError> {
        Ok(identities_db.read(|db| {
            let mut wotb_index: HashMap<PubKey, NodeId> = HashMap::new();
            for (pubkey, member_datas) in db {
                let wotb_id = member_datas.wotb_id;
                wotb_index.insert(*pubkey, wotb_id);
            }
            wotb_index
        })?)
    }

    pub fn create_identity(
        currency_params: &CurrencyParameters,
        idty_doc: &IdentityDocument,
        wotb_id: NodeId,
        current_blockstamp: Blockstamp,
        current_bc_time: u64,
    ) -> DALIdentity {
        let mut idty_doc = idty_doc.clone();
        idty_doc.reduce();
        DALIdentity {
            hash: "0".to_string(),
            state: DALIdentityState::Member(vec![0]),
            joined_on: current_blockstamp,
            expired_on: None,
            revoked_on: None,
            idty_doc,
            wotb_id,
            ms_chainable_on: vec![current_bc_time + currency_params.ms_period],
            cert_chainable_on: vec![],
        }
    }

    pub fn revoke_identity<B: Backend + Debug>(
        identities_db: &BinDB<IdentitiesV10Datas, B>,
        pubkey: &PubKey,
        renewal_blockstamp: &Blockstamp,
        explicit: bool,
        revert: bool,
    ) -> Result<(), DALError> {
        let mut member_datas = identities_db
            .read(|db| db.get(pubkey).cloned())?
            .expect("Fatal error : Try to revoke unknow idty !");

        member_datas.state = if revert {
            match member_datas.state {
                DALIdentityState::ExplicitRevoked(renewed_counts) => {
                    DALIdentityState::Member(renewed_counts)
                }
                DALIdentityState::ExplicitExpireRevoked(renewed_counts)
                | DALIdentityState::ImplicitRevoked(renewed_counts) => {
                    DALIdentityState::ExpireMember(renewed_counts)
                }
                _ => panic!("Try to revert revoke_identity() for a no revoked idty !"),
            }
        } else {
            match member_datas.state {
                DALIdentityState::ExpireMember(renewed_counts) => {
                    DALIdentityState::ExplicitExpireRevoked(renewed_counts)
                }
                DALIdentityState::Member(renewed_counts) => if explicit {
                    DALIdentityState::ExplicitRevoked(renewed_counts)
                } else {
                    DALIdentityState::ImplicitRevoked(renewed_counts)
                },
                _ => panic!("Try to revert revoke an already revoked idty !"),
            }
        };
        member_datas.revoked_on = if revert {
            None
        } else {
            Some(*renewal_blockstamp)
        };

        identities_db.write(|db| {
            db.insert(*pubkey, member_datas);
        })?;
        Ok(())
    }

    pub fn renewal_identity<B: Backend + Debug>(
        &mut self,
        currency_params: &CurrencyParameters,
        identities_db: &BinDB<IdentitiesV10Datas, B>,
        ms_db: &BinDB<MsExpirV10Datas, B>,
        pubkey: &PubKey,
        idty_wot_id: NodeId,
        renewal_timestamp: u64,
        ms_created_block_id: BlockId,
        revert: bool,
    ) -> Result<(), DALError> {
        // Get idty_datas
        let mut idty_datas = identities_db
            .read(|db| db.get(pubkey).cloned())?
            .expect("Fatal error : try to renewal unknow identity !");
        // Calculate new state value
        idty_datas.state = if revert {
            match idty_datas.state {
                DALIdentityState::Member(renewed_counts) => {
                    let mut new_renewed_counts = renewed_counts.clone();
                    new_renewed_counts[renewed_counts.len() - 1] -= 1;
                    if new_renewed_counts[renewed_counts.len() - 1] > 0 {
                        DALIdentityState::Member(new_renewed_counts)
                    } else {
                        DALIdentityState::ExpireMember(new_renewed_counts)
                    }
                }
                _ => panic!("Try to revert renewal_identity() for an excluded or revoked idty !"),
            }
        } else {
            match idty_datas.state {
                DALIdentityState::Member(renewed_counts) => {
                    let mut new_renewed_counts = renewed_counts.clone();
                    new_renewed_counts[renewed_counts.len() - 1] += 1;
                    DALIdentityState::Member(new_renewed_counts)
                }
                DALIdentityState::ExpireMember(renewed_counts) => {
                    let mut new_renewed_counts = renewed_counts.clone();
                    new_renewed_counts.push(0);
                    DALIdentityState::Member(new_renewed_counts)
                }
                _ => panic!("Try to renewed a revoked identity !"),
            }
        };
        // Calculate new ms_chainable_on value
        if revert {
            idty_datas.ms_chainable_on.pop();
        } else {
            idty_datas
                .ms_chainable_on
                .push(renewal_timestamp + currency_params.ms_period);
        }
        // Write new identity datas
        identities_db.write(|db| {
            db.insert(*pubkey, idty_datas);
        })?;
        // Update MsExpirV10DB
        ms_db.write(|db| {
            let mut memberships = db.get(&ms_created_block_id).cloned().unwrap_or_default();
            memberships.insert(idty_wot_id);
            db.insert(ms_created_block_id, memberships);
        })?;
        Ok(())
    }

    pub fn remove_identity<B: Backend + Debug>(
        db: &BinDB<IdentitiesV10Datas, B>,
        pubkey: PubKey,
    ) -> Result<(), DALError> {
        db.write(|db| {
            db.remove(&pubkey);
        })?;
        Ok(())
    }

    pub fn get_identity<B: Backend + Debug>(
        db: &BinDB<IdentitiesV10Datas, B>,
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
}
