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

/// Total size of Ed25519 SigBox
pub static ED25519_SIG_BOX_SIZE: &'static usize = &64;

/// ReadSigBoxError
#[derive(Debug)]
pub enum ReadSigBoxError {
    /// IoError
    IoError(::std::io::Error),
    /// TooShort
    TooShort(),
    /// TooLong
    TooLong(),
    /// AlgoNotSupported
    AlgoNotSupported(),
}

impl From<::std::io::Error> for ReadSigBoxError {
    fn from(e: ::std::io::Error) -> Self {
        ReadSigBoxError::IoError(e)
    }
}

/// WriteSigBoxError
#[derive(Debug)]
pub enum WriteSigBoxError {
    /// IoError
    IoError(::std::io::Error),
    /// AlgoNotSupported
    AlgoNotSupported(),
}

impl From<::std::io::Error> for WriteSigBoxError {
    fn from(e: ::std::io::Error) -> Self {
        WriteSigBoxError::IoError(e)
    }
}

/// Read sig_box
pub fn read_sig_box(datas: &[u8], algo: u8) -> Result<Sig, ReadSigBoxError> {
    if !datas.is_empty() {
        match algo {
            0x00 => {
                if datas.len() < *ED25519_SIG_BOX_SIZE {
                    Err(ReadSigBoxError::TooShort())
                } else if datas.len() > *ED25519_SIG_BOX_SIZE {
                    Err(ReadSigBoxError::TooLong())
                } else {
                    let mut sig_bytes: [u8; 64] = [0u8; 64];
                    sig_bytes.copy_from_slice(&datas[0..(*ED25519_SIG_BOX_SIZE)]);
                    Ok(Sig::Ed25519(ed25519::Signature(sig_bytes)))
                }
            }
            _ => Err(ReadSigBoxError::AlgoNotSupported()),
        }
    } else {
        Err(ReadSigBoxError::TooShort())
    }
}

/// Write sig_box
pub fn write_sig_box(buffer: &mut Vec<u8>, sig: Sig) -> Result<(), WriteSigBoxError> {
    match sig {
        Sig::Ed25519(ed25519_sig) => {
            write_u16_be(buffer, *ED25519_SIG_BOX_SIZE as u16)?;
            buffer.extend_from_slice(&ed25519_sig.0);
            Ok(())
        }
        _ => Err(WriteSigBoxError::AlgoNotSupported()),
    }
}
