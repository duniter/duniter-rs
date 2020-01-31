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

//! Verifies if transaction document verifies the local rules

use dubp_common_doc::errors::DocumentSigsErr;
use dubp_common_doc::traits::text::CompactTextDocument;
use dubp_common_doc::traits::Document;
use dubp_user_docs::documents::transaction::v10::TransactionDocumentV10;
use dubp_user_docs::documents::transaction::TransactionDocumentTrait;
use durs_common_tools::traits::bool_ext::BoolExt;

#[derive(Debug, PartialEq)]
/// Transaction Document Error
pub enum TransactionDocumentError {
    /// Length is too long
    TooLong {
        expected_max_length: usize,
        actual_length: usize,
    },
    /// There is no input
    MissingInput,
    /// Signature error    
    TxSignatureError(DocumentSigsErr),
}

/// Local verification of a Tx Document
pub fn local_verify_tx_doc_v10(
    dubp_version: usize,
    tx_doc: &TransactionDocumentV10,
) -> Result<(), TransactionDocumentError> {
    // A transaction in compact format must measure less than 100 lines
    (tx_doc.as_compact_text().lines().count() < 100).or_err(TransactionDocumentError::TooLong {
        expected_max_length: 100,
        actual_length: tx_doc.as_compact_text().lines().count(),
    })?;

    // A transaction must have at least 1 input
    (tx_doc.get_inputs().is_empty().not()).or_err(TransactionDocumentError::MissingInput)?;

    ////////////////////////////////////////////////////////////////////////////////////
    // A transaction **must** have signatures matching its content **for each issuer**
    // Signatures are ordered by issuer
    // Signatures are made over the transaction's content, signatures excepted
    ////////////////////////////////////////////////////////////////////////////////////
    // Temporary disabled due to #183
    if dubp_version >= 12 {
        tx_doc
            .verify_signatures()
            .map_err(TransactionDocumentError::TxSignatureError)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dubp_common_doc::traits::DocumentBuilder;
    use dubp_common_doc::Blockstamp;
    use dubp_user_docs::documents::transaction::v10::TransactionDocumentV10Builder;
    use dubp_user_docs::documents::transaction::v10::TransactionInputUnlocksV10;
    use dubp_user_docs::documents::transaction::v10::TransactionInputV10;
    use dubp_user_docs::documents::transaction::v10::TransactionOutputV10;
    use dubp_user_docs::documents::transaction::OutputIndex;
    use dubp_user_docs::documents::transaction::TransactionDocument;
    use dubp_user_docs::documents::transaction::TransactionUnlockProof;
    use dubp_user_docs::documents::transaction::TxAmount;
    use dubp_user_docs::documents::transaction::TxBase;
    use dubp_user_docs_tests_tools::mocks::tx::gen_mock_tx_doc;
    use dup_crypto::hashs::Hash;
    use dup_crypto::keys::*;
    use std::str::FromStr;
    use unwrap::unwrap;

    #[inline]
    fn blockstamp() -> Blockstamp {
        Blockstamp::from_string(
            "0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
        )
        .expect("invalid blockstamp")
    }

    #[inline]
    fn issuers() -> Vec<PubKey> {
        let keypair = ed25519::KeyPairFromSeed32Generator::generate(
            Seed32::from_base58("DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV")
                .expect("invalid seed32"),
        );
        vec![PubKey::Ed25519(keypair.public_key())]
    }

    #[inline]
    fn sig1() -> Sig {
        Sig::Ed25519(unwrap!(ed25519::Signature::from_base64(
            "cq86RugQlqAEyS8zFkB9o0PlWPSb+a6D/MEnLe8j+okyFYf/WzI6pFiBkQ9PSOVn5I0dwzVXg7Q4N1apMWeGAg==")))
    }

    #[inline]
    fn input1() -> TransactionInputV10 {
        TransactionInputV10::T(
            TxAmount(950),
            TxBase(0),
            Hash::from_hex("2CF1ACD8FE8DC93EE39A1D55881C50D87C55892AE8E4DB71D4EBAB3D412AA8FD")
                .expect("invalid hash"),
            OutputIndex(1),
        )
    }

    #[inline]
    fn unlocks() -> Vec<TransactionInputUnlocksV10> {
        vec![TransactionInputUnlocksV10 {
            index: 0,
            unlocks: vec![TransactionUnlockProof::Sig(0)],
        }]
    }

    #[inline]
    fn outputs() -> Vec<TransactionOutputV10> {
        vec![TransactionOutputV10::from_str(
            "10:0:SIG(FD9wujR7KABw88RyKEGBYRLz8PA6jzVCbcBAsrBXBqSa)",
        )
        .expect("fail to parse output !")]
    }

    fn tx_builder<'a>(
        blockstamp: &'a Blockstamp,
        issuers: &'a Vec<PubKey>,
        inputs: &'a Vec<TransactionInputV10>,
        unlocks: &'a Vec<TransactionInputUnlocksV10>,
        outputs: &'a Vec<TransactionOutputV10>,
    ) -> TransactionDocumentV10Builder<'a> {
        TransactionDocumentV10Builder {
            currency: "duniter_unit_test_currency",
            blockstamp,
            locktime: &0,
            issuers,
            inputs,
            unlocks,
            outputs,
            comment: "test",
            hash: None,
        }
    }

    #[test]
    fn test_tx_valid() {
        let TransactionDocument::V10(tx) = gen_mock_tx_doc();
        assert_eq!(Ok(()), local_verify_tx_doc_v10(10, &tx));
    }

    #[test]
    fn test_tx_empty_inputs() {
        let blockstamp = blockstamp();
        let issuers = issuers();
        let inputs = vec![];
        let unlocks = unlocks();
        let outputs = outputs();
        let tx_builder = tx_builder(&blockstamp, &issuers, &inputs, &unlocks, &outputs);
        let tx = tx_builder.build_with_signature(vec![sig1()]);

        let expected = Err(TransactionDocumentError::MissingInput);
        let actual = local_verify_tx_doc_v10(10, &tx);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_tx_too_long() {
        let blockstamp = blockstamp();
        let issuers = issuers();
        let inputs = vec![input1(); 100];
        let unlocks = unlocks();
        let outputs = outputs();
        let tx_builder = tx_builder(&blockstamp, &issuers, &inputs, &unlocks, &outputs);
        let tx = tx_builder.build_with_signature(vec![sig1()]);

        let expected = Err(TransactionDocumentError::TooLong {
            expected_max_length: 100,
            actual_length: 107,
        });
        let actual = local_verify_tx_doc_v10(10, &tx);
        assert_eq!(expected, actual);
    }

    /*#[test]
    fn test_tx_invalid_sig() {
        let blockstamp = blockstamp();
        let issuers = issuers();
        let inputs = vec![input1(); 2];
        let unlocks = unlocks();
        let outputs = outputs();
        let tx_builder = tx_builder(&blockstamp, &issuers, &inputs, &unlocks, &outputs);
        let tx = tx_builder.build_with_signature(vec![sig1()]);

        let expected = Err(TransactionDocumentError::TxSignatureError(
            DocumentSigsErr::Invalid(maplit::hashmap![
                0 => SigError::InvalidSig,
            ]),
        ));
        let actual = local_verify_tx_doc(&tx);
        assert_eq!(expected, actual);
    }*/
}
