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

use crate::entities::block::DALBlock;
use crate::entities::sources::SourceAmount;
use crate::writers::transaction::DALTxV10;
use crate::*;
use dubp_block_doc::block::{BlockDocument, BlockDocumentTrait};
use dubp_common_doc::Blockstamp;
use dubp_currency_params::CurrencyParameters;
use dubp_user_docs::documents::certification::CompactCertificationDocumentV10;
use dubp_user_docs::documents::identity::IdentityDocumentV10;
use dup_crypto::keys::PubKey;
use durs_wot::WotId;
use std::ops::Deref;

#[derive(Debug, Clone)]
/// Contain a pending write request for databases
pub enum DBsWriteRequest {
    /// Contain a pending write request for blocks database
    BlocksDB(BlocksDBsWriteQuery),
    /// Contain a pending write request for wots databases
    WotDBs(WotsDBsWriteQuery),
    /// Contain a pending write request for currency databases
    CurrencyDBs(CurrencyDBsWriteQuery),
}

#[derive(Debug, Clone)]
/// Contain a pending write request for blocks databases
pub enum BlocksDBsWriteQuery {
    /// Write block
    WriteBlock(DALBlock),
    /// Revert block
    RevertBlock(DALBlock),
}

impl BlocksDBsWriteQuery {
    /// Get copy of block document
    pub fn get_block_doc_copy(&self) -> BlockDocument {
        match self {
            BlocksDBsWriteQuery::WriteBlock(dal_block) => dal_block.block.clone(),
            BlocksDBsWriteQuery::RevertBlock(dal_block) => dal_block.block.clone(),
        }
    }
    /// BlocksDBsWriteQuery
    pub fn apply(
        self,
        blockchain_db: &BinDB<LocalBlockchainV10Datas>,
        forks_db: &ForksDBs,
        fork_window_size: usize,
        sync_target: Option<Blockstamp>,
    ) -> Result<(), DALError> {
        match self {
            BlocksDBsWriteQuery::WriteBlock(dal_block) => {
                let dal_block: DALBlock = dal_block;
                trace!("BlocksDBsWriteQuery::WriteBlock...");
                if sync_target.is_none()
                    || dal_block.blockstamp().id.0 + fork_window_size as u32
                        >= sync_target.expect("safe unwrap").id.0
                {
                    super::block::insert_new_head_block(blockchain_db, forks_db, dal_block)?;
                } else {
                    // Insert block in blockchain
                    blockchain_db.write(|db| {
                        db.insert(dal_block.block.number(), dal_block);
                    })?;
                }
            }
            BlocksDBsWriteQuery::RevertBlock(dal_block) => {
                trace!("BlocksDBsWriteQuery::WriteBlock...");
                // Remove block in blockchain
                blockchain_db.write(|db| {
                    db.remove(&dal_block.block.number());
                })?;
                trace!("BlocksDBsWriteQuery::WriteBlock...finish");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
/// Contain a pending write request for wots databases
pub enum WotsDBsWriteQuery {
    /// Newcomer (wot_id, blockstamp, current_bc_time, idty_doc, ms_created_block_id)
    CreateIdentity(
        WotId,
        Blockstamp,
        u64,
        Box<IdentityDocumentV10>,
        BlockNumber,
    ),
    /// Revert newcomer event (wot_id, blockstamp, current_bc_time, idty_doc, ms_created_block_id)
    RevertCreateIdentity(PubKey),
    /// Active (pubKey, idty_wot_id, current_bc_time, ms_created_block_id)
    RenewalIdentity(PubKey, WotId, u64, BlockNumber),
    /// Revert active (pubKey, idty_wot_id, current_bc_time, ms_created_block_id)
    RevertRenewalIdentity(PubKey, WotId, u64, BlockNumber),
    /// Excluded
    ExcludeIdentity(PubKey, Blockstamp),
    /// Revert exclusion
    RevertExcludeIdentity(PubKey, Blockstamp),
    /// Revoked
    RevokeIdentity(PubKey, Blockstamp, bool),
    /// Revert revocation
    RevertRevokeIdentity(PubKey, Blockstamp, bool),
    /// Certification (source_pubkey, source, target, created_block_id, median_time)
    CreateCert(PubKey, WotId, WotId, BlockNumber, u64),
    /// Revert certification (source_pubkey, source, target, created_block_id, median_time)
    RevertCert(CompactCertificationDocumentV10, WotId, WotId),
    /// Certification expiry (source, target, created_block_id)
    ExpireCerts(BlockNumber),
    /// Revert certification expiry event (source, target, created_block_id)
    RevertExpireCert(WotId, WotId, BlockNumber),
}

impl WotsDBsWriteQuery {
    /// Apply WotsDBsWriteQuery
    pub fn apply(
        &self,
        _blockstamp: &Blockstamp,
        currency_params: &CurrencyParameters,
        databases: &WotsV10DBs,
    ) -> Result<(), DALError> {
        match *self {
            WotsDBsWriteQuery::CreateIdentity(
                ref wot_id,
                ref blockstamp,
                ref current_bc_time,
                ref idty_doc,
                ref ms_created_block_id,
            ) => {
                writers::identity::create_identity(
                    currency_params,
                    &databases.identities_db,
                    &databases.ms_db,
                    idty_doc.deref(),
                    *ms_created_block_id,
                    *wot_id,
                    *blockstamp,
                    *current_bc_time,
                )?;
            }
            WotsDBsWriteQuery::RevertCreateIdentity(ref pubkey) => {
                writers::identity::revert_create_identity(
                    &databases.identities_db,
                    &databases.ms_db,
                    pubkey,
                )?;
            }
            WotsDBsWriteQuery::RenewalIdentity(
                ref pubkey,
                ref idty_wot_id,
                ref current_bc_time,
                ms_created_block_id,
            ) => {
                trace!("WotsDBsWriteQuery::RenewalIdentity...");
                writers::identity::renewal_identity(
                    currency_params,
                    &databases.identities_db,
                    &databases.ms_db,
                    pubkey,
                    *idty_wot_id,
                    *current_bc_time,
                    ms_created_block_id,
                    false,
                )?;
                trace!("DBWrWotsDBsWriteQueryiteRequest::RenewalIdentity...");
            }
            WotsDBsWriteQuery::RevertRenewalIdentity(
                ref pubkey,
                ref idty_wot_id,
                ref current_bc_time,
                ms_created_block_id,
            ) => {
                writers::identity::renewal_identity(
                    currency_params,
                    &databases.identities_db,
                    &databases.ms_db,
                    pubkey,
                    *idty_wot_id,
                    *current_bc_time,
                    ms_created_block_id,
                    true,
                )?;
            }
            WotsDBsWriteQuery::ExcludeIdentity(ref pubkey, ref blockstamp) => {
                writers::identity::exclude_identity(
                    &databases.identities_db,
                    pubkey,
                    blockstamp,
                    false,
                )?;
            }
            WotsDBsWriteQuery::RevertExcludeIdentity(ref pubkey, ref blockstamp) => {
                writers::identity::exclude_identity(
                    &databases.identities_db,
                    pubkey,
                    blockstamp,
                    true,
                )?;
            }
            WotsDBsWriteQuery::RevokeIdentity(ref pubkey, ref blockstamp, ref explicit) => {
                writers::identity::revoke_identity(
                    &databases.identities_db,
                    pubkey,
                    blockstamp,
                    *explicit,
                    false,
                )?;
            }
            WotsDBsWriteQuery::RevertRevokeIdentity(ref pubkey, ref blockstamp, ref explicit) => {
                writers::identity::revoke_identity(
                    &databases.identities_db,
                    pubkey,
                    blockstamp,
                    *explicit,
                    true,
                )?;
            }
            WotsDBsWriteQuery::CreateCert(
                ref source_pubkey,
                ref source,
                ref target,
                ref created_block_id,
                ref median_time,
            ) => {
                trace!("WotsDBsWriteQuery::CreateCert...");
                writers::certification::write_certification(
                    currency_params,
                    &databases.identities_db,
                    &databases.certs_db,
                    *source_pubkey,
                    *source,
                    *target,
                    *created_block_id,
                    *median_time,
                )?;
                trace!("WotsDBsWriteQuery::CreateCert...finish");
            }
            WotsDBsWriteQuery::RevertCert(ref compact_doc, ref source, ref target) => {
                trace!("WotsDBsWriteQuery::CreateCert...");
                writers::certification::revert_write_cert(
                    &databases.identities_db,
                    &databases.certs_db,
                    *compact_doc,
                    *source,
                    *target,
                )?;
                trace!("WotsDBsWriteQuery::CreateCert...finish");
            }
            WotsDBsWriteQuery::ExpireCerts(ref created_block_id) => {
                super::certification::expire_certs(&databases.certs_db, *created_block_id)?;
            }
            WotsDBsWriteQuery::RevertExpireCert(ref source, ref target, ref created_block_id) => {
                super::certification::revert_expire_cert(
                    &databases.certs_db,
                    *source,
                    *target,
                    *created_block_id,
                )?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
/// Contain a pending write request for currency databases
pub enum CurrencyDBsWriteQuery {
    /// Write transaction
    WriteTx(Box<TransactionDocument>),
    /// Revert transaction
    RevertTx(Box<DALTxV10>),
    /// Create dividend
    CreateUD(SourceAmount, BlockNumber, Vec<PubKey>),
    /// Revert dividend
    RevertUD(SourceAmount, BlockNumber, Vec<PubKey>),
}

impl CurrencyDBsWriteQuery {
    /// Apply CurrencyDBsWriteQuery
    pub fn apply(
        &self,
        blockstamp: &Blockstamp,
        databases: &CurrencyV10DBs,
    ) -> Result<(), DALError> {
        match *self {
            CurrencyDBsWriteQuery::WriteTx(ref tx_doc) => {
                super::transaction::apply_and_write_tx(blockstamp, &databases, tx_doc.deref())?;
            }
            CurrencyDBsWriteQuery::RevertTx(ref dal_tx) => {
                super::transaction::revert_tx(blockstamp, &databases, dal_tx.deref())?;
            }
            CurrencyDBsWriteQuery::CreateUD(ref du_amount, ref block_id, ref members) => {
                super::dividend::create_du(
                    &databases.du_db,
                    &databases.balances_db,
                    du_amount,
                    *block_id,
                    members,
                    false,
                )?;
            }
            CurrencyDBsWriteQuery::RevertUD(ref du_amount, ref block_id, ref members) => {
                super::dividend::create_du(
                    &databases.du_db,
                    &databases.balances_db,
                    du_amount,
                    *block_id,
                    members,
                    true,
                )?;
            }
        }
        Ok(())
    }
}
