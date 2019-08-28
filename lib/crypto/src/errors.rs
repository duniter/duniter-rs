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

//! Manage cryptographic errors.

#[derive(Debug)]
/// Cryptographic error
pub enum CryptoError {
    /// Fail to compute agreement
    FailToComputeAgreement,
    /// Fail to decrypt datas
    FailToDecryptDatas(chacha20_poly1305_aead::DecryptError),
    /// Fail to encrypt datas
    FailToEncryptDatas(std::io::Error),
    /// Fail to generate ephemeral key pair
    FailToGenEphemerKeyPair,
    /// Fail to generate ephemeral public key
    FailToGenEphemerPubKey,
}
