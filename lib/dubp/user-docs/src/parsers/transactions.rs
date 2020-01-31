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

use crate::documents::transaction::v10::*;
use crate::documents::transaction::*;
use crate::parsers::DefaultHasher;
use crate::*;
use dubp_common_doc::blockstamp::Blockstamp;
use dubp_common_doc::parser::TextDocumentParseError;
use dubp_common_doc::traits::DocumentBuilder;
use dup_crypto::bases::BaseConvertionError;
use dup_crypto::hashs::Hash;
use dup_crypto::keys::*;
use failure::Error;
use json_pest_parser::*;
use std::str::FromStr;

#[derive(Debug, Fail, Copy, Clone)]
pub enum ParseTxError {
    #[fail(display = "Fail to parse transaction : wrong format !")]
    WrongFormat,
}

/// Parse transactions documents from array of str
pub fn parse_json_transactions(
    array_transactions: &[&JSONValue<DefaultHasher>],
) -> Result<Vec<TransactionDocumentV10>, Error> {
    array_transactions
        .iter()
        .map(|tx| {
            parse_json_transaction(tx).map(|tx_doc| match tx_doc {
                TransactionDocument::V10(tx_doc_v10) => tx_doc_v10,
            })
        })
        .collect::<Result<Vec<TransactionDocumentV10>, Error>>()
}

/// Parse transaction from json value
pub fn parse_json_transaction(
    json_tx: &JSONValue<DefaultHasher>,
) -> Result<TransactionDocument, Error> {
    if !json_tx.is_object() {
        return Err(ParseJsonError {
            cause: "Json transaction must be an object !".to_owned(),
        }
        .into());
    }

    let json_tx = json_tx.to_object().expect("safe unwrap");

    match get_u64(json_tx, "version")? {
        10 => Ok(
            TransactionDocumentBuilder::V10(TransactionDocumentV10Builder {
                currency: get_str(json_tx, "currency")?,
                blockstamp: &Blockstamp::from_string(get_str(json_tx, "blockstamp")?)?,
                locktime: &(get_number(json_tx, "locktime")?.trunc() as u64),
                issuers: &get_str_array(json_tx, "issuers")?
                    .iter()
                    .map(|p| ed25519::PublicKey::from_base58(p))
                    .map(|p| p.map(PubKey::Ed25519))
                    .collect::<Result<Vec<PubKey>, BaseConvertionError>>()?,
                inputs: &get_str_array(json_tx, "inputs")?
                    .iter()
                    .map(|i| TransactionInputV10::from_str(i))
                    .collect::<Result<Vec<TransactionInputV10>, TextDocumentParseError>>()?,
                unlocks: &get_str_array(json_tx, "unlocks")?
                    .iter()
                    .map(|i| TransactionInputUnlocksV10::from_str(i))
                    .collect::<Result<Vec<TransactionInputUnlocksV10>, TextDocumentParseError>>()?,
                outputs: &get_str_array(json_tx, "outputs")?
                    .iter()
                    .map(|i| TransactionOutputV10::from_str(i))
                    .collect::<Result<Vec<TransactionOutputV10>, TextDocumentParseError>>()?,
                comment: &durs_common_tools::fns::str_escape::unescape_str(get_str(
                    json_tx, "comment",
                )?),
                hash: get_optional_str(json_tx, "hash")?
                    .map(Hash::from_hex)
                    .transpose()?,
            })
            .build_with_signature(
                get_str_array(json_tx, "signatures")?
                    .iter()
                    .map(|p| ed25519::Signature::from_base64(p))
                    .map(|p| p.map(Sig::Ed25519))
                    .collect::<Result<Vec<Sig>, BaseConvertionError>>()?,
            ),
        ),
        version => Err(ParseJsonError {
            cause: format!("Unhandled json transaction version: {} !", version),
        }
        .into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dubp_common_doc::traits::DocumentBuilder;
    use dubp_common_doc::Blockstamp;
    use std::str::FromStr;

    pub fn first_g1_tx_doc() -> TransactionDocument {
        let expected_tx_builder = TransactionDocumentV10Builder {
            currency: &"g1",
            blockstamp: &Blockstamp::from_string(
                "50-00001DAA4559FEDB8320D1040B0F22B631459F36F237A0D9BC1EB923C12A12E7",
            )
            .expect("Fail to parse blockstamp"),
            locktime: &0,
            issuers: &vec![PubKey::Ed25519(
                ed25519::PublicKey::from_base58("2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ")
                    .expect("Fail to parse issuer !"),
            )],
            inputs: &vec![TransactionInputV10::from_str(
                "1000:0:D:2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ:1",
            )
            .expect("Fail to parse inputs")],
            unlocks: &vec![
                TransactionInputUnlocksV10::from_str("0:SIG(0)").expect("Fail to parse unlocks")
            ],
            outputs: &vec![
                TransactionOutputV10::from_str(
                    "1:0:SIG(Com8rJukCozHZyFao6AheSsfDQdPApxQRnz7QYFf64mm)",
                )
                .expect("Fail to parse outputs"),
                TransactionOutputV10::from_str(
                    "999:0:SIG(2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ)",
                )
                .expect("Fail to parse outputs"),
            ],
            comment: "TEST",
            hash: None,
        };

        TransactionDocumentBuilder::V10(expected_tx_builder).build_with_signature(vec![Sig::Ed25519(
                ed25519::Signature::from_base64("fAH5Gor+8MtFzQZ++JaJO6U8JJ6+rkqKtPrRr/iufh3MYkoDGxmjzj6jCADQL+hkWBt8y8QzlgRkz0ixBcKHBw==").expect("Fail to parse sig !")
            )])
    }

    #[test]
    fn test_parse_json_tx() {
        let tx_json_str = r#"{
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
    }"#;

        let tx_json_value =
            json_pest_parser::parse_json_string(tx_json_str).expect("Fail to parse json tx !");

        assert_eq!(
            first_g1_tx_doc(),
            parse_json_transaction(&tx_json_value).expect("Fail to parse tx_json_value !")
        );
    }
}
