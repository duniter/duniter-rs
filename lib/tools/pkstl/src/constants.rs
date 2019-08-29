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

//! Declare PKSTL constants.

/// Sig algo length
pub const SIG_ALGO_LEN: usize = 4;

/// Current version
pub(crate) const CURRENT_VERSION: [u8; 4] = [0, 0, 0, 1];

/// Challenge size
pub(crate) const CHALLENGE_SIZE: usize = 32;

/// Hash size
pub(crate) const HASH_SIZE: usize = 32;

/// Ephemeral public key size
pub(crate) const EPK_SIZE: usize = 32;

/// Magic value (at the beginning of all messages)
pub(crate) const MAGIC_VALUE: [u8; 4] = [0xE2, 0xC2, 0xE2, 0xD2];

/// Message type length
pub(crate) const MSG_TYPE_LEN: usize = 2;

/// User message type
pub(crate) const USER_MSG_TYPE: &[u8] = &[0, 0];

/// Connect message type
pub(crate) const CONNECT_MSG_TYPE: &[u8] = &[0, 1];

/// Ack message type
pub(crate) const ACK_MSG_TYPE: &[u8] = &[0, 2];

/// Sig pubkey begin
pub(crate) const SIG_PUBKEY_BEGIN: usize = MSG_TYPE_LEN + EPK_SIZE + SIG_ALGO_LEN;
