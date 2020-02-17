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

//! Wrappers around Revocation documents V//  Copyright (C) 2017-2019  The AXIOM TEAM Association.
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

//! Wrappers around Revocation documents V10.

use dup_crypto::keys::*;

use crate::documents::*;
use dubp_common_doc::blockstamp::Blockstamp;
use dubp_common_doc::parser::TextDocumentParseError;
use dubp_common_doc::traits::text::*;
use dubp_common_doc::traits::{Document, DocumentBuilder, ToStringObject};
use dubp_common_doc::{BlockHash, BlockNumber};
use dup_crypto::hashs::Hash;
use durs_common_tools::UsizeSer32;

#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// Wrap an Compact Revocation document (in block content)
pub struct CompactRevocationDocumentV10 {
    /// Issuer
    pub issuer: PubKey,
    /// Signature
    pub signature: Sig,
}

impl CompactTextDocument for CompactRevocationDocumentV10 {
    fn as_compact_text(&self) -> String {
        format!(
            "{issuer}:{signature}",
            issuer = self.issuer,
            signature = self.signature,
        )
    }
}

#[derive(Clone, Debug, Deserialize, Hash, Serialize, PartialEq, Eq)]
/// Revocation document for jsonification
pub struct CompactRevocationDocumentV10Stringified {
    /// Document issuer
    pub issuer: String,
    /// Document signature
    pub signature: String,
}

impl ToStringObject for CompactRevocationDocumentV10 {
    type StringObject = CompactRevocationDocumentV10Stringified;
    /// Transforms an object into a json object
    fn to_string_object(&self) -> CompactRevocationDocumentV10Stringified {
        CompactRevocationDocumentV10Stringified {
            issuer: format!("{}", self.issuer),
            signature: format!("{}", self.signature),
        }
    }
}

/// Wrap an Revocation document.
///
/// Must be created by parsing a text document or using a builder.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct RevocationDocumentV10 {
    /// Document as text.
    ///
    /// Is used to check signatures, and other values mut be extracted from it.
    text: String,

    /// Name of the currency.
    currency: String,
    /// Document issuer (there should be only one).
    issuers: Vec<PubKey>,
    /// Username of target identity
    identity_username: String,
    /// Target Identity document blockstamp.
    identity_blockstamp: Blockstamp,
    /// Target Identity document signature.
    identity_sig: Sig,
    /// Document signature (there should be only one).
    signatures: Vec<Sig>,
}

#[derive(Clone, Debug, Deserialize, Hash, Serialize, PartialEq, Eq)]
/// Revocation document for jsonification
pub struct RevocationDocumentV10Stringified {
    /// Name of the currency.
    currency: String,
    /// Document issuer
    issuer: String,
    /// Username of target identity
    identity_username: String,
    /// Target Identity document blockstamp.
    identity_blockstamp: String,
    /// Target Identity document signature.
    identity_sig: String,
    /// Document signature
    signature: String,
}

impl ToStringObject for RevocationDocumentV10 {
    type StringObject = RevocationDocumentV10Stringified;
    /// Transforms an object into a json object
    fn to_string_object(&self) -> RevocationDocumentV10Stringified {
        RevocationDocumentV10Stringified {
            currency: self.currency.clone(),
            issuer: format!("{}", self.issuers[0]),
            identity_username: self.identity_username.clone(),
            identity_blockstamp: format!("{}", self.identity_blockstamp),
            identity_sig: format!("{}", self.identity_sig),
            signature: format!("{}", self.signatures[0]),
        }
    }
}

impl RevocationDocumentV10 {
    /// Username of target identity
    pub fn identity_username(&self) -> &str {
        &self.identity_username
    }
    /// From pest parser pair
    pub fn from_pest_pair(
        pair: Pair<Rule>,
    ) -> Result<RevocationDocumentV10, TextDocumentParseError> {
        let doc = pair.as_str();
        let mut currency = "";
        let mut pubkeys = Vec::with_capacity(1);
        let mut uid = "";
        let mut sigs = Vec::with_capacity(2);
        let mut blockstamps = Vec::with_capacity(1);
        for field in pair.into_inner() {
            match field.as_rule() {
                Rule::currency => currency = field.as_str(),
                Rule::pubkey => pubkeys.push(PubKey::Ed25519(
                    unwrap!(ed25519::PublicKey::from_base58(field.as_str())), // Grammar ensures that we have a base58 string.
                )),
                Rule::uid => {
                    uid = field.as_str();
                }
                Rule::blockstamp => {
                    let mut inner_rules = field.into_inner(); // { integer ~ "-" ~ hash }

                    let block_id: &str = unwrap!(inner_rules.next()).as_str();
                    let block_hash: &str = unwrap!(inner_rules.next()).as_str();
                    blockstamps.push(Blockstamp {
                        id: BlockNumber(unwrap!(block_id.parse())), // Grammar ensures that we have a digits string.
                        hash: BlockHash(unwrap!(Hash::from_hex(block_hash))), // Grammar ensures that we have an hexadecimal string.
                    });
                }
                Rule::ed25519_sig => {
                    sigs.push(Sig::Ed25519(
                        unwrap!(ed25519::Signature::from_base64(field.as_str())), // Grammar ensures that we have a base64 string.
                    ));
                }
                Rule::EOI => (),
                _ => fatal_error!("unexpected rule"), // Grammar ensures that we never reach this line
            }
        }
        Ok(RevocationDocumentV10 {
            text: doc.to_owned(),
            issuers: vec![pubkeys[0]],
            currency: currency.to_owned(),
            identity_username: uid.to_owned(),
            identity_blockstamp: blockstamps[0],
            identity_sig: sigs[0],
            signatures: vec![sigs[1]],
        })
    }
}

impl Document for RevocationDocumentV10 {
    type PublicKey = PubKey;

    fn version(&self) -> UsizeSer32 {
        UsizeSer32(10)
    }

    fn currency(&self) -> &str {
        &self.currency
    }

    fn blockstamp(&self) -> Blockstamp {
        unimplemented!()
    }

    fn issuers(&self) -> &Vec<PubKey> {
        &self.issuers
    }

    fn signatures(&self) -> &Vec<Sig> {
        &self.signatures
    }

    fn as_bytes(&self) -> &[u8] {
        self.as_text_without_signature().as_bytes()
    }
}

impl TextDocument for RevocationDocumentV10 {
    type CompactTextDocument_ = CompactRevocationDocumentV10;

    fn as_text(&self) -> &str {
        &self.text
    }

    fn to_compact_document(&self) -> Self::CompactTextDocument_ {
        CompactRevocationDocumentV10 {
            issuer: self.issuers[0],
            signature: self.signatures[0],
        }
    }
}

/// Revocation document builder.
#[derive(Debug, Copy, Clone)]
pub struct RevocationDocumentV10Builder<'a> {
    /// Document currency.
    pub currency: &'a str,
    /// Revocation issuer.
    pub issuer: &'a PubKey,
    /// Username of target Identity.
    pub identity_username: &'a str,
    /// Blockstamp of target Identity.
    pub identity_blockstamp: &'a Blockstamp,
    /// Signature of target Identity.
    pub identity_sig: &'a Sig,
}

impl<'a> RevocationDocumentV10Builder<'a> {
    fn build_with_text_and_sigs(self, text: String, signatures: Vec<Sig>) -> RevocationDocumentV10 {
        RevocationDocumentV10 {
            text,
            currency: self.currency.to_string(),
            issuers: vec![*self.issuer],
            identity_username: self.identity_username.to_string(),
            identity_blockstamp: *self.identity_blockstamp,
            identity_sig: *self.identity_sig,
            signatures,
        }
    }
}

impl<'a> DocumentBuilder for RevocationDocumentV10Builder<'a> {
    type Document = RevocationDocumentV10;
    type Signator = SignatorEnum;

    fn build_with_signature(&self, signatures: Vec<Sig>) -> RevocationDocumentV10 {
        self.build_with_text_and_sigs(self.generate_text(), signatures)
    }

    fn build_and_sign(&self, private_keys: Vec<SignatorEnum>) -> RevocationDocumentV10 {
        let (text, signatures) = self.build_signed_text(private_keys);
        self.build_with_text_and_sigs(text, signatures)
    }
}

impl<'a> TextDocumentBuilder for RevocationDocumentV10Builder<'a> {
    fn generate_text(&self) -> String {
        format!(
            "Version: 10
Type: Revocation
Currency: {currency}
Issuer: {issuer}
IdtyUniqueID: {idty_uid}
IdtyTimestamp: {idty_blockstamp}
IdtySignature: {idty_sig}
",
            currency = self.currency,
            issuer = self.issuer,
            idty_uid = self.identity_username,
            idty_blockstamp = self.identity_blockstamp,
            idty_sig = self.identity_sig,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dup_crypto::keys::Signature;

    #[test]
    fn generate_real_document() {
        let keypair = ed25519::KeyPairFromSeed32Generator::generate(unwrap!(
            Seed32::from_base58("DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV"),
            "fail to build Seed32"
        ));
        let pubkey = PubKey::Ed25519(keypair.public_key());
        let signator =
            SignatorEnum::Ed25519(keypair.generate_signator().expect("fail to gen signator"));

        let sig = Sig::Ed25519(unwrap!(ed25519::Signature::from_base64(
            "gBD2mCr7E/tW8u3wqVK7IWtQB6IKxddg13UMl9ypVsv/VhqhAFTBba9BwoK5t6H9eqF1d+4sCB3WY2eJ/yuUAg==",
        ), "Fail to build Signature"));

        let identity_blockstamp = unwrap!(
            Blockstamp::from_string(
                "0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
            ),
            "Fail to build Blockstamp"
        );

        let identity_sig = Sig::Ed25519(unwrap!(ed25519::Signature::from_base64(
            "1eubHHbuNfilHMM0G2bI30iZzebQ2cQ1PC7uPAw08FGMMmQCRerlF/3pc4sAcsnexsxBseA/3lY03KlONqJBAg==",
        ), "Fail to build Signature"));

        let builder = RevocationDocumentV10Builder {
            currency: "g1",
            issuer: &pubkey,
            identity_username: "tic",
            identity_blockstamp: &identity_blockstamp,
            identity_sig: &identity_sig,
        };

        println!(
            "Signatures = {:?}",
            builder
                .build_and_sign(vec![SignatorEnum::Ed25519(
                    keypair.generate_signator().expect("fail to gen signator")
                )])
                .signatures()
        );

        assert!(builder
            .build_with_signature(vec![sig])
            .verify_signatures()
            .is_ok());

        assert!(builder
            .build_and_sign(vec![signator])
            .verify_signatures()
            .is_ok());
    }

    #[test]
    fn revocation_document() {
        let doc = "Version: 10
Type: Revocation
Currency: g1
Issuer: DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV
IdtyUniqueID: tic
IdtyTimestamp: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
IdtySignature: 1eubHHbuNfilHMM0G2bI30iZzebQ2cQ1PC7uPAw08FGMMmQCRerlF/3pc4sAcsnexsxBseA/3lY03KlONqJBAg==
XXOgI++6qpY9O31ml/FcfbXCE6aixIrgkT5jL7kBle3YOMr+8wrp7Rt+z9hDVjrNfYX2gpeJsuMNfG4T/fzVDQ==";

        let doc =
            RevocationDocumentParser::parse(doc).expect("fail to parse test revocation document !");
        println!("Doc : {:?}", doc);
        assert!(doc.verify_signatures().is_ok())
    }
}
