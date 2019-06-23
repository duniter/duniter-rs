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

//! Wrappers around Block document V10.

use dup_crypto::hashs::Hash;
use dup_crypto::keys::*;
use dup_currency_params::genesis_block_params::v10::BlockV10Parameters;
use dup_currency_params::CurrencyName;
use durs_common_tools::fatal_error;
use std::ops::Deref;
use unwrap::unwrap;

use super::BlockDocumentTrait;
use crate::blockstamp::Blockstamp;
use crate::documents::certification::CertificationDocument;
use crate::documents::identity::IdentityDocumentV10;
use crate::documents::membership::v10::{MembershipDocumentV10, MembershipDocumentV10Stringified};
use crate::documents::revocation::RevocationDocument;
use crate::documents::transaction::TransactionDocument;
use crate::documents::*;
use crate::text_document_traits::*;

/// Store a transaction document or just its hash.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum TxDocOrTxHash {
    /// Transaction document
    TxDoc(Box<TransactionDocument>),
    /// transaction hash
    TxHash(Hash),
}

impl TxDocOrTxHash {
    /// Lightens the TxDocOrTxHash (for example to store it while minimizing the space required)
    /// lightening consists in transforming the document by its hash.
    pub fn reduce(&self) -> TxDocOrTxHash {
        if let TxDocOrTxHash::TxDoc(ref tx_doc) = self {
            let tx_doc = tx_doc.deref();
            if let Some(ref hash) = tx_doc.get_hash_opt() {
                TxDocOrTxHash::TxHash(*hash)
            } else {
                TxDocOrTxHash::TxHash(tx_doc.clone().compute_hash())
            }
        } else {
            self.clone()
        }
    }
    /// Get TxDoc variant
    pub fn unwrap_doc(&self) -> TransactionDocument {
        if let TxDocOrTxHash::TxDoc(ref tx_doc) = self {
            tx_doc.deref().clone()
        } else {
            fatal_error!("Try to unwrap_doc() in a TxHash() variant of TxDocOrTxHash !")
        }
    }
}

/// Wrap a Block document.
///
/// Must be created by parsing a text document or using a builder.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct BlockDocumentV10 {
    /// Version
    pub version: u32,
    /// Nonce
    pub nonce: u64,
    /// number
    pub number: BlockNumber,
    /// Minimal proof of work difficulty
    pub pow_min: usize,
    /// Local time of the block issuer
    pub time: u64,
    /// Average time
    pub median_time: u64,
    /// Members count
    pub members_count: usize,
    /// Monetary mass
    pub monetary_mass: usize,
    /// Unit base (power of ten)
    pub unit_base: usize,
    /// Number of compute members in the current frame
    pub issuers_count: usize,
    /// Current frame size (in blocks)
    pub issuers_frame: usize,
    /// Current frame variation buffer
    pub issuers_frame_var: isize,
    /// Currency.
    pub currency: CurrencyName,
    /// Document issuer (there should be only one).
    pub issuers: Vec<PubKey>,
    /// Document signature (there should be only one).
    /// This vector is empty, when the block is generated but the proof of work has not yet started
    pub signatures: Vec<Sig>,
    /// The hash is None, when the block is generated but the proof of work has not yet started
    pub hash: Option<BlockHash>,
    /// Currency parameters (only in genesis block)
    pub parameters: Option<BlockV10Parameters>,
    /// Hash of the previous block
    pub previous_hash: Option<Hash>,
    /// Issuer of the previous block
    pub previous_issuer: Option<PubKey>,
    /// Hash of the deterministic content of the block
    pub inner_hash: Option<Hash>,
    /// Amount of new dividend created at this block, None if no dividend is created at this block
    pub dividend: Option<usize>,
    /// Identities
    pub identities: Vec<IdentityDocumentV10>,
    /// joiners
    pub joiners: Vec<MembershipDocumentV10>,
    /// Actives (=renewals)
    pub actives: Vec<MembershipDocumentV10>,
    /// Leavers
    pub leavers: Vec<MembershipDocumentV10>,
    /// Revokeds
    pub revoked: Vec<TextDocumentFormat<RevocationDocument>>,
    /// Excludeds
    pub excluded: Vec<PubKey>,
    /// Certifications
    pub certifications: Vec<TextDocumentFormat<CertificationDocument>>,
    /// Transactions
    pub transactions: Vec<TxDocOrTxHash>,
}

impl BlockDocumentTrait for BlockDocumentV10 {
    fn common_time(&self) -> u64 {
        self.median_time
    }
    fn compute_hash(&self) -> BlockHash {
        BlockHash(Hash::compute_str(&self.compute_will_hashed_string()))
    }
    fn compute_will_hashed_string(&self) -> String {
        format!(
            "{}{}\n",
            self.compute_will_signed_string(),
            self.signatures[0]
        )
    }
    fn compute_will_signed_string(&self) -> String {
        format!(
            "InnerHash: {}\nNonce: {}\n",
            self.inner_hash
                .expect("compute_will_signed_string(): Try to get inner_hash of an uncompleted or reduce block !")
                .to_hex(),
            self.nonce
        )
    }
    fn current_frame_size(&self) -> usize {
        self.issuers_frame
    }
    fn generate_compact_inner_text(&self) -> String {
        let mut identities_str = String::from("");
        for identity in self.identities.clone() {
            identities_str.push_str("\n");
            identities_str.push_str(&identity.generate_compact_text());
        }
        let mut joiners_str = String::from("");
        for joiner in self.joiners.clone() {
            joiners_str.push_str("\n");
            joiners_str.push_str(&joiner.generate_compact_text());
        }
        let mut actives_str = String::from("");
        for active in self.actives.clone() {
            actives_str.push_str("\n");
            actives_str.push_str(&active.generate_compact_text());
        }
        let mut leavers_str = String::from("");
        for leaver in self.leavers.clone() {
            leavers_str.push_str("\n");
            leavers_str.push_str(&leaver.generate_compact_text());
        }
        let mut identities_str = String::from("");
        for identity in self.identities.clone() {
            identities_str.push_str("\n");
            identities_str.push_str(&identity.generate_compact_text());
        }
        let mut revokeds_str = String::from("");
        for revocation in self.revoked.clone() {
            revokeds_str.push_str("\n");
            revokeds_str.push_str(&revocation.as_compact_text());
        }
        let mut excludeds_str = String::from("");
        for exclusion in self.excluded.clone() {
            excludeds_str.push_str("\n");
            excludeds_str.push_str(&exclusion.to_string());
        }
        let mut certifications_str = String::from("");
        for certification in self.certifications.clone() {
            certifications_str.push_str("\n");
            certifications_str.push_str(&certification.as_compact_text());
        }
        let mut transactions_str = String::from("");
        for transaction in self.transactions.clone() {
            if let TxDocOrTxHash::TxDoc(transaction) = transaction {
                transactions_str.push_str("\n");
                transactions_str.push_str(&transaction.deref().generate_compact_text());
            }
        }
        let mut dividend_str = String::from("");
        if let Some(dividend) = self.dividend {
            if dividend > 0 {
                dividend_str.push_str("UniversalDividend: ");
                dividend_str.push_str(&dividend.to_string());
                dividend_str.push_str("\n");
            }
        }
        let mut parameters_str = String::from("");
        if let Some(params) = self.parameters {
            parameters_str.push_str("Parameters: ");
            parameters_str.push_str(&params.to_string());
            parameters_str.push_str("\n");
        }
        let mut previous_hash_str = String::from("");
        if self.number.0 > 0 {
            previous_hash_str.push_str("PreviousHash: ");
            previous_hash_str.push_str(&unwrap!(self.previous_hash).to_string());
            previous_hash_str.push_str("\n");
        }
        let mut previous_issuer_str = String::from("");
        if self.number.0 > 0 {
            previous_issuer_str.push_str("PreviousIssuer: ");
            previous_issuer_str.push_str(
                &self
                    .previous_issuer
                    .expect("No genesis block must have previous issuer")
                    .to_string(),
            );
            previous_issuer_str.push_str("\n");
        }
        format!(
            "Version: {version}
Type: Block
Currency: {currency}
Number: {block_number}
PoWMin: {pow_min}
Time: {time}
MedianTime: {median_time}
{dividend}UnitBase: {unit_base}
Issuer: {issuer}
IssuersFrame: {issuers_frame}
IssuersFrameVar: {issuers_frame_var}
DifferentIssuersCount: {issuers_count}
{parameters}{previous_hash}{previous_issuer}MembersCount: {members_count}
Identities:{identities}
Joiners:{joiners}
Actives:{actives}
Leavers:{leavers}
Revoked:{revoked}
Excluded:{excluded}
Certifications:{certifications}
Transactions:{transactions}
",
            version = self.version,
            currency = self.currency,
            block_number = self.number,
            pow_min = self.pow_min,
            time = self.time,
            median_time = self.median_time,
            dividend = dividend_str,
            unit_base = self.unit_base,
            issuer = self.issuers[0],
            issuers_frame = self.issuers_frame,
            issuers_frame_var = self.issuers_frame_var,
            issuers_count = self.issuers_count,
            parameters = parameters_str,
            previous_hash = previous_hash_str,
            previous_issuer = previous_issuer_str,
            members_count = self.members_count,
            identities = identities_str,
            joiners = joiners_str,
            actives = actives_str,
            leavers = leavers_str,
            revoked = revokeds_str,
            excluded = excludeds_str,
            certifications = certifications_str,
            transactions = transactions_str,
        )
    }
    fn generate_hash(&mut self) {
        self.hash = Some(self.compute_hash());
    }
    fn generate_inner_hash(&mut self) {
        self.inner_hash = Some(self.compute_inner_hash());
    }
    fn hash(&self) -> Option<BlockHash> {
        self.hash
    }
    fn increment_nonce(&mut self) {
        self.nonce += 1;
    }
    fn inner_hash(&self) -> Option<Hash> {
        self.inner_hash
    }
    fn issuers_count(&self) -> usize {
        self.issuers_count
    }
    fn number(&self) -> BlockNumber {
        self.number
    }
    fn previous_blockstamp(&self) -> Blockstamp {
        if self.number.0 > 0 {
            Blockstamp {
                id: BlockNumber(self.number.0 - 1),
                hash: BlockHash(unwrap!(self.previous_hash)),
            }
        } else {
            Blockstamp::default()
        }
    }
    fn previous_hash(&self) -> Option<Hash> {
        self.previous_hash
    }
    fn reduce(&mut self) {
        //self.hash = None;
        self.inner_hash = None;
        for i in &mut self.identities {
            i.reduce();
        }
        for i in &mut self.joiners {
            i.reduce();
        }
        for i in &mut self.actives {
            i.reduce();
        }
        for i in &mut self.leavers {
            i.reduce();
        }
        for i in &mut self.transactions {
            i.reduce();
        }
    }
    fn sign(&mut self, privkey: PrivKey) {
        self.signatures = vec![privkey.sign(self.compute_will_signed_string().as_bytes())];
    }
    fn verify_inner_hash(&self) -> bool {
        match self.inner_hash {
            Some(inner_hash) => inner_hash == self.compute_inner_hash(),
            None => false,
        }
    }
    fn verify_hash(&self) -> bool {
        match self.hash {
            Some(hash) => {
                let expected_hash = self.compute_hash();
                if hash == expected_hash {
                    true
                } else {
                    warn!(
                        "Block #{} have invalid hash (expected='{}', actual='{}', datas='{}').",
                        self.number.0,
                        expected_hash,
                        hash,
                        self.compute_will_hashed_string()
                    );
                    false
                }
            }
            None => false,
        }
    }
}

impl Document for BlockDocumentV10 {
    type PublicKey = PubKey;

    #[inline]
    fn version(&self) -> u16 {
        10
    }

    #[inline]
    fn currency(&self) -> &str {
        &self.currency.0
    }

    #[inline]
    fn blockstamp(&self) -> Blockstamp {
        Blockstamp {
            id: self.number,
            hash: self
                .hash
                .expect("Fatal error : try to get blockstamp of an uncomplete or reduce block !"),
        }
    }

    #[inline]
    fn issuers(&self) -> &Vec<PubKey> {
        &self.issuers
    }

    #[inline]
    fn signatures(&self) -> &Vec<Sig> {
        &self.signatures
    }

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        fatal_error!("as_bytes() must not be call for BlockDocumentV10 !")
    }

    #[inline]
    fn no_as_bytes(&self) -> bool {
        true
    }

    #[inline]
    fn to_bytes(&self) -> Vec<u8> {
        self.compute_will_signed_string().as_bytes().to_vec()
    }
}

impl CompactTextDocument for BlockDocumentV10 {
    fn as_compact_text(&self) -> String {
        let compact_inner_text = self.generate_compact_inner_text();
        format!(
            "{}InnerHash: {}\nNonce: ",
            compact_inner_text,
            self.inner_hash
                .expect(
                    "as_compact_text(): Try to get inner_hash of an uncompleted or reduce block !"
                )
                .to_hex()
        )
    }
}

impl TextDocument for BlockDocumentV10 {
    type CompactTextDocument_ = BlockDocumentV10;

    fn as_text(&self) -> &str {
        fatal_error!(
            "Dev error: function not implemented. Please use to_compact_document() instead"
        );
    }

    fn to_compact_document(&self) -> Self::CompactTextDocument_ {
        self.clone()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockDocumentV10Stringified {
    /// Version
    pub version: u64,
    /// Nonce
    pub nonce: u64,
    /// number
    pub number: u64,
    /// Minimal proof of work difficulty
    pub pow_min: u64,
    /// Local time of the block issuer
    pub time: u64,
    /// Average time
    pub median_time: u64,
    /// Members count
    pub members_count: u64,
    /// Monetary mass
    pub monetary_mass: u64,
    /// Unit base (power of ten)
    pub unit_base: u64,
    /// Number of compute members in the current frame
    pub issuers_count: u64,
    /// Current frame size (in blocks)
    pub issuers_frame: i64,
    /// Current frame variation buffer
    pub issuers_frame_var: i64,
    /// Currency.
    pub currency: String,
    /// Document issuer (there should be only one).
    pub issuers: Vec<String>,
    /// Document signature (there should be only one).
    /// This vector is empty, when the block is generated but the proof of work has not yet started
    pub signatures: Vec<String>,
    /// The hash is None, when the block is generated but the proof of work has not yet started
    pub hash: Option<String>,
    /// Currency parameters (only in genesis block)
    pub parameters: Option<String>,
    /// Hash of the previous block
    pub previous_hash: Option<String>,
    /// Issuer of the previous block
    pub previous_issuer: Option<String>,
    /// Hash of the deterministic content of the block
    pub inner_hash: Option<String>,
    /// Amount of new dividend created at this block, None if no dividend is created at this block
    pub dividend: Option<u64>,
    /// Identities
    pub identities: Vec<IdentityDocumentV10Stringified>,
    /// joiners
    pub joiners: Vec<MembershipDocumentV10Stringified>,
    /// Actives (=renewals)
    pub actives: Vec<MembershipDocumentV10Stringified>,
    /// Leavers
    pub leavers: Vec<MembershipDocumentV10Stringified>,
    /// Revokeds
    pub revoked: Vec<CompactRevocationStringDocument>,
    /// Excludeds
    pub excluded: Vec<String>,
    /// Certifications
    pub certifications: Vec<CompactCertificationStringDocument>,
    /// Transactions
    pub transactions: Vec<TransactionDocumentStringified>,
}

impl ToStringObject for BlockDocumentV10 {
    type StringObject = BlockDocumentV10Stringified;
    /// Transforms an object into a json object
    fn to_string_object(&self) -> BlockDocumentV10Stringified {
        BlockDocumentV10Stringified {
            version: u64::from(self.version),
            nonce: self.nonce,
            number: u64::from(self.number.0),
            pow_min: self.pow_min as u64,
            time: self.time,
            median_time: self.median_time,
            members_count: self.members_count as u64,
            monetary_mass: self.monetary_mass as u64,
            unit_base: self.unit_base as u64,
            issuers_count: self.issuers_count as u64,
            issuers_frame: self.issuers_frame as i64,
            issuers_frame_var: self.issuers_frame_var as i64,
            currency: self.currency.to_string(),
            issuers: self.issuers.iter().map(ToString::to_string).collect(),
            signatures: self.signatures.iter().map(ToString::to_string).collect(),
            hash: self.hash.map(|hash| hash.to_string()),
            parameters: self.parameters.map(|parameters| parameters.to_string()),
            previous_hash: self.previous_hash.map(|hash| hash.to_string()),
            previous_issuer: self.previous_issuer.map(|p| p.to_string()),
            inner_hash: self.inner_hash.map(|hash| hash.to_string()),
            dividend: self.dividend.map(|dividend| dividend as u64),
            identities: self
                .identities
                .iter()
                .map(ToStringObject::to_string_object)
                .collect(),
            joiners: self
                .joiners
                .iter()
                .map(ToStringObject::to_string_object)
                .collect(),
            actives: self
                .actives
                .iter()
                .map(ToStringObject::to_string_object)
                .collect(),
            leavers: self
                .leavers
                .iter()
                .map(ToStringObject::to_string_object)
                .collect(),
            revoked: self
                .revoked
                .iter()
                .map(|revocation_doc| match revocation_doc {
                    TextDocumentFormat::Complete(complete_revoc_doc) => {
                        complete_revoc_doc.to_compact_document().to_string_object()
                    }
                    TextDocumentFormat::Compact(compact_revoc_doc) => {
                        compact_revoc_doc.to_string_object()
                    }
                })
                .collect(),
            excluded: self.excluded.iter().map(ToString::to_string).collect(),
            certifications: self
                .certifications
                .iter()
                .map(|cert_doc| match cert_doc {
                    TextDocumentFormat::Complete(complete_cert_doc) => {
                        complete_cert_doc.to_compact_document().to_string_object()
                    }
                    TextDocumentFormat::Compact(compact_cert_doc) => {
                        compact_cert_doc.to_string_object()
                    }
                })
                .collect(),
            transactions: self
                .transactions
                .iter()
                .map(|tx_doc_or_tx_hash| match tx_doc_or_tx_hash {
                    TxDocOrTxHash::TxDoc(tx_doc) => tx_doc.to_string_object(),
                    TxDocOrTxHash::TxHash(_) => {
                        fatal_error!("Try to stringify block without their tx documents")
                    }
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::certification::CertificationDocumentParser;
    use super::transaction::TransactionDocumentParser;
    use super::*;
    use crate::{Document, VerificationResult};

    #[test]
    fn generate_and_verify_empty_block() {
        let mut block = BlockDocumentV10 {
            nonce: 100_010_200_000_006_940,
            version: 10,
            number: BlockNumber(174_260),
            pow_min: 68,
            time: 1_525_296_873,
            median_time: 1_525_292_577,
            members_count: 33,
            monetary_mass: 15_633_687,
            unit_base: 0,
            issuers_count: 8,
            issuers_frame: 41,
            issuers_frame_var: 0,
            currency: CurrencyName(String::from("g1-test")),
            issuers: vec![PubKey::Ed25519(ed25519::PublicKey::from_base58("39Fnossy1GrndwCnAXGDw3K5UYXhNXAFQe7yhYZp8ELP").unwrap())],
            signatures: vec![Sig::Ed25519(ed25519::Signature::from_base64("lqXrNOopjM39oM7hgB7Vq13uIohdCuLlhh/q8RVVEZ5UVASphow/GXikCdhbWID19Bn0XrXzTbt/R7akbE9xAg==").unwrap())],
            hash: None,
            parameters: None,
            previous_hash: Some(Hash::from_hex("0000A7D4361B9EBF4CE974A521149A73E8A5DE9B73907AB3BC918726AED7D40A").expect("fail to parse previous_hash")),
            previous_issuer: Some(PubKey::Ed25519(ed25519::PublicKey::from_base58("EPKuZA1Ek5y8S1AjAmAPtGrVCMFqUGzUEAa7Ei62CY2L").unwrap())),
            inner_hash: None,
            dividend: None,
            identities: Vec::new(),
            joiners: Vec::new(),
            actives: Vec::new(),
            leavers: Vec::new(),
            revoked: Vec::new(),
            excluded: Vec::new(),
            certifications: Vec::new(),
            transactions: Vec::new(),
        };
        // test inner_hash computation
        block.generate_inner_hash();
        println!("{}", block.generate_compact_text());
        assert_eq!(
            block
                .inner_hash
                .expect("tests::generate_and_verify_empty_block: Try to get inner_hash of an uncompleted or reduce block !")
                .to_hex(),
            "58E4865A47A46E0DF1449AABC449B5406A12047C413D61B5E17F86BE6641E7B0"
        );
        // Test signature validity
        assert_eq!(block.verify_signatures(), VerificationResult::Valid());
        // Test hash computation
        block.generate_hash();
        assert_eq!(
            block
                .hash
                .expect("Try to get hash of an uncompleted or reduce block !")
                .0
                .to_hex(),
            "00002EE584F36C15D3EB21AAC78E0896C75EF9070E73B4EC33BFA2C3D561EEB2"
        );
    }

    #[test]
    fn generate_and_verify_block() {
        let cert1 = CertificationDocumentParser::parse("Version: 10
Type: Certification
Currency: g1
Issuer: 6TAzLWuNcSqgNDNpAutrKpPXcGJwy1ZEMeVvZSZNs2e3
IdtyIssuer: CYPsYTdt87Tx6cCiZs9KD4jqPgYxbcVEqVZpRgJ9jjoV
IdtyUniqueID: PascaleM
IdtyTimestamp: 97401-0000003821911909F98519CC773D2D3E5CFE3D5DBB39F4F4FF33B96B4D41800E
IdtySignature: QncUVXxZ2NfARjdJOn6luILvDuG1NuK9qSoaU4CST2Ij8z7oeVtEgryHl+EXOjSe6XniALsCT0gU8wtadcA/Cw==
CertTimestamp: 106669-000003682E6FE38C44433DCE92E8B2A26C69B6D7867A2BAED231E788DDEF4251
UmseG2XKNwKcY8RFi6gUCT91udGnnNmSh7se10J1jeRVlwf+O2Tyb2Cccot9Dt7BO4+Kx2P6vFJB3oVGGHMxBA==").expect("Fail to parse cert1");

        let tx1 = TransactionDocumentParser::parse("Version: 10
Type: Transaction
Currency: g1
Blockstamp: 107982-000001242F6DA51C06A915A96C58BAA37AB3D1EB51F6E1C630C707845ACF764B
Locktime: 0
Issuers:
8dkCwvAqSczUjKsoVMDPVbQ3i6bBQeBQYawL87kqTSQ3
Inputs:
1002:0:D:8dkCwvAqSczUjKsoVMDPVbQ3i6bBQeBQYawL87kqTSQ3:106345
Unlocks:
0:SIG(0)
Outputs:
1002:0:SIG(CitdnuQgZ45tNFCagay7Wh12gwwHM8VLej1sWmfHWnQX)
Comment: DU symbolique pour demander le codage de nouvelles fonctionnalites cf. https://forum.monnaie-libre.fr/t/creer-de-nouvelles-fonctionnalites-dans-cesium-les-autres-applications/2025  Merci
T0LlCcbIn7xDFws48H8LboN6NxxwNXXTovG4PROLf7tkUAueHFWjfwZFKQXeZEHxfaL1eYs3QspGtLWUHPRVCQ==").expect("Fail to parse tx1");

        let tx2 = TransactionDocumentParser::parse("Version: 10
Type: Transaction
Currency: g1
Blockstamp: 107982-000001242F6DA51C06A915A96C58BAA37AB3D1EB51F6E1C630C707845ACF764B
Locktime: 0
Issuers:
8dkCwvAqSczUjKsoVMDPVbQ3i6bBQeBQYawL87kqTSQ3
Inputs:
1002:0:D:8dkCwvAqSczUjKsoVMDPVbQ3i6bBQeBQYawL87kqTSQ3:106614
Unlocks:
0:SIG(0)
Outputs:
1002:0:SIG(78ZwwgpgdH5uLZLbThUQH7LKwPgjMunYfLiCfUCySkM8)
Comment: DU symbolique pour demander le codage de nouvelles fonctionnalites cf. https://forum.monnaie-libre.fr/t/creer-de-nouvelles-fonctionnalites-dans-cesium-les-autres-applications/2025  Merci
a9PHPuSfw7jW8FRQHXFsGi/bnLjbtDnTYvEVgUC9u0WlR7GVofa+Xb+l5iy6NwuEXiwvueAkf08wPVY8xrNcCg==").expect("Fail to parse tx2");

        let mut block = BlockDocumentV10 {
            nonce: 10_300_000_018_323,
            version: 10,
            number: BlockNumber(107_984),
            pow_min: 88,
            time: 1_522_685_861,
            median_time: 1522683184,
            members_count: 896,
            monetary_mass: 140_469_765,
            unit_base: 0,
            issuers_count: 42,
            issuers_frame: 211,
            issuers_frame_var: 0,
            currency: CurrencyName(String::from("g1")),
            issuers: vec![PubKey::Ed25519(ed25519::PublicKey::from_base58("DA4PYtXdvQqk1nCaprXH52iMsK5Ahxs1nRWbWKLhpVkQ").unwrap())],
            signatures: vec![Sig::Ed25519(ed25519::Signature::from_base64("92id58VmkhgVNee4LDqBGSm8u/ooHzAD67JM6fhAE/CV8LCz7XrMF1DvRl+eRpmlaVkp6I+Iy8gmZ1WUM5C8BA==").unwrap())],
            hash: None,
            parameters: None,
            previous_hash: Some(Hash::from_hex("000001144968D0C3516BE6225E4662F182E28956AF46DD7FB228E3D0F9413FEB").expect("fail to parse previous_hash")),
            previous_issuer: Some(PubKey::Ed25519(ed25519::PublicKey::from_base58("D3krfq6J9AmfpKnS3gQVYoy7NzGCc61vokteTS8LJ4YH").unwrap())),
            inner_hash: None,
            dividend: None,
            identities: Vec::new(),
            joiners: Vec::new(),
            actives: Vec::new(),
            leavers: Vec::new(),
            revoked: Vec::new(),
            excluded: Vec::new(),
            certifications: vec![TextDocumentFormat::Complete(cert1)],
            transactions: vec![TxDocOrTxHash::TxDoc(Box::new(tx1)), TxDocOrTxHash::TxDoc(Box::new(tx2))],
        };
        // test inner_hash computation
        block.generate_inner_hash();
        println!("{}", block.generate_compact_text());
        assert_eq!(
            block
                .inner_hash
                .expect("tests::generate_and_verify_block: Try to get inner_hash of an uncompleted or reduce block !")
                .to_hex(),
            "C8AB69E33ECE2612EADC7AB30D069B1F1A3D8C95EBBFD50DE583AC8E3666CCA1"
        );
        // test generate_compact_text()
        assert_eq!(
            block.generate_compact_text(),
            "Version: 10
Type: Block
Currency: g1
Number: 107984
PoWMin: 88
Time: 1522685861
MedianTime: 1522683184
UnitBase: 0
Issuer: DA4PYtXdvQqk1nCaprXH52iMsK5Ahxs1nRWbWKLhpVkQ
IssuersFrame: 211
IssuersFrameVar: 0
DifferentIssuersCount: 42
PreviousHash: 000001144968D0C3516BE6225E4662F182E28956AF46DD7FB228E3D0F9413FEB
PreviousIssuer: D3krfq6J9AmfpKnS3gQVYoy7NzGCc61vokteTS8LJ4YH
MembersCount: 896
Identities:
Joiners:
Actives:
Leavers:
Revoked:
Excluded:
Certifications:
6TAzLWuNcSqgNDNpAutrKpPXcGJwy1ZEMeVvZSZNs2e3:CYPsYTdt87Tx6cCiZs9KD4jqPgYxbcVEqVZpRgJ9jjoV:106669:UmseG2XKNwKcY8RFi6gUCT91udGnnNmSh7se10J1jeRVlwf+O2Tyb2Cccot9Dt7BO4+Kx2P6vFJB3oVGGHMxBA==
Transactions:
TX:10:1:1:1:1:1:0
107982-000001242F6DA51C06A915A96C58BAA37AB3D1EB51F6E1C630C707845ACF764B
8dkCwvAqSczUjKsoVMDPVbQ3i6bBQeBQYawL87kqTSQ3
1002:0:D:8dkCwvAqSczUjKsoVMDPVbQ3i6bBQeBQYawL87kqTSQ3:106345
0:SIG(0)
1002:0:SIG(CitdnuQgZ45tNFCagay7Wh12gwwHM8VLej1sWmfHWnQX)
DU symbolique pour demander le codage de nouvelles fonctionnalites cf. https://forum.monnaie-libre.fr/t/creer-de-nouvelles-fonctionnalites-dans-cesium-les-autres-applications/2025  Merci
T0LlCcbIn7xDFws48H8LboN6NxxwNXXTovG4PROLf7tkUAueHFWjfwZFKQXeZEHxfaL1eYs3QspGtLWUHPRVCQ==
TX:10:1:1:1:1:1:0
107982-000001242F6DA51C06A915A96C58BAA37AB3D1EB51F6E1C630C707845ACF764B
8dkCwvAqSczUjKsoVMDPVbQ3i6bBQeBQYawL87kqTSQ3
1002:0:D:8dkCwvAqSczUjKsoVMDPVbQ3i6bBQeBQYawL87kqTSQ3:106614
0:SIG(0)
1002:0:SIG(78ZwwgpgdH5uLZLbThUQH7LKwPgjMunYfLiCfUCySkM8)
DU symbolique pour demander le codage de nouvelles fonctionnalites cf. https://forum.monnaie-libre.fr/t/creer-de-nouvelles-fonctionnalites-dans-cesium-les-autres-applications/2025  Merci
a9PHPuSfw7jW8FRQHXFsGi/bnLjbtDnTYvEVgUC9u0WlR7GVofa+Xb+l5iy6NwuEXiwvueAkf08wPVY8xrNcCg==
InnerHash: C8AB69E33ECE2612EADC7AB30D069B1F1A3D8C95EBBFD50DE583AC8E3666CCA1
Nonce: "
        );
        // Test signature validity
        assert_eq!(block.verify_signatures(), VerificationResult::Valid());
        // Test hash computation
        block.generate_hash();
        assert_eq!(
            block
                .hash
                .expect("Try to get hash of an uncompleted or reduce block !")
                .0
                .to_hex(),
            "000004F8B84A3590243BA562E5F2BA379F55A0B387C5D6FAC1022DFF7FFE6014"
        );
    }

    #[test]
    fn generate_and_verify_block_2() {
        let ms1 = MembershipDocumentV10Parser::parse(
            "Version: 10
Type: Membership
Currency: g1
Issuer: 4VZkro3N7VonygybESHngKUABA6gSrbW77Ktb94zE969
Block: 165645-000002D30130881939961A38D51CA233B3C696AA604439036DB1AAA4ED5046D2
Membership: IN
UserID: piaaf31
CertTS: 74077-0000022816648B2F7801E059F67CCD0C023FF0ED84459D52C70494D74DDCC6F6
gvaZ1QnJf8FjjRDJ0cYusgpBgQ8r0NqEz39BooH6DtIrgX+WTeXuLSnjZDl35VCBjokvyjry+v0OkTT8FKpABA==",
        )
        .expect("Fail to parse ms1");

        let tx1 = TransactionDocumentParser::parse(
            "Version: 10
Type: Transaction
Currency: g1
Blockstamp: 165645-000002D30130881939961A38D51CA233B3C696AA604439036DB1AAA4ED5046D2
Locktime: 0
Issuers:
51EFVNZwpfmTXU7BSLpeh3PZFgfdmm5hq5MzCDopdH2
Inputs:
1004:0:D:51EFVNZwpfmTXU7BSLpeh3PZFgfdmm5hq5MzCDopdH2:163766
1004:0:D:51EFVNZwpfmTXU7BSLpeh3PZFgfdmm5hq5MzCDopdH2:164040
1004:0:D:51EFVNZwpfmTXU7BSLpeh3PZFgfdmm5hq5MzCDopdH2:164320
1004:0:D:51EFVNZwpfmTXU7BSLpeh3PZFgfdmm5hq5MzCDopdH2:164584
1004:0:D:51EFVNZwpfmTXU7BSLpeh3PZFgfdmm5hq5MzCDopdH2:164849
1004:0:D:51EFVNZwpfmTXU7BSLpeh3PZFgfdmm5hq5MzCDopdH2:165118
1004:0:D:51EFVNZwpfmTXU7BSLpeh3PZFgfdmm5hq5MzCDopdH2:165389
Unlocks:
0:SIG(0)
1:SIG(0)
2:SIG(0)
3:SIG(0)
4:SIG(0)
5:SIG(0)
6:SIG(0)
Outputs:
7000:0:SIG(98wxzS683Tc1WWm1YxpL5WpxS7wBa1mZBccKSsYpaant)
28:0:SIG(51EFVNZwpfmTXU7BSLpeh3PZFgfdmm5hq5MzCDopdH2)
Comment: Panier mixte plus 40 pommes merci
7o/yIh0BNSAv5pNmHz04uUBl8TuP2s4HRFMtKeGFQfXNYJPUyJTP/dj6hdrgKtJkm5dCfbxT4KRy6wJf+dj1Cw==",
        )
        .expect("Fail to parse tx1");

        let tx2 = TransactionDocumentParser::parse(
            "Version: 10
Type: Transaction
Currency: g1
Blockstamp: 165645-000002D30130881939961A38D51CA233B3C696AA604439036DB1AAA4ED5046D2
Locktime: 0
Issuers:
3Uwq4qNp2A97P1XQueEBCxmnvgtAKMdfrEq6VB7Ph2qX
Inputs:
1002:0:D:3Uwq4qNp2A97P1XQueEBCxmnvgtAKMdfrEq6VB7Ph2qX:148827
1002:0:D:3Uwq4qNp2A97P1XQueEBCxmnvgtAKMdfrEq6VB7Ph2qX:149100
1002:0:D:3Uwq4qNp2A97P1XQueEBCxmnvgtAKMdfrEq6VB7Ph2qX:149370
1002:0:D:3Uwq4qNp2A97P1XQueEBCxmnvgtAKMdfrEq6VB7Ph2qX:149664
1002:0:D:3Uwq4qNp2A97P1XQueEBCxmnvgtAKMdfrEq6VB7Ph2qX:149943
1002:0:D:3Uwq4qNp2A97P1XQueEBCxmnvgtAKMdfrEq6VB7Ph2qX:150222
Unlocks:
0:SIG(0)
1:SIG(0)
2:SIG(0)
3:SIG(0)
4:SIG(0)
5:SIG(0)
Outputs:
6000:0:SIG(AopwTfXhj8VqZReFJYGGWnoWnXNj3RgaqFcGGywXpZrD)
12:0:SIG(3Uwq4qNp2A97P1XQueEBCxmnvgtAKMdfrEq6VB7Ph2qX)
Comment: En reglement de tes bons bocaux de fruits et legumes
nxr4exGrt16jteN9ZX3XZPP9l+X0OUbZ1o/QjE1hbWQNtVU3HhH9SJoEvNj2iVl3gCRr9u2OA9uj9vCyUDyjAg==
",
        )
        .expect("Fail to parse tx2");

        let mut block = BlockDocumentV10 {
            nonce: 10_300_000_090_296,
            version: 10,
            number: BlockNumber(165_647),
            pow_min: 90,
            time: 1_540_633_175,
            median_time: 1_540_627_811,
            members_count: 1402,
            monetary_mass: 386_008_811,
            unit_base: 0,
            issuers_count: 37,
            issuers_frame: 186,
            issuers_frame_var: 0,
            currency: CurrencyName(String::from("g1")),
            issuers: vec![PubKey::Ed25519(ed25519::PublicKey::from_base58("A4pc9Uuk4NXkWG8CibicjjPpEPdiup1mhjMoRWUZsonq").unwrap())],
            signatures: vec![Sig::Ed25519(ed25519::Signature::from_base64("2Z/+9ADdZvHXs19YR8+qDzgfl8WJlBG5PcbFvBG9TOuUJbjAdxhcgxrFrSRIABGWcCrIgLkB805fZVLP8jOjBA==").unwrap())],
            hash: None,
            parameters: None,
            previous_hash: Some(Hash::from_hex("000003E78FA4133F2C13B416F330C8DFB5A41EB87E37190615DB334F2C914A51").expect("fail to parse previous_hash")),
            previous_issuer: Some(PubKey::Ed25519(ed25519::PublicKey::from_base58("8NmGZmGjL1LUgJQRg282yQF7KTdQuRNAg8QfSa2qvd65").unwrap())),
            inner_hash: None,//Some(Hash::from_hex("3B49ECC1475549CFD94CA7B399311548A0FD0EC93C8EDD5670DAA5A958A41846").expect("fail to parse inner_hash")),
            dividend: None,
            identities: vec![],
            joiners: vec![],
            actives: vec![ms1],
            leavers: vec![],
            revoked: vec![],
            excluded: vec![],
            certifications: vec![],
            transactions: vec![TxDocOrTxHash::TxDoc(Box::new(tx1)), TxDocOrTxHash::TxDoc(Box::new(tx2))],
        };
        // test inner_hash computation
        block.generate_inner_hash();
        println!("{}", block.generate_compact_text());
        assert_eq!(
            block
                .inner_hash
                .expect("tests::generate_and_verify_block_2: Try to get inner_hash of an uncompleted or reduce block !")
                .to_hex(),
            "3B49ECC1475549CFD94CA7B399311548A0FD0EC93C8EDD5670DAA5A958A41846"
        );
        // test generate_compact_text()
        let block_compact_text = block.generate_compact_text();
        assert_eq!(
            block_compact_text,
            "Version: 10\nType: Block\nCurrency: g1\nNumber: 165647\nPoWMin: 90\nTime: 1540633175\nMedianTime: 1540627811\nUnitBase: 0\nIssuer: A4pc9Uuk4NXkWG8CibicjjPpEPdiup1mhjMoRWUZsonq\nIssuersFrame: 186\nIssuersFrameVar: 0\nDifferentIssuersCount: 37\nPreviousHash: 000003E78FA4133F2C13B416F330C8DFB5A41EB87E37190615DB334F2C914A51\nPreviousIssuer: 8NmGZmGjL1LUgJQRg282yQF7KTdQuRNAg8QfSa2qvd65\nMembersCount: 1402\nIdentities:\nJoiners:\nActives:\n4VZkro3N7VonygybESHngKUABA6gSrbW77Ktb94zE969:gvaZ1QnJf8FjjRDJ0cYusgpBgQ8r0NqEz39BooH6DtIrgX+WTeXuLSnjZDl35VCBjokvyjry+v0OkTT8FKpABA==:165645-000002D30130881939961A38D51CA233B3C696AA604439036DB1AAA4ED5046D2:74077-0000022816648B2F7801E059F67CCD0C023FF0ED84459D52C70494D74DDCC6F6:piaaf31\nLeavers:\nRevoked:\nExcluded:\nCertifications:\nTransactions:\nTX:10:1:7:7:2:1:0\n165645-000002D30130881939961A38D51CA233B3C696AA604439036DB1AAA4ED5046D2\n51EFVNZwpfmTXU7BSLpeh3PZFgfdmm5hq5MzCDopdH2\n1004:0:D:51EFVNZwpfmTXU7BSLpeh3PZFgfdmm5hq5MzCDopdH2:163766\n1004:0:D:51EFVNZwpfmTXU7BSLpeh3PZFgfdmm5hq5MzCDopdH2:164040\n1004:0:D:51EFVNZwpfmTXU7BSLpeh3PZFgfdmm5hq5MzCDopdH2:164320\n1004:0:D:51EFVNZwpfmTXU7BSLpeh3PZFgfdmm5hq5MzCDopdH2:164584\n1004:0:D:51EFVNZwpfmTXU7BSLpeh3PZFgfdmm5hq5MzCDopdH2:164849\n1004:0:D:51EFVNZwpfmTXU7BSLpeh3PZFgfdmm5hq5MzCDopdH2:165118\n1004:0:D:51EFVNZwpfmTXU7BSLpeh3PZFgfdmm5hq5MzCDopdH2:165389\n0:SIG(0)\n1:SIG(0)\n2:SIG(0)\n3:SIG(0)\n4:SIG(0)\n5:SIG(0)\n6:SIG(0)\n7000:0:SIG(98wxzS683Tc1WWm1YxpL5WpxS7wBa1mZBccKSsYpaant)\n28:0:SIG(51EFVNZwpfmTXU7BSLpeh3PZFgfdmm5hq5MzCDopdH2)\nPanier mixte plus 40 pommes merci\n7o/yIh0BNSAv5pNmHz04uUBl8TuP2s4HRFMtKeGFQfXNYJPUyJTP/dj6hdrgKtJkm5dCfbxT4KRy6wJf+dj1Cw==\nTX:10:1:6:6:2:1:0\n165645-000002D30130881939961A38D51CA233B3C696AA604439036DB1AAA4ED5046D2\n3Uwq4qNp2A97P1XQueEBCxmnvgtAKMdfrEq6VB7Ph2qX\n1002:0:D:3Uwq4qNp2A97P1XQueEBCxmnvgtAKMdfrEq6VB7Ph2qX:148827\n1002:0:D:3Uwq4qNp2A97P1XQueEBCxmnvgtAKMdfrEq6VB7Ph2qX:149100\n1002:0:D:3Uwq4qNp2A97P1XQueEBCxmnvgtAKMdfrEq6VB7Ph2qX:149370\n1002:0:D:3Uwq4qNp2A97P1XQueEBCxmnvgtAKMdfrEq6VB7Ph2qX:149664\n1002:0:D:3Uwq4qNp2A97P1XQueEBCxmnvgtAKMdfrEq6VB7Ph2qX:149943\n1002:0:D:3Uwq4qNp2A97P1XQueEBCxmnvgtAKMdfrEq6VB7Ph2qX:150222\n0:SIG(0)\n1:SIG(0)\n2:SIG(0)\n3:SIG(0)\n4:SIG(0)\n5:SIG(0)\n6000:0:SIG(AopwTfXhj8VqZReFJYGGWnoWnXNj3RgaqFcGGywXpZrD)\n12:0:SIG(3Uwq4qNp2A97P1XQueEBCxmnvgtAKMdfrEq6VB7Ph2qX)\nEn reglement de tes bons bocaux de fruits et legumes\nnxr4exGrt16jteN9ZX3XZPP9l+X0OUbZ1o/QjE1hbWQNtVU3HhH9SJoEvNj2iVl3gCRr9u2OA9uj9vCyUDyjAg==\nInnerHash: 3B49ECC1475549CFD94CA7B399311548A0FD0EC93C8EDD5670DAA5A958A41846\nNonce: "
        );
        // Test signature validity
        assert_eq!(block.verify_signatures(), VerificationResult::Valid());
        // Test hash computation
        block.generate_hash();
        assert_eq!(
            block
                .hash
                .expect("Try to get hash of an uncompleted or reduce block !")
                .0
                .to_hex(),
            "000002026E32A3D649B34968AAF9D03C4F19A5954229C54A801BBB1CD216B230"
        );
    }
}
