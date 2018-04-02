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

use std::ops::Deref;

use duniter_crypto::keys::{PublicKey, ed25519};
use regex::Regex;
use regex::RegexBuilder;

use Blockstamp;
use blockchain::{BlockchainProtocol, Document, DocumentBuilder, IntoSpecializedDocument};
use blockchain::v10::documents::{StandardTextDocumentParser, TextDocument, TextDocumentBuilder,
                                 V10Document, V10DocumentParsingError};

lazy_static! {
    static ref TRANSACTION_REGEX_SIZE: &'static usize = &40_000_000;
    static ref TRANSACTION_REGEX_BUILDER: &'static str =
        r"^Blockstamp: (?P<blockstamp>[0-9]+-[0-9A-F]{64})\nLocktime: (?P<locktime>[0-9]+)\nIssuers:(?P<issuers>(?:\n[1-9A-Za-z][^OIl]{43,44})+)Inputs:\n(?P<inputs>([0-9A-Za-z:]+\n)+)Unlocks:\n(?P<unlocks>([0-9]+:(SIG\([0-9]+\) ?|XHX\(\w+\) ?)+\n)+)Outputs:\n(?P<outputs>([0-9A-Za-z()&|: ]+\n)+)Comment:(?P<comment>[\\\w:/;*\[\]()?!^+=@&~#{}|<>%. -]{0,255})\n$";
    static ref ISSUER_REGEX: Regex = Regex::new("(?P<issuer>[1-9A-Za-z][^OIl]{43,44})\n").unwrap();
    static ref D_INPUT_REGEX: Regex = Regex::new(
        "^(?P<amount>[1-9][0-9]*):(?P<base>[0-9]+):D:(?P<pubkey>[1-9A-Za-z][^OIl]{43,44}):(?P<block_number>[0-9]+)$"
    ).unwrap();
    static ref T_INPUT_REGEX: Regex = Regex::new(
        "^(?P<amount>[1-9][0-9]*):(?P<base>[0-9]+):T:(?P<tx_hash>[0-9A-F]{64}):(?P<tx_index>[0-9]+)$"
    ).unwrap();
    static ref UNLOCKS_REGEX: Regex = Regex::new(
        r"^(?P<index>[0-9]+):(?P<unlocks>(SIG\([0-9]+\) ?|XHX\(\w+\) ?)+)$"
    ).unwrap();
    static ref UNLOCK_SIG_REGEX: Regex =
        Regex::new(r"^SIG\((?P<index>[0-9]+)\)$").unwrap();
    static ref UNLOCK_XHX_REGEX: Regex = Regex::new(r"^XHX\((?P<code>\w+)\)$").unwrap();
    static ref OUTPUT_COND_SIG_REGEX: Regex = Regex::new(r"^SIG\((?P<pubkey>[1-9A-Za-z][^OIl]{43,44})\)$").unwrap();
    static ref OUTPUT_COND_XHX_REGEX: Regex = Regex::new(r"^XHX\((?P<hash>[0-9A-F]{64})\)$").unwrap();
    static ref OUTPUT_COND_CLTV_REGEX: Regex = Regex::new(r"^CLTV\((?P<timestamp>[0-9]+)\)$").unwrap();
    static ref OUTPUT_COND_CSV_REGEX: Regex = Regex::new(r"^CSV\((?P<timestamp>[0-9]+)\)$").unwrap();
    static ref OUPUT_CONDS_BRAKETS: Regex = Regex::new(r"^\((?P<conditions>[0-9A-Za-z()&| ]+)\)$").unwrap();
    static ref OUPUT_CONDS_AND: Regex = Regex::new(r"^(?P<conditions_group_1>[0-9A-Za-z()&| ]+) && (?P<conditions_group_2>[0-9A-Za-z()&| ]+)$").unwrap();
    static ref OUPUT_CONDS_OR: Regex = Regex::new(r"^(?P<conditions_group_1>[0-9A-Za-z()&| ]+) \|\| (?P<conditions_group_2>[0-9A-Za-z()&| ]+)$").unwrap();
    static ref OUTPUT_REGEX: Regex = Regex::new(
        "^(?P<amount>[1-9][0-9]+):(?P<base>[0-9]+):(?P<conditions>[0-9A-Za-z()&| ]+)$"
    ).unwrap();
}

/// Wrap a transaction input
#[derive(Debug, Clone)]
pub enum TransactionInput {
    /// Universal Dividend Input
    D(isize, usize, ed25519::PublicKey, u64),
    /// Previous Transaction Input
    T(isize, usize, String, usize),
}

impl ToString for TransactionInput {
    fn to_string(&self) -> String {
        match *self {
            TransactionInput::D(amount, base, pubkey, block_number) => {
                format!("{}:{}:D:{}:{}", amount, base, pubkey, block_number)
            }
            TransactionInput::T(amount, base, ref tx_hash, tx_index) => {
                format!("{}:{}:T:{}:{}", amount, base, tx_hash, tx_index)
            }
        }
    }
}

impl TransactionInput {
    fn parse_from_str(source: &str) -> Result<TransactionInput, V10DocumentParsingError> {
        if let Some(caps) = D_INPUT_REGEX.captures(source) {
            let amount = &caps["amount"];
            let base = &caps["base"];
            let pubkey = &caps["pubkey"];
            let block_number = &caps["block_number"];
            Ok(TransactionInput::D(
                amount.parse().expect("fail to parse input amount !"),
                base.parse().expect("fail to parse input base !"),
                ed25519::PublicKey::from_base58(pubkey).expect("fail to parse input pubkey !"),
                block_number
                    .parse()
                    .expect("fail to parse input block_number !"),
            ))
        //Ok(TransactionInput::D(10, 0, PublicKey::from_base58("FD9wujR7KABw88RyKEGBYRLz8PA6jzVCbcBAsrBXBqSa").unwrap(), 0))
        } else if let Some(caps) = T_INPUT_REGEX.captures(source) {
            let amount = &caps["amount"];
            let base = &caps["base"];
            let tx_hash = &caps["tx_hash"];
            let tx_index = &caps["tx_index"];
            Ok(TransactionInput::T(
                amount.parse().expect("fail to parse input amount"),
                base.parse().expect("fail to parse base amount"),
                String::from(tx_hash),
                tx_index.parse().expect("fail to parse tx_index amount"),
            ))
        } else {
            println!("Fail to parse this input = {:?}", source);
            Err(V10DocumentParsingError::InvalidInnerFormat(String::from(
                "Transaction2",
            )))
        }
    }
}

/// Wrap a transaction unlock proof
#[derive(Debug, Clone)]
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

impl TransactionUnlockProof {
    fn parse_from_str(source: &str) -> Result<TransactionUnlockProof, V10DocumentParsingError> {
        if let Some(caps) = UNLOCK_SIG_REGEX.captures(source) {
            let index = &caps["index"];
            Ok(TransactionUnlockProof::Sig(
                index.parse().expect("fail to parse SIG unlock"),
            ))
        } else if let Some(caps) = UNLOCK_XHX_REGEX.captures(source) {
            let code = &caps["code"];
            Ok(TransactionUnlockProof::Xhx(String::from(code)))
        } else {
            Err(V10DocumentParsingError::InvalidInnerFormat(String::from(
                "Transaction3",
            )))
        }
    }
}

/// Wrap a transaction unlocks input
#[derive(Debug, Clone)]
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
    fn parse_from_str(source: &str) -> Result<TransactionInputUnlocks, V10DocumentParsingError> {
        if let Some(caps) = UNLOCKS_REGEX.captures(source) {
            let index = &caps["index"].parse().expect("fail to parse unlock index");
            let unlocks = &caps["unlocks"];

            let unlocks_array: Vec<&str> = unlocks.split(' ').collect();
            let mut unlocks = Vec::new();
            for unlock in unlocks_array {
                unlocks.push(TransactionUnlockProof::parse_from_str(unlock)?);
            }
            Ok(TransactionInputUnlocks {
                index: *index,
                unlocks,
            })
        } else {
            println!("Fail to parse this unlock = {:?}", source);
            Err(V10DocumentParsingError::InvalidInnerFormat(String::from(
                "Transaction4",
            )))
        }
    }
}

/// Wrap a transaction ouput condition
#[derive(Debug, Clone)]
pub enum TransactionOuputCondition {
    /// The consumption of funds will require a valid signature of the specified key
    Sig(ed25519::PublicKey),
    /// The consumption of funds will require to provide a code with the hash indicated
    Xhx(String),
    /// Funds may not be consumed until the blockchain reaches the timestamp indicated.
    Cltv(u64),
    /// Funds may not be consumed before the duration indicated, starting from the timestamp of the block where the transaction is written.
    Csv(u64),
}

impl ToString for TransactionOuputCondition {
    fn to_string(&self) -> String {
        match *self {
            TransactionOuputCondition::Sig(ref pubkey) => format!("SIG({})", pubkey),
            TransactionOuputCondition::Xhx(ref hash) => format!("XHX({})", hash),
            TransactionOuputCondition::Cltv(timestamp) => format!("CLTV({})", timestamp),
            TransactionOuputCondition::Csv(duration) => format!("CSV({})", duration),
        }
    }
}

impl TransactionOuputCondition {
    fn parse_from_str(source: &str) -> Result<TransactionOuputCondition, V10DocumentParsingError> {
        if let Some(caps) = OUTPUT_COND_SIG_REGEX.captures(source) {
            Ok(TransactionOuputCondition::Sig(
                ed25519::PublicKey::from_base58(&caps["pubkey"])
                    .expect("fail to parse SIG TransactionOuputCondition"),
            ))
        } else if let Some(caps) = OUTPUT_COND_XHX_REGEX.captures(source) {
            Ok(TransactionOuputCondition::Xhx(String::from(&caps["hash"])))
        } else if let Some(caps) = OUTPUT_COND_CLTV_REGEX.captures(source) {
            Ok(TransactionOuputCondition::Cltv(
                caps["timestamp"]
                    .parse()
                    .expect("fail to parse CLTV TransactionOuputCondition"),
            ))
        } else if let Some(caps) = OUTPUT_COND_CSV_REGEX.captures(source) {
            Ok(TransactionOuputCondition::Csv(
                caps["duration"]
                    .parse()
                    .expect("fail to parse CSV TransactionOuputCondition"),
            ))
        } else {
            Err(V10DocumentParsingError::InvalidInnerFormat(
                "Transaction5".to_string(),
            ))
        }
    }
}

/// Wrap a transaction ouput condition group
#[derive(Debug, Clone)]
pub enum TransactionOuputConditionGroup {
    /// Single
    Single(TransactionOuputCondition),
    /// Brackets
    Brackets(Box<TransactionOuputConditionGroup>),
    /// And operator
    And(
        Box<TransactionOuputConditionGroup>,
        Box<TransactionOuputConditionGroup>,
    ),
    /// Or operator
    Or(
        Box<TransactionOuputConditionGroup>,
        Box<TransactionOuputConditionGroup>,
    ),
}

impl ToString for TransactionOuputConditionGroup {
    fn to_string(&self) -> String {
        match *self {
            TransactionOuputConditionGroup::Single(ref condition) => condition.to_string(),
            TransactionOuputConditionGroup::Brackets(ref condition_group) => {
                format!("({})", condition_group.deref().to_string())
            }
            TransactionOuputConditionGroup::And(ref condition_group_1, ref condition_group_2) => {
                format!(
                    "{} && {}",
                    condition_group_1.deref().to_string(),
                    condition_group_2.deref().to_string()
                )
            }
            TransactionOuputConditionGroup::Or(ref condition_group_1, ref condition_group_2) => {
                format!(
                    "{} || {}",
                    condition_group_1.deref().to_string(),
                    condition_group_2.deref().to_string()
                )
            }
        }
    }
}

impl TransactionOuputConditionGroup {
    fn parse_from_str(
        conditions: &str,
    ) -> Result<TransactionOuputConditionGroup, V10DocumentParsingError> {
        if let Ok(single_condition) = TransactionOuputCondition::parse_from_str(conditions) {
            Ok(TransactionOuputConditionGroup::Single(single_condition))
        } else if let Some(caps) = OUPUT_CONDS_BRAKETS.captures(conditions) {
            let inner_conditions =
                TransactionOuputConditionGroup::parse_from_str(&caps["conditions"])?;
            Ok(TransactionOuputConditionGroup::Brackets(Box::new(
                inner_conditions,
            )))
        } else if let Some(caps) = OUPUT_CONDS_AND.captures(conditions) {
            let conditions_group_1 =
                TransactionOuputConditionGroup::parse_from_str(&caps["conditions_group_1"])?;
            let conditions_group_2 =
                TransactionOuputConditionGroup::parse_from_str(&caps["conditions_group_2"])?;
            Ok(TransactionOuputConditionGroup::And(
                Box::new(conditions_group_1),
                Box::new(conditions_group_2),
            ))
        } else if let Some(caps) = OUPUT_CONDS_OR.captures(conditions) {
            let conditions_group_1 =
                TransactionOuputConditionGroup::parse_from_str(&caps["conditions_group_1"])?;
            let conditions_group_2 =
                TransactionOuputConditionGroup::parse_from_str(&caps["conditions_group_2"])?;
            Ok(TransactionOuputConditionGroup::Or(
                Box::new(conditions_group_1),
                Box::new(conditions_group_2),
            ))
        } else {
            println!("fail to parse this output condition = {:?}", conditions);
            Err(V10DocumentParsingError::InvalidInnerFormat(String::from(
                "Transaction6",
            )))
        }
    }
}

/// Wrap a transaction ouput
#[derive(Debug, Clone)]
pub struct TransactionOuput {
    /// Amount
    pub amount: isize,
    /// Base
    pub base: usize,
    /// List of conditions for consum this output
    pub conditions: TransactionOuputConditionGroup,
}

impl ToString for TransactionOuput {
    fn to_string(&self) -> String {
        format!(
            "{}:{}:{}",
            self.amount,
            self.base,
            self.conditions.to_string()
        )
    }
}

impl TransactionOuput {
    fn parse_from_str(source: &str) -> Result<TransactionOuput, V10DocumentParsingError> {
        if let Some(caps) = OUTPUT_REGEX.captures(source) {
            let amount = caps["amount"].parse().expect("fail to parse output amount");
            let base = caps["base"].parse().expect("fail to parse base amount");
            let conditions = TransactionOuputConditionGroup::parse_from_str(&caps["conditions"])?;
            Ok(TransactionOuput {
                conditions,
                amount,
                base,
            })
        } else {
            Err(V10DocumentParsingError::InvalidInnerFormat(
                "Transaction7".to_string(),
            ))
        }
    }
}

/// Wrap a Transaction document.
///
/// Must be created by parsing a text document or using a builder.
#[derive(Debug, Clone)]
pub struct TransactionDocument {
    /// Document as text.
    ///
    /// Is used to check signatures, and other values
    /// must be extracted from it.
    text: String,

    /// Currency.
    currency: String,
    /// Blockstamp
    blockstamp: Blockstamp,
    /// Locktime
    locktime: u64,
    /// Document issuer (there should be only one).
    issuers: Vec<ed25519::PublicKey>,
    /// Transaction inputs.
    inputs: Vec<TransactionInput>,
    /// Inputs unlocks.
    unlocks: Vec<TransactionInputUnlocks>,
    /// Transaction outputs.
    outputs: Vec<TransactionOuput>,
    /// Transaction comment
    comment: String,
    /// Document signature (there should be only one).
    signatures: Vec<ed25519::Signature>,
}

impl Document for TransactionDocument {
    type PublicKey = ed25519::PublicKey;
    type CurrencyType = str;

    fn version(&self) -> u16 {
        10
    }

    fn currency(&self) -> &str {
        &self.currency
    }

    fn issuers(&self) -> &Vec<ed25519::PublicKey> {
        &self.issuers
    }

    fn signatures(&self) -> &Vec<ed25519::Signature> {
        &self.signatures
    }

    fn as_bytes(&self) -> &[u8] {
        self.as_text().as_bytes()
    }
}

impl TextDocument for TransactionDocument {
    fn as_text(&self) -> &str {
        &self.text
    }

    fn generate_compact_text(&self) -> String {
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
        let mut signatures_str = String::from("");
        for sig in self.signatures.clone() {
            signatures_str.push_str("\n");
            signatures_str.push_str(&sig.to_string());
        }
        format!(
            "TX:10:{issuers_count}:{inputs_count}:{unlocks_count}:{outputs_count}:{has_comment}:{locktime}
{blockstamp}{issuers}{inputs}{unlocks}{outputs}
{comment}{signatures}",
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
            comment = self.comment,
            signatures = signatures_str,
        )
    }
}

impl IntoSpecializedDocument<BlockchainProtocol> for TransactionDocument {
    fn into_specialized(self) -> BlockchainProtocol {
        BlockchainProtocol::V10(Box::new(V10Document::Transaction(Box::new(self))))
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
    pub issuers: &'a Vec<ed25519::PublicKey>,
    /// Transaction inputs.
    pub inputs: &'a Vec<TransactionInput>,
    /// Inputs unlocks.
    pub unlocks: &'a Vec<TransactionInputUnlocks>,
    /// Transaction ouputs.
    pub outputs: &'a Vec<TransactionOuput>,
    /// Transaction comment
    pub comment: &'a str,
}

impl<'a> TransactionDocumentBuilder<'a> {
    fn build_with_text_and_sigs(
        self,
        text: String,
        signatures: Vec<ed25519::Signature>,
    ) -> TransactionDocument {
        TransactionDocument {
            text,
            currency: self.currency.to_string(),
            blockstamp: *self.blockstamp,
            locktime: *self.locktime,
            issuers: self.issuers.clone(),
            inputs: self.inputs.clone(),
            unlocks: self.unlocks.clone(),
            outputs: self.outputs.clone(),
            comment: String::from(self.comment),
            signatures,
        }
    }
}

impl<'a> DocumentBuilder for TransactionDocumentBuilder<'a> {
    type Document = TransactionDocument;
    type PrivateKey = ed25519::PrivateKey;

    fn build_with_signature(&self, signatures: Vec<ed25519::Signature>) -> TransactionDocument {
        self.build_with_text_and_sigs(self.generate_text(), signatures)
    }

    fn build_and_sign(&self, private_keys: Vec<ed25519::PrivateKey>) -> TransactionDocument {
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

impl StandardTextDocumentParser for TransactionDocumentParser {
    fn parse_standard(
        doc: &str,
        body: &str,
        currency: &str,
        signatures: Vec<ed25519::Signature>,
    ) -> Result<V10Document, V10DocumentParsingError> {
        let tx_regex: Regex = RegexBuilder::new(&TRANSACTION_REGEX_BUILDER)
            .size_limit(**TRANSACTION_REGEX_SIZE)
            .build()
            .expect("fail to build TRANSACTION_REGEX !");
        if let Some(caps) = tx_regex.captures(body) {
            let blockstamp =
                Blockstamp::from_string(&caps["blockstamp"]).expect("fail to parse blockstamp");
            let locktime = caps["locktime"].parse().expect("fail to parse locktime");
            let issuers_str = &caps["issuers"];
            let inputs = &caps["inputs"];
            let unlocks = &caps["unlocks"];
            let outputs = &caps["outputs"];
            let comment = String::from(&caps["comment"]);

            let mut issuers = Vec::new();
            for caps in ISSUER_REGEX.captures_iter(issuers_str) {
                println!("{:?}", &caps["issuer"]);
                issuers.push(
                    ed25519::PublicKey::from_base58(&caps["issuer"]).expect("fail to parse issuer"),
                );
            }
            let inputs_array: Vec<&str> = inputs.split('\n').collect();
            let mut inputs = Vec::new();
            for input in inputs_array {
                if !input.is_empty() {
                    inputs.push(TransactionInput::parse_from_str(input)?);
                }
            }
            let unlocks_array: Vec<&str> = unlocks.split('\n').collect();
            let mut unlocks = Vec::new();
            for unlock in unlocks_array {
                if !unlock.is_empty() {
                    unlocks.push(TransactionInputUnlocks::parse_from_str(unlock)?);
                }
            }
            let outputs_array: Vec<&str> = outputs.split('\n').collect();
            let mut outputs = Vec::new();
            for output in outputs_array {
                if !output.is_empty() {
                    outputs.push(TransactionOuput::parse_from_str(output)?);
                }
            }

            Ok(V10Document::Transaction(Box::new(TransactionDocument {
                text: doc.to_owned(),
                currency: currency.to_owned(),
                blockstamp,
                locktime,
                issuers,
                inputs,
                unlocks,
                outputs,
                comment,
                signatures,
            })))
        } else {
            Err(V10DocumentParsingError::InvalidInnerFormat(
                "Transaction1".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use duniter_crypto::keys::{PrivateKey, PublicKey, Signature};
    use blockchain::{Document, VerificationResult};

    #[test]
    fn generate_real_document() {
        let pubkey = ed25519::PublicKey::from_base58(
            "DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV",
        ).unwrap();

        let prikey = ed25519::PrivateKey::from_base58(
            "468Q1XtTq7h84NorZdWBZFJrGkB18CbmbHr9tkp9snt5G\
             iERP7ySs3wM8myLccbAAGejgMRC9rqnXuW3iAfZACm7",
        ).unwrap();

        let sig = ed25519::Signature::from_base64(
            "pRQeKlzCsvPNmYAAkEP5jPPQO1RwrtFMRfCajEfkkrG0UQE0DhoTkxG3Zs2JFmvAFLw67pn1V5NQ08zsSfJkBg==",
        ).unwrap();

        let block = Blockstamp::from_string(
            "0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
        ).unwrap();

        let builder = TransactionDocumentBuilder {
            currency: "duniter_unit_test_currency",
            blockstamp: &block,
            locktime: &0,
            issuers: &vec![pubkey],
            inputs: &vec![
                TransactionInput::parse_from_str(
                    "10:0:D:DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV:0",
                ).expect("fail to parse input !"),
            ],
            unlocks: &vec![
                TransactionInputUnlocks::parse_from_str("0:SIG(0)")
                    .expect("fail to parse unlock !"),
            ],
            outputs: &vec![
                TransactionOuput::parse_from_str(
                    "10:0:SIG(FD9wujR7KABw88RyKEGBYRLz8PA6jzVCbcBAsrBXBqSa)",
                ).expect("fail to parse output !"),
            ],
            comment: "test",
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
    fn transaction_standard_regex() {
        let tx_regex: Regex = RegexBuilder::new(&TRANSACTION_REGEX_BUILDER)
            .size_limit(**TRANSACTION_REGEX_SIZE)
            .build()
            .expect("fail to build TRANSACTION_REGEX !");
        assert!(tx_regex.is_match(
            "Blockstamp: 204-00003E2B8A35370BA5A7064598F628A62D4E9EC1936BE8651CE9A85F2E06981B
Locktime: 0
Issuers:
HsLShAtzXTVxeUtQd7yi5Z5Zh4zNvbu8sTEZ53nfKcqY
CYYjHsNyg3HMRMpTHqCJAN9McjH5BwFLmDKGV3PmCuKp
FD9wujR7KABw88RyKEGBYRLz8PA6jzVCbcBAsrBXBqSa
Inputs:
40:2:T:6991C993631BED4733972ED7538E41CCC33660F554E3C51963E2A0AC4D6453D3:2
70:2:T:3A09A20E9014110FD224889F13357BAB4EC78A72F95CA03394D8CCA2936A7435:8
20:2:D:HsLShAtzXTVxeUtQd7yi5Z5Zh4zNvbu8sTEZ53nfKcqY:46
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
49:2:(SIG(6DyGr5LFtFmbaJYRvcs9WmBsr4cbJbJ1EV9zBbqG7A6i) || XHX(3EB4702F2AC2FD3FA4FDC46A4FC05AE8CDEE1A85))
Comment: -----@@@----- (why not this comment?)
"
        ));
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
";

        let body =
            "Blockstamp: 204-00003E2B8A35370BA5A7064598F628A62D4E9EC1936BE8651CE9A85F2E06981B
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
";

        let currency = "duniter_unit_test_currency";

        let signatures = vec![
            Signature::from_base64(
"kL59C1izKjcRN429AlKdshwhWbasvyL7sthI757zm1DfZTdTIctDWlKbYeG/tS7QyAgI3gcfrTHPhu1E1lKCBw=="
        ).expect("fail to parse test signature"),
            Signature::from_base64(
"e3LpgB2RZ/E/BCxPJsn+TDDyxGYzrIsMyDt//KhJCjIQD6pNUxr5M5jrq2OwQZgwmz91YcmoQ2XRQAUDpe4BAw=="
            ).expect("fail to parse test signature"),
            Signature::from_base64(
"w69bYgiQxDmCReB0Dugt9BstXlAKnwJkKCdWvCeZ9KnUCv0FJys6klzYk/O/b9t74tYhWZSX0bhETWHiwfpWBw=="
            ).expect("fail to parse test signature"),
        ];

        let doc = TransactionDocumentParser::parse_standard(doc, body, currency, signatures)
            .expect("fail to parse test transaction document !");
        if let V10Document::Transaction(doc) = doc {
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
        } else {
            panic!("Wrong document type");
        }
    }
}
