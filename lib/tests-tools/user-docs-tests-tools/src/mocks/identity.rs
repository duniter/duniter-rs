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

//! Mocks for projects use dubp-user-docs

use dubp_common_doc::blockstamp::Blockstamp;
use dubp_common_doc::traits::DocumentBuilder;
use dubp_common_doc::{BlockHash, BlockNumber};
use dubp_user_docs::documents::identity::v10::IdentityDocumentV10Builder;
use dubp_user_docs::documents::identity::*;
use dup_crypto::hashs::Hash;
use dup_crypto::keys::PubKey;

/// Generate mock identity document
pub fn gen_mock_idty(pubkey: PubKey, created_on: BlockNumber) -> IdentityDocumentV10 {
    let idty_builder = IdentityDocumentV10Builder {
        currency: "",
        username: "",
        blockstamp: &Blockstamp {
            id: created_on,
            hash: BlockHash(Hash::default()),
        },
        issuer: &pubkey,
    };
    idty_builder.build_with_signature(vec![])
}
