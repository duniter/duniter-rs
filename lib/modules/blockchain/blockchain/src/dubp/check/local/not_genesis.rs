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

//! Verifies if a normal block verifies the local rules specific to normal blocks

use dubp_block_doc::BlockDocument;
use dubp_currency_params::CurrencyParameters;

#[derive(Debug, Clone, PartialEq)]
/// Local verification error specific to a not-genesis Block
pub enum LocalVerifyNotGenesisBlockError {
    /// Is Genesis block
    IsGenesisBlock,
    /// No previous hash is provided
    MissingPreviousHash,
    /// No previous issuer is provided
    MissingPreviousIssuer,
    /// Some Paramters are provided in the Block Document
    UnexpectedParameters,
    /// Block Time
    BlockTime {
        actual_block_time: u64,
        lower_block_time_bound: u64,
        upper_block_time_bound: u64,
    },
}

/// Local verification of rules specific of a not genesis block
pub fn local_validation_not_genesis_block(
    block: &BlockDocument,
    currency_parameters: CurrencyParameters,
) -> Result<(), LocalVerifyNotGenesisBlockError> {
    let BlockDocument::V10(block) = block;

    // Provided block is a genesis block
    if block.number.0 == 0 {
        return Err(LocalVerifyNotGenesisBlockError::IsGenesisBlock);
    }

    // PreviousHash
    block
        .previous_hash
        .ok_or(LocalVerifyNotGenesisBlockError::MissingPreviousHash)?;

    // PreviousIssuer
    block
        .previous_issuer
        .ok_or(LocalVerifyNotGenesisBlockError::MissingPreviousIssuer)?;

    // Parameters
    if block.parameters.is_some() {
        return Err(LocalVerifyNotGenesisBlockError::UnexpectedParameters);
    }

    // Dates
    let avg_gen_time = currency_parameters.avg_gen_time as f64;
    let max_gen_time = (avg_gen_time * 1.189).ceil();
    let median_time_blocks = currency_parameters.median_time_blocks as f64;
    let max_acceleration = max_gen_time * median_time_blocks;
    let max_acceleration = max_acceleration.ceil() as u64;
    if block.time < block.median_time || block.median_time + max_acceleration < block.time {
        return Err(LocalVerifyNotGenesisBlockError::BlockTime {
            actual_block_time: block.time,
            lower_block_time_bound: block.median_time,
            upper_block_time_bound: block.median_time + max_acceleration,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dubp_block_doc::BlockDocument;
    use dubp_blocks_tests_tools::mocks::block_params::gen_mock_currency_parameters;
    use dubp_blocks_tests_tools::mocks::gen_mock_normal_block_v10;
    use dubp_common_doc::BlockNumber;

    #[test]
    fn local_verify_normal_block() -> Result<(), LocalVerifyNotGenesisBlockError> {
        let currency_params = gen_mock_currency_parameters();

        // Normal Rules : IsGenesisBlock
        let mut block = gen_mock_normal_block_v10();
        block.number = BlockNumber(0);

        let expected = Err(LocalVerifyNotGenesisBlockError::IsGenesisBlock);
        let actual =
            local_validation_not_genesis_block(&BlockDocument::V10(block), currency_params);
        assert_eq!(expected, actual);

        // Normal Rules : Missing Previous Hash
        let mut block = gen_mock_normal_block_v10();
        block.previous_hash = None;

        let expected = Err(LocalVerifyNotGenesisBlockError::MissingPreviousHash);
        let actual =
            local_validation_not_genesis_block(&BlockDocument::V10(block), currency_params);
        assert_eq!(expected, actual);

        // Normal Rules : Missing Previous Hash
        let mut block = gen_mock_normal_block_v10();
        block.previous_hash = None;

        let expected = Err(LocalVerifyNotGenesisBlockError::MissingPreviousHash);
        let actual =
            local_validation_not_genesis_block(&BlockDocument::V10(block), currency_params);
        assert_eq!(expected, actual);

        // Normal Rules : Missing Previous Issuer
        let mut block = gen_mock_normal_block_v10();
        block.previous_issuer = None;

        let expected = Err(LocalVerifyNotGenesisBlockError::MissingPreviousIssuer);
        let actual =
            local_validation_not_genesis_block(&BlockDocument::V10(block), currency_params);
        assert_eq!(expected, actual);

        // Normal Rules : Unexpected Parameters
        let mut block = gen_mock_normal_block_v10();
        block.parameters =
            Some(dubp_currency_params::genesis_block_params::v10::BlockV10Parameters::default());

        let expected = Err(LocalVerifyNotGenesisBlockError::UnexpectedParameters);
        let actual =
            local_validation_not_genesis_block(&BlockDocument::V10(block), currency_params);
        assert_eq!(expected, actual);

        // Normal Rules : Block Time
        let mut block = gen_mock_normal_block_v10();
        block.time = 685_861;

        let expected = Err(LocalVerifyNotGenesisBlockError::BlockTime {
            actual_block_time: 685_861,
            lower_block_time_bound: 1_522_683_184,
            upper_block_time_bound: 1_522_695_084,
        });
        let actual =
            local_validation_not_genesis_block(&BlockDocument::V10(block), currency_params);
        assert_eq!(expected, actual);

        Ok(())
    }
}
