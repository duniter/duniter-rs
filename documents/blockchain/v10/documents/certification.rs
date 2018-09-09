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

//! Wrappers around Certification documents.

use duniter_crypto::keys::*;
use regex::Regex;

use blockchain::v10::documents::*;
use blockchain::{BlockchainProtocol, Document, DocumentBuilder, IntoSpecializedDocument};
use BlockId;
use Blockstamp;

lazy_static! {
    static ref CERTIFICATION_REGEX: Regex = Regex::new(
        "^Issuer: (?P<issuer>[1-9A-Za-z]{43,44})\nIdtyIssuer: (?P<target>[1-9A-Za-z]{43,44})\nIdtyUniqueID: (?P<idty_uid>[[:alnum:]_-]+)\nIdtyTimestamp: (?P<idty_blockstamp>[0-9]+-[0-9A-F]{64})\nIdtySignature: (?P<idty_sig>(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?)\nCertTimestamp: (?P<blockstamp>[0-9]+-[0-9A-F]{64})\n$"
    ).unwrap();
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
/// Wrap an Compact Revocation document (in block content)
pub struct CompactCertificationDocument {
    /// Issuer
    pub issuer: PubKey,
    /// Target
    pub target: PubKey,
    /// Blockstamp
    pub block_number: BlockId,
    /// Signature
    pub signature: Sig,
}

impl CompactTextDocument for CompactCertificationDocument {
    fn as_compact_text(&self) -> String {
        format!(
            "{issuer}:{target}:{block_number}:{signature}",
            issuer = self.issuer,
            target = self.target,
            block_number = self.block_number.0,
            signature = self.signature,
        )
    }
}

/// Wrap an Certification document.
///
/// Must be created by parsing a text document or using a builder.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CertificationDocument {
    /// Document as text.
    ///
    /// Is used to check signatures, and other values mut be extracted from it.
    text: String,

    /// Name of the currency.
    currency: String,
    /// Document issuer (there should be only one).
    issuers: Vec<PubKey>,
    /// issuer of target identity.
    target: PubKey,
    /// Username of target identity
    identity_username: String,
    /// Target Identity document blockstamp.
    identity_blockstamp: Blockstamp,
    /// Target Identity document signature.
    identity_sig: Sig,
    /// Blockstamp
    blockstamp: Blockstamp,
    /// Document signature (there should be only one).
    signatures: Vec<Sig>,
}

impl CertificationDocument {
    /// Username of target identity
    pub fn identity_username(&self) -> &str {
        &self.identity_username
    }

    /// Pubkey of source identity
    pub fn source(&self) -> &PubKey {
        &self.issuers[0]
    }

    /// Pubkey of target identity
    pub fn target(&self) -> &PubKey {
        &self.target
    }
}

impl Document for CertificationDocument {
    type PublicKey = PubKey;
    type CurrencyType = str;

    fn version(&self) -> u16 {
        10
    }

    fn currency(&self) -> &str {
        &self.currency
    }

    fn blockstamp(&self) -> Blockstamp {
        self.blockstamp
    }

    fn issuers(&self) -> &Vec<PubKey> {
        &self.issuers
    }

    fn signatures(&self) -> &Vec<Sig> {
        &self.signatures
    }

    fn as_bytes(&self) -> &[u8] {
        self.as_text().as_bytes()
    }
}

impl TextDocument for CertificationDocument {
    type CompactTextDocument_ = CompactCertificationDocument;

    fn as_text(&self) -> &str {
        &self.text
    }

    fn to_compact_document(&self) -> Self::CompactTextDocument_ {
        CompactCertificationDocument {
            issuer: self.issuers[0],
            target: self.target,
            block_number: self.blockstamp().id,
            signature: self.signatures()[0],
        }
    }
}

impl IntoSpecializedDocument<BlockchainProtocol> for CertificationDocument {
    fn into_specialized(self) -> BlockchainProtocol {
        BlockchainProtocol::V10(Box::new(V10Document::Certification(Box::new(self))))
    }
}

/// Certification document builder.
#[derive(Debug, Copy, Clone)]
pub struct CertificationDocumentBuilder<'a> {
    /// Document currency.
    pub currency: &'a str,
    /// Certification issuer (=source).
    pub issuer: &'a PubKey,
    /// Reference blockstamp.
    pub blockstamp: &'a Blockstamp,
    /// Pubkey of target identity.
    pub target: &'a PubKey,
    /// Username of target Identity.
    pub identity_username: &'a str,
    /// Blockstamp of target Identity.
    pub identity_blockstamp: &'a Blockstamp,
    /// Signature of target Identity.
    pub identity_sig: &'a Sig,
}

impl<'a> CertificationDocumentBuilder<'a> {
    fn build_with_text_and_sigs(self, text: String, signatures: Vec<Sig>) -> CertificationDocument {
        CertificationDocument {
            text,
            currency: self.currency.to_string(),
            issuers: vec![*self.issuer],
            blockstamp: *self.blockstamp,
            target: *self.target,
            identity_username: self.identity_username.to_string(),
            identity_blockstamp: *self.identity_blockstamp,
            identity_sig: *self.identity_sig,
            signatures,
        }
    }
}

impl<'a> DocumentBuilder for CertificationDocumentBuilder<'a> {
    type Document = CertificationDocument;
    type PrivateKey = PrivKey;

    fn build_with_signature(&self, signatures: Vec<Sig>) -> CertificationDocument {
        self.build_with_text_and_sigs(self.generate_text(), signatures)
    }

    fn build_and_sign(&self, private_keys: Vec<PrivKey>) -> CertificationDocument {
        let (text, signatures) = self.build_signed_text(private_keys);
        self.build_with_text_and_sigs(text, signatures)
    }
}

impl<'a> TextDocumentBuilder for CertificationDocumentBuilder<'a> {
    fn generate_text(&self) -> String {
        format!(
            "Version: 10
Type: Certification
Currency: {currency}
Issuer: {issuer}
IdtyIssuer: {target}
IdtyUniqueID: {idty_uid}
IdtyTimestamp: {idty_blockstamp}
IdtySignature: {idty_sig}
CertTimestamp: {blockstamp}
",
            currency = self.currency,
            issuer = self.issuer,
            target = self.target,
            idty_uid = self.identity_username,
            idty_blockstamp = self.identity_blockstamp,
            idty_sig = self.identity_sig,
            blockstamp = self.blockstamp,
        )
    }
}

/// Certification document parser
#[derive(Debug, Clone, Copy)]
pub struct CertificationDocumentParser;

impl StandardTextDocumentParser for CertificationDocumentParser {
    fn parse_standard(
        doc: &str,
        body: &str,
        currency: &str,
        signatures: Vec<Sig>,
    ) -> Result<V10Document, V10DocumentParsingError> {
        if let Some(caps) = CERTIFICATION_REGEX.captures(body) {
            let issuer = &caps["issuer"];
            let target = &caps["target"];
            let identity_username = &caps["idty_uid"];
            let identity_blockstamp = &caps["idty_blockstamp"];
            let identity_sig = &caps["idty_sig"];
            let blockstamp = &caps["blockstamp"];

            // Regex match so should not fail.
            // TODO : Test it anyway
            let issuer = PubKey::Ed25519(ed25519::PublicKey::from_base58(issuer).unwrap());
            let target = PubKey::Ed25519(ed25519::PublicKey::from_base58(target).unwrap());
            let identity_username = String::from(identity_username);
            let identity_blockstamp = Blockstamp::from_string(identity_blockstamp).unwrap();
            let identity_sig = Sig::Ed25519(ed25519::Signature::from_base64(identity_sig).unwrap());
            let blockstamp = Blockstamp::from_string(blockstamp).unwrap();

            Ok(V10Document::Certification(Box::new(
                CertificationDocument {
                    text: doc.to_owned(),
                    issuers: vec![issuer],
                    currency: currency.to_owned(),
                    target,
                    identity_username,
                    identity_blockstamp,
                    identity_sig,
                    blockstamp,
                    signatures,
                },
            )))
        } else {
            Err(V10DocumentParsingError::InvalidInnerFormat(
                "Certification".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blockchain::VerificationResult;
    use duniter_crypto::keys::{PrivateKey, PublicKey, Signature};

    #[test]
    fn generate_real_document() {
        let pubkey = PubKey::Ed25519(
            ed25519::PublicKey::from_base58("4tNQ7d9pj2Da5wUVoW9mFn7JjuPoowF977au8DdhEjVR")
                .unwrap(),
        );

        let prikey = PrivKey::Ed25519(ed25519::PrivateKey::from_base58(
            "3XGWuuU1dQ7zaYPzE76ATfY71STzRkbT3t4DE1bSjMhYje81XdJFeXVG9uMPi3oDeRTosT2dmBAFH8VydrAUWXRZ",
        ).unwrap());

        let sig = Sig::Ed25519(ed25519::Signature::from_base64(
            "qfR6zqT1oJbqIsppOi64gC9yTtxb6g6XA9RYpulkq9ehMvqg2VYVigCbR0yVpqKFsnYiQTrnjgFuFRSJCJDfCw==",
        ).unwrap());

        let target = PubKey::Ed25519(
            ed25519::PublicKey::from_base58("DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV")
                .unwrap(),
        );

        let identity_blockstamp = Blockstamp::from_string(
            "0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
        ).unwrap();

        let identity_sig = Sig::Ed25519(ed25519::Signature::from_base64(
            "1eubHHbuNfilHMM0G2bI30iZzebQ2cQ1PC7uPAw08FGMMmQCRerlF/3pc4sAcsnexsxBseA/3lY03KlONqJBAg==",
        ).unwrap());

        let blockstamp = Blockstamp::from_string(
            "36-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B865",
        ).unwrap();

        let builder = CertificationDocumentBuilder {
            currency: "duniter_unit_test_currency",
            issuer: &pubkey,
            target: &target,
            identity_username: "tic",
            identity_blockstamp: &identity_blockstamp,
            identity_sig: &identity_sig,
            blockstamp: &blockstamp,
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
    fn certification_standard_regex() {
        assert!(CERTIFICATION_REGEX.is_match(
            "Issuer: DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV
IdtyIssuer: 7jzkd8GiFnpys4X7mP78w2Y3y3kwdK6fVSLEaojd3aH9
IdtyUniqueID: fbarbut
IdtyTimestamp: 98221-000000575AC04F5164F7A307CDB766139EA47DD249E4A2444F292BC8AAB408B3
IdtySignature: DjeipIeb/RF0tpVCnVnuw6mH1iLJHIsDfPGLR90Twy3PeoaDz6Yzhc/UjLWqHCi5Y6wYajV0dNg4jQRUneVBCQ==
CertTimestamp: 99956-00000472758331FDA8388E30E50CA04736CBFD3B7C21F34E74707107794B56DD
"
        ));
    }

    #[test]
    fn certification_document() {
        let doc = "Version: 10
Type: Certification
Currency: g1-test
Issuer: 5B8iMAzq1dNmFe3ZxFTBQkqhq4fsztg1gZvxHXCk1XYH
IdtyIssuer: mMPioknj2MQCX9KyKykdw8qMRxYR2w1u3UpdiEJHgXg
IdtyUniqueID: mmpio
IdtyTimestamp: 7543-000044410C5370DE8DBA911A358F318096B7A269CFC2BB93272E397CC513EA0A
IdtySignature: SmSweUD4lEMwiZfY8ux9maBjrQQDkC85oMNsin6oSQCPdXG8sFCZ4FisUaWqKsfOlZVb/HNa+TKzD2t0Yte+DA==
CertTimestamp: 167884-0001DFCA28002A8C96575E53B8CEF8317453A7B0BA255542CCF0EC8AB5E99038
";

        let body = "Issuer: 5B8iMAzq1dNmFe3ZxFTBQkqhq4fsztg1gZvxHXCk1XYH
IdtyIssuer: mMPioknj2MQCX9KyKykdw8qMRxYR2w1u3UpdiEJHgXg
IdtyUniqueID: mmpio
IdtyTimestamp: 7543-000044410C5370DE8DBA911A358F318096B7A269CFC2BB93272E397CC513EA0A
IdtySignature: SmSweUD4lEMwiZfY8ux9maBjrQQDkC85oMNsin6oSQCPdXG8sFCZ4FisUaWqKsfOlZVb/HNa+TKzD2t0Yte+DA==
CertTimestamp: 167884-0001DFCA28002A8C96575E53B8CEF8317453A7B0BA255542CCF0EC8AB5E99038
";

        let currency = "g1-test";

        let signatures = vec![Sig::Ed25519(ed25519::Signature::from_base64(
"wqZxPEGxLrHGv8VdEIfUGvUcf+tDdNTMXjLzVRCQ4UhlhDRahOMjfcbP7byNYr5OfIl83S1MBxF7VJgu8YasCA=="
        ).unwrap())];

        let doc =
            CertificationDocumentParser::parse_standard(doc, body, currency, signatures).unwrap();
        if let V10Document::Certification(doc) = doc {
            println!("Doc : {:?}", doc);
            assert_eq!(doc.verify_signatures(), VerificationResult::Valid());
        /*assert_eq!(
                doc.generate_compact_text(),
                "2sZF6j2PkxBDNAqUde7Dgo5x3crkerZpQ4rBqqJGn8QT:\
                7jzkd8GiFnpys4X7mP78w2Y3y3kwdK6fVSLEaojd3aH9:99956:\
                Hkps1QU4HxIcNXKT8YmprYTVByBhPP1U2tIM7Z8wENzLKIWAvQClkAvBE7pW9dnVa18sJIJhVZUcRrPAZfmjBA=="
            );*/
        } else {
            panic!("Wrong document type");
        }
    }
}
