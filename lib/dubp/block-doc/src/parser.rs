//  Copyright (C) 2017-2019  The AXIOM TEAM Association.
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

//! Parsers for block.

use crate::block::{BlockDocument, BlockDocumentV10};
use dubp_common_doc::{BlockHash, BlockNumber};
use dubp_currency_params::genesis_block_params::v10::BlockV10Parameters;
use dubp_currency_params::CurrencyName;
use dubp_user_docs::documents::membership::v10::MembershipType;
use dubp_user_docs::parsers::{serde_json_value_to_pest_json_value, DefaultHasher};
use dup_crypto::bases::BaseConvertionError;
use dup_crypto::hashs::Hash;
use dup_crypto::keys::*;
use durs_common_tools::UsizeSer32;
use failure::Error;
use json_pest_parser::*;
use std::convert::TryFrom;
use std::str::FromStr;

pub fn parse_json_block_from_serde_value(
    serde_json_value: &serde_json::Value,
) -> Result<BlockDocument, Error> {
    parse_json_block(&serde_json_value_to_pest_json_value(serde_json_value)?)
}

pub fn parse_json_block(json_block: &JSONValue<DefaultHasher>) -> Result<BlockDocument, Error> {
    if !json_block.is_object() {
        return Err(ParseJsonError {
            cause: "Json block must be an object !".to_owned(),
        }
        .into());
    }

    let json_block = json_block.to_object().expect("safe unwrap");

    let currency = get_str(json_block, "currency")?;

    let block_number = u32::try_from(get_u64(json_block, "number")?)?;

    Ok(BlockDocument::V10(BlockDocumentV10 {
        version: UsizeSer32(get_u64(json_block, "version")? as usize),
        nonce: get_u64(json_block, "nonce")?,
        number: BlockNumber(block_number),
        pow_min: UsizeSer32(get_u64(json_block, "powMin")? as usize),
        time: get_u64(json_block, "time")?,
        median_time: get_u64(json_block, "medianTime")?,
        members_count: UsizeSer32(get_u64(json_block, "membersCount")? as usize),
        monetary_mass: get_u64(json_block, "monetaryMass").unwrap_or(0),
        unit_base: UsizeSer32(get_u64(json_block, "unitbase")? as usize),
        issuers_count: UsizeSer32(get_u64(json_block, "issuersCount")? as usize),
        issuers_frame: UsizeSer32(get_u64(json_block, "issuersFrame")? as usize),
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
        parameters: get_optional_str_not_empty(json_block, "parameters")?
            .map(BlockV10Parameters::from_str)
            .transpose()?,
        previous_hash: if block_number == 0 {
            None
        } else {
            Some(Hash::from_hex(get_str(json_block, "previousHash")?)?)
        },
        previous_issuer: if block_number == 0 {
            None
        } else {
            Some(PubKey::Ed25519(ed25519::PublicKey::from_base58(get_str(
                json_block,
                "previousIssuer",
            )?)?))
        },
        inner_hash: Some(Hash::from_hex(get_str(json_block, "inner_hash")?)?),
        dividend: get_optional_usize(json_block, "dividend")?.map(UsizeSer32),
        identities: dubp_user_docs::parsers::identities::parse_compact_identities(
            currency,
            get_str_array(json_block, "identities")?,
        )?,
        joiners: dubp_user_docs::parsers::memberships::parse_compact_memberships(
            currency,
            MembershipType::In(),
            &get_str_array(json_block, "joiners")?,
        )?,
        actives: dubp_user_docs::parsers::memberships::parse_compact_memberships(
            currency,
            MembershipType::In(),
            &get_str_array(json_block, "actives")?,
        )?,
        leavers: dubp_user_docs::parsers::memberships::parse_compact_memberships(
            currency,
            MembershipType::Out(),
            &get_str_array(json_block, "leavers")?,
        )?,
        revoked: dubp_user_docs::parsers::revoked::parse_revocations_into_compact(&get_str_array(
            json_block, "revoked",
        )?),
        excluded: get_str_array(json_block, "excluded")?
            .iter()
            .map(|p| ed25519::PublicKey::from_base58(p))
            .map(|p| p.map(PubKey::Ed25519))
            .collect::<Result<Vec<PubKey>, BaseConvertionError>>()?,
        certifications: dubp_user_docs::parsers::certifications::parse_certifications_into_compact(
            &get_str_array(json_block, "certifications")?,
        ),
        transactions: dubp_user_docs::parsers::transactions::parse_json_transactions(&get_array(
            json_block,
            "transactions",
        )?)?,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::*;
    use dubp_user_docs::documents::transaction::TransactionDocument;

    #[test]
    fn parse_empty_json_block() {
        let block_json_str = r#"{
   "version": 10,
   "nonce": 10200000037108,
   "number": 7,
   "powMin": 70,
   "time": 1488987677,
   "medianTime": 1488987394,
   "membersCount": 59,
   "monetaryMass": 59000,
   "unitbase": 0,
   "issuersCount": 1,
   "issuersFrame": 6,
   "issuersFrameVar": 0,
   "currency": "g1",
   "issuer": "2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ",
   "signature": "xaWNjdFeE4yr9+AKckgR6QuAvMzmKUWfY+uIlC3HKjn2apJqG70Gf59A71W+Ucz6E9WPXRzDDF/xOrf6GCGHCA==",
   "hash": "0000407900D981FC17B5A6FBCF8E8AFA4C00FAD7AFC5BEA9A96FF505E5D105EC",
   "parameters": "",
   "previousHash": "0000379BBE6ABC18DCFD6E4733F9F76CB06593D10FAEDF722BE190C277AC16EA",
   "previousIssuer": "2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ",
   "inner_hash": "CF2701092D5A34A55802E343B5F8D61D9B7E8089F1F13A19721234DF5B2F0F38",
   "dividend": null,
   "identities": [],
   "joiners": [],
   "actives": [],
   "leavers": [],
   "revoked": [],
   "excluded": [],
   "certifications": [],
   "transactions": [],
   "raw": "Version: 10\nType: Block\nCurrency: g1\nNumber: 7\nPoWMin: 70\nTime: 1488987677\nMedianTime: 1488987394\nUnitBase: 0\nIssuer: 2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ\nIssuersFrame: 6\nIssuersFrameVar: 0\nDifferentIssuersCount: 1\nPreviousHash: 0000379BBE6ABC18DCFD6E4733F9F76CB06593D10FAEDF722BE190C277AC16EA\nPreviousIssuer: 2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ\nMembersCount: 59\nIdentities:\nJoiners:\nActives:\nLeavers:\nRevoked:\nExcluded:\nCertifications:\nTransactions:\nInnerHash: CF2701092D5A34A55802E343B5F8D61D9B7E8089F1F13A19721234DF5B2F0F38\nNonce: 10200000037108\n"
  }"#;

        let block_json_value = json_pest_parser::parse_json_string(block_json_str)
            .expect("Fail to parse json block !");
        assert_eq!(
            BlockDocument::V10(BlockDocumentV10 {
                version: UsizeSer32(10),
                nonce: 10_200_000_037_108,
                number: BlockNumber(7),
                pow_min: UsizeSer32(70),
                time: 1_488_987_677,
                median_time: 1_488_987_394,
                members_count: UsizeSer32(59),
                monetary_mass: 59000,
                unit_base: UsizeSer32(0),
                issuers_count: UsizeSer32(1),
                issuers_frame: UsizeSer32(6),
                issuers_frame_var: 0,
                currency: CurrencyName("g1".to_owned()),
                issuers: vec![PubKey::Ed25519(
                    ed25519::PublicKey::from_base58("2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ")
                        .expect("Fail to parse issuer !")
                )],
                signatures: vec![Sig::Ed25519(
                    ed25519::Signature::from_base64("xaWNjdFeE4yr9+AKckgR6QuAvMzmKUWfY+uIlC3HKjn2apJqG70Gf59A71W+Ucz6E9WPXRzDDF/xOrf6GCGHCA==").expect("Fail to parse sig !")
                )],
                hash: Some(BlockHash(
                    Hash::from_hex(
                        "0000407900D981FC17B5A6FBCF8E8AFA4C00FAD7AFC5BEA9A96FF505E5D105EC"
                    )
                    .expect("Fail to parse hash !")
                )),
                parameters: None,
                previous_hash: Some(Hash::from_hex(
                    "0000379BBE6ABC18DCFD6E4733F9F76CB06593D10FAEDF722BE190C277AC16EA"
                )
                .expect("Fail to parse previous_hash !")),
                previous_issuer: Some(PubKey::Ed25519(
                    ed25519::PublicKey::from_base58("2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ")
                        .expect("Fail to parse previous issuer !")
                )),
                inner_hash: Some(
                    Hash::from_hex(
                        "CF2701092D5A34A55802E343B5F8D61D9B7E8089F1F13A19721234DF5B2F0F38"
                    )
                    .expect("Fail to parse inner hash !")
                ),
                dividend: None,
                identities: vec![],
                joiners: vec![],
                actives: vec![],
                leavers: vec![],
                revoked: vec![],
                excluded: vec![],
                certifications: vec![],
                transactions: vec![],
            }),
            parse_json_block(&block_json_value).expect("Fail to parse block_json_value !")
        );
    }

    #[test]
    fn parse_json_block_with_one_tx() -> Result<(), VerifyBlockHashError> {
        let block_json_str = r#"{
   "version": 10,
   "nonce": 10100000033688,
   "number": 52,
   "powMin": 74,
   "time": 1488990898,
   "medianTime": 1488990117,
   "membersCount": 59,
   "monetaryMass": 59000,
   "unitbase": 0,
   "issuersCount": 1,
   "issuersFrame": 6,
   "issuersFrameVar": 0,
   "currency": "g1",
   "issuer": "2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ",
   "signature": "4/UIwXzWQekbYw7fpD8ueMH4GnDEwCM+DvDaTfquBXOvFXLRYo/S+Vrk5u7so/98gYaZ2O7Myh20xgQvhh5FDQ==",
   "hash": "000057D4B29AF6DADB16F841F19C54C00EB244CECA9C8F2D4839D54E5F91451C",
   "parameters": "",
   "previousHash": "00000FEDA61240DD125A26886FEB2E6995B52A94778C71224CAF8492FF257D47",
   "previousIssuer": "2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ",
   "inner_hash": "6B27ACDA51F416449E5A61FC69438F8974D11FC27EB7A992410C276FC0B9BA5F",
   "dividend": null,
   "identities": [],
   "joiners": [],
   "actives": [],
   "leavers": [],
   "revoked": [],
   "excluded": [],
   "certifications": [],
   "transactions": [
    {
     "version": 10,
     "currency": "g1",
     "locktime": 0,
     "blockstamp": "50-00001DAA4559FEDB8320D1040B0F22B631459F36F237A0D9BC1EB923C12A12E7",
     "blockstampTime": 1488990016,
     "issuers": [
      "2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ"
     ],
     "inputs": [
      "1000:0:D:2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ:1"
     ],
     "outputs": [
      "1:0:SIG(Com8rJukCozHZyFao6AheSsfDQdPApxQRnz7QYFf64mm)",
      "999:0:SIG(2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ)"
     ],
     "unlocks": [
      "0:SIG(0)"
     ],
     "signatures": [
      "fAH5Gor+8MtFzQZ++JaJO6U8JJ6+rkqKtPrRr/iufh3MYkoDGxmjzj6jCADQL+hkWBt8y8QzlgRkz0ixBcKHBw=="
     ],
     "comment": "TEST",
     "block_number": 0,
     "time": 0
    }
   ],
   "raw": "Version: 10\nType: Block\nCurrency: g1\nNumber: 52\nPoWMin: 74\nTime: 1488990898\nMedianTime: 1488990117\nUnitBase: 0\nIssuer: 2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ\nIssuersFrame: 6\nIssuersFrameVar: 0\nDifferentIssuersCount: 1\nPreviousHash: 00000FEDA61240DD125A26886FEB2E6995B52A94778C71224CAF8492FF257D47\nPreviousIssuer: 2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ\nMembersCount: 59\nIdentities:\nJoiners:\nActives:\nLeavers:\nRevoked:\nExcluded:\nCertifications:\nTransactions:\nTX:10:1:1:1:2:1:0\n50-00001DAA4559FEDB8320D1040B0F22B631459F36F237A0D9BC1EB923C12A12E7\n2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ\n1000:0:D:2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ:1\n0:SIG(0)\n1:0:SIG(Com8rJukCozHZyFao6AheSsfDQdPApxQRnz7QYFf64mm)\n999:0:SIG(2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ)\nTEST\nfAH5Gor+8MtFzQZ++JaJO6U8JJ6+rkqKtPrRr/iufh3MYkoDGxmjzj6jCADQL+hkWBt8y8QzlgRkz0ixBcKHBw==\nInnerHash: 6B27ACDA51F416449E5A61FC69438F8974D11FC27EB7A992410C276FC0B9BA5F\nNonce: 10100000033688\n"
  }"#;

        let block_json_value = json_pest_parser::parse_json_string(block_json_str)
            .expect("Fail to parse json block !");

        let expected_block = BlockDocument::V10(BlockDocumentV10 {
                version: UsizeSer32(10),
                nonce: 10_100_000_033_688,
                number: BlockNumber(52),
                pow_min: UsizeSer32(74),
                time: 1_488_990_898,
                median_time: 1_488_990_117,
                members_count: UsizeSer32(59),
                monetary_mass: 59000,
                unit_base: UsizeSer32(0),
                issuers_count: UsizeSer32(1),
                issuers_frame: UsizeSer32(6),
                issuers_frame_var: 0,
                currency: CurrencyName("g1".to_owned()),
                issuers: vec![PubKey::Ed25519(
                    ed25519::PublicKey::from_base58("2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ")
                        .expect("Fail to parse issuer !")
                )],
                signatures: vec![Sig::Ed25519(
                    ed25519::Signature::from_base64("4/UIwXzWQekbYw7fpD8ueMH4GnDEwCM+DvDaTfquBXOvFXLRYo/S+Vrk5u7so/98gYaZ2O7Myh20xgQvhh5FDQ==").expect("Fail to parse sig !")
                )],
                hash: Some(BlockHash(
                    Hash::from_hex(
                        "000057D4B29AF6DADB16F841F19C54C00EB244CECA9C8F2D4839D54E5F91451C"
                    )
                    .expect("Fail to parse hash !")
                )),
                parameters: None,
                previous_hash: Some(Hash::from_hex(
                    "00000FEDA61240DD125A26886FEB2E6995B52A94778C71224CAF8492FF257D47"
                )
                .expect("Fail to parse previous_hash !")),
                previous_issuer: Some(PubKey::Ed25519(
                    ed25519::PublicKey::from_base58("2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ")
                        .expect("Fail to parse previous issuer !")
                )),
                inner_hash: Some(
                    Hash::from_hex(
                        "6B27ACDA51F416449E5A61FC69438F8974D11FC27EB7A992410C276FC0B9BA5F"
                    )
                    .expect("Fail to parse inner hash !")
                ),
                dividend: None,
                identities: vec![],
                joiners: vec![],
                actives: vec![],
                leavers: vec![],
                revoked: vec![],
                excluded: vec![],
                certifications: vec![],
                transactions: vec![match dubp_user_docs_tests_tools::mocks::tx::first_g1_tx_doc() {
                    TransactionDocument::V10(tx_doc) => tx_doc,
                }],
            });
        assert_eq!(
            expected_block,
            parse_json_block(&block_json_value).expect("Fail to parse block_json_value !")
        );

        expected_block.verify_inner_hash()?;
        Ok(())
    }
}
