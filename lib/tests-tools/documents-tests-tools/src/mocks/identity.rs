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

//! Mocks for projects use dubp-documents

use dubp_documents::documents::identity::*;
use dubp_documents::*;
use dup_crypto::hashs::Hash;
use dup_crypto::keys::PubKey;

/// Generate mock identity document
pub fn gen_mock_idty(pubkey: PubKey, created_on: BlockId) -> IdentityDocument {
    let idty_builder = IdentityDocumentBuilder {
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
