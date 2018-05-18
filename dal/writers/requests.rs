extern crate duniter_crypto;
extern crate duniter_documents;
extern crate duniter_wotb;
extern crate serde;
extern crate serde_json;
extern crate sqlite;

use self::duniter_crypto::keys::ed25519;
use self::duniter_documents::blockchain::v10::documents::certification::CompactCertificationDocument;
use self::duniter_documents::blockchain::v10::documents::identity::IdentityDocument;
use self::duniter_documents::Blockstamp;
use self::duniter_wotb::NodeId;
use super::super::block::DALBlock;
use super::super::identity::DALIdentity;
use super::super::DuniterDB;

#[derive(Debug)]
/// Contain a pending write request for blockchain database
pub enum DBWriteRequest {
    /// Newcomer
    CreateIdentity(NodeId, Blockstamp, u64, IdentityDocument),
    /// Active
    RenewalIdentity(ed25519::PublicKey, Blockstamp, u64),
    /// Excluded
    ExcludeIdentity(NodeId, Blockstamp, u64),
    /// Revoked
    RevokeIdentity(NodeId, Blockstamp, u64),
    /// Certification
    CreateCert(Blockstamp, u64, CompactCertificationDocument),
    /// Certification expiry
    CertExpiry(NodeId, NodeId, Blockstamp, u64),
    /// Write block
    WriteBlock(DALBlock),
    /// Revert block
    RevertBlock(DALBlock),
}

impl DBWriteRequest {
    pub fn apply(&self, currency: &str, db: &DuniterDB) {
        match *self {
            DBWriteRequest::CreateIdentity(
                ref wotb_id,
                ref blockstamp,
                ref median_time,
                ref idty_doc,
            ) => {
                trace!("DBWriteRequest::CreateIdentity...");
                let idty = DALIdentity::create_identity(db, idty_doc, blockstamp.clone());
                super::identity::write(&idty, wotb_id, db, blockstamp.clone(), *median_time);
                trace!("DBWriteRequest::CreateIdentity...finish.");
            }
            DBWriteRequest::RenewalIdentity(ref pubkey, ref blockstamp, ref median_time) => {
                trace!("DBWriteRequest::RenewalIdentity...");
                let mut idty = DALIdentity::get_identity(currency, db, pubkey)
                    .expect("Fatal error : impossible ton renewal an identidy that don't exist !");
                idty.renewal_identity(db, pubkey, blockstamp, *median_time, false);
                trace!("DBWriteRequest::RenewalIdentity...");
            }
            DBWriteRequest::ExcludeIdentity(ref wotb_id, ref blockstamp, ref _median_time) => {
                DALIdentity::exclude_identity(db, *wotb_id, *blockstamp, false);
            }
            DBWriteRequest::RevokeIdentity(ref wotb_id, ref blockstamp, ref _median_time) => {
                DALIdentity::revoke_identity(db, *wotb_id, blockstamp, false);
            }
            DBWriteRequest::CreateCert(ref blockstamp, ref median_time, ref compact_cert) => {
                trace!("DBWriteRequest::CreateCert...");
                super::certification::write_certification(
                    compact_cert,
                    db,
                    blockstamp.clone(),
                    *median_time,
                );
                trace!("DBWriteRequest::CreateCert...finish");
            }
            DBWriteRequest::WriteBlock(ref dal_block) => {
                trace!("DBWriteRequest::WriteBlock...");
                super::block::write(db, &dal_block.block, dal_block.fork, dal_block.isolate);
                trace!("DBWriteRequest::WriteBlock...finish");
            }
            _ => {}
        }
    }
}
