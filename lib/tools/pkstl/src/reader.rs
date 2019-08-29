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

//! Define PKSTL reader.

use crate::constants::*;
use crate::encryption::{decrypt, EncryptAlgoWithSecretKey};
use crate::errors::IncomingMsgErr;
use crate::message::MsgTypeHeaders;
use crate::signature::SIG_ALGO_ED25519;
use crate::{Error, Result};
use std::io::{BufWriter, Write};

const MAGIC_VALUE_END: usize = 4;
const VERSION_END: usize = 8;
pub(crate) const ENCAPSULED_MSG_BEGIN: usize = 16;

#[derive(Debug, PartialEq)]
pub(crate) struct DecryptedIncomingDatas {
    pub(crate) datas: Vec<u8>,
    pub(crate) user_msg_begin: usize,
    pub(crate) user_msg_end: usize,
    pub(crate) msg_type_headers: MsgTypeHeaders,
}

/// Read incoming datas
pub(crate) fn read(
    encrypt_algo_with_secret_opt: Option<&EncryptAlgoWithSecretKey>,
    incoming_datas: &[u8],
    check_encrypt_state: bool,
) -> std::result::Result<DecryptedIncomingDatas, Error> {
    // Decrypt datas
    let datas_encrypted;
    let mut buffer = BufWriter::new(Vec::with_capacity(incoming_datas.len()));
    if incoming_datas[..MAGIC_VALUE_END] == MAGIC_VALUE {
        // Datas are not encrypted
        datas_encrypted = false;
        buffer.write(incoming_datas).map_err(Error::WriteError)?;
    } else {
        // Datas are encrypted
        datas_encrypted = true;
        if let Some(encrypt_algo_with_secret) = encrypt_algo_with_secret_opt {
            decrypt(incoming_datas, encrypt_algo_with_secret, &mut buffer)?;
        } else {
            return Err(Error::RecvInvalidMsg(IncomingMsgErr::UnexpectedMessage));
        }
    }
    let decrypted_datas = buffer.into_inner().map_err(|_| Error::BufferFlushError)?;

    // Check magic value
    if decrypted_datas[..MAGIC_VALUE_END] != MAGIC_VALUE {
        return Err(IncomingMsgErr::InvalidMagicValue.into());
    }

    // Check version
    if decrypted_datas[MAGIC_VALUE_END..VERSION_END] != CURRENT_VERSION {
        return Err(IncomingMsgErr::UnsupportedVersion.into());
    }

    // Read ENCAPSULED_MSG_SIZE
    let mut buffer_8_bytes: [u8; 8] = <[u8; 8]>::default();
    buffer_8_bytes.copy_from_slice(&decrypted_datas[VERSION_END..ENCAPSULED_MSG_BEGIN]);
    let encapsuled_msg_size = u64::from_be_bytes(buffer_8_bytes) as usize;
    let user_msg_end = ENCAPSULED_MSG_BEGIN + encapsuled_msg_size;

    // Read type headers
    let (msg_type_headers, type_headers_len) =
        read_type_headers(&decrypted_datas[ENCAPSULED_MSG_BEGIN..])?;

    if check_encrypt_state && datas_encrypted != msg_type_headers.must_be_encrypted() {
        Err(Error::RecvInvalidMsg(
            IncomingMsgErr::UnexpectedEncryptionState,
        ))
    } else {
        Ok(DecryptedIncomingDatas {
            datas: decrypted_datas,
            user_msg_begin: ENCAPSULED_MSG_BEGIN + type_headers_len,
            user_msg_end,
            msg_type_headers,
        })
    }
}

fn read_type_headers(type_headers: &[u8]) -> Result<(MsgTypeHeaders, usize)> {
    // Match message type
    match &type_headers[..MSG_TYPE_LEN] {
        USER_MSG_TYPE => Ok((MsgTypeHeaders::UserMsg, MSG_TYPE_LEN)),
        CONNECT_MSG_TYPE => {
            // Read PEER_EPHEMERAL_PUBKEY
            let mut peer_ephemeral_pk = [0u8; EPK_SIZE];
            peer_ephemeral_pk.copy_from_slice(&type_headers[MSG_TYPE_LEN..MSG_TYPE_LEN + EPK_SIZE]);
            // Read SIG_ALGO and SIG_PUBKEY
            match &type_headers[(MSG_TYPE_LEN + EPK_SIZE)..(SIG_PUBKEY_BEGIN)] {
                SIG_ALGO_ED25519 => {
                    let mut sig_algo = [0u8; SIG_ALGO_LEN];
                    sig_algo.copy_from_slice(
                        &type_headers
                            [(MSG_TYPE_LEN + EPK_SIZE)..(MSG_TYPE_LEN + EPK_SIZE + SIG_ALGO_LEN)],
                    );
                    Ok((
                        MsgTypeHeaders::Connect {
                            peer_ephemeral_pk,
                            sig_algo,
                            sig_pubkey: type_headers[SIG_PUBKEY_BEGIN..(SIG_PUBKEY_BEGIN + 32)]
                                .to_vec(),
                        },
                        SIG_PUBKEY_BEGIN + 32,
                    ))
                }
                _ => Err(IncomingMsgErr::UnsupportedSigAlgo.into()),
            }
        }
        ACK_MSG_TYPE => {
            let mut challenge = [0u8; CHALLENGE_SIZE];
            challenge
                .copy_from_slice(&&type_headers[MSG_TYPE_LEN..(MSG_TYPE_LEN + CHALLENGE_SIZE)]);
            Ok((
                MsgTypeHeaders::Ack { challenge },
                MSG_TYPE_LEN + CHALLENGE_SIZE,
            ))
        }
        _ => Err(IncomingMsgErr::UnknownMessageType.into()),
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::digest::sha256;
    use crate::encryption::{encrypt, tests::gen_random_encrypt_algo_with_secret};
    use crate::signature::{SIG_ALGO_ED25519, SIG_ALGO_ED25519_ARRAY};
    use pretty_assertions::assert_eq;
    use std::io::BufReader;

    #[test]
    fn test_unexpected_user_msg() {
        let fake_encrypted_incoming_datas = &[0, 0, 0, 0];
        let result = read(None, fake_encrypted_incoming_datas, true);
        if let Err(Error::RecvInvalidMsg(e)) = result {
            assert_eq!(IncomingMsgErr::UnexpectedMessage, e);
        } else {
            panic!("unexpected result")
        }
    }

    #[test]
    fn test_msg_with_unsupported_version() {
        let mut fake_incoming_datas = MAGIC_VALUE.to_vec();
        fake_incoming_datas.append(&mut vec![0, 0, 0, 2]);

        let result = read(None, &fake_incoming_datas, true);
        if let Err(Error::RecvInvalidMsg(e)) = result {
            assert_eq!(IncomingMsgErr::UnsupportedVersion, e);
        } else {
            panic!("unexpected result")
        }
    }

    #[test]
    fn test_unencrypted_usr_msg() {
        let mut empty_user_msg = MAGIC_VALUE.to_vec();
        empty_user_msg.append(&mut CURRENT_VERSION.to_vec());
        empty_user_msg.append(&mut vec![0, 0, 0, 0, 0, 0, 0, 2]); // ENCAPSULED_MSG_SIZE
        empty_user_msg.append(&mut USER_MSG_TYPE.to_vec());

        let result = read(None, &empty_user_msg, true);
        if let Err(Error::RecvInvalidMsg(e)) = result {
            assert_eq!(IncomingMsgErr::UnexpectedEncryptionState, e);
        } else {
            panic!("unexpected result")
        }
    }

    #[test]
    fn test_user_msg_with_wrong_magiv_value() -> Result<()> {
        let wrong_magic_value = vec![0, 0, 0, 0];
        let encrypt_algo_with_secret = gen_random_encrypt_algo_with_secret();
        let mut encrypted_datas = BufWriter::new(Vec::new());

        encrypt(
            &mut BufReader::new(&wrong_magic_value[..]),
            &encrypt_algo_with_secret,
            &mut encrypted_datas,
        )?;
        let encrypted_incoming_datas = encrypted_datas.into_inner().expect("buffer flush error");

        let result = read(
            Some(&encrypt_algo_with_secret),
            &encrypted_incoming_datas[..],
            true,
        );
        if let Err(Error::RecvInvalidMsg(e)) = result {
            assert_eq!(IncomingMsgErr::InvalidMagicValue, e);
        } else {
            panic!("unexpected result")
        }
        Ok(())
    }

    #[test]
    fn test_read() -> Result<()> {
        // Crate fake keys
        let fake_ephem_pk = &[0u8; 32][..];
        let fake_sig_pk = [0u8; 32].to_vec();
        let _fake_signature_opt = Some(&[0u8; 32][..]);

        // Create fake challenge
        let mut fake_challenge = [0u8; 32];
        fake_challenge.copy_from_slice(sha256(fake_ephem_pk).as_ref());

        // Create encrypt_algo_with_secret
        let encrypt_algo_with_secret = gen_random_encrypt_algo_with_secret();

        /////////////////////
        // CONNECT MSG
        /////////////////////

        let mut incoming_datas = Vec::with_capacity(100);
        incoming_datas.append(&mut MAGIC_VALUE.to_vec());
        incoming_datas.append(&mut CURRENT_VERSION.to_vec());
        incoming_datas.append(&mut 74u64.to_be_bytes().to_vec()); // Encapsuled message length
        incoming_datas.append(&mut vec![0, 1]); // CONNECT type
        incoming_datas.append(&mut fake_ephem_pk.to_vec()); // EPK
        incoming_datas.append(&mut SIG_ALGO_ED25519.to_vec()); // SIG_ALGO
        incoming_datas.append(&mut fake_sig_pk.clone()); // SIG_PK
        incoming_datas.append(&mut vec![5, 4, 4, 5]); // User custom datas
        incoming_datas.append(&mut [0u8; 32].to_vec()); // fake sig
        assert_eq!(
            DecryptedIncomingDatas {
                datas: incoming_datas.clone(),
                user_msg_begin: 86,
                user_msg_end: 90,
                msg_type_headers: MsgTypeHeaders::Connect {
                    peer_ephemeral_pk: [0u8; EPK_SIZE],
                    sig_algo: SIG_ALGO_ED25519_ARRAY,
                    sig_pubkey: fake_sig_pk,
                }
            },
            read(Some(&encrypt_algo_with_secret), &incoming_datas[..], true)?,
        );

        /////////////////////
        // ACK MSG
        /////////////////////

        // Read incoming ack message without custom datas
        let mut incoming_datas = Vec::with_capacity(100);
        incoming_datas.append(&mut MAGIC_VALUE.to_vec());
        incoming_datas.append(&mut CURRENT_VERSION.to_vec());
        incoming_datas.append(&mut 34u64.to_be_bytes().to_vec()); // Encapsuled message length
        incoming_datas.append(&mut vec![0, 2]); // ACK type
        incoming_datas.append(&mut fake_challenge.to_vec()); // ACK challenge
        incoming_datas.append(&mut [0u8; 32].to_vec()); // fake sig
        assert_eq!(
            DecryptedIncomingDatas {
                datas: incoming_datas.clone(),
                user_msg_begin: 50,
                user_msg_end: 50,
                msg_type_headers: MsgTypeHeaders::Ack {
                    challenge: fake_challenge,
                }
            },
            read(Some(&encrypt_algo_with_secret), &incoming_datas[..], true)?,
        );

        Ok(())
    }

    #[test]
    fn test_read_user_type_headers() -> Result<()> {
        let type_headers = vec![0, 0]; // USER_MSG_TYPE

        let expected = (MsgTypeHeaders::UserMsg, 2);

        assert_eq!(expected, read_type_headers(&type_headers[..])?);

        Ok(())
    }

    #[test]
    fn test_read_unknown_type_headers() {
        let type_headers = vec![1, 0]; // Unknown type

        let result = read_type_headers(&type_headers[..]);

        if let Err(Error::RecvInvalidMsg(e)) = read_type_headers(&type_headers[..]) {
            assert_eq!(e, IncomingMsgErr::UnknownMessageType);
        } else {
            println!("{:?}", result);
            panic!("Unexpected result");
        }
    }

    #[test]
    fn test_read_ack_type_headers() -> Result<()> {
        let challenge = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8,
            9, 0, 1,
        ];
        let mut type_headers = vec![
            0, 2, // ACK_MSG_TYPE
        ];
        type_headers.append(&mut challenge.to_vec());

        let expected = (MsgTypeHeaders::Ack { challenge }, 34);

        assert_eq!(expected, read_type_headers(&type_headers[..])?);

        Ok(())
    }

    #[test]
    fn test_read_connect_type_headers() -> Result<()> {
        let type_headers = vec![
            0, 1, // CONNECT_MSG_TYPE
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8,
            9, 0, 1, // EPK (32 bytes)
            0, 0, 0, 0, // SIG_ALGO_ED25519
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8,
            9, 0, 1, // Sig pubkey (32 bytes)
        ];

        let expected = (
            MsgTypeHeaders::Connect {
                peer_ephemeral_pk: [
                    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5,
                    6, 7, 8, 9, 0, 1,
                ],
                sig_algo: [0, 0, 0, 0],
                sig_pubkey: type_headers[38..].to_vec(),
            },
            70,
        );

        assert_eq!(expected, read_type_headers(&type_headers[..])?);

        Ok(())
    }

    #[test]
    fn test_read_connect_type_headers_with_unsupported_sig_algo() {
        let type_headers = vec![
            0, 1, // CONNECT_MSG_TYPE
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8,
            9, 0, 1, // EPK (32 bytes)
            0, 0, 0, 1, // Unsupported sig algo
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8,
            9, 0, 1, // Sig pubkey (32 bytes)
        ];

        let result = read_type_headers(&type_headers[..]);

        if let Err(Error::RecvInvalidMsg(e)) = read_type_headers(&type_headers[..]) {
            assert_eq!(e, IncomingMsgErr::UnsupportedSigAlgo);
        } else {
            println!("{:?}", result);
            panic!("Unexpected result");
        }
    }
}
