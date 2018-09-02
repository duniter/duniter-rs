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

//use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use duniter_crypto::hashs::Hash;
use duniter_documents::{Blockstamp, ReadBytesBlockstampError};
use dup_binarizer::*;
//use std::io::Cursor;
//use std::mem;
use std::num::NonZeroU16;

#[derive(Clone, Debug, Eq, PartialEq)]
/// WS2Pv2OkMsg
pub struct WS2Pv2OkMsg {
    /// If this field is zero, it means that the remote node does not want to reveal its prefix (the prefix being necessarily greater than or equal to 1).
    pub prefix: Option<NonZeroU16>,
    /// WS2Pv2SyncTarget
    pub sync_target: Option<WS2Pv2SyncTarget>,
}

impl Default for WS2Pv2OkMsg {
    fn default() -> Self {
        WS2Pv2OkMsg {
            prefix: None,
            sync_target: None,
        }
    }
}

/// ReadWS2Pv2OkMsgError
#[derive(Debug)]
pub enum ReadWS2Pv2OkMsgError {
    /// TooShort
    TooShort(&'static str),
    /// IoError
    IoError(::std::io::Error),
    /// ReadWS2Pv2SyncTargetError
    ReadWS2Pv2SyncTargetError(ReadWS2Pv2SyncTargetError),
}

impl From<ReadWS2Pv2SyncTargetError> for ReadWS2Pv2OkMsgError {
    fn from(e: ReadWS2Pv2SyncTargetError) -> Self {
        ReadWS2Pv2OkMsgError::ReadWS2Pv2SyncTargetError(e)
    }
}

impl From<::std::io::Error> for ReadWS2Pv2OkMsgError {
    fn from(e: ::std::io::Error) -> Self {
        ReadWS2Pv2OkMsgError::IoError(e)
    }
}

impl BinMessage for WS2Pv2OkMsg {
    type ReadBytesError = ReadWS2Pv2OkMsgError;
    fn from_bytes(datas: &[u8]) -> Result<Self, Self::ReadBytesError> {
        match datas.len() {
            0 => Ok(WS2Pv2OkMsg::default()),
            1 => Err(ReadWS2Pv2OkMsgError::TooShort(
                "Size of WS2Pv2OkMsg cannot be 1",
            )),
            2 => Ok(WS2Pv2OkMsg {
                prefix: NonZeroU16::new(u16::read_u16_be(&datas)?),
                sync_target: None,
            }),
            _ => Ok(WS2Pv2OkMsg {
                prefix: NonZeroU16::new(u16::read_u16_be(&datas[0..2])?),
                sync_target: Some(WS2Pv2SyncTarget::from_bytes(&datas[2..])?),
            }),
        }
    }
    fn to_bytes_vector(&self) -> Vec<u8> {
        if let Some(ref sync_target) = self.sync_target {
            let sync_target_bytes = sync_target.to_bytes_vector();
            let mut ok_msg_bytes = Vec::with_capacity(2 + sync_target_bytes.len());
            if let Some(prefix) = self.prefix {
                u16::write_u16_be(&mut ok_msg_bytes, prefix.get())
                    .expect("Fail to write prefix in WS2Pv2OkMsg !");
            } else {
                u16::write_u16_be(&mut ok_msg_bytes, 0)
                    .expect("Fail to write prefix in WS2Pv2OkMsg !");;
            }
            ok_msg_bytes.extend(sync_target_bytes);
            ok_msg_bytes
        } else {
            if let Some(prefix) = self.prefix {
                let mut ok_msg_bytes = Vec::with_capacity(2);
                u16::write_u16_be(&mut ok_msg_bytes, prefix.get())
                    .expect("Fail to write prefix in WS2Pv2OkMsg !");;
                ok_msg_bytes
            } else {
                vec![]
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// WS2Pv2SyncTarget
pub struct WS2Pv2SyncTarget {
    /// Indicates the current blockstamp of the message sender node. This blockstamp will be the target to reach for the node being synchronized.
    pub target_blockstamp: Blockstamp,
    /// Hash table of the last block of each chunk. We do not need the block numbers, we know them. Here the remote node sends the hashs of all these chunk, which correspond to the current hashs of all the blocks having a number in 250 module 249, in ascending order.
    pub chunks_hash: Vec<Hash>,
}

/// ReadWS2Pv2SyncTargetError
#[derive(Debug)]
pub enum ReadWS2Pv2SyncTargetError {
    /// TooShort
    TooShort(String),
    /// ReadBytesBlockstampError
    ReadBytesBlockstampError(ReadBytesBlockstampError),
    /// IoError
    IoError(::std::io::Error),
}

impl From<ReadBytesBlockstampError> for ReadWS2Pv2SyncTargetError {
    fn from(e: ReadBytesBlockstampError) -> Self {
        ReadWS2Pv2SyncTargetError::ReadBytesBlockstampError(e)
    }
}

impl From<::std::io::Error> for ReadWS2Pv2SyncTargetError {
    fn from(e: ::std::io::Error) -> Self {
        ReadWS2Pv2SyncTargetError::IoError(e)
    }
}

impl BinMessage for WS2Pv2SyncTarget {
    type ReadBytesError = ReadWS2Pv2SyncTargetError;
    fn from_bytes(datas: &[u8]) -> Result<Self, Self::ReadBytesError> {
        // target_blockstamp
        let target_blockstamp = if datas.len() < (Blockstamp::SIZE_IN_BYTES + 2) {
            return Err(ReadWS2Pv2SyncTargetError::TooShort(String::from(
                "blockstamp",
            )));
        } else {
            Blockstamp::from_bytes(&datas[0..Blockstamp::SIZE_IN_BYTES])?
        };
        // chunks_hash_count
        let mut index = Blockstamp::SIZE_IN_BYTES + 2;
        let chunks_hash_count = u16::read_u16_be(&datas[index - 2..index])? as usize;
        let chunks_hash = if datas.len() < (index + (chunks_hash_count * Hash::SIZE_IN_BYTES)) {
            return Err(ReadWS2Pv2SyncTargetError::TooShort(String::from(
                "chunks_hash",
            )));
        } else {
            let mut chunks_hash = Vec::with_capacity(chunks_hash_count);
            for _ in 0..chunks_hash_count {
                let mut hash_bytes: [u8; 32] = [0u8; 32];
                hash_bytes.copy_from_slice(&datas[index..index + Hash::SIZE_IN_BYTES]);
                chunks_hash.push(Hash(hash_bytes));
                index += Hash::SIZE_IN_BYTES;
            }
            chunks_hash
        };
        Ok(WS2Pv2SyncTarget {
            target_blockstamp,
            chunks_hash,
        })
    }
    fn to_bytes_vector(&self) -> Vec<u8> {
        let chunks_hash_size = self.chunks_hash.len() * Hash::SIZE_IN_BYTES;
        let mut bytes = Vec::with_capacity(Blockstamp::SIZE_IN_BYTES + 2 + chunks_hash_size);
        bytes.extend(self.target_blockstamp.to_bytes_vector());
        u16::write_u16_be(&mut bytes, self.chunks_hash.len() as u16)
            .expect("Fail to write chunks_hash_count in WS2Pv2SyncTarget !");
        for hash in &self.chunks_hash {
            bytes.extend_from_slice(&hash.0);
        }
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;
    use duniter_documents::Blockstamp;
    use std::num::NonZeroU16;

    fn keypair1() -> ed25519::KeyPair {
        ed25519::KeyPairFromSaltedPasswordGenerator::with_default_parameters().generate(
            "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV".as_bytes(),
            "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV_".as_bytes(),
        )
    }

    #[test]
    fn test_ws2p_message_ok() {
        let keypair1 = keypair1();

        let ok_msg = WS2Pv2OkMsg {
            prefix: NonZeroU16::new(1),
            sync_target: Some(WS2Pv2SyncTarget {
                target_blockstamp: Blockstamp::from_string(
                    "500-000011BABEEE1020B1F6B2627E2BC1C35BCD24375E114349634404D2C266D84F",
                ).unwrap(),
                chunks_hash: vec![
                    Hash::from_hex(
                        "000007722B243094269E548F600BD34D73449F7578C05BD370A6D301D20B5F10",
                    ).unwrap(),
                    Hash::from_hex(
                        "0000095FD4C8EA96DE2844E3A4B62FD18761E9B4C13A74FAB716A4C81F438D91",
                    ).unwrap(),
                ],
            }),
        };
        let mut ws2p_message = WS2Pv2Message {
            currency_code: CurrencyName(String::from("g1")),
            ws2p_version: 2u16,
            issuer_node_id: NodeId(0),
            issuer_pubkey: PubKey::Ed25519(keypair1.public_key()),
            payload: WS2Pv2MessagePayload::Ok(ok_msg),
            message_hash: None,
            signature: None,
        };

        let sign_result = ws2p_message.sign(PrivKey::Ed25519(keypair1.private_key()));
        if let Ok(bin_msg) = sign_result {
            // Test binarization
            assert_eq!(ws2p_message.to_bytes_vector(), bin_msg);
            // Test sign
            assert_eq!(ws2p_message.verify(), Ok(()));
            // Test debinarization
            let debinarization_result = WS2Pv2Message::from_bytes(&bin_msg);
            if let Ok(ws2p_message2) = debinarization_result {
                assert_eq!(ws2p_message, ws2p_message2);
            } else {
                panic!(
                    "Fail to debinarize ws2p_message : {:?}",
                    debinarization_result.err().unwrap()
                );
            }
        } else {
            panic!(
                "Fail to sign ws2p_message : {:?}",
                sign_result.err().unwrap()
            );
        }
    }
}
