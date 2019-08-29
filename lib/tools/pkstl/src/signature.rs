//  Copyright (C) 2019  Eloïs SANCHEZ.
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

//! Define PKSTL Signature.

use ring::signature::UnparsedPublicKey;

/// Signature algorithm Ed25519
pub const SIG_ALGO_ED25519: &[u8] = &[0, 0, 0, 0];

/// Signature algorithm Ed25519 array
pub const SIG_ALGO_ED25519_ARRAY: [u8; 4] = [0, 0, 0, 0];

pub(crate) fn verify_sig(pubkey: &[u8], message: &[u8], sig: &[u8]) -> bool {
    UnparsedPublicKey::new(&ring::signature::ED25519, pubkey)
        .verify(message, sig)
        .is_ok()
}
