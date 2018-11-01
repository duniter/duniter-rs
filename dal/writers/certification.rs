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

use currency_params::CurrencyParameters;
use duniter_documents::v10::certification::CompactCertificationDocument;
use duniter_documents::BlockId;
use dup_crypto::keys::*;
use durs_wot::NodeId;
use {BinDB, CertsExpirV10Datas, DALError, IdentitiesV10Datas};

/// Apply "certification" event in databases
pub fn write_certification(
    currency_params: &CurrencyParameters,
    identities_db: &BinDB<IdentitiesV10Datas>,
    certs_db: &BinDB<CertsExpirV10Datas>,
    source_pubkey: PubKey,
    source: NodeId,
    target: NodeId,
    created_block_id: BlockId,
    written_timestamp: u64,
) -> Result<(), DALError> {
    // Get cert_chainable_on
    let mut member_datas = identities_db.read(|db| {
        db.get(&source_pubkey)
            .expect("Database Corrupted, please reset data !")
            .clone()
    })?;
    // Push new cert_chainable_on
    member_datas
        .cert_chainable_on
        .push(written_timestamp + currency_params.sig_period);
    // Write new identity datas
    identities_db.write(|db| {
        db.insert(source_pubkey, member_datas);
    })?;
    // Add cert in certs_db
    certs_db.write(|db| {
        let mut created_certs = db.get(&created_block_id).cloned().unwrap_or_default();
        created_certs.insert((source, target));
        db.insert(created_block_id, created_certs);
    })?;
    Ok(())
}

/// Revert writtent certification
pub fn revert_write_cert(
    identities_db: &BinDB<IdentitiesV10Datas>,
    certs_db: &BinDB<CertsExpirV10Datas>,
    compact_doc: CompactCertificationDocument,
    source: NodeId,
    target: NodeId,
) -> Result<(), DALError> {
    // Remove CertsExpirV10Datas entry
    certs_db.write(|db| {
        let mut certs = db
            .get(&compact_doc.block_number)
            .cloned()
            .unwrap_or_default();
        certs.remove(&(source, target));
        db.insert(compact_doc.block_number, certs);
    })?;
    // Pop last cert_chainable_on
    identities_db.write(|db| {
        if let Some(mut member_datas) = db.get(&compact_doc.issuer).cloned() {
            member_datas.cert_chainable_on.pop();
            db.insert(compact_doc.issuer, member_datas);
        }
    })?;
    Ok(())
}

/// Revert "certification expiry" event in databases
pub fn revert_expire_cert(
    certs_db: &BinDB<CertsExpirV10Datas>,
    source: NodeId,
    target: NodeId,
    created_block_id: BlockId,
) -> Result<(), DALError> {
    // Reinsert CertsExpirV10Datas entry
    certs_db.write(|db| {
        let mut certs = db.get(&created_block_id).cloned().unwrap_or_default();
        certs.insert((source, target));
        db.insert(created_block_id, certs);
    })?;
    Ok(())
}

/// Apply "certification expiry" event in databases
pub fn expire_certs(
    certs_db: &BinDB<CertsExpirV10Datas>,
    created_block_id: BlockId,
) -> Result<(), DALError> {
    // Remove CertsExpirV10Datas entries
    certs_db.write(|db| {
        db.remove(&created_block_id);
    })?;
    Ok(())
}
