//  Copyright (C) 2019  Éloïs SANCHEZ
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

//! Mocks for projects use dubp-block-doc

pub mod block_params;

use dubp_block_doc::block::BlockDocumentTrait;
use dubp_block_doc::{BlockDocument, BlockDocumentV10};
use dubp_common_doc::blockstamp::Blockstamp;
use dubp_common_doc::parser::TextDocumentParser;
use dubp_common_doc::traits::text::*;
use dubp_common_doc::{BlockHash, BlockNumber};
use dubp_currency_params::genesis_block_params::v10::BlockV10Parameters;
use dubp_currency_params::CurrencyName;
use dubp_user_docs::documents::certification::*;
use dubp_user_docs::documents::transaction::*;
use dup_crypto::bases::b16::str_hex_to_32bytes;
use dup_crypto::hashs::Hash;
use dup_crypto::keys::{ed25519, PubKey, PublicKey, Sig, Signator, Signature};
use durs_common_tools::UsizeSer32;

/// Generate n mock blockstamps
pub fn generate_blockstamps(n: usize) -> Vec<Blockstamp> {
    (0..n)
        .map(|i| Blockstamp {
            id: BlockNumber(i as u32),
            hash: BlockHash(dup_crypto_tests_tools::mocks::hash_from_byte(
                (i % 255) as u8,
            )),
        })
        .collect()
}

/// Generate n empty timed block document
pub fn gen_empty_timed_blocks_v10(n: usize, time_step: u64) -> Vec<BlockDocument> {
    (0..n)
        .map(|i| {
            BlockDocument::V10(gen_empty_timed_block_v10(
                Blockstamp {
                    id: BlockNumber(i as u32),
                    hash: BlockHash(dup_crypto_tests_tools::mocks::hash_from_byte(
                        (i % 255) as u8,
                    )),
                },
                time_step * n as u64,
                if i == 0 {
                    Hash::default()
                } else {
                    dup_crypto_tests_tools::mocks::hash_from_byte(((i - 1) % 255) as u8)
                },
            ))
        })
        .collect()
}

/// Generate empty timed and issued and hashed block document
/// (usefull for tests that need valid hashs and issuer and signature and median_time field)
pub fn gen_empty_timed_issued_hashed_block_v10(
    block_number: BlockNumber,
    time: u64,
    previous_issuer: PubKey,
    previous_hash: Hash,
    signator: &dup_crypto::keys::SignatorEnum,
) -> BlockDocumentV10 {
    let mut block = gen_empty_block_v10(block_number);
    block.time = time;
    block.median_time = time;
    block.issuers = vec![signator.public_key()];
    block.previous_issuer = Some(previous_issuer);
    block.previous_hash = Some(previous_hash);
    block.inner_hash = Some(block.compute_inner_hash());
    block.sign(signator);
    block.hash = Some(block.compute_hash());
    block
}

/// Generate empty timed block document with provided hashs
/// (usefull for tests that only need blockstamp and median_time fields)
pub fn gen_empty_timed_block_v10(
    blockstamp: Blockstamp,
    time: u64,
    previous_hash: Hash,
) -> BlockDocumentV10 {
    let mut block = gen_empty_block_v10(blockstamp.id);
    block.median_time = time;
    block.previous_hash = Some(previous_hash);
    block.hash = Some(blockstamp.hash);
    block
}

/// Generate empty issued block document
/// (usefull for tests that only need issuer field)
pub fn gen_empty_issued_block_v10(issuer: PubKey) -> BlockDocumentV10 {
    let mut block = gen_empty_block_v10(BlockNumber(0));
    block.issuers = vec![issuer];
    block
}

fn gen_empty_block_v10(block_number: BlockNumber) -> BlockDocumentV10 {
    BlockDocumentV10 {
        version: UsizeSer32(10),
        nonce: 0,
        number: block_number,
        pow_min: UsizeSer32(0),
        time: 0,
        median_time: 0,
        members_count: UsizeSer32(0),
        monetary_mass: 0,
        unit_base: UsizeSer32(0),
        issuers_count: UsizeSer32(0),
        issuers_frame: UsizeSer32(0),
        issuers_frame_var: 0,
        currency: CurrencyName("test_currency".to_owned()),
        issuers: vec![],
        signatures: vec![],
        hash: None,
        parameters: None,
        previous_hash: None,
        previous_issuer: None,
        dividend: None,
        identities: vec![],
        joiners: vec![],
        actives: vec![],
        leavers: vec![],
        revoked: vec![],
        excluded: vec![],
        certifications: vec![],
        transactions: vec![],
        inner_hash: None,
    }
}

/// Generate mock block which is not a genesis block
pub fn gen_mock_normal_block_v10() -> BlockDocumentV10 {
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
    let CertificationDocument::V10(cert1) = cert1;

    let TransactionDocument::V10(tx1) = dubp_user_docs_tests_tools::mocks::tx::gen_mock_tx_doc();
    let TransactionDocument::V10(tx2) = TransactionDocumentParser::parse("Version: 10
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

    BlockDocumentV10 {
            nonce: 10_300_000_018_323,
            version: UsizeSer32(10),
            number: BlockNumber(107_984),
            pow_min: UsizeSer32(88),
            time: 1_522_685_861,
            median_time: 1_522_683_184,
            members_count: UsizeSer32(896),
            monetary_mass: 140_469_765,
            unit_base: UsizeSer32(0),
            issuers_count: UsizeSer32(42),
            issuers_frame: UsizeSer32(211),
            issuers_frame_var: 0,
            currency: CurrencyName(String::from("g1")),
            issuers: vec![PubKey::Ed25519(ed25519::PublicKey::from_base58("DA4PYtXdvQqk1nCaprXH52iMsK5Ahxs1nRWbWKLhpVkQ").expect("fail to parse issuers"))],
            signatures: vec![Sig::Ed25519(ed25519::Signature::from_base64("92id58VmkhgVNee4LDqBGSm8u/ooHzAD67JM6fhAE/CV8LCz7XrMF1DvRl+eRpmlaVkp6I+Iy8gmZ1WUM5C8BA==").expect("fail to parse signatures"))],
            hash: None,
            parameters: None,
            previous_hash: Some(Hash::from_hex("000001144968D0C3516BE6225E4662F182E28956AF46DD7FB228E3D0F9413FEB").expect("fail to parse previous_hash")),
            previous_issuer: Some(PubKey::Ed25519(ed25519::PublicKey::from_base58("D3krfq6J9AmfpKnS3gQVYoy7NzGCc61vokteTS8LJ4YH").expect("fail to parse previous_issuer"))),
            inner_hash: Some(Hash(
                    str_hex_to_32bytes(
                        "C8AB69E33ECE2612EADC7AB30D069B1F1A3D8C95EBBFD50DE583AC8E3666CCA1",
                    ).expect("fail to parse inner_hash"))),
            dividend: None,
            identities: Vec::new(),
            joiners: Vec::new(),
            actives: Vec::new(),
            leavers: Vec::new(),
            revoked: Vec::new(),
            excluded: Vec::new(),
            certifications: vec![TextDocumentFormat::Complete(cert1)],
            transactions: vec![tx1, tx2],
        }
}

/// Generate a mock genesis block
pub fn gen_mock_genesis_block_v10() -> BlockDocumentV10 {
    BlockDocumentV10 {
            nonce: 0,
            version: UsizeSer32(10),
            number: BlockNumber(0),
            pow_min: UsizeSer32(0),
            time: 0,
            median_time: 0,
            members_count: UsizeSer32(0),
            monetary_mass: 0,
            unit_base: UsizeSer32(0),
            issuers_count: UsizeSer32(0),
            issuers_frame: UsizeSer32(0),
            issuers_frame_var: 0,
            currency: CurrencyName(String::from("g1")),
            issuers: vec![PubKey::Ed25519(ed25519::PublicKey::from_base58("DA4PYtXdvQqk1nCaprXH52iMsK5Ahxs1nRWbWKLhpVkQ").expect("fail to parse issuers"))],
            signatures: vec![Sig::Ed25519(ed25519::Signature::from_base64("92id58VmkhgVNee4LDqBGSm8u/ooHzAD67JM6fhAE/CV8LCz7XrMF1DvRl+eRpmlaVkp6I+Iy8gmZ1WUM5C8BA==").expect("fail to parse signatures"))],
            hash: None,
            parameters: Some(BlockV10Parameters::default()),
            previous_hash: None,
            previous_issuer: None,
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
        }
}
