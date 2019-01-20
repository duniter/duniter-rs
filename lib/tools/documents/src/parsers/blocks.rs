//  Copyright (C) 2018  The Durs Project Developers.
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

use crate::documents::block::BlockDocument;
use crate::documents::membership::MembershipType;
use crate::parsers::*;
use crate::*;
use dup_crypto::hashs::Hash;
use dup_crypto::keys::*;
use failure::Error;
use json_pest_parser::JSONValue;
use std::collections::HashMap;

pub fn parse_json_block(json_block: &JSONValue) -> Result<BlockDocument, Error> {
    if !json_block.is_object() {
        return Err(ParseBlockError {
            cause: "Json block must be an object !".to_owned(),
        }
        .into());
    }

    let json_block = json_block.to_object().expect("safe unwrap");

    let currency = get_str(json_block, "currency")?;

    Ok(BlockDocument {
        version: get_number(json_block, "version")?.trunc() as u32,
        nonce: get_number(json_block, "nonce")?.trunc() as u64,
        number: BlockId(get_number(json_block, "number")?.trunc() as u32),
        pow_min: get_number(json_block, "powMin")?.trunc() as usize,
        time: get_number(json_block, "time")?.trunc() as u64,
        median_time: get_number(json_block, "medianTime")?.trunc() as u64,
        members_count: get_number(json_block, "membersCount")?.trunc() as usize,
        monetary_mass: get_number(json_block, "monetaryMass")?.trunc() as usize,
        unit_base: get_number(json_block, "unitbase")?.trunc() as usize,
        issuers_count: get_number(json_block, "issuersCount")?.trunc() as usize,
        issuers_frame: get_number(json_block, "issuersFrame")?.trunc() as isize,
        issuers_frame_var: get_number(json_block, "issuersFrameVar")?.trunc() as isize,
        currency: CurrencyName(currency.to_owned()),
        issuers: vec![PubKey::Ed25519(ed25519::PublicKey::from_base58(get_str(
            json_block, "issuer",
        )?)?)],
        signatures: vec![Sig::Ed25519(ed25519::Signature::from_base64(get_str(
            json_block,
            "signature",
        )?)?)],
        hash: Some(BlockHash(Hash::from_hex(get_str(json_block, "hash")?)?)),
        parameters: None,
        previous_hash: Hash::from_hex(get_str(json_block, "previousHash")?)?,
        previous_issuer: Some(PubKey::Ed25519(ed25519::PublicKey::from_base58(get_str(
            json_block,
            "previousIssuer",
        )?)?)),
        inner_hash: Some(Hash::from_hex(get_str(json_block, "inner_hash")?)?),
        dividend: get_optional_usize(json_block, "dividend")?,
        identities: crate::parsers::identities::parse_compact_identities(
            currency,
            get_str_array(json_block, "identities")?,
        )?,
        joiners: crate::parsers::memberships::parse_compact_memberships(
            currency,
            MembershipType::In(),
            &get_str_array(json_block, "joiners")?,
        )?,
        actives: crate::parsers::memberships::parse_compact_memberships(
            currency,
            MembershipType::In(),
            &get_str_array(json_block, "actives")?,
        )?,
        leavers: crate::parsers::memberships::parse_compact_memberships(
            currency,
            MembershipType::Out(),
            &get_str_array(json_block, "leavers")?,
        )?,
        revoked: crate::parsers::revoked::parse_revocations_into_compact(&get_str_array(
            json_block, "revoked",
        )?),
        excluded: crate::parsers::excluded::parse_excluded(&get_str_array(
            json_block, "excluded",
        )?)?,
        certifications: crate::parsers::certifications::parse_certifications_into_compact(
            &get_str_array(json_block, "certifications")?,
        ),
        transactions: vec![],
        inner_hash_and_nonce_str: "".to_owned(),
    })
}

fn get_optional_usize(
    json_block: &HashMap<&str, JSONValue>,
    field: &str,
) -> Result<Option<usize>, ParseBlockError> {
    Ok(match json_block.get(field) {
        Some(value) => {
            if !value.is_null() {
                Some(
                    value
                        .to_number()
                        .ok_or_else(|| ParseBlockError {
                            cause: format!("Json block {} field must be a number !", field),
                        })?
                        .trunc() as usize,
                )
            } else {
                None
            }
        }
        None => None,
    })
}

fn get_number(json_block: &HashMap<&str, JSONValue>, field: &str) -> Result<f64, ParseBlockError> {
    Ok(json_block
        .get(field)
        .ok_or_else(|| ParseBlockError {
            cause: format!("Json block must have {} field !", field),
        })?
        .to_number()
        .ok_or_else(|| ParseBlockError {
            cause: format!("Json block {} field must be a number !", field),
        })?)
}

fn get_str<'a>(
    json_block: &'a HashMap<&str, JSONValue>,
    field: &str,
) -> Result<&'a str, ParseBlockError> {
    Ok(json_block
        .get(field)
        .ok_or_else(|| ParseBlockError {
            cause: format!("Json block must have {} field !", field),
        })?
        .to_str()
        .ok_or_else(|| ParseBlockError {
            cause: format!("Json block {} field must be a string !", field),
        })?)
}

fn get_str_array<'a>(
    json_block: &'a HashMap<&str, JSONValue>,
    field: &str,
) -> Result<Vec<&'a str>, ParseBlockError> {
    json_block
        .get(field)
        .ok_or_else(|| ParseBlockError {
            cause: format!("Json block must have {} field !", field),
        })?
        .to_array()
        .ok_or_else(|| ParseBlockError {
            cause: format!("Json block {} field must be an array !", field),
        })?
        .iter()
        .map(|v| {
            v.to_str().ok_or_else(|| ParseBlockError {
                cause: format!("Json block {} field must be an array of string !", field),
            })
        })
        .collect()
}
