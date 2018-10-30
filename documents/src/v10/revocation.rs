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

//! Wrappers around Revocation documents.

use duniter_crypto::keys::*;
use pest::Parser;

use blockstamp::Blockstamp;
use v10::*;
use *;

#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// Wrap an Compact Revocation document (in block content)
pub struct CompactRevocationDocument {
    /// Issuer
    pub issuer: PubKey,
    /// Signature
    pub signature: Sig,
}

impl CompactTextDocument for CompactRevocationDocument {
    fn as_compact_text(&self) -> String {
        format!(
            "{issuer}:{signature}",
            issuer = self.issuer,
            signature = self.signature,
        )
    }
}

/// Wrap an Revocation document.
///
/// Must be created by parsing a text document or using a builder.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct RevocationDocument {
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

impl RevocationDocument {
    /// Username of target identity
    pub fn identity_username(&self) -> &str {
        &self.identity_username
    }
}

impl Document for RevocationDocument {
    type PublicKey = PubKey;
    type CurrencyType = str;

    fn version(&self) -> u16 {
        10
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

impl TextDocument for RevocationDocument {
    type CompactTextDocument_ = CompactRevocationDocument;

    fn as_text(&self) -> &str {
        &self.text
    }

    fn to_compact_document(&self) -> Self::CompactTextDocument_ {
        CompactRevocationDocument {
            issuer: self.issuers[0],
            signature: self.signatures[0],
        }
    }
}

impl IntoSpecializedDocument<BlockchainProtocol> for RevocationDocument {
    fn into_specialized(self) -> BlockchainProtocol {
        BlockchainProtocol::V10(Box::new(V10Document::Revocation(Box::new(self))))
    }
}

/// Revocation document builder.
#[derive(Debug, Copy, Clone)]
pub struct RevocationDocumentBuilder<'a> {
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

impl<'a> RevocationDocumentBuilder<'a> {
    fn build_with_text_and_sigs(self, text: String, signatures: Vec<Sig>) -> RevocationDocument {
        RevocationDocument {
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

impl<'a> DocumentBuilder for RevocationDocumentBuilder<'a> {
    type Document = RevocationDocument;
    type PrivateKey = PrivKey;

    fn build_with_signature(&self, signatures: Vec<Sig>) -> RevocationDocument {
        self.build_with_text_and_sigs(self.generate_text(), signatures)
    }

    fn build_and_sign(&self, private_keys: Vec<PrivKey>) -> RevocationDocument {
        let (text, signatures) = self.build_signed_text(private_keys);
        self.build_with_text_and_sigs(text, signatures)
    }
}

impl<'a> TextDocumentBuilder for RevocationDocumentBuilder<'a> {
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

/// Revocation document parser
#[derive(Debug, Clone, Copy)]
pub struct RevocationDocumentParser;

impl TextDocumentParser for RevocationDocumentParser {
    fn parse(doc: &str, currency: &str) -> Result<V10Document, V10DocumentParsingError> {
        match DocumentsParser::parse(Rule::revoc, doc) {
            Ok(mut doc_ast) => {
                let revoc_ast = doc_ast.next().unwrap(); // get and unwrap the `revoc` rule; never fails
                let revoc_vx_ast = revoc_ast.into_inner().next().unwrap(); // get and unwrap the `revoc_vX` rule; never fails

                match revoc_vx_ast.as_rule() {
                    Rule::revoc_v10 => {
                        let mut pubkeys = Vec::with_capacity(1);
                        let mut uid = "";
                        let mut sigs = Vec::with_capacity(2);
                        let mut blockstamps = Vec::with_capacity(1);
                        for field in revoc_vx_ast.into_inner() {
                            match field.as_rule() {
                                Rule::currency => {
                                    if currency != field.as_str() {
                                        return Err(V10DocumentParsingError::InvalidCurrency());
                                    }
                                }
                                Rule::pubkey => {
                                    if !pubkeys.is_empty() {
                                        return Err(V10DocumentParsingError::InvalidInnerFormat(
                                            "Revocation document must contain exactly one pubkey !",
                                        ));
                                    }
                                    pubkeys.push(PubKey::Ed25519(
                                        ed25519::PublicKey::from_base58(field.as_str()).unwrap(), // Grammar ensures that we have a base58 string.
                                    ));
                                }
                                Rule::uid => {
                                    uid = field.as_str();
                                }
                                Rule::blockstamp => {
                                    if blockstamps.len() > 1 {
                                        return Err(V10DocumentParsingError::InvalidInnerFormat(
                                            "Revocation document must contain exactly one blockstamp !",
                                        ));
                                    }
                                    let mut inner_rules = field.into_inner(); // { integer ~ "-" ~ hash }

                                    let block_id: &str = inner_rules.next().unwrap().as_str();
                                    let block_hash: &str = inner_rules.next().unwrap().as_str();
                                    blockstamps.push(Blockstamp {
                                        id: BlockId(block_id.parse().unwrap()), // Grammar ensures that we have a digits string.
                                        hash: BlockHash(Hash::from_hex(block_hash).unwrap()), // Grammar ensures that we have an hexadecimal string.
                                    });
                                }
                                Rule::ed25519_sig => {
                                    sigs.push(Sig::Ed25519(
                                        ed25519::Signature::from_base64(field.as_str()).unwrap(), // Grammar ensures that we have a base64 string.
                                    ));
                                }
                                Rule::EOI => (),
                                _ => panic!("unexpected rule"), // Grammar ensures that we never reach this line
                            }
                        }
                        Ok(V10Document::Revocation(Box::new(RevocationDocument {
                            text: doc.to_owned(),
                            issuers: vec![pubkeys[0]],
                            currency: currency.to_owned(),
                            identity_username: uid.to_owned(),
                            identity_blockstamp: blockstamps[0],
                            identity_sig: sigs[0],
                            signatures: vec![sigs[1]],
                        })))
                    }
                    _ => Err(V10DocumentParsingError::UnexpectedVersion()),
                }
            }
            Err(pest_error) => panic!("{}", pest_error), //Err(V10DocumentParsingError::PestError()),
        }

        /*if let Some(caps) = REVOCATION_REGEX.captures(body) {
            let issuer = &caps["issuer"];
            let identity_username = &caps["idty_uid"];
            let identity_blockstamp = &caps["idty_blockstamp"];
            let identity_sig = &caps["idty_sig"];
        
            // Regex match so should not fail.
            // TODO : Test it anyway
            let issuer = PubKey::Ed25519(ed25519::PublicKey::from_base58(issuer).unwrap());
            let identity_username = String::from(identity_username);
            let identity_blockstamp = Blockstamp::from_string(identity_blockstamp).unwrap();
            let identity_sig = Sig::Ed25519(Signature::from_base64(identity_sig).unwrap());
        
            Ok(V10Document::Revocation(Box::new(RevocationDocument {
                text: doc.to_owned(),
                issuers: vec![issuer],
                currency: currency.to_owned(),
                identity_username,
                identity_blockstamp,
                identity_sig,
                signatures,
            })))
        } else {
            Err(V10DocumentParsingError::InvalidInnerFormat("Revocation"))
        }*/
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use duniter_crypto::keys::{PrivateKey, PublicKey, Signature};
    use VerificationResult;

    #[test]
    fn generate_real_document() {
        let pubkey = PubKey::Ed25519(
            ed25519::PublicKey::from_base58("DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV")
                .unwrap(),
        );

        let prikey = PrivKey::Ed25519(
            ed25519::PrivateKey::from_base58(
                "468Q1XtTq7h84NorZdWBZFJrGkB18CbmbHr9tkp9snt5G\
                 iERP7ySs3wM8myLccbAAGejgMRC9rqnXuW3iAfZACm7",
            )
            .unwrap(),
        );

        let sig = Sig::Ed25519(ed25519::Signature::from_base64(
            "XXOgI++6qpY9O31ml/FcfbXCE6aixIrgkT5jL7kBle3YOMr+8wrp7Rt+z9hDVjrNfYX2gpeJsuMNfG4T/fzVDQ==",
        ).unwrap());

        let identity_blockstamp = Blockstamp::from_string(
            "0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
        )
        .unwrap();

        let identity_sig = Sig::Ed25519(ed25519::Signature::from_base64(
            "1eubHHbuNfilHMM0G2bI30iZzebQ2cQ1PC7uPAw08FGMMmQCRerlF/3pc4sAcsnexsxBseA/3lY03KlONqJBAg==",
        ).unwrap());

        let builder = RevocationDocumentBuilder {
            currency: "g1",
            issuer: &pubkey,
            identity_username: "tic",
            identity_blockstamp: &identity_blockstamp,
            identity_sig: &identity_sig,
        };

        assert_eq!(
            builder.build_with_signature(vec![sig]).verify_signatures(),
            VerificationResult::Valid()
        );

        assert_eq!(
            builder.build_and_sign(vec![prikey]).verify_signatures(),
            VerificationResult::Valid()
        );
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

        let currency = "g1";

        let doc = RevocationDocumentParser::parse(doc, currency).unwrap();
        if let V10Document::Revocation(doc) = doc {
            println!("Doc : {:?}", doc);
            assert_eq!(doc.verify_signatures(), VerificationResult::Valid())
        } else {
            panic!("Wrong document type");
        }
    }
}
