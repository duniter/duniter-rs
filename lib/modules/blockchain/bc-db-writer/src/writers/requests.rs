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

use crate::*;
use dubp_block_doc::block::{BlockDocument, BlockDocumentTrait};
use dubp_common_doc::Blockstamp;
use dubp_currency_params::CurrencyParameters;
use dubp_user_docs::documents::certification::CompactCertificationDocumentV10;
use dubp_user_docs::documents::identity::IdentityDocumentV10;
use dup_crypto::keys::PubKey;
use durs_bc_db_reader::blocks::fork_tree::ForkTree;
use durs_bc_db_reader::blocks::BlockDb;
use durs_bc_db_reader::indexes::sources::SourceAmount;
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
    WriteBlock(BlockDb),
    /// Revert block
    RevertBlock(BlockDb),
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
        db: &Db,
        w: &mut DbWriter,
        fork_tree: &mut ForkTree,
        fork_window_size: usize,
        sync_target: Option<Blockstamp>,
    ) -> Result<(), DbError> {
        match self {
            BlocksDBsWriteQuery::WriteBlock(mut block_db) => {
                trace!("BlocksDBsWriteQuery::WriteBlock...");
                block_db.block.reduce();
                crate::current_metadata::update_current_metadata(db, w, &block_db.block)?;
                if sync_target.is_none()
                    || block_db.blockstamp().id.0 + fork_window_size as u32
                        >= sync_target.expect("safe unwrap").id.0
                {
                    crate::blocks::insert_new_head_block(db, w, Some(fork_tree), block_db)?;
                } else {
                    crate::blocks::insert_new_head_block(db, w, None, block_db)?;
                }
            }
            BlocksDBsWriteQuery::RevertBlock(block_db) => {
                trace!("BlocksDBsWriteQuery::WriteBlock...");
                crate::current_metadata::revert_current_metadata(db, w, &block_db.block)?;
                crate::blocks::remove_block(db, w, block_db.block.number())?;
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
        db: &Db,
        w: &mut DbWriter,
        _blockstamp: &Blockstamp,
        currency_params: &CurrencyParameters,
    ) -> Result<(), DbError> {
        match *self {
            WotsDBsWriteQuery::CreateIdentity(
                ref wot_id,
                ref blockstamp,
                ref current_bc_time,
                ref idty_doc,
                ref ms_created_block_id,
            ) => {
                crate::indexes::identities::create_identity(
                    currency_params,
                    &db,
                    w,
                    idty_doc.deref(),
                    *ms_created_block_id,
                    *wot_id,
                    *blockstamp,
                    *current_bc_time,
                )?;
            }
            WotsDBsWriteQuery::RevertCreateIdentity(ref pubkey) => {
                crate::indexes::identities::revert_create_identity(&db, w, pubkey)?;
            }
            WotsDBsWriteQuery::RenewalIdentity(
                _,
                ref idty_wot_id,
                ref current_bc_time,
                ms_created_block_id,
            ) => {
                trace!("WotsDBsWriteQuery::RenewalIdentity...");
                crate::indexes::identities::renewal_identity(
                    currency_params,
                    &db,
                    w,
                    *idty_wot_id,
                    *current_bc_time,
                    ms_created_block_id,
                    false,
                )?;
                trace!("DBWrWotsDBsWriteQueryiteRequest::RenewalIdentity...");
            }
            WotsDBsWriteQuery::RevertRenewalIdentity(
                _,
                ref idty_wot_id,
                ref current_bc_time,
                ms_created_block_id,
            ) => {
                crate::indexes::identities::renewal_identity(
                    currency_params,
                    &db,
                    w,
                    *idty_wot_id,
                    *current_bc_time,
                    ms_created_block_id,
                    true,
                )?;
            }
            WotsDBsWriteQuery::ExcludeIdentity(ref pubkey, ref blockstamp) => {
                crate::indexes::identities::exclude_identity(&db, w, pubkey, blockstamp, false)?;
            }
            WotsDBsWriteQuery::RevertExcludeIdentity(ref pubkey, ref blockstamp) => {
                crate::indexes::identities::exclude_identity(&db, w, pubkey, blockstamp, true)?;
            }
            WotsDBsWriteQuery::RevokeIdentity(ref pubkey, ref blockstamp, ref explicit) => {
                crate::indexes::identities::revoke_identity(
                    &db, w, pubkey, blockstamp, *explicit, false,
                )?;
            }
            WotsDBsWriteQuery::RevertRevokeIdentity(ref pubkey, ref blockstamp, ref explicit) => {
                crate::indexes::identities::revoke_identity(
                    &db, w, pubkey, blockstamp, *explicit, true,
                )?;
            }
            WotsDBsWriteQuery::CreateCert(
                _,
                ref source,
                ref target,
                ref created_block_id,
                ref median_time,
            ) => {
                trace!("WotsDBsWriteQuery::CreateCert...");
                crate::indexes::certs::write_certification(
                    currency_params,
                    &db,
                    w,
                    *source,
                    *target,
                    *created_block_id,
                    *median_time,
                )?;
                trace!("WotsDBsWriteQuery::CreateCert...finish");
            }
            WotsDBsWriteQuery::RevertCert(ref compact_doc, ref source, ref target) => {
                trace!("WotsDBsWriteQuery::CreateCert...");
                crate::indexes::certs::revert_write_cert(&db, w, *compact_doc, *source, *target)?;
                trace!("WotsDBsWriteQuery::CreateCert...finish");
            }
            WotsDBsWriteQuery::ExpireCerts(ref created_block_id) => {
                crate::indexes::certs::expire_certs(&db, w, *created_block_id)?;
            }
            WotsDBsWriteQuery::RevertExpireCert(ref source, ref target, ref created_block_id) => {
                crate::indexes::certs::revert_expire_cert(
                    &db,
                    w,
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
/// Contain a pending write request for currency indexes
pub enum CurrencyDBsWriteQuery {
    /// Write transaction
    WriteTx(Box<TransactionDocument>),
    /// Revert transaction
    RevertTx(Box<TransactionDocument>),
    /// Create dividend
    CreateUD(SourceAmount, BlockNumber, Vec<PubKey>),
    /// Revert dividend
    RevertUD(SourceAmount, BlockNumber, Vec<PubKey>),
}

impl CurrencyDBsWriteQuery {
    /// Apply CurrencyDBsWriteQuery
    pub fn apply(
        &self,
        db: &Db,
        w: &mut DbWriter,
        block_consumed_sources_opt: Option<&mut HashMap<UniqueIdUTXOv10, TransactionOutputV10>>,
        in_fork_window: bool,
    ) -> Result<(), DbError> {
        match *self {
            CurrencyDBsWriteQuery::WriteTx(ref tx_doc) => {
                crate::indexes::transactions::apply_and_write_tx(
                    db,
                    w,
                    tx_doc.deref(),
                    in_fork_window,
                )?;
            }
            CurrencyDBsWriteQuery::RevertTx(ref tx_doc) => {
                if let Some(block_consumed_sources) = block_consumed_sources_opt {
                    crate::indexes::transactions::revert_tx(
                        db,
                        w,
                        tx_doc.deref(),
                        block_consumed_sources,
                    )?;
                } else {
                    durs_common_tools::fatal_error!(
                        "Try to revert tx without block_consumed_sources."
                    )
                }
            }
            CurrencyDBsWriteQuery::CreateUD(ref du_amount, ref block_id, ref members) => {
                crate::indexes::dividends::create_du(db, w, du_amount, *block_id, members, false)?;
            }
            CurrencyDBsWriteQuery::RevertUD(ref du_amount, ref block_id, ref members) => {
                crate::indexes::dividends::create_du(db, w, du_amount, *block_id, members, true)?;
            }
        }
        Ok(())
    }
}
