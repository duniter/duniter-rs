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

use crate::sync::BlockHeader;
use dubp_documents::documents::block::{BlockDocument, BlockV10Parameters, TxDocOrTxHash};
use dubp_documents::documents::identity::*;
use dubp_documents::documents::membership::*;
use dubp_documents::documents::transaction::*;
use dubp_documents::CurrencyName;
use dubp_documents::DocumentBuilder;
use dubp_documents::{BlockHash, BlockId, Blockstamp};
use dup_crypto::hashs::Hash;
use dup_crypto::keys::*;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// MembershipParseError
pub enum MembershipParseError {
    WrongFormat(),
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
            )
            .iter()
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
        match TransactionInput::from_str(input_str) {
            Ok(input) => inputs.push(input),
            Err(_) => {
                return None;
            }
        }
    }
    let unlocks_array = source.get("unlocks")?.as_array()?;
    let mut unlocks = Vec::with_capacity(unlocks_array.len());
    for unlock in unlocks_array {
        match TransactionInputUnlocks::from_str(unlock.as_str()?) {
            Ok(unlock) => unlocks.push(unlock),
            Err(_) => {
                return None;
            }
        }
    }
    let outputs_array = source.get("outputs")?.as_array()?;
    let mut outputs = Vec::with_capacity(outputs_array.len());
    for output in outputs_array {
        outputs.push(
            TransactionOutput::from_str(
                output
                    .as_str()
                    .unwrap_or_else(|| panic!("Fail to parse output : {:?}", output)),
            )
            .unwrap_or_else(|_| panic!("Fail to parse output : {:?}", output)),
        );
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
