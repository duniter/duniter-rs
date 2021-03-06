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

//! Wrappers around Membership documents v10.

use crate::documents::*;
use dubp_common_doc::blockstamp::Blockstamp;
use dubp_common_doc::parser::TextDocumentParseError;
use dubp_common_doc::traits::text::*;
use dubp_common_doc::traits::{Document, DocumentBuilder, ToStringObject};
use dubp_common_doc::{BlockHash, BlockNumber};
use dup_crypto::hashs::Hash;
use dup_crypto::keys::*;
use durs_common_tools::{fatal_error, UsizeSer32};

/// Type of a Membership.
#[derive(Debug, Deserialize, Clone, Copy, Hash, Serialize, PartialEq, Eq)]
pub enum MembershipType {
    /// The member wishes to opt-in.
    In(),
    /// The member wishes to opt-out.
    Out(),
}

/// Wrap an Membership document.
///
/// Must be created by parsing a text document or using a builder.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct MembershipDocumentV10 {
    /// Document as text.
    ///
    /// Is used to check signatures, and other values mut be extracted from it.
    text: Option<String>,

    /// Name of the currency.
    currency: String,
    /// Document issuer (there should be only one).
    issuers: Vec<PubKey>,
    /// Blockstamp
    blockstamp: Blockstamp,
    /// Membership message.
    membership: MembershipType,
    /// Identity to use for this public key.
    identity_username: String,
    /// Identity document blockstamp.
    identity_blockstamp: Blockstamp,
    /// Document signature (there should be only one).
    signatures: Vec<Sig>,
}

#[derive(Clone, Debug, Deserialize, Hash, Serialize, PartialEq, Eq)]
/// identity document for jsonification
pub struct MembershipDocumentV10Stringified {
    /// Currency.
    pub currency: String,
    /// Document issuer
    pub issuer: String,
    /// Blockstamp
    pub blockstamp: String,
    /// Membership message.
    pub membership: String,
    /// Unique ID
    pub username: String,
    /// Identity document blockstamp.
    pub identity_blockstamp: String,
    /// Document signature
    pub signature: String,
}

impl ToStringObject for MembershipDocumentV10 {
    type StringObject = MembershipDocumentV10Stringified;
    /// Transforms an object into a json object
    fn to_string_object(&self) -> MembershipDocumentV10Stringified {
        MembershipDocumentV10Stringified {
            currency: self.currency.clone(),
            issuer: format!("{}", self.issuers[0]),
            blockstamp: format!("{}", self.blockstamp),
            membership: match self.membership {
                MembershipType::In() => "IN".to_owned(),
                MembershipType::Out() => "OUT".to_owned(),
            },
            username: self.identity_username.clone(),
            identity_blockstamp: format!("{}", self.identity_blockstamp),
            signature: format!("{}", self.signatures[0]),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Deserialize, Serialize)]
/// Membership event type (blockchain event)
pub enum MembershipEventType {
    /// Newcomer
    Join(),
    /// Renewal
    Renewal(),
    /// Renewal after expire or leave
    Rejoin(),
    /// Expire
    Expire(),
}

#[derive(Debug, Clone, PartialEq, Hash, Deserialize, Serialize)]
/// Membership event (blockchain event)
pub struct MembershipEvent {
    /// Blockstamp of block event
    pub blockstamp: Blockstamp,
    /// Membership document
    pub doc: MembershipDocumentV10,
    /// Event type
    pub event_type: MembershipEventType,
    /// Chainable time
    pub chainable_on: u64,
}

impl MembershipDocumentV10 {
    /// Membership message.
    pub fn membership(&self) -> MembershipType {
        self.membership
    }

    /// Identity to use for this public key.
    pub fn identity_username(&self) -> &str {
        &self.identity_username
    }

    /// Lightens the membership (for example to store it while minimizing the space required)
    pub fn reduce(&mut self) {
        self.text = None;
    }

    /// From pest parser pair
    pub fn from_pest_pair(
        pair: Pair<Rule>,
    ) -> Result<MembershipDocumentV10, TextDocumentParseError> {
        let doc = pair.as_str();
        let mut currency = "";
        let mut pubkey_str = "";
        let mut uid = "";
        let mut blockstamps = Vec::with_capacity(2);
        let mut membership = MembershipType::In();
        let mut sig_str = "";
        for field in pair.into_inner() {
            match field.as_rule() {
                Rule::currency => currency = field.as_str(),
                Rule::uid => uid = field.as_str(),
                Rule::pubkey => pubkey_str = field.as_str(),
                Rule::membership_in => membership = MembershipType::In(),
                Rule::membership_out => membership = MembershipType::Out(),
                Rule::blockstamp => {
                    let mut inner_rules = field.into_inner(); // { integer ~ "-" ~ hash }

                    let block_id: &str = unwrap!(inner_rules.next()).as_str();
                    let block_hash: &str = unwrap!(inner_rules.next()).as_str();
                    blockstamps.push(Blockstamp {
                        id: BlockNumber(unwrap!(block_id.parse())), // Grammar ensures that we have a digits string.
                        hash: BlockHash(unwrap!(Hash::from_hex(block_hash))), // Grammar ensures that we have an hexadecimal string.
                    });
                }
                Rule::ed25519_sig => sig_str = field.as_str(),
                Rule::EOI => (),
                _ => fatal_error!("unexpected rule"), // Grammar ensures that we never reach this line
            }
        }

        Ok(MembershipDocumentV10 {
            text: Some(doc.to_owned()),
            issuers: vec![PubKey::Ed25519(unwrap!(ed25519::PublicKey::from_base58(
                pubkey_str
            )))], // Grammar ensures that we have a base58 string.
            currency: currency.to_owned(),
            blockstamp: blockstamps[0],
            membership,
            identity_username: uid.to_owned(),
            identity_blockstamp: blockstamps[1],
            signatures: vec![Sig::Ed25519(unwrap!(ed25519::Signature::from_base64(
                sig_str
            )))], // Grammar ensures that we have a base64 string.
        })
    }
}

impl Document for MembershipDocumentV10 {
    type PublicKey = PubKey;

    fn version(&self) -> UsizeSer32 {
        UsizeSer32(10)
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
        self.as_text_without_signature().as_bytes()
    }
}

impl CompactTextDocument for MembershipDocumentV10 {
    fn as_compact_text(&self) -> String {
        format!(
            "{issuer}:{signature}:{blockstamp}:{idty_blockstamp}:{username}",
            issuer = self.issuers[0],
            signature = self.signatures[0],
            blockstamp = self.blockstamp,
            idty_blockstamp = self.identity_blockstamp,
            username = self.identity_username,
        )
    }
}

/// CompactPoolMembershipDoc
#[derive(Copy, Clone, Debug, Deserialize, Hash, Serialize, PartialEq, Eq)]
pub struct CompactPoolMembershipDoc {
    /// Document creation blockstamp
    pub blockstamp: Blockstamp,
    /// Signature
    pub signature: Sig,
}

impl TextDocument for MembershipDocumentV10 {
    type CompactTextDocument_ = MembershipDocumentV10;

    fn as_text(&self) -> &str {
        if let Some(ref text) = self.text {
            text
        } else {
            fatal_error!("Try to get text of reduce membership !")
        }
    }

    fn to_compact_document(&self) -> Self::CompactTextDocument_ {
        self.clone()
    }
}

/// Membership document builder.
#[derive(Debug, Copy, Clone)]
pub struct MembershipDocumentV10Builder<'a> {
    /// Document currency.
    pub currency: &'a str,
    /// Document/identity issuer.
    pub issuer: &'a PubKey,
    /// Reference blockstamp.
    pub blockstamp: &'a Blockstamp,
    /// Membership message.
    pub membership: MembershipType,
    /// Identity username.
    pub identity_username: &'a str,
    /// Identity document blockstamp.
    pub identity_blockstamp: &'a Blockstamp,
}

impl<'a> MembershipDocumentV10Builder<'a> {
    fn build_with_text_and_sigs(self, text: String, signatures: Vec<Sig>) -> MembershipDocumentV10 {
        MembershipDocumentV10 {
            text: Some(text),
            currency: self.currency.to_string(),
            issuers: vec![*self.issuer],
            blockstamp: *self.blockstamp,
            membership: self.membership,
            identity_username: self.identity_username.to_string(),
            identity_blockstamp: *self.identity_blockstamp,
            signatures,
        }
    }
}

impl<'a> DocumentBuilder for MembershipDocumentV10Builder<'a> {
    type Document = MembershipDocumentV10;
    type Signator = SignatorEnum;

    fn build_with_signature(&self, signatures: Vec<Sig>) -> MembershipDocumentV10 {
        self.build_with_text_and_sigs(self.generate_text(), signatures)
    }

    fn build_and_sign(&self, private_keys: Vec<SignatorEnum>) -> MembershipDocumentV10 {
        let (text, signatures) = self.build_signed_text(private_keys);
        self.build_with_text_and_sigs(text, signatures)
    }
}

impl<'a> TextDocumentBuilder for MembershipDocumentV10Builder<'a> {
    fn generate_text(&self) -> String {
        format!(
            "Version: 10
Type: Membership
Currency: {currency}
Issuer: {issuer}
Block: {blockstamp}
Membership: {membership}
UserID: {username}
CertTS: {ity_blockstamp}
",
            currency = self.currency,
            issuer = self.issuer,
            blockstamp = self.blockstamp,
            membership = match self.membership {
                MembershipType::In() => "IN",
                MembershipType::Out() => "OUT",
            },
            username = self.identity_username,
            ity_blockstamp = self.identity_blockstamp,
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

        let sig = Sig::Ed25519(
            unwrap!(ed25519::Signature::from_base64(
                "cUgoc8AI+Tae/AZmRfTnW+xq3XFtmYoUi2LXlmXr8/7LaXiUccQb8+Ds1nZoBp/8+t031HMwqAUpVIqww2FGCg==",
            )
            , "fail to build Signature"),
        );

        let block = unwrap!(
            Blockstamp::from_string(
                "0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
            ),
            "fail to build Blockstamp"
        );

        let builder = MembershipDocumentV10Builder {
            currency: "duniter_unit_test_currency",
            issuer: &pubkey,
            blockstamp: &block,
            membership: MembershipType::In(),
            identity_username: "tic",
            identity_blockstamp: &block,
        };

        /*println!(
            "Signatures = {:?}",
            builder
                .build_and_sign(vec![SignatorEnum::Ed25519(
                    keypair.generate_signator().expect("fail to gen signator")
                )])
                .signatures()
        );*/

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
    fn membership_identity_document() {
        let doc = "Version: 10
Type: Membership
Currency: duniter_unit_test_currency
Issuer: DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV
Block: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
Membership: IN
UserID: tic
CertTS: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
s2hUbokkibTAWGEwErw6hyXSWlWFQ2UWs2PWx8d/kkElAyuuWaQq4Tsonuweh1xn4AC1TVWt4yMR3WrDdkhnAw==";

        let doc =
            MembershipDocumentParser::parse(doc).expect("fail to parse test membership document !");
        println!("Doc : {:?}", doc);
        assert!(doc.verify_signatures().is_ok());
        assert_eq!(
            doc.generate_compact_text(),
                "DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV:\
                s2hUbokkibTAWGEwErw6hyXSWlWFQ2UWs2PWx8d/kkElAyuuWaQq4Tsonuweh1xn4AC1TVWt4yMR3WrDdkhnAw==:\
                0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855:\
                0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855:\
                tic"
            );
    }
}
