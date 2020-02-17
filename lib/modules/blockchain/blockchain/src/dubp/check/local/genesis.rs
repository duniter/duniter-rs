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

//! Verifies if a genesis block verifies the local rules specific to genesis blocks

use dubp_block_doc::block::v10::BlockDocumentV10;
use dubp_block_doc::BlockDocument;
use dubp_common_doc::BlockNumber;
use durs_common_tools::traits::bool_ext::BoolExt;

/// Local verification of errors specific to a Genesis Block
#[derive(Debug, PartialEq)]
pub enum LocalVerifyGenesisBlockError {
    /// Block number is not zero
    NonZeroBlockNumber { block_number: BlockNumber },
    /// A dividend is provided
    UnexpectedDividend,
    /// A previous hash is provided
    UnexpectedPreviousHash,
    /// A previous issuer is provided
    UnexpectedPreviousIssuer,
    /// No paramters are provided
    MissingParameters,
    /// Unit base is not zero
    NonZeroUnitBase { unit_base: usize },
    /// The block time is different from the median time
    TimeError { block_time: u64, median_time: u64 },
}

/// Verifies local rules specific to a genesis block
pub fn local_validation_genesis_block(
    block: &BlockDocument,
) -> Result<(), LocalVerifyGenesisBlockError> {
    match block {
        BlockDocument::V10(block) => local_validation_genesis_block_v10(block),
    }
}

fn local_validation_genesis_block_v10(
    block: &BlockDocumentV10,
) -> Result<(), LocalVerifyGenesisBlockError> {
    // block_number must be equal to zero
    (block.number == BlockNumber(0)).or_err(LocalVerifyGenesisBlockError::NonZeroBlockNumber {
        block_number: block.number,
    })?;

    // Dividend must be none
    block
        .dividend
        .is_none()
        .or_err(LocalVerifyGenesisBlockError::UnexpectedDividend)?;

    // Previous Hash must be none
    block
        .previous_hash
        .is_none()
        .or_err(LocalVerifyGenesisBlockError::UnexpectedPreviousHash)?;

    // Previous issuer must be none
    block
        .previous_issuer
        .is_none()
        .or_err(LocalVerifyGenesisBlockError::UnexpectedPreviousIssuer)?;

    // Parameters
    block
        .parameters
        .ok_or(LocalVerifyGenesisBlockError::MissingParameters)?;

    // unit_base must be equal to zero
    (usize::from(block.unit_base) == 0).or_err(LocalVerifyGenesisBlockError::NonZeroUnitBase {
        unit_base: block.unit_base.into(),
    })?;

    // time must be equal to median_time
    (block.time == block.median_time).or_err(LocalVerifyGenesisBlockError::TimeError {
        block_time: block.time,
        median_time: block.median_time,
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dubp_block_doc::BlockDocument;
    use dubp_blocks_tests_tools::mocks::gen_mock_genesis_block_v10;
    use dup_crypto::hashs::Hash;
    use dup_crypto::keys::*;
    use durs_common_tools::UsizeSer32;

    #[test]
    fn test_genesis_block_valid() {
        let block = gen_mock_genesis_block_v10();
        assert_eq!(
            Ok(()),
            local_validation_genesis_block(&BlockDocument::V10(block))
        );
    }

    #[test]
    fn test_genesis_block_wrong_number() {
        let mut block = gen_mock_genesis_block_v10();
        block.number = BlockNumber(1);

        let expected = Err(LocalVerifyGenesisBlockError::NonZeroBlockNumber {
            block_number: BlockNumber(1),
        });
        let actual = local_validation_genesis_block(&BlockDocument::V10(block));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_genesis_block_unexpected_dividend() {
        let mut block = gen_mock_genesis_block_v10();
        block.dividend = Some(UsizeSer32(10));

        let expected = Err(LocalVerifyGenesisBlockError::UnexpectedDividend);
        let actual = local_validation_genesis_block(&BlockDocument::V10(block));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_genesis_block_unexpected_previous_hash() {
        let mut block = gen_mock_genesis_block_v10();
        block.previous_hash = Some(
            Hash::from_hex("000001144968D0C3516BE6225E4662F182E28956AF46DD7FB228E3D0F9413FEB")
                .expect("invalid hash"),
        );

        let expected = Err(LocalVerifyGenesisBlockError::UnexpectedPreviousHash);
        let actual = local_validation_genesis_block(&BlockDocument::V10(block));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_genesis_block_unexpected_previous_issuer() {
        let mut block = gen_mock_genesis_block_v10();
        block.previous_issuer = Some(PubKey::Ed25519(
            ed25519::PublicKey::from_base58("D3krfq6J9AmfpKnS3gQVYoy7NzGCc61vokteTS8LJ4YH")
                .expect("invalid pubkey"),
        ));

        let expected = Err(LocalVerifyGenesisBlockError::UnexpectedPreviousIssuer);
        let actual = local_validation_genesis_block(&BlockDocument::V10(block));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_genesis_block_missing_parameters() {
        let mut block = gen_mock_genesis_block_v10();
        block.parameters = None;

        let expected = Err(LocalVerifyGenesisBlockError::MissingParameters);
        let actual = local_validation_genesis_block(&BlockDocument::V10(block));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_genesis_block_non_zero_unit_base() {
        let mut block = gen_mock_genesis_block_v10();
        block.unit_base = UsizeSer32(3);

        let expected = Err(LocalVerifyGenesisBlockError::NonZeroUnitBase { unit_base: 3 });
        let actual = local_validation_genesis_block(&BlockDocument::V10(block));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_genesis_block_ntime_error() {
        let mut block = gen_mock_genesis_block_v10();
        block.time = 3;
        block.median_time = 4;

        let expected = Err(LocalVerifyGenesisBlockError::TimeError {
            block_time: 3,
            median_time: 4,
        });
        let actual = local_validation_genesis_block(&BlockDocument::V10(block));
        assert_eq!(expected, actual);
    }
}
