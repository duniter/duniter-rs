//  Copyright (C) 2018  The Duniter Project Developers.
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

extern crate serde;
extern crate serde_json;

use block::DALBlock;
use currency_params::CurrencyParameters;
use duniter_crypto::keys::PubKey;
use duniter_documents::blockchain::v10::documents::certification::CompactCertificationDocument;
use duniter_documents::blockchain::v10::documents::identity::IdentityDocument;
use duniter_documents::Blockstamp;
use duniter_wotb::NodeId;
use identity::DALIdentity;
use sources::SourceAmount;
use std::ops::Deref;
use writers::transaction::DALTxV10;
use *;

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
    WriteBlock(Box<DALBlock>, Option<ForkId>, PreviousBlockstamp, BlockHash),
    /// Revert block
    RevertBlock(Box<DALBlock>, Option<ForkId>),
}

impl BlocksDBsWriteQuery {
    /// BlocksDBsWriteQuery
    pub fn apply(&self, databases: &BlocksV10DBs, sync: bool) -> Result<(), DALError> {
        match *self {
            BlocksDBsWriteQuery::WriteBlock(ref dal_block, ref old_fork_id, _, _) => {
                let dal_block = dal_block.deref();
                trace!("BlocksDBsWriteQuery::WriteBlock...");
                super::block::write(
                    &databases.blockchain_db,
                    &databases.forks_db,
                    &databases.forks_blocks_db,
                    &dal_block,
                    *old_fork_id,
                    sync,
                    false,
                )?;
                trace!("BlocksDBsWriteQuery::WriteBlock...finish");
            }
            BlocksDBsWriteQuery::RevertBlock(ref dal_block, ref to_fork_id) => {
                let dal_block = dal_block.deref();
                trace!("BlocksDBsWriteQuery::WriteBlock...");
                super::block::write(
                    &databases.blockchain_db,
                    &databases.forks_db,
                    &databases.forks_blocks_db,
                    &dal_block,
                    *to_fork_id,
                    sync,
                    true,
                )?;
                trace!("BlocksDBsWriteQuery::WriteBlock...finish");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
/// Contain a pending write request for wots databases
pub enum WotsDBsWriteQuery {
    /// Newcomer (wotb_id, blockstamp, current_bc_time, idty_doc, ms_created_block_id)
    CreateIdentity(NodeId, Blockstamp, u64, Box<IdentityDocument>, BlockId),
    /// Revert newcomer event (wotb_id, blockstamp, current_bc_time, idty_doc, ms_created_block_id)
    RevertCreateIdentity(PubKey),
    /// Active (pubKey, idty_wot_id, current_bc_time, ms_created_block_id)
    RenewalIdentity(PubKey, NodeId, u64, BlockId),
    /// Revert active (pubKey, idty_wot_id, current_bc_time, ms_created_block_id)
    RevertRenewalIdentity(PubKey, NodeId, u64, BlockId),
    /// Excluded
    ExcludeIdentity(PubKey, Blockstamp),
    /// Revert exclusion
    RevertExcludeIdentity(PubKey, Blockstamp),
    /// Revoked
    RevokeIdentity(PubKey, Blockstamp, bool),
    /// Revert revocation
    RevertRevokeIdentity(PubKey, Blockstamp, bool),
    /// Certification (source_pubkey, source, target, created_block_id, median_time)
    CreateCert(PubKey, NodeId, NodeId, BlockId, u64),
    /// Revert certification (source_pubkey, source, target, created_block_id, median_time)
    RevertCert(CompactCertificationDocument, NodeId, NodeId),
    /// Certification expiry (source, target, created_block_id)
    ExpireCerts(BlockId),
    /// Revert certification expiry event (source, target, created_block_id)
    RevertExpireCert(NodeId, NodeId, BlockId),
}

impl WotsDBsWriteQuery {
    /// Apply WotsDBsWriteQuery
    pub fn apply(
        &self,
        databases: &WotsV10DBs,
        currency_params: &CurrencyParameters,
    ) -> Result<(), DALError> {
        match *self {
            WotsDBsWriteQuery::CreateIdentity(
                ref wotb_id,
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
                    *wotb_id,
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
                let mut idty = DALIdentity::get_identity(&databases.identities_db, pubkey)?
                    .expect("Fatal error : impossible to renewal an identidy that don't exist !");
                idty.renewal_identity(
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
                let mut idty = DALIdentity::get_identity(&databases.identities_db, pubkey)?
                    .expect("Fatal error : impossible to renewal an identidy that don't exist !");
                idty.renewal_identity(
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
                DALIdentity::exclude_identity(&databases.identities_db, pubkey, blockstamp, false)?;
            }
            WotsDBsWriteQuery::RevertExcludeIdentity(ref pubkey, ref blockstamp) => {
                DALIdentity::exclude_identity(&databases.identities_db, pubkey, blockstamp, true)?;
            }
            WotsDBsWriteQuery::RevokeIdentity(ref pubkey, ref blockstamp, ref explicit) => {
                DALIdentity::revoke_identity(
                    &databases.identities_db,
                    pubkey,
                    blockstamp,
                    *explicit,
                    false,
                )?;
            }
            WotsDBsWriteQuery::RevertRevokeIdentity(ref pubkey, ref blockstamp, ref explicit) => {
                DALIdentity::revoke_identity(
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
    CreateUD(SourceAmount, BlockId, Vec<PubKey>),
    /// Revert dividend
    RevertUD(SourceAmount, BlockId, Vec<PubKey>),
}

impl CurrencyDBsWriteQuery {
    /// Apply CurrencyDBsWriteQuery
    pub fn apply(&self, databases: &CurrencyV10DBs) -> Result<(), DALError> {
        match *self {
            CurrencyDBsWriteQuery::WriteTx(ref tx_doc) => {
                super::transaction::apply_and_write_tx(&databases, tx_doc.deref())?;
            }
            CurrencyDBsWriteQuery::RevertTx(ref dal_tx) => {
                super::transaction::revert_tx(&databases, dal_tx.deref())?;
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
