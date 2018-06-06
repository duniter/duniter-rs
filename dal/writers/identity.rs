use super::super::identity::DALIdentity;
use duniter_documents::blockchain::Document;
use duniter_documents::BlockId;
use duniter_wotb::NodeId;
use {BinFileDB, DALError, IdentitiesV10Datas, MsExpirV10Datas};

pub fn write(
    idty: &DALIdentity,
    idty_wot_id: NodeId,
    identities_db: &BinFileDB<IdentitiesV10Datas>,
    ms_db: &BinFileDB<MsExpirV10Datas>,
    ms_created_block_id: BlockId,
) -> Result<(), DALError> {
    // Write Identity
    identities_db.write(|db| {
        db.insert(idty.idty_doc.issuers()[0], idty.clone());
    })?;
    // Update IdentitiesV10DB
    ms_db.write(|db| {
        let mut memberships = db.get(&ms_created_block_id).cloned().unwrap_or_default();
        memberships.insert(idty_wot_id);
        db.insert(ms_created_block_id, memberships);
    })?;
    Ok(())
}
