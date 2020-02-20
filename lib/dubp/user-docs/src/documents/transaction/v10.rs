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

use crate::documents::*;
use dubp_common_doc::blockstamp::Blockstamp;
use dubp_common_doc::parser::{DocumentsParser, TextDocumentParseError, TextDocumentParser};
use dubp_common_doc::traits::text::*;
use dubp_common_doc::traits::{Document, DocumentBuilder, ToStringObject};
use dubp_common_doc::{BlockHash, BlockNumber};
use dup_crypto::hashs::*;
use dup_crypto::keys::*;
use durs_common_tools::fatal_error;
use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;
use std::str::FromStr;
use unwrap::unwrap;

/// Wrap a transaction input
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum TransactionInputV10 {
    /// Universal Dividend Input
    D(TxAmount, TxBase, PubKey, BlockNumber),
    /// Previous Transaction Input
    T(TxAmount, TxBase, Hash, OutputIndex),
}

impl ToString for TransactionInputV10 {
    fn to_string(&self) -> String {
        match *self {
            TransactionInputV10::D(amount, base, pubkey, block_number) => {
                format!("{}:{}:D:{}:{}", amount.0, base.0, pubkey, block_number.0)
            }
            TransactionInputV10::T(amount, base, ref tx_hash, tx_index) => {
                format!("{}:{}:T:{}:{}", amount.0, base.0, tx_hash, tx_index.0)
            }
        }
    }
}

impl TransactionInputV10 {
    fn from_pest_pair(mut pairs: Pairs<Rule>) -> TransactionInputV10 {
        let tx_input_type_pair = unwrap!(pairs.next());
        match tx_input_type_pair.as_rule() {
            Rule::tx_input_du => {
                let mut inner_rules = tx_input_type_pair.into_inner(); // ${ tx_amount ~ ":" ~ tx_amount_base ~ ":D:" ~ pubkey ~ ":" ~ du_block_id }

                TransactionInputV10::D(
                    TxAmount(unwrap!(unwrap!(inner_rules.next()).as_str().parse())),
                    TxBase(unwrap!(unwrap!(inner_rules.next()).as_str().parse())),
                    PubKey::Ed25519(unwrap!(ed25519::PublicKey::from_base58(
                        unwrap!(inner_rules.next()).as_str()
                    ))),
                    BlockNumber(unwrap!(unwrap!(inner_rules.next()).as_str().parse())),
                )
            }
            Rule::tx_input_tx => {
                let mut inner_rules = tx_input_type_pair.into_inner(); // ${ tx_amount ~ ":" ~ tx_amount_base ~ ":D:" ~ pubkey ~ ":" ~ du_block_id }

                TransactionInputV10::T(
                    TxAmount(unwrap!(unwrap!(inner_rules.next()).as_str().parse())),
                    TxBase(unwrap!(unwrap!(inner_rules.next()).as_str().parse())),
                    unwrap!(Hash::from_hex(unwrap!(inner_rules.next()).as_str())),
                    OutputIndex(unwrap!(unwrap!(inner_rules.next()).as_str().parse())),
                )
            }
            _ => fatal_error!("unexpected rule: {:?}", tx_input_type_pair.as_rule()), // Grammar ensures that we never reach this line
        }
    }
}

impl FromStr for TransactionInputV10 {
    type Err = TextDocumentParseError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        match DocumentsParser::parse(Rule::tx_input, source) {
            Ok(mut pairs) => Ok(TransactionInputV10::from_pest_pair(
                unwrap!(pairs.next()).into_inner(),
            )),
            Err(_) => Err(TextDocumentParseError::InvalidInnerFormat(
                "Invalid unlocks !".to_owned(),
            )),
        }
    }
}

impl<'a> TransactionDocumentTrait<'a> for TransactionDocumentV10 {
    type Input = TransactionInputV10;
    type Inputs = &'a [TransactionInputV10];
    type Output = TransactionOutputV10;
    type Outputs = &'a [TransactionOutputV10];
    fn get_inputs(&'a self) -> Self::Inputs {
        &self.inputs
    }
    fn get_outputs(&'a self) -> Self::Outputs {
        &self.outputs
    }
}

/// Wrap a transaction unlocks input
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TransactionInputUnlocksV10 {
    /// Input index
    pub index: usize,
    /// List of proof to unlock funds
    pub unlocks: Vec<TransactionUnlockProof>,
}

impl ToString for TransactionInputUnlocksV10 {
    fn to_string(&self) -> String {
        let mut result: String = format!("{}:", self.index);
        for unlock in &self.unlocks {
            result.push_str(&format!("{} ", unlock.to_string()));
        }
        let new_size = result.len() - 1;
        result.truncate(new_size);
        result
    }
}

impl TransactionInputUnlocksV10 {
    fn from_pest_pair(pairs: Pairs<Rule>) -> TransactionInputUnlocksV10 {
        let mut input_index = 0;
        let mut unlock_conds = Vec::new();
        for unlock_field in pairs {
            // ${ input_index ~ ":" ~ unlock_cond ~ (" " ~ unlock_cond)* }
            match unlock_field.as_rule() {
                Rule::input_index => input_index = unwrap!(unlock_field.as_str().parse()),
                Rule::unlock_sig => {
                    unlock_conds.push(TransactionUnlockProof::Sig(unwrap!(unwrap!(unlock_field
                        .into_inner()
                        .next())
                    .as_str()
                    .parse())))
                }
                Rule::unlock_xhx => unlock_conds.push(TransactionUnlockProof::Xhx(String::from(
                    unwrap!(unlock_field.into_inner().next()).as_str(),
                ))),
                _ => fatal_error!("unexpected rule: {:?}", unlock_field.as_rule()), // Grammar ensures that we never reach this line
            }
        }
        TransactionInputUnlocksV10 {
            index: input_index,
            unlocks: unlock_conds,
        }
    }
}

impl FromStr for TransactionInputUnlocksV10 {
    type Err = TextDocumentParseError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        match DocumentsParser::parse(Rule::tx_unlock, source) {
            Ok(mut pairs) => Ok(TransactionInputUnlocksV10::from_pest_pair(
                unwrap!(pairs.next()).into_inner(),
            )),
            Err(_) => Err(TextDocumentParseError::InvalidInnerFormat(
                "Invalid unlocks !".to_owned(),
            )),
        }
    }
}

/// Wrap a transaction ouput
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TransactionOutputV10 {
    /// Amount
    pub amount: TxAmount,
    /// Base
    pub base: TxBase,
    /// List of conditions for consum this output
    pub conditions: UTXOConditions,
}

impl TransactionOutputV10 {
    /// Lightens the TransactionOutputV10 (for example to store it while minimizing the space required)
    fn reduce(&mut self) {
        self.conditions.reduce()
    }
    /// Check validity of this output
    pub fn check(&self) -> bool {
        self.conditions.check()
    }
}

impl ToString for TransactionOutputV10 {
    fn to_string(&self) -> String {
        format!(
            "{}:{}:{}",
            self.amount.0,
            self.base.0,
            self.conditions.to_string()
        )
    }
}

impl TransactionOutputV10 {
    fn from_pest_pair(mut utxo_pairs: Pairs<Rule>) -> TransactionOutputV10 {
        let amount = TxAmount(unwrap!(unwrap!(utxo_pairs.next()).as_str().parse()));
        let base = TxBase(unwrap!(unwrap!(utxo_pairs.next()).as_str().parse()));
        let conditions_pairs = unwrap!(utxo_pairs.next());
        let conditions_origin_str = conditions_pairs.as_str();
        TransactionOutputV10 {
            amount,
            base,
            conditions: UTXOConditions {
                origin_str: Some(String::from(conditions_origin_str)),
                conditions: UTXOConditionsGroup::from_pest_pair(conditions_pairs),
            },
        }
    }
}

impl FromStr for TransactionOutputV10 {
    type Err = TextDocumentParseError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let output_parts: Vec<&str> = source.split(':').collect();
        let amount = output_parts.get(0);
        let base = output_parts.get(1);
        let conditions_origin_str = output_parts.get(2);

        let str_to_parse = if amount.is_some() && base.is_some() && conditions_origin_str.is_some()
        {
            format!(
                "{}:{}:({})",
                unwrap!(amount),
                unwrap!(base),
                unwrap!(conditions_origin_str)
            )
        } else {
            source.to_owned()
        };

        match DocumentsParser::parse(Rule::tx_output, &str_to_parse) {
            Ok(mut utxo_pairs) => {
                let mut output =
                    TransactionOutputV10::from_pest_pair(unwrap!(utxo_pairs.next()).into_inner());
                output.conditions.origin_str = conditions_origin_str.map(ToString::to_string);
                Ok(output)
            }
            Err(_) => match DocumentsParser::parse(Rule::tx_output, source) {
                Ok(mut utxo_pairs) => {
                    let mut output = TransactionOutputV10::from_pest_pair(
                        unwrap!(utxo_pairs.next()).into_inner(),
                    );
                    output.conditions.origin_str = conditions_origin_str.map(ToString::to_string);
                    Ok(output)
                }
                Err(e) => Err(TextDocumentParseError::InvalidInnerFormat(format!(
                    "Invalid output : {}",
                    e
                ))),
            },
        }
    }
}

/// Wrap a Transaction document.
///
/// Must be created by parsing a text document or using a builder.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TransactionDocumentV10 {
    /// Document as text.
    ///
    /// Is used to check signatures, and other values
    /// must be extracted from it.
    text: Option<String>,

    /// Currency.
    currency: String,
    /// Blockstamp
    blockstamp: Blockstamp,
    /// Locktime
    locktime: u64,
    /// Document issuers.
    issuers: Vec<PubKey>,
    /// Transaction inputs.
    inputs: Vec<TransactionInputV10>,
    /// Inputs unlocks.
    unlocks: Vec<TransactionInputUnlocksV10>,
    /// Transaction outputs.
    outputs: Vec<TransactionOutputV10>,
    /// Transaction comment
    comment: String,
    /// Document signatures.
    signatures: Vec<Sig>,
    /// Transaction hash
    hash: Option<Hash>,
}

#[derive(Clone, Debug, Deserialize, Hash, Serialize, PartialEq, Eq)]
/// Transaction document stringifed
pub struct TransactionDocumentV10Stringified {
    /// Currency.
    pub currency: String,
    /// Blockstamp
    pub blockstamp: String,
    /// Locktime
    pub locktime: u64,
    /// Document issuers.
    pub issuers: Vec<String>,
    /// Transaction inputs.
    pub inputs: Vec<String>,
    /// Inputs unlocks.
    pub unlocks: Vec<String>,
    /// Transaction outputs.
    pub outputs: Vec<String>,
    /// Transaction comment
    pub comment: String,
    /// Document signatures
    pub signatures: Vec<String>,
    /// Transaction hash
    pub hash: Option<String>,
}

impl ToStringObject for TransactionDocumentV10 {
    type StringObject = TransactionDocumentV10Stringified;

    fn to_string_object(&self) -> TransactionDocumentV10Stringified {
        TransactionDocumentV10Stringified {
            currency: self.currency.clone(),
            blockstamp: format!("{}", self.blockstamp),
            locktime: self.locktime,
            issuers: self.issuers.iter().map(|p| format!("{}", p)).collect(),
            inputs: self
                .inputs
                .iter()
                .map(TransactionInputV10::to_string)
                .collect(),
            unlocks: self
                .unlocks
                .iter()
                .map(TransactionInputUnlocksV10::to_string)
                .collect(),
            outputs: self
                .outputs
                .iter()
                .map(TransactionOutputV10::to_string)
                .collect(),
            comment: self.comment.clone(),
            signatures: self.signatures.iter().map(|s| format!("{}", s)).collect(),
            hash: if let Some(hash) = self.hash {
                Some(hash.to_string())
            } else {
                None
            },
        }
    }
}

impl TransactionDocumentV10 {
    /// Compute transaction hash
    pub fn compute_hash(&self) -> Hash {
        let mut hashing_text = if let Some(ref text) = self.text {
            text.clone()
        } else {
            fatal_error!("Try to compute_hash of tx with None text !")
        };
        for sig in &self.signatures {
            hashing_text.push_str(&sig.to_string());
            hashing_text.push_str("\n");
        }
        Hash::compute_str(&hashing_text)
    }
    /// get transaction hash option
    pub fn get_hash_opt(&self) -> Option<Hash> {
        self.hash
    }
    /// Get transaction hash
    pub fn get_hash(&mut self) -> Hash {
        if let Some(hash) = self.hash {
            hash
        } else {
            self.hash = Some(self.compute_hash());
            self.hash.expect("unreach")
        }
    }
    /// Lightens the transaction (for example to store it while minimizing the space required)
    /// WARNING: do not remove the hash as it's necessary to reverse the transaction !
    pub fn reduce(&mut self) {
        self.hash = Some(self.compute_hash());
        self.text = None;
        for output in &mut self.outputs {
            output.reduce()
        }
    }
    /// from pest parser pair
    pub fn from_pest_pair(
        pair: Pair<Rule>,
    ) -> Result<TransactionDocumentV10, TextDocumentParseError> {
        let doc = pair.as_str();
        let mut currency = "";
        let mut blockstamp = Blockstamp::default();
        let mut locktime = 0;
        let mut issuers = Vec::new();
        let mut inputs = Vec::new();
        let mut unlocks = Vec::new();
        let mut outputs = Vec::new();
        let mut comment = "";
        let mut sigs = Vec::new();

        for field in pair.into_inner() {
            match field.as_rule() {
                Rule::currency => currency = field.as_str(),
                Rule::blockstamp => {
                    let mut inner_rules = field.into_inner(); // ${ block_id ~ "-" ~ hash }

                    let block_id: &str = unwrap!(inner_rules.next()).as_str();
                    let block_hash: &str = unwrap!(inner_rules.next()).as_str();
                    blockstamp = Blockstamp {
                        id: BlockNumber(unwrap!(block_id.parse())), // Grammar ensures that we have a digits string.
                        hash: BlockHash(unwrap!(Hash::from_hex(block_hash))), // Grammar ensures that we have an hexadecimal string.
                    };
                }
                Rule::tx_locktime => locktime = unwrap!(field.as_str().parse()), // Grammar ensures that we have digits characters.
                Rule::pubkey => issuers.push(PubKey::Ed25519(
                    unwrap!(ed25519::PublicKey::from_base58(field.as_str())), // Grammar ensures that we have a base58 string.
                )),
                Rule::tx_input => {
                    inputs.push(TransactionInputV10::from_pest_pair(field.into_inner()))
                }
                Rule::tx_unlock => unlocks.push(TransactionInputUnlocksV10::from_pest_pair(
                    field.into_inner(),
                )),
                Rule::tx_output => {
                    outputs.push(TransactionOutputV10::from_pest_pair(field.into_inner()))
                }
                Rule::tx_comment => comment = field.as_str(),
                Rule::ed25519_sig => {
                    sigs.push(Sig::Ed25519(
                        unwrap!(ed25519::Signature::from_base64(field.as_str())), // Grammar ensures that we have a base64 string.
                    ));
                }
                Rule::EOI => (),
                _ => fatal_error!("unexpected rule: {:?}", field.as_rule()), // Grammar ensures that we never reach this line
            }
        }

        Ok(TransactionDocumentV10 {
            text: Some(doc.to_owned()),
            currency: currency.to_owned(),
            blockstamp,
            locktime,
            issuers,
            inputs,
            unlocks,
            outputs,
            comment: comment.to_owned(),
            signatures: sigs,
            hash: None,
        })
    }
}

impl Document for TransactionDocumentV10 {
    type PublicKey = PubKey;

    fn version(&self) -> usize {
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
        self.as_text_without_signature().as_bytes()
    }
}

impl CompactTextDocument for TransactionDocumentV10 {
    fn as_compact_text(&self) -> String {
        let mut issuers_str = String::from("");
        for issuer in self.issuers.clone() {
            issuers_str.push_str("\n");
            issuers_str.push_str(&issuer.to_string());
        }
        let mut inputs_str = String::from("");
        for input in self.inputs.clone() {
            inputs_str.push_str("\n");
            inputs_str.push_str(&input.to_string());
        }
        let mut unlocks_str = String::from("");
        for unlock in self.unlocks.clone() {
            unlocks_str.push_str("\n");
            unlocks_str.push_str(&unlock.to_string());
        }
        let mut outputs_str = String::from("");
        for output in self.outputs.clone() {
            outputs_str.push_str("\n");
            outputs_str.push_str(&output.to_string());
        }
        let mut comment_str = self.comment.clone();
        if !comment_str.is_empty() {
            comment_str.push_str("\n");
        }
        let mut signatures_str = String::from("");
        for sig in self.signatures.clone() {
            signatures_str.push_str(&sig.to_string());
            signatures_str.push_str("\n");
        }
        // Remove end line step
        signatures_str.pop();
        format!(
            "TX:10:{issuers_count}:{inputs_count}:{unlocks_count}:{outputs_count}:{has_comment}:{locktime}
{blockstamp}{issuers}{inputs}{unlocks}{outputs}\n{comment}{signatures}",
            issuers_count = self.issuers.len(),
            inputs_count = self.inputs.len(),
            unlocks_count = self.unlocks.len(),
            outputs_count = self.outputs.len(),
            has_comment = if self.comment.is_empty() { 0 } else { 1 },
            locktime = self.locktime,
            blockstamp = self.blockstamp,
            issuers = issuers_str,
            inputs = inputs_str,
            unlocks = unlocks_str,
            outputs = outputs_str,
            comment = comment_str,
            signatures = signatures_str,
        )
    }
}

impl TextDocument for TransactionDocumentV10 {
    type CompactTextDocument_ = TransactionDocumentV10;

    fn as_text(&self) -> &str {
        if let Some(ref text) = self.text {
            text
        } else {
            fatal_error!("Try to get text of tx with None text !")
        }
    }

    fn to_compact_document(&self) -> Self::CompactTextDocument_ {
        self.clone()
    }
}

/// Transaction document builder.
#[derive(Debug, Copy, Clone)]
pub struct TransactionDocumentV10Builder<'a> {
    /// Document currency.
    pub currency: &'a str,
    /// Reference blockstamp.
    pub blockstamp: &'a Blockstamp,
    /// Locktime
    pub locktime: &'a u64,
    /// Transaction Document issuers.
    pub issuers: &'a [PubKey],
    /// Transaction inputs.
    pub inputs: &'a [TransactionInputV10],
    /// Inputs unlocks.
    pub unlocks: &'a [TransactionInputUnlocksV10],
    /// Transaction ouputs.
    pub outputs: &'a [TransactionOutputV10],
    /// Transaction comment
    pub comment: &'a str,
    /// Transaction hash
    pub hash: Option<Hash>,
}

impl<'a> TransactionDocumentV10Builder<'a> {
    fn build_with_text_and_sigs(
        self,
        text: String,
        signatures: Vec<Sig>,
    ) -> TransactionDocumentV10 {
        TransactionDocumentV10 {
            text: Some(text),
            currency: self.currency.to_string(),
            blockstamp: *self.blockstamp,
            locktime: *self.locktime,
            issuers: self.issuers.to_vec(),
            inputs: self.inputs.to_vec(),
            unlocks: self.unlocks.to_vec(),
            outputs: self.outputs.to_vec(),
            comment: String::from(self.comment),
            signatures,
            hash: self.hash,
        }
    }
}

impl<'a> DocumentBuilder for TransactionDocumentV10Builder<'a> {
    type Document = TransactionDocumentV10;
    type Signator = SignatorEnum;

    fn build_with_signature(&self, signatures: Vec<Sig>) -> TransactionDocumentV10 {
        self.build_with_text_and_sigs(self.generate_text(), signatures)
    }

    fn build_and_sign(&self, private_keys: Vec<SignatorEnum>) -> TransactionDocumentV10 {
        let (text, signatures) = self.build_signed_text(private_keys);
        self.build_with_text_and_sigs(text, signatures)
    }
}

impl<'a> TextDocumentBuilder for TransactionDocumentV10Builder<'a> {
    fn generate_text(&self) -> String {
        let mut issuers_string: String = "".to_owned();
        let mut inputs_string: String = "".to_owned();
        let mut unlocks_string: String = "".to_owned();
        let mut outputs_string: String = "".to_owned();
        for issuer in self.issuers {
            issuers_string.push_str(&format!("{}\n", issuer.to_string()))
        }
        for input in self.inputs {
            inputs_string.push_str(&format!("{}\n", input.to_string()))
        }
        for unlock in self.unlocks {
            unlocks_string.push_str(&format!("{}\n", unlock.to_string()))
        }
        for output in self.outputs {
            outputs_string.push_str(&format!("{}\n", output.to_string()))
        }
        format!(
            "Version: 10
Type: Transaction
Currency: {currency}
Blockstamp: {blockstamp}
Locktime: {locktime}
Issuers:
{issuers}Inputs:
{inputs}Unlocks:
{unlocks}Outputs:
{outputs}Comment: {comment}
",
            currency = self.currency,
            blockstamp = self.blockstamp,
            locktime = self.locktime,
            issuers = issuers_string,
            inputs = inputs_string,
            unlocks = unlocks_string,
            outputs = outputs_string,
            comment = self.comment,
        )
    }
}

/// Transaction document parser
#[derive(Debug, Clone, Copy)]
pub struct TransactionDocumentV10Parser;

impl TextDocumentParser<Rule> for TransactionDocumentV10Parser {
    type DocumentType = TransactionDocumentV10;

    fn parse(doc: &str) -> Result<Self::DocumentType, TextDocumentParseError> {
        let mut tx_pairs = DocumentsParser::parse(Rule::tx, doc)?;
        let tx_pair = unwrap!(tx_pairs.next()); // get and unwrap the `tx` rule; never fails
        Self::from_pest_pair(tx_pair)
    }
    #[inline]
    fn from_pest_pair(pair: Pair<Rule>) -> Result<Self::DocumentType, TextDocumentParseError> {
        let tx_vx_pair = unwrap!(pair.into_inner().next()); // get and unwrap the `tx_vX` rule; never fails

        match tx_vx_pair.as_rule() {
            Rule::tx_v10 => TransactionDocumentV10::from_pest_pair(tx_vx_pair),
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
            10 => Ok(TransactionDocumentV10::from_pest_pair(pair)?),
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
        let mut tx_doc = builder.build_with_signature(vec![sig]);
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

        let doc = TransactionDocumentV10Parser::parse(doc)
            .expect("fail to parse test transaction document !");
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

    #[test]
    fn transaction_input_str() {
        let expected_du = TransactionInputV10::D(
            TxAmount(10),
            TxBase(0),
            PubKey::Ed25519(unwrap!(
                ed25519::PublicKey::from_base58("DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV"),
                "Fail to parse PublicKey"
            )),
            BlockNumber(0),
        );
        let du = TransactionInputV10::from_str(&expected_du.to_string());
        assert!(du.is_ok());
        assert_eq!(expected_du, unwrap!(du));

        let expected_tx = TransactionInputV10::T(
            TxAmount(950),
            TxBase(0),
            unwrap!(
                Hash::from_hex("2CF1ACD8FE8DC93EE39A1D55881C50D87C55892AE8E4DB71D4EBAB3D412AA8FD"),
                "Fail to parse Hash"
            ),
            OutputIndex(1),
        );
        let tx = TransactionInputV10::from_str(&expected_tx.to_string());
        assert!(tx.is_ok());
        assert_eq!(expected_tx, unwrap!(tx));
    }
}
