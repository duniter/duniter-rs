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

//! Test Public Key Secure Transport Layer in minimal mode.

use pkstl::*;
use ring::signature::{Ed25519KeyPair, KeyPair};
use std::io::{BufWriter, Write};

trait AsOptRef {
    fn as_opt_ref(&self) -> Option<&[u8]>;
}

impl AsOptRef for Option<Vec<u8>> {
    fn as_opt_ref(&self) -> Option<&[u8]> {
        match self {
            Some(ref datas) => Some(&datas[..]),
            None => None,
        }
    }
}

fn client_infos(server_sig_kp: &[u8]) -> Result<(MinimalSecureLayer, Ed25519KeyPair)> {
    // Create client sig keypair
    let client_sig_kp = Ed25519KeyPair::from_seed_unchecked(Seed32::random().as_ref())
        .map_err(|_| Error::FailtoGenSigKeyPair)?;

    // Create client secure layer
    let client_msl =
        MinimalSecureLayer::create(SecureLayerConfig::default(), Some(server_sig_kp.to_vec()))?;

    Ok((client_msl, client_sig_kp))
}

fn server_infos() -> Result<(MinimalSecureLayer, Ed25519KeyPair)> {
    // Create server secure layer
    let server_msl = MinimalSecureLayer::create(SecureLayerConfig::default(), None)?;

    // Create server sig keypair
    let server_sig_kp = Ed25519KeyPair::from_seed_unchecked(Seed32::random().as_ref())
        .map_err(|_| Error::FailtoGenSigKeyPair)?;

    Ok((server_msl, server_sig_kp))
}

fn send_connect_msg_inner(
    sender_msl: &mut MinimalSecureLayer,
    sender_sig_kp: &Ed25519KeyPair,
    receiver_msl: &mut MinimalSecureLayer,
    custom_datas: Option<Vec<u8>>,
) -> Result<Option<Message>> {
    // Create and sign connect message
    let connect_msg = sender_msl.create_connect_message(
        sender_sig_kp.public_key().as_ref(),
        custom_datas.as_opt_ref(),
    )?;
    let sig = sender_sig_kp.sign(&connect_msg);

    // Write connect message and it's sig in channel
    let mut channel = BufWriter::new(Vec::with_capacity(1_000));
    channel
        .write(&connect_msg)
        .map_err(|_| Error::BufferFlushError)?;
    channel
        .write(sig.as_ref())
        .map_err(|_| Error::BufferFlushError)?;

    // Receiver read connect message from channel
    let channel = channel.into_inner().map_err(|_| Error::BufferFlushError)?;
    receiver_msl.read(&channel[..])
}

#[inline]
fn send_connect_msg(
    sender_msl: &mut MinimalSecureLayer,
    sender_sig_kp: &Ed25519KeyPair,
    receiver_msl: &mut MinimalSecureLayer,
    custom_datas: Option<Vec<u8>>,
) -> Result<()> {
    let connect_msg_received = send_connect_msg_inner(
        sender_msl,
        sender_sig_kp,
        receiver_msl,
        custom_datas.clone(),
    )?
    .expect("Must be receive a message");
    assert_eq!(
        Message::Connect {
            sig_algo: SIG_ALGO_ED25519_ARRAY,
            sig_pubkey: sender_sig_kp.public_key().as_ref().to_vec(),
            custom_datas,
        },
        connect_msg_received,
    );
    Ok(())
}

#[inline]
fn send_ack_msg(
    sender_msl: &mut MinimalSecureLayer,
    sender_sig_kp: &Ed25519KeyPair,
    receiver_msl: &mut MinimalSecureLayer,
    custom_datas: Option<Vec<u8>>,
) -> Result<()> {
    let msg_received = send_ack_msg_inner(
        sender_msl,
        sender_sig_kp,
        receiver_msl,
        custom_datas.as_opt_ref(),
    )?
    .expect("Must be receive a message");
    assert_eq!(Message::Ack { custom_datas }, msg_received,);
    Ok(())
}

fn send_ack_msg_inner<'a>(
    sender_msl: &mut MinimalSecureLayer,
    sender_sig_kp: &Ed25519KeyPair,
    receiver_msl: &'a mut MinimalSecureLayer,
    custom_datas: Option<&[u8]>,
) -> Result<Option<Message>> {
    // Create and sign server ack message
    let ack_msg = sender_msl.create_ack_message(custom_datas)?;
    let sig = sender_sig_kp.sign(&ack_msg);

    // Write server ack message and it's sig in channel
    let mut channel = BufWriter::new(Vec::with_capacity(1_000));
    channel
        .write(&ack_msg)
        .map_err(|_| Error::BufferFlushError)?;
    channel
        .write(sig.as_ref())
        .map_err(|_| Error::BufferFlushError)?;

    // Client read server ack message from channel
    let channel = channel.into_inner().map_err(|_| Error::BufferFlushError)?;
    receiver_msl.read(&channel[..])
}

#[inline]
fn send_user_msg(
    sender_msl: &mut MinimalSecureLayer,
    receiver_msl: &mut MinimalSecureLayer,
    datas: Vec<u8>,
) -> Result<()> {
    let msg_received = send_user_msg_inner(sender_msl, receiver_msl, datas.clone())?
        .expect("Must be receive a message");
    assert_eq!(
        Message::Message {
            custom_datas: Some(datas),
        },
        msg_received,
    );
    Ok(())
}

fn send_user_msg_inner(
    sender_msl: &mut MinimalSecureLayer,
    receiver_msl: &mut MinimalSecureLayer,
    datas: Vec<u8>,
) -> Result<Option<Message>> {
    // Client write one message in channel
    let mut channel = BufWriter::new(Vec::with_capacity(1_000));
    sender_msl.write_message(&datas, &mut channel)?;

    // Server read client message from channel
    let channel = channel.into_inner().map_err(|_| Error::BufferFlushError)?;
    receiver_msl.read(&channel[..])
}

#[test]
fn server_recv_ack_early() -> Result<()> {
    //////////////////////////
    // SERVER INFOS
    //////////////////////////

    let (mut server_msl, server_sig_kp) = server_infos()?;

    //////////////////////////
    // CLIENT INFOS
    //////////////////////////

    let (mut client_msl, client_sig_kp) = client_infos(server_sig_kp.public_key().as_ref())?;

    //////////////////////////
    // SERVER CONNECT MSG
    //////////////////////////

    send_connect_msg(
        &mut server_msl,
        &server_sig_kp,
        &mut client_msl,
        Some(vec![5, 1, 1, 5]),
    )?;

    //////////////////////////
    // CLIENT ACK MSG
    //////////////////////////

    assert_eq!(
        None,
        send_ack_msg_inner(
            &mut client_msl,
            &client_sig_kp,
            &mut server_msl,
            Some(&[7, 1, 1, 7]),
        )?
    );

    //////////////////////////
    // CLIENT CONNECT MSG
    //////////////////////////

    send_connect_msg(
        &mut client_msl,
        &client_sig_kp,
        &mut server_msl,
        Some(vec![5, 4, 4, 5]),
    )?;

    //////////////////////////
    // SERVER ACK MSG
    //////////////////////////

    send_ack_msg(
        &mut server_msl,
        &server_sig_kp,
        &mut client_msl,
        Some(vec![3, 1, 1, 3]),
    )?;

    /////////////////////////////////////////
    // RECEIVE CLIENT USER MSG TOO EARLY
    /////////////////////////////////////////

    assert_eq!(
        None,
        send_user_msg_inner(&mut client_msl, &mut server_msl, vec![5, 2, 2, 5],)?
    );

    //////////////////////////////////////////
    // GET CLIENT ACK MSG RECEIVED TOO EARLY
    //////////////////////////////////////////

    assert_eq!(
        Some(Message::Ack {
            custom_datas: Some(vec![7, 1, 1, 7]),
        }),
        server_msl.take_ack_msg_recv_too_early()?
    );

    assert_eq!(None, server_msl.take_ack_msg_recv_too_early()?);

    //////////////////////////////////////////
    // GET CLIENT USER MSG RECEIVED TOO EARLY
    //////////////////////////////////////////

    assert_eq!(
        vec![Message::Message {
            custom_datas: Some(vec![5, 2, 2, 5]),
        }],
        server_msl.drain_tmp_stack_user_msgs()?
    );

    assert_eq!(
        Vec::<Message>::with_capacity(0),
        server_msl.drain_tmp_stack_user_msgs()?
    );

    Ok(())
}

#[test]
fn disordered_passing_case() -> Result<()> {
    //////////////////////////
    // SERVER INFOS
    //////////////////////////

    let (mut server_msl, server_sig_kp) = server_infos()?;

    //////////////////////////
    // CLIENT INFOS
    //////////////////////////

    let (mut client_msl, client_sig_kp) = client_infos(server_sig_kp.public_key().as_ref())?;

    //////////////////////////
    // CLIENT CONNECT MSG
    //////////////////////////

    send_connect_msg(
        &mut client_msl,
        &client_sig_kp,
        &mut server_msl,
        Some(vec![5, 4, 4, 5]),
    )?;

    //////////////////////////
    // SERVER ACK MSG
    //////////////////////////

    send_ack_msg(
        &mut server_msl,
        &server_sig_kp,
        &mut client_msl,
        Some(vec![5, 8, 8, 5]),
    )?;

    //////////////////////////
    // SERVER CONNECT MSG
    //////////////////////////

    send_connect_msg(
        &mut server_msl,
        &server_sig_kp,
        &mut client_msl,
        Some(vec![5, 1, 1, 5]),
    )?;

    //////////////////////////
    // CLIENT ACK MSG
    //////////////////////////

    send_ack_msg(
        &mut client_msl,
        &client_sig_kp,
        &mut server_msl,
        Some(vec![7, 4, 4, 7]),
    )?;

    //////////////////////////
    // SERVER USER MSG
    //////////////////////////

    send_user_msg(&mut server_msl, &mut client_msl, vec![1, 5, 5, 1])?;

    Ok(())
}

#[test]
fn test_middle_man_detection() -> Result<()> {
    //////////////////////////
    // SERVER INFOS
    //////////////////////////

    let (_server_msl, server_sig_kp) = server_infos()?;

    //////////////////////////
    // MIDDLE MAN INFOS
    //////////////////////////

    let (mut middle_msl, middle_sig_kp) = server_infos()?;

    //////////////////////////
    // CLIENT INFOS
    //////////////////////////

    let (mut client_msl, client_sig_kp) = client_infos(server_sig_kp.public_key().as_ref())?;

    //////////////////////////
    // CLIENT CONNECT MSG
    //////////////////////////

    send_connect_msg(
        &mut client_msl,
        &client_sig_kp,
        &mut middle_msl,
        Some(vec![7, 6, 5, 4]),
    )?;

    //////////////////////////
    // MIDDLE MAN CONNECT MSG
    //////////////////////////

    let result = send_connect_msg(
        &mut middle_msl,
        &middle_sig_kp,
        &mut client_msl,
        Some(vec![7, 5, 6, 4]),
    );
    if let Err(Error::UnexpectedRemoteSigPubKey) = result {
        Ok(())
    } else {
        println!("unexpected result={:?}", result);
        panic!();
    }
}

#[test]
fn ordered_passing_case() -> Result<()> {
    //////////////////////////
    // SERVER INFOS
    //////////////////////////

    let (mut server_msl, server_sig_kp) = server_infos()?;

    //////////////////////////
    // CLIENT INFOS
    //////////////////////////

    let (mut client_msl, client_sig_kp) = client_infos(server_sig_kp.public_key().as_ref())?;

    //////////////////////////
    // CLIENT CONNECT MSG
    //////////////////////////

    send_connect_msg(
        &mut client_msl,
        &client_sig_kp,
        &mut server_msl,
        Some(vec![5, 4, 4, 5]),
    )?;

    //////////////////////////
    // SERVER CONNECT MSG
    //////////////////////////

    send_connect_msg(
        &mut server_msl,
        &server_sig_kp,
        &mut client_msl,
        Some(vec![5, 6, 6, 5]),
    )?;

    //////////////////////////
    // SERVER ACK MSG
    //////////////////////////

    send_ack_msg(
        &mut server_msl,
        &server_sig_kp,
        &mut client_msl,
        Some(vec![5, 8, 8, 5]),
    )?;

    //////////////////////////
    // CLIENT ACK MSG
    //////////////////////////

    send_ack_msg(
        &mut client_msl,
        &client_sig_kp,
        &mut server_msl,
        Some(vec![5, 9, 9, 5]),
    )?;

    // Negociation must be successfull, so we can clone secure layer
    client_msl.try_clone()?;
    server_msl.try_clone()?;

    // After clone, we can't change config
    let result = client_msl.change_config(SecureLayerConfig::default());
    if let Err(Error::ForbidChangeConfAfterClone) = result {
        // OK
    } else {
        println!("unexpected result={:?}", result);
        panic!();
    }

    //////////////////////////
    // CLIENT USER MSG
    //////////////////////////

    send_user_msg(&mut client_msl, &mut server_msl, vec![5, 7, 7, 5])?;

    //////////////////////////
    // SERVER USER MSG
    //////////////////////////

    send_user_msg(&mut server_msl, &mut client_msl, vec![7, 4, 4, 7])?;

    Ok(())
}
