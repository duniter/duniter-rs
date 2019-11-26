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

//! Sub-module checking if the content of a block is consistent.

pub mod genesis;
pub mod not_genesis;
pub mod tx_doc;

use self::genesis::LocalVerifyGenesisBlockError;
use self::not_genesis::LocalVerifyNotGenesisBlockError;
use self::tx_doc::TransactionDocumentError;

use dubp_block_doc::block::v10::BlockDocumentV10;
use dubp_block_doc::{block::BlockDocumentTrait, BlockDocument};
use dubp_common_doc::errors::DocumentSigsErr;
use dubp_common_doc::traits::Document;
use dubp_common_doc::BlockNumber;
use dubp_currency_params::CurrencyParameters;

#[derive(Debug, PartialEq)]
/// Local verification of a block error
pub enum LocalVerifyBlockError {
    /// Wrong block version
    Version {
        expected_version: Vec<usize>,
        actual_version: u32,
    },
    /// Genesis block specific rules
    LocalVerifyGenesisBlockError(LocalVerifyGenesisBlockError),
    /// Not-genesis block specific rules
    LocalVerifyNotGenesisBlockError(LocalVerifyNotGenesisBlockError),
    /// Signature error
    _BlockSignatureError(DocumentSigsErr),
    /// Identity signature error
    IdentitySignatureError(DocumentSigsErr),
    /// Joiner signature error
    JoinerSignatureError(DocumentSigsErr),
    /// Active signature error
    ActiveSignatureError(DocumentSigsErr),
    /// Leaver signature error
    LeaverSignatureError(DocumentSigsErr),
    /// Missing issuer
    MissingIssuer,
    /// Too many issuers (> 1)
    TooManyIssuers,
    /// Transaction Document Error
    TransactionDocumentError(TransactionDocumentError),
    /// Receive not genesis block wityhout blockchain
    RecvNotGenesisWithoutBlockchain,
}

impl From<LocalVerifyGenesisBlockError> for LocalVerifyBlockError {
    fn from(err: LocalVerifyGenesisBlockError) -> Self {
        Self::LocalVerifyGenesisBlockError(err)
    }
}

impl From<LocalVerifyNotGenesisBlockError> for LocalVerifyBlockError {
    fn from(err: LocalVerifyNotGenesisBlockError) -> Self {
        Self::LocalVerifyNotGenesisBlockError(err)
    }
}

impl From<TransactionDocumentError> for LocalVerifyBlockError {
    fn from(err: TransactionDocumentError) -> Self {
        Self::TransactionDocumentError(err)
    }
}

/// Local verification of a block document according to rules of RFC0009
pub fn verify_local_validity_block(
    block: &BlockDocument,
    currency_parameters_opt: Option<CurrencyParameters>,
) -> Result<(), LocalVerifyBlockError> {
    if block.number() == BlockNumber(0) {
        // Check the local rules specific to genesis blocks
        self::genesis::local_validation_genesis_block(block)?;
    } else if let Some(currency_parameters) = currency_parameters_opt {
        // Check the local rules specific to non-genesis blocks
        self::not_genesis::local_validation_not_genesis_block(block, currency_parameters)?;
    } else {
        return Err(LocalVerifyBlockError::RecvNotGenesisWithoutBlockchain);
    }

    match block {
        BlockDocument::V10(block) => verify_local_validity_block_v10(block),
    }
}

/// Local verification of a block document V10 according to rules of RFC0009
pub fn verify_local_validity_block_v10(
    block: &BlockDocumentV10,
) -> Result<(), LocalVerifyBlockError> {
    // Version
    if !(block.version == 10 || block.version == 11) {
        return Err(LocalVerifyBlockError::Version {
            expected_version: vec![10, 11],
            actual_version: block.version,
        });
    }

    // Issuers
    if block.issuers.is_empty() {
        return Err(LocalVerifyBlockError::MissingIssuer);
    } else if block.issuers.len() > 1 {
        return Err(LocalVerifyBlockError::TooManyIssuers);
    }

    // Check signatures of block and wot events
    // As it has been checked that block.issuers.len() == 1 and as
    // block.issuers.len() == block.signatures.len() is check in block.verify_signatures()
    // there is no need to check that block.signatures.len() == 1
    // check bloc sig tmp disabled because #183
    /*block
    .verify_signatures()
    .map_err(LocalVerifyBlockError::BlockSignatureError)?;*/
    for identity in &block.identities {
        identity
            .verify_signatures()
            .map_err(LocalVerifyBlockError::IdentitySignatureError)?;
    }
    for joiner in &block.joiners {
        joiner
            .verify_signatures()
            .map_err(LocalVerifyBlockError::JoinerSignatureError)?;
    }
    for active in &block.actives {
        active
            .verify_signatures()
            .map_err(LocalVerifyBlockError::ActiveSignatureError)?;
    }
    for leaver in &block.leavers {
        leaver
            .verify_signatures()
            .map_err(LocalVerifyBlockError::LeaverSignatureError)?;
    }

    // Check transactions
    for tx in &block.transactions {
        self::tx_doc::local_verify_tx_doc(tx)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use dubp_block_doc::BlockDocument;
    use dubp_blocks_tests_tools::mocks::block_params::gen_mock_currency_parameters;
    use dubp_blocks_tests_tools::mocks::gen_mock_normal_block_v10;

    #[test]
    fn test_verify_not_genesis_block_valid() {
        let currency_params = gen_mock_currency_parameters();
        let block = gen_mock_normal_block_v10();
        assert!(
            verify_local_validity_block(&BlockDocument::V10(block), Some(currency_params)).is_ok()
        );
    }

    #[test]
    fn test_verify_not_genesis_block_wrong_version() {
        let currency_params = gen_mock_currency_parameters();
        let mut block = gen_mock_normal_block_v10();
        block.version = 14;

        let expected = Err(LocalVerifyBlockError::Version {
            expected_version: vec![10, 11],
            actual_version: 14,
        });
        let actual = verify_local_validity_block(&BlockDocument::V10(block), Some(currency_params));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_verify_not_genesis_block_issuers_empty() {
        let currency_params = gen_mock_currency_parameters();
        let mut block = gen_mock_normal_block_v10();
        block.issuers.clear();

        let expected = Err(LocalVerifyBlockError::MissingIssuer);
        let actual = verify_local_validity_block(&BlockDocument::V10(block), Some(currency_params));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_verify_not_genesis_block_none_too_many_issuers() {
        let currency_params = gen_mock_currency_parameters();
        let mut block = gen_mock_normal_block_v10();
        block.issuers.push(block.issuers[0]);

        let expected = Err(LocalVerifyBlockError::TooManyIssuers);
        let actual = verify_local_validity_block(&BlockDocument::V10(block), Some(currency_params));
        assert_eq!(expected, actual);
    }
}
