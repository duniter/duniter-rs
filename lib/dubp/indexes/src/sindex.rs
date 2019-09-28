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

//! Provides the definition of the source index (SINDEX) described in the DUBP RFC.

pub mod v11;

use dubp_common_doc::BlockNumber;
use dubp_user_docs::documents::transaction::OutputIndex;
use dup_crypto::hashs::Hash;
use dup_crypto::keys::PubKey;
use serde::{Deserialize, Serialize};

const UTXO_ID_SIZE: usize = 36;

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
/// Unique identifier for Unused tx output v10
pub struct UniqueIdUTXOv10(pub Hash, pub OutputIndex);

impl Into<Vec<u8>> for UniqueIdUTXOv10 {
    fn into(self) -> Vec<u8> {
        let mut bytes = [0u8; UTXO_ID_SIZE];

        bytes[..Hash::SIZE_IN_BYTES].copy_from_slice(&(self.0).0[..]);
        bytes[Hash::SIZE_IN_BYTES..UTXO_ID_SIZE]
            .copy_from_slice(&((self.1).0 as u32).to_be_bytes()[..]);

        bytes.to_vec()
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
/// Index of a V10 source
pub enum SourceUniqueIdV10 {
    /// unused Transaction Output
    UTXO(UniqueIdUTXOv10),
    /// universal Dividend
    UD(PubKey, BlockNumber),
}
