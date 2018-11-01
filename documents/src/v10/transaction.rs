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

//! Wrappers around Transaction documents.

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;
use std::ops::{Add, Deref, Sub};
use std::str::FromStr;

use blockstamp::Blockstamp;
use v10::*;
use *;

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

/// Wrap a transaction index
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct TxIndex(pub usize);

/// Wrap a transaction input
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum TransactionInput {
    /// Universal Dividend Input
    D(TxAmount, TxBase, PubKey, BlockId),
    /// Previous Transaction Input
    T(TxAmount, TxBase, Hash, TxIndex),
}

impl ToString for TransactionInput {
    fn to_string(&self) -> String {
        match *self {
            TransactionInput::D(amount, base, pubkey, block_number) => {
                format!("{}:{}:D:{}:{}", amount.0, base.0, pubkey, block_number.0)
            }
            TransactionInput::T(amount, base, ref tx_hash, tx_index) => {
                format!("{}:{}:T:{}:{}", amount.0, base.0, tx_hash, tx_index.0)
            }
        }
    }
}

impl TransactionInput {
    fn from_pest_pair(mut pairs: Pairs<Rule>) -> TransactionInput {
        let tx_input_type_pair = pairs.next().unwrap();
        match tx_input_type_pair.as_rule() {
            Rule::tx_input_du => {
                let mut inner_rules = tx_input_type_pair.into_inner(); // ${ tx_amount ~ ":" ~ tx_amount_base ~ ":D:" ~ pubkey ~ ":" ~ du_block_id }

                TransactionInput::D(
                    TxAmount(inner_rules.next().unwrap().as_str().parse().unwrap()),
                    TxBase(inner_rules.next().unwrap().as_str().parse().unwrap()),
                    PubKey::Ed25519(
                        ed25519::PublicKey::from_base58(inner_rules.next().unwrap().as_str())
                            .unwrap(),
                    ),
                    BlockId(inner_rules.next().unwrap().as_str().parse().unwrap()),
                )
            }
            Rule::tx_input_tx => {
                let mut inner_rules = tx_input_type_pair.into_inner(); // ${ tx_amount ~ ":" ~ tx_amount_base ~ ":D:" ~ pubkey ~ ":" ~ du_block_id }

                TransactionInput::T(
                    TxAmount(inner_rules.next().unwrap().as_str().parse().unwrap()),
                    TxBase(inner_rules.next().unwrap().as_str().parse().unwrap()),
                    Hash::from_hex(inner_rules.next().unwrap().as_str()).unwrap(),
                    TxIndex(inner_rules.next().unwrap().as_str().parse().unwrap()),
                )
            }
            _ => panic!("unexpected rule: {:?}", tx_input_type_pair.as_rule()), // Grammar ensures that we never reach this line
        }
    }
}

impl FromStr for TransactionInput {
    type Err = TextDocumentParseError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        match DocumentsParser::parse(Rule::tx_input, source) {
            Ok(mut pairs) => Ok(TransactionInput::from_pest_pair(
                pairs.next().unwrap().into_inner(),
            )),
            Err(_) => Err(TextDocumentParseError::InvalidInnerFormat(
                "Invalid unlocks !",
            )),
        }
    }
}

/*impl TransactionInput {
    /// Parse Transaction Input from string.
    pub fn from_str(source: &str) -> Result<TransactionInput, TextDocumentParseError> {
        if let Some(caps) = D_INPUT_REGEX.captures(source) {
            let amount = &caps["amount"];
            let base = &caps["base"];
            let pubkey = &caps["pubkey"];
            let block_number = &caps["block_number"];
            Ok(TransactionInput::D(
                TxAmount(amount.parse().expect("fail to parse input amount !")),
                TxBase(base.parse().expect("fail to parse input base !")),
                PubKey::Ed25519(
                    ed25519::PublicKey::from_base58(pubkey).expect("fail to parse input pubkey !"),
                ),
                BlockId(
                    block_number
                        .parse()
                        .expect("fail to parse input block_number !"),
                ),
            ))
        //Ok(TransactionInput::D(10, 0, PubKey::Ed25519(ed25519::PublicKey::from_base58("FD9wujR7KABw88RyKEGBYRLz8PA6jzVCbcBAsrBXBqSa").unwrap(), 0)))
        } else if let Some(caps) = T_INPUT_REGEX.captures(source) {
            let amount = &caps["amount"];
            let base = &caps["base"];
            let tx_hash = &caps["tx_hash"];
            let tx_index = &caps["tx_index"];
            Ok(TransactionInput::T(
                TxAmount(amount.parse().expect("fail to parse input amount")),
                TxBase(base.parse().expect("fail to parse base amount")),
                Hash::from_hex(tx_hash).expect("fail to parse tx_hash"),
                TxIndex(tx_index.parse().expect("fail to parse tx_index amount")),
            ))
        } else {
            println!("Fail to parse this input = {:?}", source);
            Err(TextDocumentParseError::InvalidInnerFormat("Transaction2"))
        }
    }
}*/

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

/// Wrap a transaction unlocks input
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TransactionInputUnlocks {
    /// Input index
    pub index: usize,
    /// List of proof to unlock funds
    pub unlocks: Vec<TransactionUnlockProof>,
}

impl ToString for TransactionInputUnlocks {
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

impl TransactionInputUnlocks {
    fn from_pest_pair(pairs: Pairs<Rule>) -> TransactionInputUnlocks {
        let mut input_index = 0;
        let mut unlock_conds = Vec::new();
        for unlock_field in pairs {
            // ${ input_index ~ ":" ~ unlock_cond ~ (" " ~ unlock_cond)* }
            match unlock_field.as_rule() {
                Rule::input_index => input_index = unlock_field.as_str().parse().unwrap(),
                Rule::unlock_sig => unlock_conds.push(TransactionUnlockProof::Sig(
                    unlock_field
                        .into_inner()
                        .next()
                        .unwrap()
                        .as_str()
                        .parse()
                        .unwrap(),
                )),
                Rule::unlock_xhx => unlock_conds.push(TransactionUnlockProof::Xhx(String::from(
                    unlock_field.into_inner().next().unwrap().as_str(),
                ))),
                _ => panic!("unexpected rule: {:?}", unlock_field.as_rule()), // Grammar ensures that we never reach this line
            }
        }
        TransactionInputUnlocks {
            index: input_index,
            unlocks: unlock_conds,
        }
    }
}

impl FromStr for TransactionInputUnlocks {
    type Err = TextDocumentParseError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        match DocumentsParser::parse(Rule::tx_unlock, source) {
            Ok(mut pairs) => Ok(TransactionInputUnlocks::from_pest_pair(
                pairs.next().unwrap().into_inner(),
            )),
            Err(_) => Err(TextDocumentParseError::InvalidInnerFormat(
                "Invalid unlocks !",
            )),
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
                let last_subgroup = conds_subgroups.pop().unwrap();
                let previous_last_subgroup = conds_subgroups.pop().unwrap();
                conds_subgroups.push($op(
                    Box::new(previous_last_subgroup),
                    Box::new(last_subgroup),
                ));
                UTXOConditionsGroup::$fn_name(conds_subgroups)
            } else {
                panic!(
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
    pub fn wrap_utxo_conds(pair: Pair<Rule>) -> UTXOConditionsGroup {
        match pair.as_rule() {
            Rule::output_and_group => {
                let mut and_pairs = pair.into_inner();
                let mut conds_subgroups: Vec<UTXOConditionsGroup> = and_pairs
                    .map(UTXOConditionsGroup::wrap_utxo_conds)
                    .collect();
                UTXOConditionsGroup::Brackets(Box::new(UTXOConditionsGroup::new_and_chain(
                    &mut conds_subgroups,
                )))
            }
            Rule::output_or_group => {
                let mut or_pairs = pair.into_inner();
                let mut conds_subgroups: Vec<UTXOConditionsGroup> =
                    or_pairs.map(UTXOConditionsGroup::wrap_utxo_conds).collect();
                UTXOConditionsGroup::Brackets(Box::new(UTXOConditionsGroup::new_or_chain(
                    &mut conds_subgroups,
                )))
            }
            Rule::output_cond_sig => {
                UTXOConditionsGroup::Single(TransactionOutputCondition::Sig(PubKey::Ed25519(
                    ed25519::PublicKey::from_base58(pair.into_inner().next().unwrap().as_str())
                        .unwrap(),
                )))
            }
            Rule::output_cond_xhx => UTXOConditionsGroup::Single(TransactionOutputCondition::Xhx(
                Hash::from_hex(pair.into_inner().next().unwrap().as_str()).unwrap(),
            )),
            Rule::output_cond_csv => UTXOConditionsGroup::Single(TransactionOutputCondition::Csv(
                pair.into_inner().next().unwrap().as_str().parse().unwrap(),
            )),
            Rule::output_cond_cltv => {
                UTXOConditionsGroup::Single(TransactionOutputCondition::Cltv(
                    pair.into_inner().next().unwrap().as_str().parse().unwrap(),
                ))
            }
            _ => panic!("unexpected rule: {:?}", pair.as_rule()), // Grammar ensures that we never reach this line
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

/// Wrap a transaction ouput
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TransactionOutput {
    /// Amount
    pub amount: TxAmount,
    /// Base
    pub base: TxBase,
    /// List of conditions for consum this output
    pub conditions: UTXOConditions,
}

impl TransactionOutput {
    /// Lightens the TransactionOutput (for example to store it while minimizing the space required)
    fn reduce(&mut self) {
        self.conditions.reduce()
    }
    /// Check validity of this output
    pub fn check(&self) -> bool {
        self.conditions.check()
    }
}

impl ToString for TransactionOutput {
    fn to_string(&self) -> String {
        format!(
            "{}:{}:{}",
            self.amount.0,
            self.base.0,
            self.conditions.to_string()
        )
    }
}

impl TransactionOutput {
    fn from_pest_pair(mut utxo_pairs: Pairs<Rule>) -> TransactionOutput {
        let amount = TxAmount(utxo_pairs.next().unwrap().as_str().parse().unwrap());
        let base = TxBase(utxo_pairs.next().unwrap().as_str().parse().unwrap());
        let conditions_pairs = utxo_pairs.next().unwrap();
        let conditions_origin_str = conditions_pairs.as_str();
        TransactionOutput {
            amount,
            base,
            conditions: UTXOConditions {
                origin_str: Some(String::from(conditions_origin_str)),
                conditions: UTXOConditionsGroup::wrap_utxo_conds(conditions_pairs),
            },
        }
    }
}

impl FromStr for TransactionOutput {
    type Err = TextDocumentParseError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        match DocumentsParser::parse(Rule::tx_output, source) {
            Ok(mut utxo_pairs) => Ok(TransactionOutput::from_pest_pair(
                utxo_pairs.next().unwrap().into_inner(),
            )),
            Err(_) => Err(TextDocumentParseError::InvalidInnerFormat(
                "Invalid output !",
            )),
        }
    }
}

/// Wrap a Transaction document.
///
/// Must be created by parsing a text document or using a builder.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TransactionDocument {
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
    /// Document issuer (there should be only one).
    issuers: Vec<PubKey>,
    /// Transaction inputs.
    inputs: Vec<TransactionInput>,
    /// Inputs unlocks.
    unlocks: Vec<TransactionInputUnlocks>,
    /// Transaction outputs.
    outputs: Vec<TransactionOutput>,
    /// Transaction comment
    comment: String,
    /// Document signature (there should be only one).
    signatures: Vec<Sig>,
    /// Transaction hash
    hash: Option<Hash>,
}

impl TransactionDocument {
    /// Compute transaction hash
    pub fn compute_hash(&mut self) -> Hash {
        let mut sha256 = Sha256::new();
        let mut hashing_text = if let Some(ref text) = self.text {
            text.clone()
        } else {
            panic!("Try to compute_hash of tx with None text !")
        };
        hashing_text.push_str(&self.signatures[0].to_string());
        hashing_text.push_str("\n");
        //println!("tx_text_hasing={}", hashing_text);
        sha256.input_str(&hashing_text);
        self.hash = Some(Hash::from_hex(&sha256.result_str()).unwrap());
        self.hash.expect("Try to get hash of a reduce tx !")
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
            self.compute_hash()
        }
    }
    /// Get transaction inputs
    pub fn get_inputs(&self) -> &[TransactionInput] {
        &self.inputs
    }
    /// Get transaction outputs
    pub fn get_outputs(&self) -> &[TransactionOutput] {
        &self.outputs
    }
    /// Lightens the transaction (for example to store it while minimizing the space required)
    pub fn reduce(&mut self) {
        self.text = None;
        self.hash = None;
        for output in &mut self.outputs {
            output.reduce()
        }
    }
}

impl Document for TransactionDocument {
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
        self.as_text_without_signature().as_bytes()
    }
}

impl CompactTextDocument for TransactionDocument {
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

impl TextDocument for TransactionDocument {
    type CompactTextDocument_ = TransactionDocument;

    fn as_text(&self) -> &str {
        if let Some(ref text) = self.text {
            text
        } else {
            panic!("Try to get text of tx whti None text !")
        }
    }

    fn to_compact_document(&self) -> Self::CompactTextDocument_ {
        self.clone()
    }
}

impl IntoSpecializedDocument<DUBPDocument> for TransactionDocument {
    fn into_specialized(self) -> DUBPDocument {
        DUBPDocument::V10(Box::new(V10Document::Transaction(Box::new(self))))
    }
}

/// Transaction document builder.
#[derive(Debug, Copy, Clone)]
pub struct TransactionDocumentBuilder<'a> {
    /// Document currency.
    pub currency: &'a str,
    /// Reference blockstamp.
    pub blockstamp: &'a Blockstamp,
    /// Locktime
    pub locktime: &'a u64,
    /// Transaction Document issuers.
    pub issuers: &'a Vec<PubKey>,
    /// Transaction inputs.
    pub inputs: &'a Vec<TransactionInput>,
    /// Inputs unlocks.
    pub unlocks: &'a Vec<TransactionInputUnlocks>,
    /// Transaction ouputs.
    pub outputs: &'a Vec<TransactionOutput>,
    /// Transaction comment
    pub comment: &'a str,
    /// Transaction hash
    pub hash: Option<Hash>,
}

impl<'a> TransactionDocumentBuilder<'a> {
    fn build_with_text_and_sigs(self, text: String, signatures: Vec<Sig>) -> TransactionDocument {
        TransactionDocument {
            text: Some(text),
            currency: self.currency.to_string(),
            blockstamp: *self.blockstamp,
            locktime: *self.locktime,
            issuers: self.issuers.clone(),
            inputs: self.inputs.clone(),
            unlocks: self.unlocks.clone(),
            outputs: self.outputs.clone(),
            comment: String::from(self.comment),
            signatures,
            hash: self.hash,
        }
    }
}

impl<'a> DocumentBuilder for TransactionDocumentBuilder<'a> {
    type Document = TransactionDocument;
    type PrivateKey = PrivKey;

    fn build_with_signature(&self, signatures: Vec<Sig>) -> TransactionDocument {
        self.build_with_text_and_sigs(self.generate_text(), signatures)
    }

    fn build_and_sign(&self, private_keys: Vec<PrivKey>) -> TransactionDocument {
        let (text, signatures) = self.build_signed_text(private_keys);
        self.build_with_text_and_sigs(text, signatures)
    }
}

impl<'a> TextDocumentBuilder for TransactionDocumentBuilder<'a> {
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
pub struct TransactionDocumentParser;

impl TextDocumentParser for TransactionDocumentParser {
    type DocumentType = TransactionDocument;

    fn parse(doc: &str) -> Result<Self::DocumentType, TextDocumentParseError> {
        match DocumentsParser::parse(Rule::tx, doc) {
            Ok(mut tx_pairs) => {
                let tx_pair = tx_pairs.next().unwrap(); // get and unwrap the `tx` rule; never fails
                let tx_vx_pair = tx_pair.into_inner().next().unwrap(); // get and unwrap the `tx_vX` rule; never fails

                match tx_vx_pair.as_rule() {
                    Rule::tx_v10 => Ok(TransactionDocumentParser::from_pest_pair(tx_vx_pair)),
                    _ => Err(TextDocumentParseError::UnexpectedVersion(format!(
                        "{:#?}",
                        tx_vx_pair.as_rule()
                    ))),
                }
            }
            Err(pest_error) => Err(TextDocumentParseError::PestError(format!("{}", pest_error))),
        }
    }
    fn from_pest_pair(pair: Pair<Rule>) -> Self::DocumentType {
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

                    let block_id: &str = inner_rules.next().unwrap().as_str();
                    let block_hash: &str = inner_rules.next().unwrap().as_str();
                    blockstamp = Blockstamp {
                        id: BlockId(block_id.parse().unwrap()), // Grammar ensures that we have a digits string.
                        hash: BlockHash(Hash::from_hex(block_hash).unwrap()), // Grammar ensures that we have an hexadecimal string.
                    };
                }
                Rule::tx_locktime => locktime = field.as_str().parse().unwrap(), // Grammar ensures that we have digits characters.
                Rule::pubkey => issuers.push(PubKey::Ed25519(
                    ed25519::PublicKey::from_base58(field.as_str()).unwrap(), // Grammar ensures that we have a base58 string.
                )),
                Rule::tx_input => inputs.push(TransactionInput::from_pest_pair(field.into_inner())),
                Rule::tx_unlock => {
                    unlocks.push(TransactionInputUnlocks::from_pest_pair(field.into_inner()))
                }
                Rule::tx_output => {
                    outputs.push(TransactionOutput::from_pest_pair(field.into_inner()))
                }
                Rule::tx_comment => comment = field.as_str(),
                Rule::ed25519_sig => {
                    sigs.push(Sig::Ed25519(
                        ed25519::Signature::from_base64(field.as_str()).unwrap(), // Grammar ensures that we have a base64 string.
                    ));
                }
                Rule::EOI => (),
                _ => panic!("unexpected rule: {:?}", field.as_rule()), // Grammar ensures that we never reach this line
            }
        }
        TransactionDocument {
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use {Document, VerificationResult};

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
            "pRQeKlzCsvPNmYAAkEP5jPPQO1RwrtFMRfCajEfkkrG0UQE0DhoTkxG3Zs2JFmvAFLw67pn1V5NQ08zsSfJkBg==",
        ).unwrap());

        let block = Blockstamp::from_string(
            "0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
        )
        .unwrap();

        let builder = TransactionDocumentBuilder {
            currency: "duniter_unit_test_currency",
            blockstamp: &block,
            locktime: &0,
            issuers: &vec![pubkey],
            inputs: &vec![TransactionInput::D(
                TxAmount(10),
                TxBase(0),
                PubKey::Ed25519(
                    ed25519::PublicKey::from_base58("DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV")
                        .unwrap(),
                ),
                BlockId(0),
            )],
            unlocks: &vec![TransactionInputUnlocks {
                index: 0,
                unlocks: vec![TransactionUnlockProof::Sig(0)],
            }],
            outputs: &vec![TransactionOutput::from_str(
                "10:0:SIG(FD9wujR7KABw88RyKEGBYRLz8PA6jzVCbcBAsrBXBqSa)",
            )
            .expect("fail to parse output !")],
            comment: "test",
            hash: None,
        };
        println!(
            "Signature = {:?}",
            builder.build_and_sign(vec![prikey]).signatures()
        );
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
    fn compute_transaction_hash() {
        let pubkey = PubKey::Ed25519(
            ed25519::PublicKey::from_base58("FEkbc4BfJukSWnCU6Hed6dgwwTuPFTVdgz5LpL4iHr9J")
                .unwrap(),
        );

        let sig = Sig::Ed25519(ed25519::Signature::from_base64(
            "XEwKwKF8AI1gWPT7elR4IN+bW3Qn02Dk15TEgrKtY/S2qfZsNaodsLofqHLI24BBwZ5aadpC88ntmjo/UW9oDQ==",
        ).unwrap());

        let block = Blockstamp::from_string(
            "60-00001FE00410FCD5991EDD18AA7DDF15F4C8393A64FA92A1DB1C1CA2E220128D",
        )
        .unwrap();

        let builder = TransactionDocumentBuilder {
            currency: "g1",
            blockstamp: &block,
            locktime: &0,
            issuers: &vec![pubkey],
            inputs: &vec![TransactionInput::T(
                TxAmount(950),
                TxBase(0),
                Hash::from_hex("2CF1ACD8FE8DC93EE39A1D55881C50D87C55892AE8E4DB71D4EBAB3D412AA8FD")
                    .unwrap(),
                TxIndex(1),
            )],
            unlocks: &vec![
                TransactionInputUnlocks::from_str("0:SIG(0)").expect("fail to parse unlock !")
            ],
            outputs: &vec![
                TransactionOutput::from_str(
                    "30:0:SIG(38MEAZN68Pz1DTvT3tqgxx4yQP6snJCQhPqEFxbDk4aE)",
                )
                .expect("fail to parse output !"),
                TransactionOutput::from_str(
                    "920:0:SIG(FEkbc4BfJukSWnCU6Hed6dgwwTuPFTVdgz5LpL4iHr9J)",
                )
                .expect("fail to parse output !"),
            ],
            comment: "Pour cesium merci",
            hash: None,
        };
        let mut tx_doc = builder.build_with_signature(vec![sig]);
        tx_doc.hash = None;
        assert_eq!(tx_doc.verify_signatures(), VerificationResult::Valid());
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
        assert_eq!(doc.verify_signatures(), VerificationResult::Valid());
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
