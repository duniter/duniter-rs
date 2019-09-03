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

//! Test Public Key Secure Transport Layer in complete mode with "ser" feature.

#[cfg(feature = "ser")]
mod tests {
    use pkstl::*;
    use ring::signature::{Ed25519KeyPair, KeyPair};
    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use std::fmt::Debug;
    use std::io::BufWriter;

    fn server_infos(format: MessageFormat) -> Result<(SecureLayer, Vec<u8>)> {
        // Create server sig keypair seed
        let seed = Seed32::random();

        // Create server secure layer
        let mut conf = SecureLayerConfig::default();
        conf.message_format = format;
        let server_msl = SecureLayer::create(conf, Some(seed.clone()), None)?;

        // Get server sig pubkey
        let server_sig_pubkey = Ed25519KeyPair::from_seed_unchecked(seed.as_ref())
            .map_err(|_| Error::FailtoGenSigKeyPair)?
            .public_key()
            .as_ref()
            .to_vec();

        Ok((server_msl, server_sig_pubkey))
    }

    fn client_infos(
        expected_server_sig_pubkey: Option<Vec<u8>>,
        format: MessageFormat,
    ) -> Result<SecureLayer> {
        // Create client secure layer
        let mut conf = SecureLayerConfig::default();
        conf.message_format = format;
        let client_msl = SecureLayer::create(conf, None, expected_server_sig_pubkey)?;

        Ok(client_msl)
    }

    fn send_connect_msg<D: Debug + PartialEq + Serialize + DeserializeOwned>(
        sender_msl: &mut SecureLayer,
        receiver_msl: &mut SecureLayer,
        custom_datas: Option<D>,
    ) -> Result<Vec<u8>> {
        // Write connect message and it's sig in channel
        let mut channel = BufWriter::new(Vec::with_capacity(1_000));
        sender_msl.write_connect_msg(custom_datas.as_ref(), &mut channel)?;

        // Receiver read connect message from channel
        let channel = channel.into_inner().map_err(|_| Error::BufferFlushError)?;
        let msg_received = receiver_msl.read(&channel[..])?;
        if let IncomingMessage::Connect {
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

    fn send_ack_msg<D: Debug + PartialEq + Serialize + DeserializeOwned>(
        sender_msl: &mut SecureLayer,
        receiver_msl: &mut SecureLayer,
        custom_datas: Option<D>,
    ) -> Result<()> {
        // Write ack message and it's sig in channel
        let mut channel = BufWriter::new(Vec::with_capacity(1_000));
        sender_msl.write_ack_msg(custom_datas.as_ref(), &mut channel)?;

        // Receiver read ack message from channel
        let channel = channel.into_inner().map_err(|_| Error::BufferFlushError)?;
        let msg_received = receiver_msl.read(&channel[..])?;
        if let IncomingMessage::Ack {
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

    fn send_user_msg<D: Debug + PartialEq + Serialize + DeserializeOwned>(
        sender_msl: &mut SecureLayer,
        receiver_msl: &mut SecureLayer,
        datas: D,
    ) -> Result<()> {
        // Write user message and it's sig in channel
        let mut channel = BufWriter::new(Vec::with_capacity(1_000));
        sender_msl.write(&datas, &mut channel)?;

        // Receiver read user message from channel
        let channel = channel.into_inner().map_err(|_| Error::BufferFlushError)?;
        let msg_received = receiver_msl.read(&channel[..])?;
        if let IncomingMessage::Message {
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

    #[cfg(feature = "bin")]
    #[test]
    fn ordered_passing_case_bincode() -> Result<()> {
        test_ordered_passing_case(
            MessageFormat::Bincode,
            Some("abc".to_owned()),
            None,
            Some("blablabla".to_owned()),
        )
    }

    #[cfg(feature = "cbor")]
    #[test]
    fn ordered_passing_case_cbor() -> Result<()> {
        test_ordered_passing_case(
            MessageFormat::Cbor,
            None,
            Some("def".to_owned()),
            Some("blablabla".to_owned()),
        )
    }

    #[cfg(feature = "json")]
    #[test]
    fn ordered_passing_case_json() -> Result<()> {
        test_ordered_passing_case(
            MessageFormat::Utf8Json,
            Some("abc".to_owned()),
            Some("def".to_owned()),
            None,
        )
    }

    fn test_ordered_passing_case<D: Clone + Debug + PartialEq + Serialize + DeserializeOwned>(
        message_format: MessageFormat,
        connect_msg_custom_datas: Option<D>,
        ack_msg_custom_datas: Option<D>,
        user_msg_datas: Option<D>,
    ) -> Result<()> {
        //////////////////////////
        // SERVER INFOS
        //////////////////////////

        let (mut server_msl, server_sig_pk) = server_infos(message_format)?;

        //////////////////////////
        // CLIENT INFOS
        //////////////////////////

        let mut client_msl = client_infos(Some(server_sig_pk.clone()), message_format)?;

        //////////////////////////
        // CLIENT CONNECT MSG
        //////////////////////////

        let _client_sig_pk_recv = send_connect_msg(
            &mut client_msl,
            &mut server_msl,
            connect_msg_custom_datas.clone(),
        )?;

        //////////////////////////
        // SERVER CONNECT MSG
        //////////////////////////

        let server_sig_pk_recv =
            send_connect_msg(&mut server_msl, &mut client_msl, connect_msg_custom_datas)?;
        assert_eq!(server_sig_pk, server_sig_pk_recv);

        //////////////////////////
        // SERVER ACK MSG
        //////////////////////////

        send_ack_msg(
            &mut server_msl,
            &mut client_msl,
            ack_msg_custom_datas.clone(),
        )?;

        //////////////////////////
        // CLIENT ACK MSG
        //////////////////////////

        send_ack_msg(&mut client_msl, &mut server_msl, ack_msg_custom_datas)?;

        //////////////////////////
        // CLIENT USER MSG
        //////////////////////////

        send_user_msg(&mut client_msl, &mut server_msl, user_msg_datas.clone())?;

        //////////////////////////
        // SERVER USER MSG
        //////////////////////////

        send_user_msg(&mut server_msl, &mut client_msl, user_msg_datas)?;

        Ok(())
    }
}
