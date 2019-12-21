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

//! Wrappers around Block document.

pub mod v10;

use dubp_common_doc::blockstamp::Blockstamp;
use dubp_common_doc::traits::{Document, ToStringObject};
use dubp_common_doc::{BlockHash, BlockNumber};
use dup_crypto::hashs::Hash;
use dup_crypto::keys::{PubKey, PublicKey, SignatorEnum};

pub use v10::{BlockDocumentV10, BlockDocumentV10Stringified};

/// Wrap a Block document.
///
/// Must be created by parsing a text document or using a builder.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum BlockDocument {
    V10(BlockDocumentV10),
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Error when verifying a hash of a block
pub enum VerifyBlockHashError {
    /// The hash is missing
    MissingHash { block_number: BlockNumber },
    /// Hash is invalid
    InvalidHash {
        block_number: BlockNumber,
        expected_hash: Hash,
        actual_hash: Hash,
    },
}

pub trait BlockDocumentTrait {
    /// Common time in block (also known as 'blockchain time')
    fn common_time(&self) -> u64;
    /// Compute hash
    fn compute_hash(&self) -> BlockHash;
    /// Compute inner hash
    fn compute_inner_hash(&self) -> Hash {
        Hash::compute_str(&self.generate_compact_inner_text())
    }
    /// Compute the character string that will be hashed
    fn compute_will_hashed_string(&self) -> String;
    /// Compute the character string that will be signed
    fn compute_will_signed_string(&self) -> String;
    /// Get current frame size (in blocks)
    fn current_frame_size(&self) -> usize;
    /// Generate compact inner text (for compute inner_hash)
    fn generate_compact_inner_text(&self) -> String;
    /// Compute hash and save it in document
    fn generate_hash(&mut self);
    /// Compute inner hash and save it in document
    fn generate_inner_hash(&mut self);
    /// Get block hash
    fn hash(&self) -> Option<BlockHash>;
    /// Increment nonce
    fn increment_nonce(&mut self);
    /// Get block inner hash
    fn inner_hash(&self) -> Option<Hash>;
    /// Get number of compute members in the current frame
    fn issuers_count(&self) -> usize;
    /// Get number of members in wot
    fn members_count(&self) -> usize;
    /// Get block number
    fn number(&self) -> BlockNumber;
    /// Get common difficulty (PoW)
    fn pow_min(&self) -> usize;
    /// Get previous hash
    fn previous_hash(&self) -> Option<Hash>;
    /// Get previous blockstamp
    fn previous_blockstamp(&self) -> Blockstamp;
    /// Lightens the block (for example to store it while minimizing the space required)
    fn reduce(&mut self);
    /// Verify inner hash
    fn verify_inner_hash(&self) -> Result<(), VerifyBlockHashError>;
    /// Verify block hash
    fn verify_hash(&self) -> Result<(), VerifyBlockHashError>;
    /// Sign block
    fn sign(&mut self, signator: &SignatorEnum);
}

impl BlockDocumentTrait for BlockDocument {
    #[inline]
    fn compute_hash(&self) -> BlockHash {
        match self {
            BlockDocument::V10(block) => block.compute_hash(),
        }
    }
    #[inline]
    fn compute_will_hashed_string(&self) -> String {
        match self {
            BlockDocument::V10(block) => block.compute_will_hashed_string(),
        }
    }
    #[inline]
    fn compute_will_signed_string(&self) -> String {
        match self {
            BlockDocument::V10(block) => block.compute_will_signed_string(),
        }
    }
    #[inline]
    fn current_frame_size(&self) -> usize {
        match self {
            BlockDocument::V10(block) => block.current_frame_size(),
        }
    }
    #[inline]
    fn generate_compact_inner_text(&self) -> String {
        match self {
            BlockDocument::V10(block) => block.generate_compact_inner_text(),
        }
    }
    #[inline]
    fn generate_hash(&mut self) {
        match self {
            BlockDocument::V10(block) => block.generate_hash(),
        }
    }
    #[inline]
    fn generate_inner_hash(&mut self) {
        match self {
            BlockDocument::V10(block) => block.generate_inner_hash(),
        }
    }
    #[inline]
    fn hash(&self) -> Option<BlockHash> {
        match self {
            BlockDocument::V10(block) => block.hash(),
        }
    }
    #[inline]
    fn increment_nonce(&mut self) {
        match self {
            BlockDocument::V10(block) => block.increment_nonce(),
        }
    }
    #[inline]
    fn inner_hash(&self) -> Option<Hash> {
        match self {
            BlockDocument::V10(block) => block.inner_hash(),
        }
    }
    #[inline]
    fn issuers_count(&self) -> usize {
        match self {
            BlockDocument::V10(block) => block.issuers_count(),
        }
    }
    #[inline]
    fn members_count(&self) -> usize {
        match self {
            BlockDocument::V10(block) => block.members_count(),
        }
    }
    #[inline]
    fn common_time(&self) -> u64 {
        match self {
            BlockDocument::V10(block) => block.common_time(),
        }
    }
    #[inline]
    fn number(&self) -> BlockNumber {
        match self {
            BlockDocument::V10(block) => block.number(),
        }
    }
    #[inline]
    fn pow_min(&self) -> usize {
        match self {
            BlockDocument::V10(block) => block.pow_min(),
        }
    }
    #[inline]
    fn previous_blockstamp(&self) -> Blockstamp {
        match self {
            BlockDocument::V10(block) => block.previous_blockstamp(),
        }
    }
    #[inline]
    fn previous_hash(&self) -> Option<Hash> {
        match self {
            BlockDocument::V10(block) => block.previous_hash(),
        }
    }
    #[inline]
    fn reduce(&mut self) {
        match self {
            BlockDocument::V10(block) => block.reduce(),
        }
    }
    #[inline]
    fn verify_inner_hash(&self) -> Result<(), VerifyBlockHashError> {
        match self {
            BlockDocument::V10(block) => block.verify_inner_hash(),
        }
    }
    #[inline]
    fn verify_hash(&self) -> Result<(), VerifyBlockHashError> {
        match self {
            BlockDocument::V10(block) => block.verify_hash(),
        }
    }
    #[inline]
    fn sign(&mut self, signator: &SignatorEnum) {
        match self {
            BlockDocument::V10(block) => block.sign(signator),
        }
    }
}

impl Document for BlockDocument {
    type PublicKey = PubKey;

    fn version(&self) -> u16 {
        match self {
            BlockDocument::V10(block_v10_12) => block_v10_12.version(),
        }
    }

    fn currency(&self) -> &str {
        match self {
            BlockDocument::V10(block) => block.currency(),
        }
    }

    fn blockstamp(&self) -> Blockstamp {
        match self {
            BlockDocument::V10(block) => block.blockstamp(),
        }
    }

    fn issuers(&self) -> &Vec<Self::PublicKey> {
        match self {
            BlockDocument::V10(block) => block.issuers(),
        }
    }

    fn signatures(&self) -> &Vec<<Self::PublicKey as PublicKey>::Signature> {
        match self {
            BlockDocument::V10(block) => block.signatures(),
        }
    }

    fn as_bytes(&self) -> &[u8] {
        match self {
            BlockDocument::V10(block) => block.as_bytes(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum BlockDocumentStringified {
    V10(BlockDocumentV10Stringified),
}

impl ToStringObject for BlockDocument {
    type StringObject = BlockDocumentStringified;

    fn to_string_object(&self) -> Self::StringObject {
        match self {
            BlockDocument::V10(block) => BlockDocumentStringified::V10(block.to_string_object()),
        }
    }
}
