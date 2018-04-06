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

//! Wrappers around Block document.

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use duniter_crypto::keys::{ed25519, PrivateKey};

use blockchain::v10::documents::certification::CertificationDocument;
use blockchain::v10::documents::identity::IdentityDocument;
use blockchain::v10::documents::membership::MembershipDocument;
use blockchain::v10::documents::revocation::RevocationDocument;
use blockchain::v10::documents::transaction::TransactionDocument;
use blockchain::v10::documents::{TextDocument, V10Document};
use blockchain::{BlockchainProtocol, Document, IntoSpecializedDocument};
use {BlockHash, BlockId, Blockstamp, Hash};

/// Currency parameters
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BlockParameters {
    /// UD target growth rate (see Relative Theorie of Money)
    pub c: f64,
    /// Duration between the creation of two DU (in seconds)
    pub dt: i64,
    /// Amount of the initial UD
    pub ud0: i64,
    /// Minimum duration between the writing of 2 certifications from the same issuer (in seconds)
    pub sig_period: u64,
    /// Maximum number of active certifications at the same time (for the same issuer)
    pub sig_stock: i64,
    /// Maximum retention period of a pending certification
    pub sig_window: i64,
    /// Time to expiry of written certification
    pub sig_validity: i64,
    /// Minimum number of certifications required to become a member
    pub sig_qty: i64,
    /// Maximum retention period of a pending identity
    pub idty_window: i64,
    /// Maximum retention period of a pending membership
    pub ms_window: i64,
    /// Percentage of referring members who must be within step_max steps of each member
    pub x_percent: f64,
    /// Time to expiry of written membership
    pub ms_validity: u64,
    /// For a member to respect the distance rule,
    /// there must exist for more than x_percent % of the referring members
    /// a path of less than step_max steps from the referring member to the evaluated member.
    pub step_max: u32,
    /// Number of blocks used for calculating median time.
    pub median_time_blocks: i64,
    /// The average time for writing 1 block (wished time)
    pub avg_gen_time: i64,
    /// The number of blocks required to evaluate again PoWMin value
    pub dt_diff_eval: i64,
    /// The percent of previous issuers to reach for personalized difficulty
    pub percent_rot: f64,
    /// Time of first UD.
    pub ud_time0: i64,
    /// Time of first reevaluation of the UD.
    pub ud_reeval_time0: i64,
    /// Time period between two re-evaluation of the UD.
    pub dt_reeval: i64,
}

/// Wrap a Block document.
///
/// Must be created by parsing a text document or using a builder.
#[derive(Debug, Clone)]
pub struct BlockDocument {
    /// Nonce
    pub nonce: u64,
    /// number
    pub number: BlockId,
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
    pub issuers_frame: isize,
    /// Current frame variation buffer
    pub issuers_frame_var: isize,
    /// Currency.
    pub currency: String,
    /// Document issuer (there should be only one).
    pub issuers: Vec<ed25519::PublicKey>,
    /// Document signature (there should be only one).
    /// This vector is empty, when the block is generated but the proof of work has not yet started
    pub signatures: Vec<ed25519::Signature>,
    /// The hash is None, when the block is generated but the proof of work has not yet started
    pub hash: Option<BlockHash>,
    /// Currency parameters (only in genesis block)
    pub parameters: Option<BlockParameters>,
    /// Hash of the previous block
    pub previous_hash: Option<Hash>,
    /// Issuer of the previous block
    pub previous_issuer: Option<ed25519::PublicKey>,
    /// Hash of the deterministic content of the block
    pub inner_hash: Option<Hash>,
    /// Amount of new dividend created at this block, None if no dividend is created at this block
    pub dividend: Option<usize>,
    /// Identities
    pub identities: Vec<IdentityDocument>,
    /// joiners
    pub joiners: Vec<MembershipDocument>,
    /// Actives (=renewals)
    pub actives: Vec<MembershipDocument>,
    /// Leavers
    pub leavers: Vec<MembershipDocument>,
    /// Revokeds
    pub revoked: Vec<RevocationDocument>,
    /// Excludeds
    pub excluded: Vec<ed25519::PublicKey>,
    /// Certifications
    pub certifications: Vec<CertificationDocument>,
    /// Transactions
    pub transactions: Vec<TransactionDocument>,
    /// Part to sign
    pub inner_hash_and_nonce_str: String,
}

impl BlockDocument {
    /// Return blockstamp
    pub fn blockstamp(&self) -> Blockstamp {
        Blockstamp {
            id: self.number,
            hash: self.hash.unwrap(),
        }
    }
    /// Compute inner hash
    pub fn compute_inner_hash(&mut self) {
        let mut sha256 = Sha256::new();
        sha256.input_str(&self.generate_compact_inner_text());
        self.inner_hash = Some(Hash::from_hex(&sha256.result_str()).unwrap());
    }
    /// Change nonce
    pub fn change_nonce(&mut self, new_nonce: u64) {
        self.nonce = new_nonce;
        self.inner_hash_and_nonce_str = format!(
            "InnerHash: {}\nNonce: {}\n",
            self.inner_hash.unwrap().to_hex(),
            self.nonce
        );
    }
    /// Sign block
    pub fn sign(&mut self, privkey: ed25519::PrivateKey) {
        self.signatures = vec![privkey.sign(self.inner_hash_and_nonce_str.as_bytes())];
    }
    /// Compute hash
    pub fn compute_hash(&mut self) {
        let mut sha256 = Sha256::new();
        sha256.input_str(&format!(
            "InnerHash: {}\nNonce: {}\n{}\n",
            self.inner_hash.unwrap().to_hex(),
            self.nonce,
            self.signatures[0]
        ));
        self.hash = Some(BlockHash(Hash::from_hex(&sha256.result_str()).unwrap()));
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
            revokeds_str.push_str(&revocation.generate_compact_text());
        }
        let mut excludeds_str = String::from("");
        for exclusion in self.excluded.clone() {
            excludeds_str.push_str("\n");
            excludeds_str.push_str(&exclusion.to_string());
        }
        let mut certifications_str = String::from("");
        for certification in self.certifications.clone() {
            certifications_str.push_str("\n");
            certifications_str.push_str(&certification.generate_compact_text());
        }
        let mut transactions_str = String::from("");
        for transaction in self.transactions.clone() {
            transactions_str.push_str("\n");
            transactions_str.push_str(&transaction.generate_compact_text());
        }
        let mut dividend_str = String::from("");
        if let Some(dividend) = self.dividend {
            if dividend > 0 {
                dividend_str.push_str("UniversalDividend: ");
                dividend_str.push_str(&dividend.to_string());
                dividend_str.push_str("\n");
            }
        }
        format!(
            "Version: 10
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
PreviousHash: {previous_hash}
PreviousIssuer: {previous_issuer}
MembersCount: {members_count}
Identities:{identities}
Joiners:{joiners}
Actives:{actives}
Leavers:{leavers}
Revoked:{revoked}
Excluded:{excluded}
Certifications:{certifications}
Transactions:{transactions}
",
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
            previous_hash = self.previous_hash.unwrap(),
            previous_issuer = self.previous_issuer.unwrap(),
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
}

impl Document for BlockDocument {
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
        self.inner_hash_and_nonce_str.as_bytes()
    }
}

impl TextDocument for BlockDocument {
    fn as_text(&self) -> &str {
        panic!();
    }

    fn generate_compact_text(&self) -> String {
        let compact_inner_text = self.generate_compact_inner_text();
        format!(
            "{}InnerHash: {}\nNonce: ",
            compact_inner_text,
            self.inner_hash.unwrap().to_hex()
        )
    }
}

impl IntoSpecializedDocument<BlockchainProtocol> for BlockDocument {
    fn into_specialized(self) -> BlockchainProtocol {
        BlockchainProtocol::V10(Box::new(V10Document::Block(Box::new(self))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blockchain::v10::documents::V10DocumentParser;
    use blockchain::{Document, DocumentParser, VerificationResult};
    use duniter_crypto::keys::{PublicKey, Signature};
    use std::ops::Deref;

    #[test]
    fn generate_and_verify_empty_block() {
        let mut block = BlockDocument {
            nonce: 10_500_000_089_933,
            number: BlockId(107_777),
            pow_min: 89,
            time: 1_522_624_657,
            median_time: 1_522_616_790,
            members_count: 894,
            monetary_mass: 139_571_973,
            unit_base: 0,
            issuers_count: 41,
            issuers_frame: 201,
            issuers_frame_var: 5,
            currency: String::from("g1"),
            issuers: vec![ed25519::PublicKey::from_base58("2sZF6j2PkxBDNAqUde7Dgo5x3crkerZpQ4rBqqJGn8QT").unwrap()],
            signatures: vec![ed25519::Signature::from_base64("FsRxB+NOiL+8zTr2d3B2j2KBItDuCa0KjFMF6hXmdQzfqXAs9g3m7DlGgYLcqzqe6JXjx/Lyzqze1HBR4cS0Aw==").unwrap()],
            hash: None,
            parameters: None,
            previous_hash: Some(Hash::from_hex("0000001F8AACF6764135F3E5D0D4E8358A3CBE537A4BF71152A00CC442EFD136").expect("fail to parse previous_hash")),
            previous_issuer: Some(ed25519::PublicKey::from_base58("38MEAZN68Pz1DTvT3tqgxx4yQP6snJCQhPqEFxbDk4aE").unwrap()),
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
            inner_hash_and_nonce_str: String::new(),
        };
        // test inner_hash computation
        block.compute_inner_hash();
        println!("{}", block.generate_compact_text());
        assert_eq!(
            block.inner_hash.unwrap().to_hex(),
            "95948AC4D45E46DA07CE0713EDE1CE0295C227EE4CA5557F73F56B7DD46FE89C"
        );
        // test generate_compact_text()
        assert_eq!(
            block.generate_compact_text(),
            "Version: 10
Type: Block
Currency: g1
Number: 107777
PoWMin: 89
Time: 1522624657
MedianTime: 1522616790
UnitBase: 0
Issuer: 2sZF6j2PkxBDNAqUde7Dgo5x3crkerZpQ4rBqqJGn8QT
IssuersFrame: 201
IssuersFrameVar: 5
DifferentIssuersCount: 41
PreviousHash: 0000001F8AACF6764135F3E5D0D4E8358A3CBE537A4BF71152A00CC442EFD136
PreviousIssuer: 38MEAZN68Pz1DTvT3tqgxx4yQP6snJCQhPqEFxbDk4aE
MembersCount: 894
Identities:
Joiners:
Actives:
Leavers:
Revoked:
Excluded:
Certifications:
Transactions:
InnerHash: 95948AC4D45E46DA07CE0713EDE1CE0295C227EE4CA5557F73F56B7DD46FE89C
Nonce: "
        );
        // Test signature validity
        block.change_nonce(10_500_000_089_933);
        assert_eq!(block.verify_signatures(), VerificationResult::Valid());
        // Test hash computation
        block.compute_hash();
        assert_eq!(
            block.hash.unwrap().0.to_hex(),
            "000002D3296A2D257D01F6FEE8AEC5C3E5779D04EA43F08901F41998FA97D9A1"
        );
    }

    #[test]
    fn generate_and_verify_block() {
        let cert1_ = V10DocumentParser::parse("Version: 10
Type: Certification
Currency: g1
Issuer: 6TAzLWuNcSqgNDNpAutrKpPXcGJwy1ZEMeVvZSZNs2e3
IdtyIssuer: CYPsYTdt87Tx6cCiZs9KD4jqPgYxbcVEqVZpRgJ9jjoV
IdtyUniqueID: PascaleM
IdtyTimestamp: 97401-0000003821911909F98519CC773D2D3E5CFE3D5DBB39F4F4FF33B96B4D41800E
IdtySignature: QncUVXxZ2NfARjdJOn6luILvDuG1NuK9qSoaU4CST2Ij8z7oeVtEgryHl+EXOjSe6XniALsCT0gU8wtadcA/Cw==
CertTimestamp: 106669-000003682E6FE38C44433DCE92E8B2A26C69B6D7867A2BAED231E788DDEF4251
UmseG2XKNwKcY8RFi6gUCT91udGnnNmSh7se10J1jeRVlwf+O2Tyb2Cccot9Dt7BO4+Kx2P6vFJB3oVGGHMxBA==").unwrap();
        let cert1 = match cert1_ {
            V10Document::Certification(doc) => (*doc.deref()).clone(),
            _ => panic!("Wrong document type"),
        };

        let tx1_ = V10DocumentParser::parse("Version: 10
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
T0LlCcbIn7xDFws48H8LboN6NxxwNXXTovG4PROLf7tkUAueHFWjfwZFKQXeZEHxfaL1eYs3QspGtLWUHPRVCQ==").unwrap();
        let tx1 = match tx1_ {
            V10Document::Transaction(doc) => (*doc.deref()).clone(),
            _ => panic!("Wrong document type"),
        };

        let tx2_ = V10DocumentParser::parse("Version: 10
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
a9PHPuSfw7jW8FRQHXFsGi/bnLjbtDnTYvEVgUC9u0WlR7GVofa+Xb+l5iy6NwuEXiwvueAkf08wPVY8xrNcCg==").unwrap();
        let tx2 = match tx2_ {
            V10Document::Transaction(doc) => (*doc.deref()).clone(),
            _ => panic!("Wrong document type"),
        };

        let mut block = BlockDocument {
            nonce: 0,
            number: BlockId(107_984),
            pow_min: 88,
            time: 1_522_685_861,
            median_time: 1522683184,
            members_count: 896,
            monetary_mass: 140_469_765,
            unit_base: 0,
            issuers_count: 42,
            issuers_frame: 211,
            issuers_frame_var: 0,
            currency: String::from("g1"),
            issuers: vec![ed25519::PublicKey::from_base58("DA4PYtXdvQqk1nCaprXH52iMsK5Ahxs1nRWbWKLhpVkQ").unwrap()],
            signatures: vec![ed25519::Signature::from_base64("92id58VmkhgVNee4LDqBGSm8u/ooHzAD67JM6fhAE/CV8LCz7XrMF1DvRl+eRpmlaVkp6I+Iy8gmZ1WUM5C8BA==").unwrap()],
            hash: None,
            parameters: None,
            previous_hash: Some(Hash::from_hex("000001144968D0C3516BE6225E4662F182E28956AF46DD7FB228E3D0F9413FEB").expect("fail to parse previous_hash")),
            previous_issuer: Some(ed25519::PublicKey::from_base58("D3krfq6J9AmfpKnS3gQVYoy7NzGCc61vokteTS8LJ4YH").unwrap()),
            inner_hash: None,
            dividend: None,
            identities: Vec::new(),
            joiners: Vec::new(),
            actives: Vec::new(),
            leavers: Vec::new(),
            revoked: Vec::new(),
            excluded: Vec::new(),
            certifications: vec![cert1],
            transactions: vec![tx1, tx2],
            inner_hash_and_nonce_str: String::new(),
        };
        // test inner_hash computation
        block.compute_inner_hash();
        println!("{}", block.generate_compact_text());
        assert_eq!(
            block.inner_hash.unwrap().to_hex(),
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
        block.change_nonce(10_300_000_018_323);
        assert_eq!(block.verify_signatures(), VerificationResult::Valid());
        // Test hash computation
        block.compute_hash();
        assert_eq!(
            block.hash.unwrap().0.to_hex(),
            "000004F8B84A3590243BA562E5F2BA379F55A0B387C5D6FAC1022DFF7FFE6014"
        );
    }
}
