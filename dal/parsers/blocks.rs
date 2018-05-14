extern crate duniter_crypto;
extern crate duniter_documents;
extern crate duniter_network;
extern crate serde_json;

use self::duniter_network::{NetworkBlock, NetworkBlockV10};
use super::excluded::parse_exclusions_from_json_value;
use super::identities::parse_compact_identity;
use super::transactions::parse_transaction;
use duniter_crypto::keys::{PublicKey, Signature};
use duniter_documents::blockchain::v10::documents::BlockDocument;
use duniter_documents::{BlockHash, BlockId, Hash};

pub fn parse_json_block(source: &serde_json::Value) -> Option<NetworkBlock> {
    let number = BlockId(source.get("number")?.as_u64()? as u32);
    let currency = source.get("currency")?.as_str()?.to_string();
    let issuer = match PublicKey::from_base58(source.get("issuer")?.as_str()?) {
        Ok(pubkey) => pubkey,
        Err(_) => return None,
    };
    let sig = match Signature::from_base64(source.get("signature")?.as_str()?) {
        Ok(sig) => sig,
        Err(_) => return None,
    };
    let hash = match Hash::from_hex(source.get("hash")?.as_str()?) {
        Ok(hash) => hash,
        Err(_) => return None,
    };
    let previous_hash = match source.get("previousHash")?.as_str() {
        Some(hash_str) => match Hash::from_hex(hash_str) {
            Ok(hash) => hash,
            Err(_) => return None,
        },
        None => if number.0 > 0 {
            return None;
        } else {
            Hash::default()
        },
    };
    let previous_issuer = match source.get("previousIssuer")?.as_str() {
        Some(pubkey_str) => match PublicKey::from_base58(pubkey_str) {
            Ok(pubkey) => Some(pubkey),
            Err(_) => return None,
        },
        None => if number.0 > 0 {
            return None;
        } else {
            None
        },
    };
    let inner_hash = match Hash::from_hex(source.get("inner_hash")?.as_str()?) {
        Ok(hash) => Some(hash),
        Err(_) => return None,
    };
    let dividend = match source.get("dividend")?.as_u64() {
        Some(dividend) => Some(dividend as usize),
        None => None,
    };
    let mut identities = Vec::new();
    for raw_idty in source.get("identities")?.as_array()? {
        identities.push(parse_compact_identity(&currency, &raw_idty)?);
    }
    let mut transactions = Vec::new();
    for json_tx in source.get("transactions")?.as_array()? {
        transactions.push(parse_transaction("g1", &json_tx)?);
    }
    let block_doc = BlockDocument {
        nonce: source.get("nonce")?.as_i64()? as u64,
        number: BlockId(source.get("number")?.as_u64()? as u32),
        pow_min: source.get("powMin")?.as_u64()? as usize,
        time: source.get("time")?.as_u64()?,
        median_time: source.get("medianTime")?.as_u64()?,
        members_count: source.get("membersCount")?.as_u64()? as usize,
        monetary_mass: source.get("monetaryMass")?.as_u64()? as usize,
        unit_base: source.get("unitbase")?.as_u64()? as usize,
        issuers_count: source.get("issuersCount")?.as_u64()? as usize,
        issuers_frame: source.get("issuersFrame")?.as_i64()? as isize,
        issuers_frame_var: source.get("issuersFrameVar")?.as_i64()? as isize,
        currency,
        issuers: vec![issuer],
        signatures: vec![sig],
        hash: Some(BlockHash(hash)),
        parameters: None,
        previous_hash,
        previous_issuer,
        inner_hash,
        dividend,
        identities,
        joiners: Vec::with_capacity(0),
        actives: Vec::with_capacity(0),
        leavers: Vec::with_capacity(0),
        revoked: Vec::with_capacity(0),
        excluded: parse_exclusions_from_json_value(&source.get("excluded")?.as_array()?),
        certifications: Vec::with_capacity(0),
        transactions,
        inner_hash_and_nonce_str: format!(
            "InnerHash: {}\nNonce: {}\n",
            inner_hash.unwrap().to_hex(),
            source.get("nonce")?.as_u64()?
        ),
    };
    Some(NetworkBlock::V10(Box::new(NetworkBlockV10 {
        uncompleted_block_doc: block_doc,
        joiners: source.get("joiners")?.as_array()?.clone(),
        actives: source.get("actives")?.as_array()?.clone(),
        leavers: source.get("leavers")?.as_array()?.clone(),
        revoked: source.get("revoked")?.as_array()?.clone(),
        certifications: source.get("certifications")?.as_array()?.clone(),
    })))
}
