//  Copyright (C) 2018  The Duniter Project Developers.
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

//! Defined all aspects of the inter-node network that concern all modules and are therefore independent of one implementation or another of this network layer.

use duniter_crypto::keys::*;
use u16::*;

/// Total size of Ed25519 PubkeyBox
pub static ED25519_PUBKEY_BOX_SIZE: &'static usize = &33;

/// ReadPubkeyBoxError
#[derive(Debug)]
pub enum ReadPubkeyBoxError {
    /// IoError
    IoError(::std::io::Error),
    /// TooShort
    TooShort(),
    /// TooLong
    TooLong(),
    /// AlgoNotSupported
    AlgoNotSupported(),
}

impl From<::std::io::Error> for ReadPubkeyBoxError {
    fn from(e: ::std::io::Error) -> Self {
        ReadPubkeyBoxError::IoError(e)
    }
}

/// WritePubkeyBoxError
#[derive(Debug)]
pub enum WritePubkeyBoxError {
    /// IoError
    IoError(::std::io::Error),
    /// AlgoNotSupported
    AlgoNotSupported(),
}

impl From<::std::io::Error> for WritePubkeyBoxError {
    fn from(e: ::std::io::Error) -> Self {
        WritePubkeyBoxError::IoError(e)
    }
}

/// Read pubkey_box
pub fn read_pubkey_box(datas: &[u8]) -> Result<(PubKey, u8), ReadPubkeyBoxError> {
    if !datas.is_empty() {
        match datas[0] {
            0x00 => {
                if datas.len() < *ED25519_PUBKEY_BOX_SIZE {
                    Err(ReadPubkeyBoxError::TooShort())
                } else if datas.len() > *ED25519_PUBKEY_BOX_SIZE {
                    Err(ReadPubkeyBoxError::TooLong())
                } else {
                    let mut pubkey_bytes: [u8; 32] = [0u8; 32];
                    pubkey_bytes.copy_from_slice(&datas[1..(*ED25519_PUBKEY_BOX_SIZE)]);
                    Ok((PubKey::Ed25519(ed25519::PublicKey(pubkey_bytes)), 0x00))
                }
            }
            _ => Err(ReadPubkeyBoxError::AlgoNotSupported()),
        }
    } else {
        Err(ReadPubkeyBoxError::TooShort())
    }
}

/// Write pubkey_box
pub fn write_pubkey_box(buffer: &mut Vec<u8>, pubkey: PubKey) -> Result<(), WritePubkeyBoxError> {
    match pubkey {
        PubKey::Ed25519(ed25519_pubkey) => {
            write_u16_be(buffer, *ED25519_PUBKEY_BOX_SIZE as u16)?;
            buffer.push(0x00);
            buffer.extend_from_slice(&ed25519_pubkey.0);
            Ok(())
        }
        _ => Err(WritePubkeyBoxError::AlgoNotSupported()),
    }
}
