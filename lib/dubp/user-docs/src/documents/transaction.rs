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

//! Wrappers around Transaction documents.

pub mod v10;

use crate::documents::*;
use dubp_common_doc::blockstamp::Blockstamp;
use dubp_common_doc::parser::{DocumentsParser, TextDocumentParseError, TextDocumentParser};
use dubp_common_doc::traits::text::*;
use dubp_common_doc::traits::{Document, DocumentBuilder, ToStringObject};
use dup_crypto::hashs::*;
use dup_crypto::keys::*;
use durs_common_tools::{fatal_error, UsizeSer32};
use std::ops::{Add, Deref, Sub};
use unwrap::unwrap;

pub use v10::{
    TransactionDocumentV10, TransactionDocumentV10Builder, TransactionDocumentV10Parser,
    TransactionDocumentV10Stringified, TransactionInputV10, TransactionOutputV10,
};

/// Wrap a transaction amount
#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Deserialize, Hash, Serialize)]
pub struct TxAmount(pub isize);

impl Add for TxAmount {
    type Output = TxAmount;
    fn add(self, a: TxAmount) -> Self::Output {
        TxAmount(self.0 + a.0)
    }
}

impl Sub for TxAmount {
    type Output = TxAmount;
    fn sub(self, a: TxAmount) -> Self::Output {
        TxAmount(self.0 - a.0)
    }
}

/// Wrap a transaction amout base
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Deserialize, Hash, Serialize)]
pub struct TxBase(pub usize);

/// Wrap an output index
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct OutputIndex(pub usize);

/// Wrap a transaction unlock proof
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum TransactionUnlockProof {
    /// Indicates that the signature of the corresponding key is at the bottom of the document
    Sig(usize),
    /// Provides the code to unlock the corresponding funds
    Xhx(String),
}

impl ToString for TransactionUnlockProof {
    fn to_string(&self) -> String {
        match *self {
            TransactionUnlockProof::Sig(ref index) => format!("SIG({})", index),
            TransactionUnlockProof::Xhx(ref hash) => format!("XHX({})", hash),
        }
    }
}

/// Wrap a transaction ouput condition
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum TransactionOutputCondition {
    /// The consumption of funds will require a valid signature of the specified key
    Sig(PubKey),
    /// The consumption of funds will require to provide a code with the hash indicated
    Xhx(Hash),
    /// Funds may not be consumed until the blockchain reaches the timestamp indicated.
    Cltv(u64),
    /// Funds may not be consumed before the duration indicated, starting from the timestamp of the block where the transaction is written.
    Csv(u64),
}

impl ToString for TransactionOutputCondition {
    fn to_string(&self) -> String {
        match *self {
            TransactionOutputCondition::Sig(ref pubkey) => format!("SIG({})", pubkey),
            TransactionOutputCondition::Xhx(ref hash) => format!("XHX({})", hash),
            TransactionOutputCondition::Cltv(timestamp) => format!("CLTV({})", timestamp),
            TransactionOutputCondition::Csv(duration) => format!("CSV({})", duration),
        }
    }
}

/// Wrap an utxo conditions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct UTXOConditions {
    /// We are obliged to allow the introduction of the original text (instead of the self-generated text),
    /// because the original text may contain errors that are unfortunately allowed by duniter-ts.
    pub origin_str: Option<String>,
    /// Store script conditions
    pub conditions: UTXOConditionsGroup,
}

impl UTXOConditions {
    /// Lightens the UTXOConditions (for example to store it while minimizing the space required)
    pub fn reduce(&mut self) {
        if self.origin_str.is_some()
            && self.origin_str.clone().expect("safe unwrap") == self.conditions.to_string()
        {
            self.origin_str = None;
        }
    }
    /// Check validity of this UTXOConditions
    pub fn check(&self) -> bool {
        !(self.origin_str.is_some()
            && self.origin_str.clone().expect("safe unwrap") != self.conditions.to_string())
    }
}

impl ToString for UTXOConditions {
    fn to_string(&self) -> String {
        if let Some(ref origin_str) = self.origin_str {
            origin_str.to_string()
        } else {
            self.conditions.to_string()
        }
    }
}

/// Wrap a transaction ouput condition group
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum UTXOConditionsGroup {
    /// Single
    Single(TransactionOutputCondition),
    /// Brackets
    Brackets(Box<UTXOConditionsGroup>),
    /// And operator
    And(Box<UTXOConditionsGroup>, Box<UTXOConditionsGroup>),
    /// Or operator
    Or(Box<UTXOConditionsGroup>, Box<UTXOConditionsGroup>),
}

macro_rules! utxo_conds_wrap_op_chain {
    ($op:expr, $fn_name:ident) => {
        fn $fn_name(conds_subgroups: &mut Vec<UTXOConditionsGroup>) -> UTXOConditionsGroup {
            if conds_subgroups.len() == 2 {
                $op(
                    Box::new(conds_subgroups[0].clone()),
                    Box::new(conds_subgroups[1].clone()),
                )
            } else if conds_subgroups.len() > 2 {
                let last_subgroup = unwrap!(conds_subgroups.pop());
                let previous_last_subgroup = unwrap!(conds_subgroups.pop());
                conds_subgroups.push($op(
                    Box::new(previous_last_subgroup),
                    Box::new(last_subgroup),
                ));
                UTXOConditionsGroup::$fn_name(conds_subgroups)
            } else {
                fatal_error!(
                    "Grammar should ensure that and chain contains at least two conditions subgroups !"
                )
            }
        }
    }
}

impl UTXOConditionsGroup {
    /// Wrap UTXO and chain
    utxo_conds_wrap_op_chain!(UTXOConditionsGroup::And, new_and_chain);
    /// Wrap UTXO or chain
    utxo_conds_wrap_op_chain!(UTXOConditionsGroup::Or, new_or_chain);

    /// Wrap UTXO conditions
    pub fn from_pest_pair(pair: Pair<Rule>) -> UTXOConditionsGroup {
        match pair.as_rule() {
            Rule::output_and_group => {
                let and_pairs = pair.into_inner();
                let mut conds_subgroups: Vec<UTXOConditionsGroup> =
                    and_pairs.map(UTXOConditionsGroup::from_pest_pair).collect();
                UTXOConditionsGroup::Brackets(Box::new(UTXOConditionsGroup::new_and_chain(
                    &mut conds_subgroups,
                )))
            }
            Rule::output_or_group => {
                let or_pairs = pair.into_inner();
                let mut conds_subgroups: Vec<UTXOConditionsGroup> =
                    or_pairs.map(UTXOConditionsGroup::from_pest_pair).collect();
                UTXOConditionsGroup::Brackets(Box::new(UTXOConditionsGroup::new_or_chain(
                    &mut conds_subgroups,
                )))
            }
            Rule::output_cond_sig => UTXOConditionsGroup::Single(TransactionOutputCondition::Sig(
                PubKey::Ed25519(unwrap!(ed25519::PublicKey::from_base58(
                    unwrap!(pair.into_inner().next()).as_str()
                ))),
            )),
            Rule::output_cond_xhx => UTXOConditionsGroup::Single(TransactionOutputCondition::Xhx(
                unwrap!(Hash::from_hex(unwrap!(pair.into_inner().next()).as_str())),
            )),
            Rule::output_cond_csv => {
                UTXOConditionsGroup::Single(TransactionOutputCondition::Csv(unwrap!(unwrap!(pair
                    .into_inner()
                    .next())
                .as_str()
                .parse())))
            }
            Rule::output_cond_cltv => UTXOConditionsGroup::Single(
                TransactionOutputCondition::Cltv(unwrap!(unwrap!(pair.into_inner().next())
                    .as_str()
                    .parse())),
            ),
            _ => fatal_error!("unexpected rule: {:?}", pair.as_rule()), // Grammar ensures that we never reach this line
        }
    }
}

impl ToString for UTXOConditionsGroup {
    fn to_string(&self) -> String {
        match *self {
            UTXOConditionsGroup::Single(ref condition) => condition.to_string(),
            UTXOConditionsGroup::Brackets(ref condition_group) => {
                format!("({})", condition_group.deref().to_string())
            }
            UTXOConditionsGroup::And(ref condition_group_1, ref condition_group_2) => format!(
                "{} && {}",
                condition_group_1.deref().to_string(),
                condition_group_2.deref().to_string()
            ),
            UTXOConditionsGroup::Or(ref condition_group_1, ref condition_group_2) => format!(
                "{} || {}",
                condition_group_1.deref().to_string(),
                condition_group_2.deref().to_string()
            ),
        }
    }
}

pub trait TransactionDocumentTrait<'a> {
    type Input: 'a;
    type Inputs: AsRef<[Self::Input]>;
    type Output: 'a;
    type Outputs: AsRef<[Self::Output]>;
    fn get_inputs(&'a self) -> Self::Inputs;
    fn get_outputs(&'a self) -> Self::Outputs;
}

/// Wrap a Transaction document.
///
/// Must be created by parsing a text document or using a builder.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum TransactionDocument {
    V10(TransactionDocumentV10),
}

#[derive(Clone, Debug, Deserialize, Hash, Serialize, PartialEq, Eq)]
/// Transaction document stringifed
pub enum TransactionDocumentStringified {
    V10(TransactionDocumentV10Stringified),
}

impl ToStringObject for TransactionDocument {
    type StringObject = TransactionDocumentStringified;

    fn to_string_object(&self) -> TransactionDocumentStringified {
        match self {
            TransactionDocument::V10(tx_v10) => {
                TransactionDocumentStringified::V10(tx_v10.to_string_object())
            }
        }
    }
}

impl TransactionDocument {
    /// Compute transaction hash
    pub fn compute_hash(&self) -> Hash {
        match self {
            TransactionDocument::V10(tx_v10) => tx_v10.compute_hash(),
        }
    }
    /// get transaction hash option
    pub fn get_hash_opt(&self) -> Option<Hash> {
        match self {
            TransactionDocument::V10(tx_v10) => tx_v10.get_hash_opt(),
        }
    }
    /// Get transaction hash
    pub fn get_hash(&mut self) -> Hash {
        match self {
            TransactionDocument::V10(tx_v10) => tx_v10.get_hash(),
        }
    }
    /// Lightens the transaction (for example to store it while minimizing the space required)
    /// WARNING: do not remove the hash as it's necessary to reverse the transaction !
    pub fn reduce(&mut self) {
        match self {
            TransactionDocument::V10(tx_v10) => tx_v10.reduce(),
        };
    }
}

impl Document for TransactionDocument {
    type PublicKey = PubKey;

    fn version(&self) -> UsizeSer32 {
        match self {
            TransactionDocument::V10(tx_v10) => tx_v10.version(),
        }
    }

    fn currency(&self) -> &str {
        match self {
            TransactionDocument::V10(tx_v10) => tx_v10.currency(),
        }
    }

    fn blockstamp(&self) -> Blockstamp {
        match self {
            TransactionDocument::V10(tx_v10) => tx_v10.blockstamp(),
        }
    }

    fn issuers(&self) -> &Vec<PubKey> {
        match self {
            TransactionDocument::V10(tx_v10) => tx_v10.issuers(),
        }
    }

    fn signatures(&self) -> &Vec<Sig> {
        match self {
            TransactionDocument::V10(tx_v10) => tx_v10.signatures(),
        }
    }

    fn as_bytes(&self) -> &[u8] {
        match self {
            TransactionDocument::V10(tx_v10) => tx_v10.as_bytes(),
        }
    }
}

impl CompactTextDocument for TransactionDocument {
    fn as_compact_text(&self) -> String {
        match self {
            TransactionDocument::V10(tx_v10) => tx_v10.as_compact_text(),
        }
    }
}

impl TextDocument for TransactionDocument {
    type CompactTextDocument_ = TransactionDocument;

    fn as_text(&self) -> &str {
        match self {
            TransactionDocument::V10(tx_v10) => tx_v10.as_text(),
        }
    }

    fn to_compact_document(&self) -> Self::CompactTextDocument_ {
        self.clone()
    }
}

/// Transaction document builder.
#[derive(Debug, Copy, Clone)]
pub enum TransactionDocumentBuilder<'a> {
    V10(TransactionDocumentV10Builder<'a>),
}

impl<'a> DocumentBuilder for TransactionDocumentBuilder<'a> {
    type Document = TransactionDocument;
    type Signator = SignatorEnum;

    fn build_with_signature(&self, signatures: Vec<Sig>) -> TransactionDocument {
        match self {
            TransactionDocumentBuilder::V10(tx_v10_builder) => {
                TransactionDocument::V10(tx_v10_builder.build_with_signature(signatures))
            }
        }
    }
    fn build_and_sign(&self, private_keys: Vec<SignatorEnum>) -> TransactionDocument {
        match self {
            TransactionDocumentBuilder::V10(tx_v10_builder) => {
                TransactionDocument::V10(tx_v10_builder.build_and_sign(private_keys))
            }
        }
    }
}

impl<'a> TextDocumentBuilder for TransactionDocumentBuilder<'a> {
    fn generate_text(&self) -> String {
        match self {
            TransactionDocumentBuilder::V10(tx_v10_builder) => tx_v10_builder.generate_text(),
        }
    }
}

/// Transaction document parser
#[derive(Debug, Clone, Copy)]
pub enum TransactionDocumentParser {
    V10(TransactionDocumentV10Parser),
}

impl TextDocumentParser<Rule> for TransactionDocumentParser {
    type DocumentType = TransactionDocument;

    fn parse(doc: &str) -> Result<Self::DocumentType, TextDocumentParseError> {
        let mut tx_pairs = DocumentsParser::parse(Rule::tx, doc)?;
        let tx_pair = unwrap!(tx_pairs.next()); // get and unwrap the `tx` rule; never fails
        Self::from_pest_pair(tx_pair)
    }
    #[inline]
    fn from_pest_pair(pair: Pair<Rule>) -> Result<Self::DocumentType, TextDocumentParseError> {
        let tx_vx_pair = unwrap!(pair.into_inner().next()); // get and unwrap the `tx_vX` rule; never fails

        match tx_vx_pair.as_rule() {
            Rule::tx_v10 => {
                TransactionDocumentV10::from_pest_pair(tx_vx_pair).map(TransactionDocument::V10)
            }
            _ => Err(TextDocumentParseError::UnexpectedRule(format!(
                "{:#?}",
                tx_vx_pair.as_rule()
            ))),
        }
    }
    #[inline]
    fn from_versioned_pest_pair(
        version: u16,
        pair: Pair<Rule>,
    ) -> Result<Self::DocumentType, TextDocumentParseError> {
        match version {
            10 => TransactionDocumentV10Parser::from_versioned_pest_pair(version, pair)
                .map(TransactionDocument::V10),
            v => Err(TextDocumentParseError::UnexpectedVersion(format!(
                "Unsupported version: {}",
                v
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dubp_common_doc::traits::Document;
    use dubp_common_doc::BlockNumber;
    use std::str::FromStr;
    use v10::{TransactionInputUnlocksV10, TransactionOutputV10};

    #[test]
    fn generate_real_document() {
        let keypair = ed25519::KeyPairFromSeed32Generator::generate(unwrap!(
            Seed32::from_base58("DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV"),
            "Fail to parse Seed32"
        ));
        let pubkey = PubKey::Ed25519(keypair.public_key());
        let signator =
            SignatorEnum::Ed25519(keypair.generate_signator().expect("fail to gen signator"));

        let sig = Sig::Ed25519(unwrap!(ed25519::Signature::from_base64(
            "cq86RugQlqAEyS8zFkB9o0PlWPSb+a6D/MEnLe8j+okyFYf/WzI6pFiBkQ9PSOVn5I0dwzVXg7Q4N1apMWeGAg==",
        ), "Fail to parse Signature"));

        let block = unwrap!(
            Blockstamp::from_string(
                "0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
            ),
            "Fail to parse blockstamp"
        );

        let builder = TransactionDocumentV10Builder {
            currency: "duniter_unit_test_currency",
            blockstamp: &block,
            locktime: &0,
            issuers: &[pubkey],
            inputs: &[TransactionInputV10::D(
                TxAmount(10),
                TxBase(0),
                PubKey::Ed25519(unwrap!(
                    ed25519::PublicKey::from_base58("DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV"),
                    "Fail to parse PublicKey"
                )),
                BlockNumber(0),
            )],
            unlocks: &[TransactionInputUnlocksV10 {
                index: 0,
                unlocks: vec![TransactionUnlockProof::Sig(0)],
            }],
            outputs: &[TransactionOutputV10::from_str(
                "10:0:SIG(FD9wujR7KABw88RyKEGBYRLz8PA6jzVCbcBAsrBXBqSa)",
            )
            .expect("fail to parse output !")],
            comment: "test",
            hash: None,
        };
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
    fn compute_transaction_hash() {
        let pubkey = PubKey::Ed25519(unwrap!(
            ed25519::PublicKey::from_base58("FEkbc4BfJukSWnCU6Hed6dgwwTuPFTVdgz5LpL4iHr9J"),
            "Fail to parse PublicKey"
        ));

        let sig = Sig::Ed25519(unwrap!(ed25519::Signature::from_base64(
            "XEwKwKF8AI1gWPT7elR4IN+bW3Qn02Dk15TEgrKtY/S2qfZsNaodsLofqHLI24BBwZ5aadpC88ntmjo/UW9oDQ==",
        ), "Fail to parse Signature"));

        let block = unwrap!(
            Blockstamp::from_string(
                "60-00001FE00410FCD5991EDD18AA7DDF15F4C8393A64FA92A1DB1C1CA2E220128D",
            ),
            "Fail to parse Blockstamp"
        );

        let builder = TransactionDocumentV10Builder {
            currency: "g1",
            blockstamp: &block,
            locktime: &0,
            issuers: &[pubkey],
            inputs: &[TransactionInputV10::T(
                TxAmount(950),
                TxBase(0),
                unwrap!(
                    Hash::from_hex(
                        "2CF1ACD8FE8DC93EE39A1D55881C50D87C55892AE8E4DB71D4EBAB3D412AA8FD"
                    ),
                    "Fail to parse Hash"
                ),
                OutputIndex(1),
            )],
            unlocks: &[
                TransactionInputUnlocksV10::from_str("0:SIG(0)").expect("fail to parse unlock !")
            ],
            outputs: &[
                TransactionOutputV10::from_str(
                    "30:0:SIG(38MEAZN68Pz1DTvT3tqgxx4yQP6snJCQhPqEFxbDk4aE)",
                )
                .expect("fail to parse output !"),
                TransactionOutputV10::from_str(
                    "920:0:SIG(FEkbc4BfJukSWnCU6Hed6dgwwTuPFTVdgz5LpL4iHr9J)",
                )
                .expect("fail to parse output !"),
            ],
            comment: "Pour cesium merci",
            hash: None,
        };
        let mut tx_doc = TransactionDocument::V10(builder.build_with_signature(vec![sig]));
        assert!(tx_doc.verify_signatures().is_ok());
        assert!(tx_doc.get_hash_opt().is_none());
        assert_eq!(
            tx_doc.get_hash(),
            Hash::from_hex("876D2430E0B66E2CE4467866D8F923D68896CACD6AA49CDD8BDD0096B834DEF1")
                .expect("fail to parse hash")
        );
    }

    #[test]
    fn parse_transaction_document() {
        let doc = "Version: 10
Type: Transaction
Currency: duniter_unit_test_currency
Blockstamp: 204-00003E2B8A35370BA5A7064598F628A62D4E9EC1936BE8651CE9A85F2E06981B
Locktime: 0
Issuers:
DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV
4tNQ7d9pj2Da5wUVoW9mFn7JjuPoowF977au8DdhEjVR
FD9wujR7KABw88RyKEGBYRLz8PA6jzVCbcBAsrBXBqSa
Inputs:
40:2:T:6991C993631BED4733972ED7538E41CCC33660F554E3C51963E2A0AC4D6453D3:2
70:2:T:3A09A20E9014110FD224889F13357BAB4EC78A72F95CA03394D8CCA2936A7435:8
20:2:D:DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV:46
70:2:T:A0D9B4CDC113ECE1145C5525873821398890AE842F4B318BD076095A23E70956:3
20:2:T:67F2045B5318777CC52CD38B424F3E40DDA823FA0364625F124BABE0030E7B5B:5
15:2:D:FD9wujR7KABw88RyKEGBYRLz8PA6jzVCbcBAsrBXBqSa:46
Unlocks:
0:SIG(0)
1:XHX(7665798292)
2:SIG(0)
3:SIG(0) SIG(2)
4:SIG(0) SIG(1) SIG(2)
5:SIG(2)
Outputs:
120:2:SIG(BYfWYFrsyjpvpFysgu19rGK3VHBkz4MqmQbNyEuVU64g)
146:2:SIG(DSz4rgncXCytsUMW2JU2yhLquZECD2XpEkpP9gG5HyAx)
49:2:(SIG(6DyGr5LFtFmbaJYRvcs9WmBsr4cbJbJ1EV9zBbqG7A6i) || XHX(3EB4702F2AC2FD3FA4FDC46A4FC05AE8CDEE1A85F2AC2FD3FA4FDC46A4FC01CA))
Comment: -----@@@----- (why not this comment?)
kL59C1izKjcRN429AlKdshwhWbasvyL7sthI757zm1DfZTdTIctDWlKbYeG/tS7QyAgI3gcfrTHPhu1E1lKCBw==
e3LpgB2RZ/E/BCxPJsn+TDDyxGYzrIsMyDt//KhJCjIQD6pNUxr5M5jrq2OwQZgwmz91YcmoQ2XRQAUDpe4BAw==
w69bYgiQxDmCReB0Dugt9BstXlAKnwJkKCdWvCeZ9KnUCv0FJys6klzYk/O/b9t74tYhWZSX0bhETWHiwfpWBw==";

        let doc = TransactionDocumentParser::parse(doc)
            .expect("fail to parse test transaction document !");
        //println!("Doc : {:?}", doc);
        println!("{}", doc.generate_compact_text());
        assert!(doc.verify_signatures().is_ok());
        assert_eq!(
            doc.generate_compact_text(),
            "TX:10:3:6:6:3:1:0
204-00003E2B8A35370BA5A7064598F628A62D4E9EC1936BE8651CE9A85F2E06981B
DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV
4tNQ7d9pj2Da5wUVoW9mFn7JjuPoowF977au8DdhEjVR
FD9wujR7KABw88RyKEGBYRLz8PA6jzVCbcBAsrBXBqSa
40:2:T:6991C993631BED4733972ED7538E41CCC33660F554E3C51963E2A0AC4D6453D3:2
70:2:T:3A09A20E9014110FD224889F13357BAB4EC78A72F95CA03394D8CCA2936A7435:8
20:2:D:DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV:46
70:2:T:A0D9B4CDC113ECE1145C5525873821398890AE842F4B318BD076095A23E70956:3
20:2:T:67F2045B5318777CC52CD38B424F3E40DDA823FA0364625F124BABE0030E7B5B:5
15:2:D:FD9wujR7KABw88RyKEGBYRLz8PA6jzVCbcBAsrBXBqSa:46
0:SIG(0)
1:XHX(7665798292)
2:SIG(0)
3:SIG(0) SIG(2)
4:SIG(0) SIG(1) SIG(2)
5:SIG(2)
120:2:SIG(BYfWYFrsyjpvpFysgu19rGK3VHBkz4MqmQbNyEuVU64g)
146:2:SIG(DSz4rgncXCytsUMW2JU2yhLquZECD2XpEkpP9gG5HyAx)
49:2:(SIG(6DyGr5LFtFmbaJYRvcs9WmBsr4cbJbJ1EV9zBbqG7A6i) || XHX(3EB4702F2AC2FD3FA4FDC46A4FC05AE8CDEE1A85F2AC2FD3FA4FDC46A4FC01CA))
-----@@@----- (why not this comment?)
kL59C1izKjcRN429AlKdshwhWbasvyL7sthI757zm1DfZTdTIctDWlKbYeG/tS7QyAgI3gcfrTHPhu1E1lKCBw==
e3LpgB2RZ/E/BCxPJsn+TDDyxGYzrIsMyDt//KhJCjIQD6pNUxr5M5jrq2OwQZgwmz91YcmoQ2XRQAUDpe4BAw==
w69bYgiQxDmCReB0Dugt9BstXlAKnwJkKCdWvCeZ9KnUCv0FJys6klzYk/O/b9t74tYhWZSX0bhETWHiwfpWBw=="
        );
    }
}
