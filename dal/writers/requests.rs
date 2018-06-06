extern crate serde;
extern crate serde_json;

use block::DALBlock;
use currency_params::CurrencyParameters;
use duniter_crypto::keys::PubKey;
use duniter_documents::blockchain::v10::documents::identity::IdentityDocument;
use duniter_documents::Blockstamp;
use duniter_wotb::NodeId;
use identity::DALIdentity;
use sources::SourceAmount;
use std::ops::Deref;
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
    RevertBlock(Box<DALBlock>),
}

impl BlocksDBsWriteQuery {
    pub fn apply(&self, databases: &BlocksV10DBs, sync: bool) -> Result<(), DALError> {
        if let BlocksDBsWriteQuery::WriteBlock(
            ref dal_block,
            ref old_fork_id,
            ref _previous_blockstamp,
            ref _block_hash,
        ) = *self
        {
            let dal_block = dal_block.deref();
            trace!("BlocksDBsWriteQuery::WriteBlock...");
            super::block::write(
                &databases.blockchain_db,
                &databases.forks_db,
                &databases.forks_blocks_db,
                &dal_block,
                *old_fork_id,
                sync,
            )?;
            trace!("BlocksDBsWriteQuery::WriteBlock...finish");
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
/// Contain a pending write request for wots databases
pub enum WotsDBsWriteQuery {
    /// Newcomer (wotb_id, blockstamp, current_bc_time, idty_doc, ms_created_block_id)
    CreateIdentity(NodeId, Blockstamp, u64, Box<IdentityDocument>, BlockId),
    /// Active (pubKey, idty_wot_id, current_bc_time, ms_created_block_id)
    RenewalIdentity(PubKey, NodeId, u64, BlockId),
    /// Excluded
    ExcludeIdentity(PubKey, Blockstamp),
    /// Revoked
    RevokeIdentity(PubKey, Blockstamp),
    /// Certification (source_pubkey, source, target, created_block_id, median_time)
    CreateCert(PubKey, NodeId, NodeId, BlockId, u64),
    /// Certification expiry (source, target, created_block_id)
    ExpireCert(NodeId, NodeId, BlockId),
}

impl WotsDBsWriteQuery {
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
                trace!("WotsDBsWriteQuery::CreateIdentity...");
                let idty = DALIdentity::create_identity(
                    currency_params,
                    idty_doc.deref(),
                    *wotb_id,
                    *blockstamp,
                    *current_bc_time,
                );
                super::identity::write(
                    &idty,
                    *wotb_id,
                    &databases.identities_db,
                    &databases.ms_db,
                    *ms_created_block_id,
                )?;
                trace!("WotsDBsWriteQuery::CreateIdentity...finish.");
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
            WotsDBsWriteQuery::ExcludeIdentity(ref pubkey, ref blockstamp) => {
                DALIdentity::exclude_identity(&databases.identities_db, pubkey, blockstamp, false)?;
            }
            WotsDBsWriteQuery::RevokeIdentity(ref pubkey, ref blockstamp) => {
                DALIdentity::revoke_identity(
                    &databases.identities_db,
                    pubkey,
                    blockstamp,
                    true,
                    false,
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
                super::certification::write_certification(
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
            WotsDBsWriteQuery::ExpireCert(ref _source, ref _target, ref _created_block_id) => {
                /*super::certification::expire_cert(
                    &databases.certs_db,
                    *source,
                    *target,
                    *created_block_id,
                )?;*/
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
    /// Create dividend
    CreateDU(SourceAmount, BlockId, Vec<PubKey>),
}

impl CurrencyDBsWriteQuery {
    pub fn apply(&self, databases: &CurrencyV10DBs) -> Result<(), DALError> {
        match *self {
            CurrencyDBsWriteQuery::WriteTx(ref tx_doc) => {
                super::transaction::apply_and_write_tx(
                    &databases.tx_db,
                    &databases.utxos_db,
                    &databases.du_db,
                    &databases.balances_db,
                    tx_doc.deref(),
                )?;
            }
            CurrencyDBsWriteQuery::CreateDU(ref du_amount, ref block_id, ref members) => {
                super::dividend::create_du(
                    &databases.du_db,
                    &databases.balances_db,
                    du_amount,
                    block_id,
                    members,
                )?;
            }
        }
        Ok(())
    }
}
