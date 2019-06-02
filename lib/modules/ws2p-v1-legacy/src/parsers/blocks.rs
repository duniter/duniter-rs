use super::excluded::parse_exclusions_from_json_value;
use super::identities::parse_compact_identity;
use super::transactions::parse_transaction;
use dubp_documents::documents::block::BlockDocument;
use dubp_documents::documents::block::TxDocOrTxHash;
use dubp_documents::documents::membership::*;
use dubp_documents::parsers::certifications::*;
use dubp_documents::parsers::revoked::*;
use dubp_documents::{BlockHash, BlockNumber};
use dup_crypto::hashs::Hash;
use dup_crypto::keys::*;
use dup_currency_params::genesis_block_params::v10::BlockV10Parameters;
use dup_currency_params::CurrencyName;
use std::str::FromStr;

fn parse_previous_hash(block_number: BlockNumber, source: &serde_json::Value) -> Option<Hash> {
    match source.get("previousHash")?.as_str() {
        Some(hash_str) => match Hash::from_hex(hash_str) {
            Ok(hash) => Some(hash),
            Err(_) => None,
        },
        None => {
            if block_number.0 > 0 {
                None
            } else {
                Some(Hash::default())
            }
        }
    }
}

fn parse_previous_issuer(source: &serde_json::Value) -> Option<PubKey> {
    match source.get("previousIssuer")?.as_str() {
        Some(pubkey_str) => match ed25519::PublicKey::from_base58(pubkey_str) {
            Ok(pubkey) => Some(PubKey::Ed25519(pubkey)),
            Err(_) => None,
        },
        None => None,
    }
}

fn parse_memberships(
    currency: &str,
    membership_type: MembershipType,
    json_memberships: &serde_json::Value,
) -> Option<Vec<MembershipDocument>> {
    let mut memberships = Vec::new();
    for membership in super::memberships::parse_memberships_from_json_value(
        currency,
        membership_type,
        &json_memberships.as_array()?,
    ) {
        if let Ok(membership) = membership {
            memberships.push(membership);
        } else {
            warn!("dal::parsers::blocks::parse_memberships() : MembershipParseError !")
        }
    }
    Some(memberships)
}

pub fn parse_json_block(source: &serde_json::Value) -> Option<BlockDocument> {
    let number = BlockNumber(source.get("number")?.as_u64()? as u32);
    let currency = source.get("currency")?.as_str()?.to_string();
    let issuer = match ed25519::PublicKey::from_base58(source.get("issuer")?.as_str()?) {
        Ok(pubkey) => PubKey::Ed25519(pubkey),
        Err(_) => return None,
    };
    let sig = match ed25519::Signature::from_base64(source.get("signature")?.as_str()?) {
        Ok(sig) => Sig::Ed25519(sig),
        Err(_) => return None,
    };
    let hash = match Hash::from_hex(source.get("hash")?.as_str()?) {
        Ok(hash) => hash,
        Err(_) => return None,
    };
    let parameters = if let Some(params_json) = source.get("parameters") {
        if let Ok(params) = BlockV10Parameters::from_str(params_json.as_str()?) {
            Some(params)
        } else {
            None
        }
    } else {
        None
    };
    let previous_hash = parse_previous_hash(number, source)?;
    let previous_issuer = parse_previous_issuer(source);
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
    let joiners = parse_memberships(&currency, MembershipType::In(), source.get("joiners")?)?;
    let actives = parse_memberships(&currency, MembershipType::In(), source.get("actives")?)?;
    let leavers = parse_memberships(&currency, MembershipType::Out(), source.get("leavers")?)?;
    let revoked: Vec<&str> = source
        .get("revoked")?
        .as_array()?
        .iter()
        .map(|v| v.as_str().unwrap_or(""))
        .collect();
    let certifications: Vec<&str> = source
        .get("certifications")?
        .as_array()?
        .iter()
        .map(|v| v.as_str().unwrap_or(""))
        .collect();
    let mut transactions = Vec::new();
    for json_tx in source.get("transactions")?.as_array()? {
        transactions.push(TxDocOrTxHash::TxDoc(Box::new(parse_transaction(
            "g1", &json_tx,
        )?)));
    }
    Some(BlockDocument {
        nonce: source.get("nonce")?.as_i64()? as u64,
        version: source.get("version")?.as_u64()? as u32,
        number: BlockNumber(source.get("number")?.as_u64()? as u32),
        pow_min: source.get("powMin")?.as_u64()? as usize,
        time: source.get("time")?.as_u64()?,
        median_time: source.get("medianTime")?.as_u64()?,
        members_count: source.get("membersCount")?.as_u64()? as usize,
        monetary_mass: source.get("monetaryMass")?.as_u64()? as usize,
        unit_base: source.get("unitbase")?.as_u64()? as usize,
        issuers_count: source.get("issuersCount")?.as_u64()? as usize,
        issuers_frame: source.get("issuersFrame")?.as_i64()? as isize,
        issuers_frame_var: source.get("issuersFrameVar")?.as_i64()? as isize,
        currency: CurrencyName(currency),
        issuers: vec![issuer],
        signatures: vec![sig],
        hash: Some(BlockHash(hash)),
        parameters,
        previous_hash,
        previous_issuer,
        inner_hash,
        dividend,
        identities,
        joiners,
        actives,
        leavers,
        revoked: parse_revocations_into_compact(&revoked),
        excluded: parse_exclusions_from_json_value(&source.get("excluded")?.as_array()?),
        certifications: parse_certifications_into_compact(&certifications),
        transactions,
        inner_hash_and_nonce_str: format!(
            "InnerHash: {}\nNonce: {}\n",
            inner_hash
                .expect("Try to get inner_hash of an uncompleted or reduce block !")
                .to_hex(),
            source.get("nonce")?.as_u64()?
        ),
    })
}
