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

extern crate serde_json;
extern crate sqlite;

use duniter_crypto::keys::*;
use duniter_documents::blockchain::v10::documents::block::{
    BlockV10Parameters, CurrencyName, TxDocOrTxHash,
};
use duniter_documents::blockchain::v10::documents::identity::IdentityDocumentBuilder;
use duniter_documents::blockchain::v10::documents::membership::*;
use duniter_documents::blockchain::v10::documents::transaction::*;
use duniter_documents::blockchain::v10::documents::*;
use duniter_documents::blockchain::DocumentBuilder;
use duniter_documents::{BlockHash, BlockId, Blockstamp, Hash};
use duniter_network::{NetworkBlock, NetworkBlockV10};
use std::str::FromStr;
use sync::BlockHeader;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// MembershipParseError
pub enum MembershipParseError {
    WrongFormat(),
}

/// Parse a block from duniter-ts database
pub fn parse_ts_block(row: &[sqlite::Value]) -> NetworkBlock {
    let current_header = BlockHeader {
        number: BlockId(row[16].as_integer().expect("Fail to parse block number") as u32),
        hash: BlockHash(
            Hash::from_hex(row[0].as_string().expect("Fail to parse block hash"))
                .expect("Fail to parse block hash (2)"),
        ),
        issuer: PubKey::Ed25519(
            ed25519::PublicKey::from_base58(
                row[4].as_string().expect("Fail to parse block issuer"),
            ).expect("Failt to parse block issuer (2)"),
        ),
    };
    let previous_header = if current_header.number.0 > 0 {
        Some(BlockHeader {
            number: BlockId(current_header.number.0 - 1),
            hash: BlockHash(
                Hash::from_hex(
                    row[6]
                        .as_string()
                        .expect("Fail to parse block previous hash"),
                ).expect("Fail to parse block previous hash (2)"),
            ),
            issuer: PubKey::Ed25519(
                ed25519::PublicKey::from_base58(
                    row[7]
                        .as_string()
                        .expect("Fail to parse previous block issuer"),
                ).expect("Fail to parse previous block issuer (2)"),
            ),
        })
    } else {
        None
    };
    let currency = row[3].as_string().expect("Fail to parse currency");
    let parameters = if let Some(params_str) = row[5].as_string() {
        if let Ok(params) = BlockV10Parameters::from_str(params_str) {
            Some(params)
        } else {
            None
        }
    } else {
        None
    };
    let dividend = match row[12].as_integer() {
        Some(dividend) => Some(dividend as usize),
        None => None,
    };
    let json_identities: serde_json::Value = serde_json::from_str(
        row[20].as_string().expect("Fail to parse block identities"),
    ).expect("Fail to parse block identities (2)");
    let mut identities = Vec::new();
    for raw_idty in json_identities
        .as_array()
        .expect("Fail to parse block identities (3)")
    {
        identities
            .push(parse_compact_identity(&currency, &raw_idty).expect("Fail to parse block idty"));
    }
    let json_txs: serde_json::Value = serde_json::from_str(
        row[18].as_string().expect("Fail to parse block txs"),
    ).expect("Fail to parse block txs (2)");
    let mut transactions = Vec::new();
    for json_tx in json_txs.as_array().expect("Fail to parse block txs (3)") {
        transactions.push(TxDocOrTxHash::TxDoc(Box::new(
            parse_transaction(currency, &json_tx).expect("Fail to parse block tx"),
        )));
    }
    let previous_hash = match previous_header.clone() {
        Some(previous_header_) => previous_header_.hash.0,
        None => Hash::default(),
    };
    let previous_issuer = match previous_header {
        Some(previous_header_) => Some(previous_header_.issuer),
        None => None,
    };
    let excluded: serde_json::Value = serde_json::from_str(
        row[25].as_string().expect("Fail to parse excluded"),
    ).expect("Fail to parse excluded (2)");
    let uncompleted_block_doc = BlockDocument {
        nonce: row[17].as_integer().expect("Fail to parse nonce") as u64,
        number: current_header.number,
        pow_min: row[15].as_integer().expect("Fail to parse pow_min") as usize,
        time: row[14].as_integer().expect("Fail to parse time") as u64,
        median_time: row[11].as_integer().expect("Fail to parse median_time") as u64,
        members_count: row[9].as_integer().expect("Fail to parse members_count") as usize,
        monetary_mass: row[10]
            .as_string()
            .expect("Fail to parse monetary_mass")
            .parse()
            .expect("Fail to parse monetary_mass (2)"),
        unit_base: row[13].as_integer().expect("Fail to parse unit_base") as usize,
        issuers_count: row[28].as_integer().expect("Fail to parse issuers_count") as usize,
        issuers_frame: row[26].as_integer().expect("Fail to parse issuers_frame") as isize,
        issuers_frame_var: row[27]
            .as_integer()
            .expect("Fail to parse issuers_frame_var") as isize,
        currency: CurrencyName(String::from(currency)),
        issuers: vec![PubKey::Ed25519(
            ed25519::PublicKey::from_base58(row[4].as_string().expect("Fail to parse issuer"))
                .expect("Fail to parse issuer '2)"),
        )],
        signatures: vec![Sig::Ed25519(
            ed25519::Signature::from_base64(row[2].as_string().expect("Fail to parse signature"))
                .expect("Fail to parse signature (2)"),
        )],
        hash: Some(current_header.hash),
        parameters,
        previous_hash,
        previous_issuer,
        inner_hash: Some(
            Hash::from_hex(row[1].as_string().expect("Fail to parse block inner_hash"))
                .expect("Fail to parse block inner_hash (2)"),
        ),
        dividend,
        identities,
        joiners: parse_memberships(
            currency,
            MembershipType::In(),
            row[21].as_string().expect("Fail to parse joiners"),
        ).expect("Fail to parse joiners (2)"),
        actives: parse_memberships(
            currency,
            MembershipType::In(),
            row[22].as_string().expect("Fail to parse actives"),
        ).expect("Fail to parse actives (2)"),
        leavers: parse_memberships(
            currency,
            MembershipType::In(),
            row[23].as_string().expect("Fail to parse leavers"),
        ).expect("Fail to parse leavers (2)"),
        revoked: Vec::new(),
        excluded: excluded
            .as_array()
            .expect("Fail to parse excluded (3)")
            .to_vec()
            .into_iter()
            .map(|e| {
                PubKey::Ed25519(
                    ed25519::PublicKey::from_base58(
                        e.as_str().expect("Fail to parse excluded (4)"),
                    ).expect("Fail to parse excluded (5)"),
                )
            })
            .collect(),
        certifications: Vec::new(),
        transactions,
        inner_hash_and_nonce_str: String::new(),
    };
    let revoked: serde_json::Value = serde_json::from_str(
        row[24].as_string().expect("Fail to parse revoked"),
    ).expect("Fail to parse revoked (2)");
    let certifications: serde_json::Value = serde_json::from_str(
        row[19].as_string().expect("Fail to parse certifications"),
    ).expect("Fail to parse certifications (2)");
    // return NetworkBlock
    NetworkBlock::V10(Box::new(NetworkBlockV10 {
        uncompleted_block_doc,
        revoked: revoked
            .as_array()
            .expect("Fail to parse revoked (3)")
            .to_vec(),
        certifications: certifications
            .as_array()
            .expect("Fail to parse certifications (3)")
            .to_vec(),
    }))
}

/// Parse a compact identity
pub fn parse_compact_identity(
    currency: &str,
    source: &serde_json::Value,
) -> Option<IdentityDocument> {
    if source.is_string() {
        let idty_elements: Vec<&str> = source.as_str().unwrap().split(':').collect();
        let issuer = match ed25519::PublicKey::from_base58(idty_elements[0]) {
            Ok(pubkey) => PubKey::Ed25519(pubkey),
            Err(_) => return None,
        };
        let signature = match ed25519::Signature::from_base64(idty_elements[1]) {
            Ok(sig) => Sig::Ed25519(sig),
            Err(_) => return None,
        };
        let blockstamp = match Blockstamp::from_string(idty_elements[2]) {
            Ok(blockstamp) => blockstamp,
            Err(_) => return None,
        };
        let username = idty_elements[3];
        let idty_doc_builder = IdentityDocumentBuilder {
            currency,
            username,
            blockstamp: &blockstamp,
            issuer: &issuer,
        };
        Some(idty_doc_builder.build_with_signature(vec![signature]))
    } else {
        None
    }
}

/// Parse memberships documents from json string
pub fn parse_memberships(
    currency: &str,
    membership_type: MembershipType,
    json_datas: &str,
) -> Option<Vec<MembershipDocument>> {
    let raw_memberships: serde_json::Value = serde_json::from_str(json_datas).unwrap();
    if raw_memberships.is_array() {
        return Some(
            parse_memberships_from_json_value(
                currency,
                membership_type,
                raw_memberships.as_array().unwrap(),
            ).iter()
                .map(|m| {
                    m.clone()
                        .expect("Fatal error : Fail to parse membership from local DB !")
                })
                .collect(),
        );
    }
    None
}

/// Parse memberships documents from array of json values
pub fn parse_memberships_from_json_value(
    currency: &str,
    membership_type: MembershipType,
    array_memberships: &[serde_json::Value],
) -> Vec<Result<MembershipDocument, MembershipParseError>> {
    //let memberships: Vec<MembershipDocument> = Vec::new();
    array_memberships
        .iter()
        .map(|membership| {
            let membership_datas: Vec<&str> = membership.as_str().unwrap().split(':').collect();
            if membership_datas.len() == 5 {
                let membership_doc_builder = MembershipDocumentBuilder {
                    currency,
                    issuer: &PubKey::Ed25519(
                        ed25519::PublicKey::from_base58(membership_datas[0]).unwrap(),
                    ),
                    blockstamp: &Blockstamp::from_string(membership_datas[2]).unwrap(),
                    membership: membership_type,
                    identity_username: membership_datas[4],
                    identity_blockstamp: &Blockstamp::from_string(membership_datas[3]).unwrap(),
                };
                let membership_sig =
                    Sig::Ed25519(ed25519::Signature::from_base64(membership_datas[1]).unwrap());
                Ok(membership_doc_builder.build_with_signature(vec![membership_sig]))
            } else {
                Err(MembershipParseError::WrongFormat())
            }
        })
        .collect()
}

/// Parse transaction from json value
pub fn parse_transaction(
    currency: &str,
    source: &serde_json::Value,
) -> Option<TransactionDocument> {
    //debug!("transaction={:#?}", source);
    let blockstamp = match Blockstamp::from_string(source.get("blockstamp")?.as_str()?) {
        Ok(blockstamp) => blockstamp,
        Err(_) => {
            return None;
        }
    };
    let locktime = source.get("locktime")?.as_i64()? as u64;
    let issuers_array = source.get("issuers")?.as_array()?;
    let mut issuers = Vec::with_capacity(issuers_array.len());
    for issuer in issuers_array {
        match ed25519::PublicKey::from_base58(issuer.as_str()?) {
            Ok(pubkey) => issuers.push(PubKey::Ed25519(pubkey)),
            Err(_) => {
                return None;
            }
        }
    }
    let inputs_array = source.get("inputs")?.as_array()?;
    let mut inputs = Vec::with_capacity(inputs_array.len());
    for input in inputs_array {
        let input_str = input.as_str()?;
        match TransactionInput::parse_from_str(input_str) {
            Ok(input) => inputs.push(input),
            Err(_) => {
                return None;
            }
        }
    }
    let unlocks_array = source.get("unlocks")?.as_array()?;
    let mut unlocks = Vec::with_capacity(unlocks_array.len());
    for unlock in unlocks_array {
        match TransactionInputUnlocks::parse_from_str(unlock.as_str()?) {
            Ok(unlock) => unlocks.push(unlock),
            Err(_) => {
                return None;
            }
        }
    }
    let outputs_array = source.get("outputs")?.as_array()?;
    let mut outputs = Vec::with_capacity(outputs_array.len());
    for output in outputs_array {
        match TransactionOutput::parse_from_str(
            output
                .as_str()
                .expect(&format!("Fail to parse output : {:?}", output)),
        ) {
            Ok(output) => outputs.push(output),
            Err(_) => panic!("Fail to parse output : {:?}", output),
        }
    }
    let signatures_array = source.get("signatures")?.as_array()?;
    let mut signatures = Vec::with_capacity(signatures_array.len());
    for signature in signatures_array {
        match ed25519::Signature::from_base64(signature.as_str()?) {
            Ok(signature) => signatures.push(Sig::Ed25519(signature)),
            Err(_) => {
                return None;
            }
        }
    }
    let comment = source.get("comment")?.as_str()?;

    let tx_doc_builder = TransactionDocumentBuilder {
        currency,
        blockstamp: &blockstamp,
        locktime: &locktime,
        issuers: &issuers,
        inputs: &inputs,
        unlocks: &unlocks,
        outputs: &outputs,
        comment,
        hash: Some(Hash::from_hex(source.get("hash")?.as_str()?).expect("Fail to parse tx hash")),
    };
    Some(tx_doc_builder.build_with_signature(signatures))
}
