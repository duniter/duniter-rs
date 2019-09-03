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

//! Test Public Key Secure Transport Layer in complete mode.

#[cfg(feature = "zip-sign")]
mod tests {
    use pkstl::*;
    use ring::signature::{Ed25519KeyPair, KeyPair};
    use std::io::BufWriter;

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

    fn server_infos() -> Result<(SecureLayer, Vec<u8>)> {
        // Create server sig keypair seed
        let seed = Seed32::random();

        // Create server secure layer
        let server_msl =
            SecureLayer::create(SecureLayerConfig::default(), Some(seed.clone()), None)?;

        // Get server sig pubkey
        let server_sig_pubkey = Ed25519KeyPair::from_seed_unchecked(seed.as_ref())
            .map_err(|_| Error::FailtoGenSigKeyPair)?
            .public_key()
            .as_ref()
            .to_vec();

        Ok((server_msl, server_sig_pubkey))
    }

    fn client_infos(expected_server_sig_pubkey: Option<Vec<u8>>) -> Result<SecureLayer> {
        // Create client secure layer
        let client_msl = SecureLayer::create(
            SecureLayerConfig::default(),
            None,
            expected_server_sig_pubkey,
        )?;

        Ok(client_msl)
    }

    fn send_connect_msg(
        sender_msl: &mut SecureLayer,
        receiver_msl: &mut SecureLayer,
        custom_datas: Option<Vec<u8>>,
    ) -> Result<Vec<u8>> {
        // Write connect message and it's sig in channel
        let mut channel = BufWriter::new(Vec::with_capacity(1_000));
        sender_msl.write_connect_msg_bin(custom_datas.as_opt_ref(), &mut channel)?;

        // Receiver read connect message from channel
        let channel = channel.into_inner().map_err(|_| Error::BufferFlushError)?;
        let msg_received = receiver_msl.read_bin(&channel[..])?;
        if let IncomingBinaryMessage::Connect {
            custom_datas: custom_datas_received,
            peer_sig_public_key,
        } = msg_received.get(0).expect("Must be receive a message")
        {
            assert_eq!(&custom_datas, custom_datas_received);
            Ok(peer_sig_public_key.to_owned())
        } else {
            print!("Unexpected incoming message={:?}", msg_received);
            panic!();
        }
    }

    fn send_ack_msg(
        sender_msl: &mut SecureLayer,
        receiver_msl: &mut SecureLayer,
        custom_datas: Option<Vec<u8>>,
    ) -> Result<()> {
        // Write ack message and it's sig in channel
        let mut channel = BufWriter::new(Vec::with_capacity(1_000));
        sender_msl.write_ack_msg_bin(custom_datas.as_opt_ref(), &mut channel)?;

        // Receiver read ack message from channel
        let channel = channel.into_inner().map_err(|_| Error::BufferFlushError)?;
        let msg_received = receiver_msl.read_bin(&channel[..])?;
        if let IncomingBinaryMessage::Ack {
            custom_datas: custom_datas_received,
        } = msg_received.get(0).expect("Must be receive a message")
        {
            assert_eq!(&custom_datas, custom_datas_received);
            Ok(())
        } else {
            print!("Unexpected incoming message={:?}", msg_received);
            panic!();
        }
    }

    fn send_user_msg(
        sender_msl: &mut SecureLayer,
        receiver_msl: &mut SecureLayer,
        datas: Vec<u8>,
    ) -> Result<()> {
        // Write user message and it's sig in channel
        let mut channel = BufWriter::new(Vec::with_capacity(1_000));
        sender_msl.write_bin(&datas[..], &mut channel)?;

        // Receiver read user message from channel
        let channel = channel.into_inner().map_err(|_| Error::BufferFlushError)?;
        let msg_received = receiver_msl.read_bin(&channel[..])?;
        if let IncomingBinaryMessage::Message {
            datas: datas_received,
        } = msg_received.get(0).expect("Must be receive a message")
        {
            assert_eq!(&Some(datas), datas_received);
            Ok(())
        } else {
            print!("Unexpected incoming message={:?}", msg_received);
            panic!();
        }
    }

    #[test]
    fn server_recv_ack_early() -> Result<()> {
        //////////////////////////
        // SERVER INFOS
        //////////////////////////

        let (mut server_msl, server_sig_pk) = server_infos()?;

        //////////////////////////
        // CLIENT INFOS
        //////////////////////////

        let mut client_msl = client_infos(Some(server_sig_pk.clone()))?;

        //////////////////////////
        // SERVER CONNECT MSG
        //////////////////////////

        send_connect_msg(&mut server_msl, &mut client_msl, Some(vec![5, 1, 1, 5]))?;

        //////////////////////////
        // CLIENT ACK MSG
        //////////////////////////

        // Client write ack message and it's sig in channel
        let mut channel = BufWriter::new(Vec::with_capacity(1_000));
        let client_ack_custom_datas = Some(vec![7, 1, 1, 7]);
        client_msl.write_ack_msg_bin(client_ack_custom_datas.as_opt_ref(), &mut channel)?;

        // Server read ack message from channel
        let channel = channel.into_inner().map_err(|_| Error::BufferFlushError)?;
        let msg_receiveds = server_msl.read_bin(&channel[..])?;

        // Server must read nothing because the ack message received too early has been set aside
        assert_eq!(
            Vec::<IncomingBinaryMessage>::with_capacity(0),
            msg_receiveds
        );

        //////////////////////////
        // CLIENT CONNECT MSG
        //////////////////////////

        // Client write connect message and it's sig in channel
        let mut channel = BufWriter::new(Vec::with_capacity(1_000));
        let client_connect_custom_datas = Some(vec![5, 1, 1, 5]);
        client_msl.write_connect_msg_bin(client_connect_custom_datas.as_opt_ref(), &mut channel)?;

        // Server read connect message from channel
        let channel = channel.into_inner().map_err(|_| Error::BufferFlushError)?;
        let msgs_received = server_msl.read_bin(&channel[..])?;
        if let IncomingBinaryMessage::Connect {
            custom_datas: custom_datas_received,
            ..
        } = msgs_received.get(0).expect("Must be receive a message")
        {
            assert_eq!(&client_connect_custom_datas, custom_datas_received);
        } else {
            print!("Unexpected incoming messages={:?}", msgs_received);
            panic!();
        }

        // Server must also receive the ack message that had been set aside
        if let IncomingBinaryMessage::Ack {
            custom_datas: custom_datas_received,
        } = msgs_received.get(1).expect("Must be receive a message")
        {
            assert_eq!(&client_ack_custom_datas, custom_datas_received);
        } else {
            print!("Unexpected incoming messages={:?}", msgs_received);
            panic!();
        }

        Ok(())
    }

    #[test]
    fn ordered_passing_case() -> Result<()> {
        //////////////////////////
        // SERVER INFOS
        //////////////////////////

        let (mut server_msl, server_sig_pk) = server_infos()?;

        //////////////////////////
        // CLIENT INFOS
        //////////////////////////

        let mut client_msl = client_infos(Some(server_sig_pk.clone()))?;

        //////////////////////////
        // CLIENT CONNECT MSG
        //////////////////////////

        let _client_sig_pk_recv =
            send_connect_msg(&mut client_msl, &mut server_msl, Some(vec![5, 4, 4, 5]))?;

        //////////////////////////
        // SERVER CONNECT MSG
        //////////////////////////

        let server_sig_pk_recv =
            send_connect_msg(&mut server_msl, &mut client_msl, Some(vec![5, 3, 3, 5]))?;
        assert_eq!(server_sig_pk, server_sig_pk_recv);

        //////////////////////////
        // SERVER ACK MSG
        //////////////////////////

        send_ack_msg(&mut server_msl, &mut client_msl, Some(vec![5, 9, 9, 5]))?;

        //////////////////////////
        // CLIENT ACK MSG
        //////////////////////////

        send_ack_msg(&mut client_msl, &mut server_msl, Some(vec![5, 0, 0, 5]))?;

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

        send_user_msg(&mut client_msl, &mut server_msl, vec![5, 5, 5, 5])?;

        //////////////////////////
        // SERVER USER MSG
        //////////////////////////

        send_user_msg(&mut server_msl, &mut client_msl, vec![9, 9, 9, 9])?;

        Ok(())
    }
}
