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

//! Manage minimal secure and decentralized transport layer.

use crate::agreement::{EphemeralKeyPair, EphemeralPublicKey};
use crate::config::SecureLayerConfig;
use crate::constants::*;
use crate::digest::sha256;
use crate::encryption::{encrypt, EncryptAlgoWithSecretKey};
use crate::errors::IncomingMsgErr;
use crate::message::{EncapsuledMessage, Message, MessageRef, MsgTypeHeaders};
use crate::reader::{self, DecryptedIncomingDatas};
use crate::signature::{self, SIG_ALGO_ED25519_ARRAY};
use crate::status::SecureLayerStatus;
use crate::{Action, ActionSideEffects, Error, MsgType, Result};
use std::io::{BufReader, BufWriter, Write};

/// Minimal secure layer
#[derive(Debug)]
pub struct MinimalSecureLayer {
    ack_msg_recv_too_early: Option<Vec<u8>>,
    cloned: bool,
    pub(crate) config: SecureLayerConfig,
    pub(crate) encrypt_algo_with_secret: Option<EncryptAlgoWithSecretKey>,
    ephemeral_kp: Option<EphemeralKeyPair>,
    pub(crate) ephemeral_pubkey: EphemeralPublicKey,
    peer_epk: Option<Vec<u8>>,
    peer_sig_pubkey: Option<Vec<u8>>,
    pub(crate) status: SecureLayerStatus,
    tmp_stack_user_msgs: Vec<Vec<u8>>,
}

impl MinimalSecureLayer {
    /// Try to clone, The negotiation must have been successful
    pub fn try_clone(&mut self) -> Result<Self> {
        if self.status == SecureLayerStatus::NegotiationSuccessful {
            self.cloned = true;
            Ok(MinimalSecureLayer {
                ack_msg_recv_too_early: None,
                cloned: true,
                config: self.config,
                encrypt_algo_with_secret: self.encrypt_algo_with_secret.clone(),
                ephemeral_kp: None,
                ephemeral_pubkey: self.ephemeral_pubkey.clone(),
                peer_epk: None,
                peer_sig_pubkey: None,
                status: SecureLayerStatus::NegotiationSuccessful,
                tmp_stack_user_msgs: self.tmp_stack_user_msgs.clone(),
            })
        } else {
            Err(Error::NegoMustHaveBeenSuccessful)
        }
    }
    /// Change configuration
    pub fn change_config(&mut self, new_config: SecureLayerConfig) -> Result<()> {
        if !self.cloned {
            self.config = new_config;
            Ok(())
        } else {
            Err(Error::ForbidChangeConfAfterClone)
        }
    }
    /// Create minimal secure layer
    pub fn create(
        config: SecureLayerConfig,
        expected_remote_sig_public_key: Option<Vec<u8>>,
    ) -> Result<Self> {
        let ephemeral_kp = EphemeralKeyPair::generate()?;
        let ephemeral_pubkey = ephemeral_kp.public_key().clone();

        let secure_layer = MinimalSecureLayer {
            ack_msg_recv_too_early: None,
            cloned: false,
            config,
            encrypt_algo_with_secret: None,
            ephemeral_pubkey,
            ephemeral_kp: Some(ephemeral_kp),
            peer_epk: None,
            peer_sig_pubkey: expected_remote_sig_public_key,
            status: SecureLayerStatus::init(),
            tmp_stack_user_msgs: Vec::new(),
        };

        Ok(secure_layer)
    }
    pub(crate) fn compute_shared_secret(&mut self, peer_ephemeral_public_key: &[u8]) -> Result<()> {
        let encrypt_algo = self.config.encrypt_algo;
        let ephemeral_kp = self.ephemeral_kp.take();
        if let Some(ephemeral_kp) = ephemeral_kp {
            let shared_secret = ephemeral_kp.compute_shared_secret(
                peer_ephemeral_public_key,
                encrypt_algo.shared_secret_len(),
            )?;

            self.encrypt_algo_with_secret =
                Some(EncryptAlgoWithSecretKey::build(encrypt_algo, shared_secret));

            Ok(())
        } else if self.encrypt_algo_with_secret.is_some() {
            // Shared secret already computed, do nothing
            Ok(())
        } else {
            unreachable!("dev error: fisrt call of compute_shared_secret() without ephemeral_kp !")
        }
    }
    /// Drain temporary stack of remote messages
    pub fn drain_tmp_stack_user_msgs(&mut self) -> Result<Vec<Message>> {
        let bin_msgs: Vec<Vec<u8>> = self.tmp_stack_user_msgs.drain(..).collect();
        let mut msgs = Vec::with_capacity(bin_msgs.len());
        for bin_msg in bin_msgs {
            if let Some(msg) = self.read_inner(&bin_msg, false)? {
                msgs.push(msg);
            }
        }
        Ok(msgs)
    }
    #[inline]
    /// Encapsulate message
    fn encapsulate_message(&mut self, message: &MessageRef) -> Result<EncapsuledMessage> {
        message.to_bytes(&self.ephemeral_pubkey.as_ref(), self.peer_epk.as_ref())
    }
    /// Take ACK message received too early
    #[inline]
    pub fn take_ack_msg_recv_too_early(&mut self) -> Result<Option<Message>> {
        if let Some(bin_ack_msg) = self.ack_msg_recv_too_early.take() {
            self.read(&bin_ack_msg)
        } else {
            Ok(None)
        }
    }
    #[inline]
    /// Read incoming datas
    pub fn read(&mut self, incoming_datas: &[u8]) -> Result<Option<Message>> {
        self.read_inner(incoming_datas, true)
    }
    fn read_inner(
        &mut self,
        incoming_datas: &[u8],
        check_encrypt_state: bool,
    ) -> Result<Option<Message>> {
        // Decrypt incoming messsage and parse headers
        let DecryptedIncomingDatas {
            mut datas,
            user_msg_begin,
            user_msg_end,
            msg_type_headers,
        } = match reader::read(
            self.encrypt_algo_with_secret.as_ref(),
            incoming_datas,
            check_encrypt_state,
        ) {
            Ok(decrypted_incoming_datas) => decrypted_incoming_datas,
            Err(e) => {
                self.status = SecureLayerStatus::Fail;
                return Err(e);
            }
        };

        //println!("DEBUG TMP: msg_type_headers={:#?}", msg_type_headers);
        match msg_type_headers {
            MsgTypeHeaders::Connect {
                peer_ephemeral_pk,
                ref sig_pubkey,
                ..
            } => {
                // Verify (or get) peer sig pubkey
                if let Some(ref peer_sig_pubkey) = self.peer_sig_pubkey {
                    if sig_pubkey != peer_sig_pubkey {
                        return Err(Error::UnexpectedRemoteSigPubKey);
                    }
                } else {
                    self.peer_sig_pubkey = Some(sig_pubkey.to_vec());
                }

                // Verify sig
                // The reader has already made sure that the signature algorithm is supported,
                // as we only support the Ed25519 algorithm, we know that it is necessarily this one.
                if !self.verify_sig(&datas, sig_pubkey, user_msg_end) {
                    return Err(IncomingMsgErr::InvalidHashOrSig.into());
                }

                // Update status
                self.status
                    .apply_action(Action::Receive(MsgType::Connect))?;

                // Get peeer EPK and compute shared secret
                self.peer_epk = Some(peer_ephemeral_pk.to_vec());
                self.compute_shared_secret(&peer_ephemeral_pk[..])?;
            }
            MsgTypeHeaders::Ack { challenge } => {
                // Verify challenge
                if challenge != sha256(self.ephemeral_pubkey.as_ref()).as_ref() {
                    return Err(IncomingMsgErr::InvalidChallenge.into());
                }

                let peer_sig_pubkey = if let Some(ref peer_sig_pubkey) = self.peer_sig_pubkey {
                    peer_sig_pubkey
                } else if self.ack_msg_recv_too_early.is_none() {
                    self.ack_msg_recv_too_early = Some(incoming_datas.to_vec());
                    return Ok(None);
                } else {
                    self.status = SecureLayerStatus::Fail;
                    return Err(IncomingMsgErr::UnexpectedAckMsg.into());
                };

                // Verify sig
                // The reader has already made sure that the signature algorithm is supported,
                // as we only support the Ed25519 algorithm, we know that it is necessarily this one.
                if !self.verify_sig(&datas, peer_sig_pubkey, user_msg_end) {
                    return Err(IncomingMsgErr::InvalidHashOrSig.into());
                }

                // Update status
                self.status.apply_action(Action::Receive(MsgType::Ack))?;
            }
            MsgTypeHeaders::UserMsg => {
                // Verify status
                if let Some(ActionSideEffects::PushUserMsgIntoTmpStack) = self
                    .status
                    .apply_action(Action::Receive(MsgType::UserMsg))?
                {
                    self.tmp_stack_user_msgs.push(datas);
                    return Ok(None);
                }

                // Verify hash
                let datas_hashed = &datas[..user_msg_end];
                let hash = &datas[user_msg_end..];
                if hash != sha256(datas_hashed).as_ref() {
                    return Err(IncomingMsgErr::InvalidHashOrSig.into());
                }
            }
        }

        // Get message
        let message = Message::from_bytes(
            datas.drain(user_msg_begin..user_msg_end).collect(),
            msg_type_headers,
        )?;

        Ok(Some(message))
    }
    /// Encrypt and write message on a writer
    #[inline]
    fn encrypt_and_write<W: Write>(
        &mut self,
        encapsuled_message: &EncapsuledMessage,
        writer: &mut BufWriter<W>,
    ) -> Result<()> {
        let encrypt_algo_with_secret =
            if let Some(ref encrypt_algo_with_secret) = self.encrypt_algo_with_secret {
                encrypt_algo_with_secret
            } else {
                panic!("Dev error: try to get encrypt_algo_with_secret before it's computed !")
            };

        let mut datas_will_encrypted = BufWriter::new(Vec::with_capacity(
            encapsuled_message.as_ref().len() + HASH_SIZE,
        ));

        // Write encapsuled message
        datas_will_encrypted
            .write(encapsuled_message.as_ref())
            .map_err(Error::WriteError)?;
        // Write encapsuled message hash
        datas_will_encrypted
            .write(sha256(encapsuled_message.as_ref()).as_ref())
            .map_err(Error::WriteError)?;

        // Flush datas_will_encrypted buffer
        let datas_will_encrypted = datas_will_encrypted
            .into_inner()
            .map_err(|_| Error::BufferFlushError)?;

        // Encrypt
        encrypt(
            &mut BufReader::new(&datas_will_encrypted[..]),
            encrypt_algo_with_secret,
            writer,
        )?;

        Ok(())
    }
    #[inline]
    /// Create connect message
    pub fn create_connect_message(
        &mut self,
        public_key: &[u8],
        custom_datas: Option<&[u8]>,
    ) -> Result<Vec<u8>> {
        // Update status
        self.status.apply_action(Action::Create(MsgType::Connect))?;

        // Create message and update status
        match self.encapsulate_message(&MessageRef::Connect {
            sig_algo: SIG_ALGO_ED25519_ARRAY,
            sig_pubkey: public_key.to_vec(),
            custom_datas,
        }) {
            Ok(encapsuled_msg) => Ok(encapsuled_msg.datas),
            Err(e) => {
                self.status = SecureLayerStatus::Fail;
                Err(e)
            }
        }
    }
    #[inline]
    /// Create ack message
    pub fn create_ack_message(&mut self, custom_datas: Option<&[u8]>) -> Result<Vec<u8>> {
        // Update status
        self.status.apply_action(Action::Create(MsgType::Ack))?;

        // Create message and update status
        match self.encapsulate_message(&MessageRef::Ack { custom_datas }) {
            Ok(encapsuled_msg) => Ok(encapsuled_msg.datas),
            Err(e) => {
                self.status = SecureLayerStatus::Fail;
                Err(e)
            }
        }
    }
    #[inline]
    /// Write message
    pub fn write_message<W: Write>(
        &mut self,
        datas: &[u8],
        writer: &mut BufWriter<W>,
    ) -> Result<()> {
        // Update status
        self.status.apply_action(Action::Create(MsgType::UserMsg))?;

        match self.encapsuled_and_encrypt_and_write_message(datas, writer) {
            Ok(()) => {
                self.status = SecureLayerStatus::NegotiationSuccessful;
                Ok(())
            }
            Err(e) => {
                self.status = SecureLayerStatus::Fail;
                Err(e)
            }
        }
    }
    #[inline]
    fn encapsuled_and_encrypt_and_write_message<W: Write>(
        &mut self,
        datas: &[u8],
        writer: &mut BufWriter<W>,
    ) -> Result<()> {
        let encapsuled_msg = self.encapsulate_message(&MessageRef::Message {
            custom_datas: Some(datas),
        })?;
        self.encrypt_and_write(&encapsuled_msg, writer)
    }
    #[inline]
    fn verify_sig(&self, datas: &[u8], sig_pubkey: &[u8], user_msg_end: usize) -> bool {
        let datas_signed = &datas[..user_msg_end];
        let sig = &datas[user_msg_end..];
        signature::verify_sig(sig_pubkey, datas_signed, sig)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::encryption::EncryptAlgo;
    use crate::signature::SIG_ALGO_ED25519;
    use crate::Seed32;
    use ring::signature::{Ed25519KeyPair, KeyPair};

    fn create_connect_msg_bytes(mut epk: Vec<u8>, sig_kp: &Ed25519KeyPair) -> Result<Vec<u8>> {
        let mut incoming_datas = Vec::with_capacity(100);
        incoming_datas.append(&mut MAGIC_VALUE.to_vec());
        incoming_datas.append(&mut CURRENT_VERSION.to_vec());
        incoming_datas.append(&mut 74u64.to_be_bytes().to_vec()); // Encapsuled message length
        incoming_datas.append(&mut vec![0, 1]); // CONNECT type
        incoming_datas.append(&mut epk); // EPK
        incoming_datas.append(&mut SIG_ALGO_ED25519.to_vec()); // SIG_ALGO
        incoming_datas.append(&mut sig_kp.public_key().as_ref().to_vec()); // SIG_PK
        incoming_datas.append(&mut vec![5, 4, 4, 5]); // User custom datas
        let sig = sig_kp.sign(&incoming_datas);
        incoming_datas.append(&mut sig.as_ref().to_vec()); // SIG
        Ok(incoming_datas)
    }

    fn create_ack_msg_bytes(remote_epk: Vec<u8>, sig_kp: &Ed25519KeyPair) -> Result<Vec<u8>> {
        let mut incoming_datas = Vec::with_capacity(100);
        incoming_datas.append(&mut MAGIC_VALUE.to_vec());
        incoming_datas.append(&mut CURRENT_VERSION.to_vec());
        incoming_datas.append(&mut 34u64.to_be_bytes().to_vec()); // Encapsuled message length
        incoming_datas.append(&mut vec![0, 2]); // ACK type
        incoming_datas.append(&mut sha256(&remote_epk).as_ref().to_vec()); // Challenge
        let sig = sig_kp.sign(&incoming_datas);
        incoming_datas.append(&mut sig.as_ref().to_vec()); // SIG
        Ok(incoming_datas)
    }

    #[test]
    fn test_change_config() -> Result<()> {
        let mut msl = MinimalSecureLayer::create(SecureLayerConfig::default(), None)?;
        msl.change_config(SecureLayerConfig {
            encrypt_algo: EncryptAlgo::Chacha20Poly1305Aead,
            ..SecureLayerConfig::default()
        })
        .expect("change config must be success");
        Ok(())
    }

    #[test]
    fn test_compute_shared_secret_twice() -> Result<()> {
        let mut msl1 = MinimalSecureLayer::create(SecureLayerConfig::default(), None)?;
        let msl2 = MinimalSecureLayer::create(SecureLayerConfig::default(), None)?;

        msl1.compute_shared_secret(msl2.ephemeral_pubkey.as_ref())?;
        msl1.compute_shared_secret(msl2.ephemeral_pubkey.as_ref())?;
        Ok(())
    }

    #[test]
    fn test_status_update_to_fail() -> Result<()> {
        let mut msl1 = MinimalSecureLayer::create(SecureLayerConfig::default(), None)?;
        let fake_encrypted_incoming_datas = &[0, 0, 0, 0];
        let result = msl1.read(fake_encrypted_incoming_datas);

        assert_eq!(SecureLayerStatus::Fail, msl1.status);

        if let Err(Error::RecvInvalidMsg(e)) = result {
            assert_eq!(IncomingMsgErr::UnexpectedMessage, e);
        } else {
            panic!("unexpected result")
        }
        Ok(())
    }

    #[test]
    fn test_ack_msg_with_wrong_challenge() -> Result<()> {
        // Create ack message
        let mut incoming_datas = Vec::with_capacity(100);
        incoming_datas.append(&mut MAGIC_VALUE.to_vec());
        incoming_datas.append(&mut CURRENT_VERSION.to_vec());
        incoming_datas.append(&mut 34u64.to_be_bytes().to_vec()); // Encapsuled message length
        incoming_datas.append(&mut vec![0, 2]); // ACK type
        incoming_datas.append(&mut [0u8; 32].to_vec()); // fake challenge
        incoming_datas.append(&mut [0u8; 32].to_vec()); // fake sig

        // Create secure layer
        let mut msl1 = MinimalSecureLayer::create(SecureLayerConfig::default(), None)?;

        // Read ack msg
        let result = msl1.read(&incoming_datas[..]);
        if let Err(Error::RecvInvalidMsg(e)) = result {
            assert_eq!(IncomingMsgErr::InvalidChallenge, e);
            Ok(())
        } else {
            println!("unexpected result={:?}", result);
            panic!();
        }
    }

    #[test]
    fn test_write_user_msg_before_nego() -> Result<()> {
        // Create secure layer
        let mut msl1 = MinimalSecureLayer::create(SecureLayerConfig::default(), None)?;

        // Try to create ack message before connect message
        let result = msl1.write_message(&[], &mut BufWriter::new(Vec::new()));
        if let Err(Error::NegoMustHaveBeenSuccessful) = result {
            Ok(())
        } else {
            println!("unexpected result={:?}", result);
            panic!();
        }
    }

    #[test]
    fn test_create_ack_msg_before_connect() -> Result<()> {
        // Create secure layer
        let mut msl1 = MinimalSecureLayer::create(SecureLayerConfig::default(), None)?;

        // Try to create ack message before connect message
        let result = msl1.create_ack_message(None);
        if let Err(Error::ForbidWriteAckMsgNow) = result {
            Ok(())
        } else {
            println!("unexpected result={:?}", result);
            panic!();
        }
    }

    #[test]
    fn test_connect_msg_twice() -> Result<()> {
        // Create sig keypair
        let sig_kp = Ed25519KeyPair::from_seed_unchecked(Seed32::random().as_ref())
            .map_err(|_| Error::FailtoGenSigKeyPair)?;

        // Create secure layer
        let mut msl1 = MinimalSecureLayer::create(SecureLayerConfig::default(), None)?;

        let _ = msl1.create_connect_message(sig_kp.public_key().as_ref(), None)?;

        // Try to create connect message twice
        let result = msl1.create_connect_message(sig_kp.public_key().as_ref(), None);
        if let Err(Error::ConnectMsgAlreadyWritten) = result {
            Ok(())
        } else {
            println!("unexpected result={:?}", result);
            panic!();
        }
    }

    #[test]
    fn test_connect_msg_with_wrong_sig() -> Result<()> {
        // Crate fake keys
        let fake_ephem_pk = &[0u8; 32][..];
        let fake_sig_pk = [0u8; 32].to_vec();
        let _fake_signature_opt = Some(&[0u8; 32][..]);

        // Create connect msg bytes
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

        // Create secure layer
        let mut msl1 = MinimalSecureLayer::create(SecureLayerConfig::default(), None)?;

        // Read connect msg
        let result = msl1.read(&incoming_datas[..]);
        if let Err(Error::RecvInvalidMsg(e)) = result {
            assert_eq!(IncomingMsgErr::InvalidHashOrSig, e);
            Ok(())
        } else {
            println!("unexpected result={:?}", result);
            panic!();
        }
    }

    #[test]
    fn test_recv_connect_msg_twice() -> Result<()> {
        // Create sig keypair
        let sig_kp = Ed25519KeyPair::from_seed_unchecked(Seed32::random().as_ref())
            .map_err(|_| Error::FailtoGenSigKeyPair)?;

        // Create EKP
        let ephemeral_kp = EphemeralKeyPair::generate()?;

        // Create connect msg bytes
        let incoming_datas =
            create_connect_msg_bytes(ephemeral_kp.public_key().as_ref().to_vec(), &sig_kp)?;

        // Create secure layer
        let mut msl1 = MinimalSecureLayer::create(SecureLayerConfig::default(), None)?;

        // Read connect message
        let _ = msl1.read(&incoming_datas[..])?;

        // Reread same connect message
        let result = msl1.read(&incoming_datas[..]);
        if let Err(Error::RecvInvalidMsg(IncomingMsgErr::UnexpectedConnectMsg)) = result {
            Ok(())
        } else {
            println!("unexpected result={:?}", result);
            panic!();
        }
    }

    #[test]
    fn test_recv_ack_msg_early_twice() -> Result<()> {
        // Create sig keypair
        let sig_kp = Ed25519KeyPair::from_seed_unchecked(Seed32::random().as_ref())
            .map_err(|_| Error::FailtoGenSigKeyPair)?;

        // Create secure layer
        let mut msl1 = MinimalSecureLayer::create(SecureLayerConfig::default(), None)?;

        // Create ack msg bytes
        let incoming_datas =
            create_ack_msg_bytes(msl1.ephemeral_pubkey.as_ref().to_vec(), &sig_kp)?;

        // Read ack message received too early
        let _ = msl1.read(&incoming_datas[..]);
        // Re-read ack message received too early
        let result = msl1.read(&incoming_datas[..]);
        if let Err(Error::RecvInvalidMsg(IncomingMsgErr::UnexpectedAckMsg)) = result {
            Ok(())
        } else {
            println!("unexpected result={:?}", result);
            panic!();
        }
    }

    #[test]
    fn test_recv_user_msg_before_nego() -> Result<()> {
        // Create secure layer
        let mut msl1 = MinimalSecureLayer::create(SecureLayerConfig::default(), None)?;

        // Create empty user msg fakely encryted
        let mut incoming_datas = Vec::with_capacity(100);
        incoming_datas.append(&mut vec![0, 0, 0, 0]);
        incoming_datas.append(&mut CURRENT_VERSION.to_vec());
        incoming_datas.append(&mut 2u64.to_be_bytes().to_vec()); // Encapsuled message length
        incoming_datas.append(&mut vec![0, 0]); // USER_MSG_TYPE
        incoming_datas.append(&mut sha256(&incoming_datas).as_ref().to_vec()); // Hash

        // Read user message received before_nego
        let result = msl1.read(&incoming_datas[..]);
        if let Err(Error::RecvInvalidMsg(IncomingMsgErr::UnexpectedMessage)) = result {
            Ok(())
        } else {
            println!("unexpected result={:?}", result);
            panic!();
        }
    }
}
