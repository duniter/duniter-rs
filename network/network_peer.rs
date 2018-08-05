//  Copyright (C) 2017  The Duniter Project Developers.
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

//! Module defining the format of network peer cards and how to handle them.

extern crate crypto;
extern crate duniter_crypto;
extern crate duniter_documents;
extern crate duniter_module;
extern crate serde;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use duniter_crypto::keys::*;
use duniter_documents::{Blockstamp, ReadBytesBlockstampError};
use dup_binarizer::*;
use network_endpoint::*;
use std::io::Cursor;
use std::mem;
use *;

/// Total size of all fixed size fields of an PeerCardV11
pub static PEER_CARDV11_FIXED_SIZE: &'static usize = &44;

#[derive(Debug)]
/// Error when converting a byte vector to peerCard
pub enum PeerCardReadBytesError {
    /// Bytes vector is too short
    TooShort(String),
    /// Bytes vector is too long
    TooLong(),
    /// IoError
    IoError(::std::io::Error),
    /// FromUtf8Error
    FromUtf8Error(::std::string::FromUtf8Error),
    /// ReadPubkeyBoxError
    ReadPubkeyBoxError(pubkey_box::ReadPubkeyBoxError),
    /// ReadSigBoxError
    ReadSigBoxError(sig_box::ReadSigBoxError),
    /// ReadBytesBlockstampError
    ReadBytesBlockstampError(ReadBytesBlockstampError),
    /// EndpointReadBytesError
    EndpointReadBytesError(EndpointReadBytesError),
    /// too early version (don't support binary format)
    TooEarlyVersion(),
    /// Version not yet supported
    VersionNotYetSupported(),
}

impl From<::std::io::Error> for PeerCardReadBytesError {
    fn from(e: ::std::io::Error) -> Self {
        PeerCardReadBytesError::IoError(e)
    }
}

impl From<pubkey_box::ReadPubkeyBoxError> for PeerCardReadBytesError {
    fn from(e: pubkey_box::ReadPubkeyBoxError) -> Self {
        PeerCardReadBytesError::ReadPubkeyBoxError(e)
    }
}

impl From<sig_box::ReadSigBoxError> for PeerCardReadBytesError {
    fn from(e: sig_box::ReadSigBoxError) -> Self {
        PeerCardReadBytesError::ReadSigBoxError(e)
    }
}

impl From<ReadBytesBlockstampError> for PeerCardReadBytesError {
    fn from(e: ReadBytesBlockstampError) -> Self {
        PeerCardReadBytesError::ReadBytesBlockstampError(e)
    }
}

impl From<EndpointReadBytesError> for PeerCardReadBytesError {
    fn from(e: EndpointReadBytesError) -> Self {
        PeerCardReadBytesError::EndpointReadBytesError(e)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Peer card V10
pub struct PeerCardV10 {
    /// Peer card Blockstamp
    pub blockstamp: Blockstamp,
    /// Peer card issuer
    pub issuer: PubKey,
    /// Peer card endpoints list
    pub endpoints: Vec<NetworkEndpoint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Peer card V11
pub struct PeerCardV11 {
    /// Currency code
    pub currency_code: u16,
    /// Peer card issuer
    pub issuer: PubKey,
    /// Issuer node id
    pub node_id: NodeId,
    /// Peer card Blockstamp
    pub blockstamp: Blockstamp,
    /// Peer card endpoints list
    pub endpoints: Vec<EndpointV11>,
    /// Signature
    pub sig: Option<Sig>,
}

impl BinMessageSignable for PeerCardV11 {
    fn issuer_pubkey(&self) -> PubKey {
        self.issuer
    }
    fn signature(&self) -> Option<Sig> {
        self.sig
    }
    fn set_signature(&mut self, signature: Sig) {
        self.sig = Some(signature)
    }
}

impl BinMessage for PeerCardV11 {
    type ReadBytesError = PeerCardReadBytesError;

    fn to_bytes_vector(&self) -> Vec<u8> {
        let endpoints_count = self.endpoints.len() as u8;
        let mut binary_endpoints = vec![];
        for ep in &self.endpoints {
            binary_endpoints.push(ep.to_bytes_vector());
        }
        let endpoints_size: usize = binary_endpoints.iter().map(|bin_ep| bin_ep.len() + 2).sum();
        let (pubkey_box_size, sig_box_size) = match self.issuer {
            PubKey::Ed25519(_) => (35, 66),
            PubKey::Schnorr() => panic!("Schnorr algo not yet supported !"),
        };
        let peer_card_size =
            *PEER_CARDV11_FIXED_SIZE + endpoints_size + pubkey_box_size + sig_box_size;
        let mut binary_peer_card = Vec::with_capacity(peer_card_size + 2);
        // peer_card_size
        let mut buffer = [0u8; mem::size_of::<u16>()];
        buffer
            .as_mut()
            .write_u16::<BigEndian>(peer_card_size as u16)
            .expect("Unable to write");
        binary_peer_card.extend_from_slice(&buffer);
        // version
        binary_peer_card.push(11u8);
        // endpoints_count
        binary_peer_card.push(endpoints_count);
        // currency_code
        let mut buffer = [0u8; mem::size_of::<u16>()];
        buffer
            .as_mut()
            .write_u16::<BigEndian>(self.currency_code)
            .expect("Unable to write");
        binary_peer_card.extend_from_slice(&buffer);
        // node_id
        let mut buffer = [0u8; mem::size_of::<u32>()];
        buffer
            .as_mut()
            .write_u32::<BigEndian>(self.node_id.0)
            .expect("Unable to write");
        binary_peer_card.extend_from_slice(&buffer);
        // Write issuer_public_key
        pubkey_box::write_pubkey_box(&mut binary_peer_card, self.issuer)
            .expect("Fail to binarize peer.issuer !");
        // blockstamp
        binary_peer_card.extend(self.blockstamp.to_bytes_vector());
        // endpoints_datas
        for bin_ep in binary_endpoints {
            let mut buffer = [0u8; mem::size_of::<u16>()];
            buffer
                .as_mut()
                .write_u16::<BigEndian>(bin_ep.len() as u16)
                .expect("Unable to write");
            binary_peer_card.extend_from_slice(&buffer);
            binary_peer_card.extend(bin_ep);
        }
        // Write signature
        if let Some(sig) = self.sig {
            sig_box::write_sig_box(&mut binary_peer_card, sig)
                .expect("Fail to binarize peer.sig !");
        }
        binary_peer_card
    }
    fn from_bytes(binary_peer_card: &[u8]) -> Result<PeerCardV11, PeerCardReadBytesError> {
        let mut index = 0;
        // read version
        let version = if !binary_peer_card.is_empty() {
            index += 1;
            binary_peer_card[index - 1]
        } else {
            return Err(PeerCardReadBytesError::TooShort(String::from(
                "Size is zero",
            )));
        };
        if binary_peer_card.len() < *PEER_CARDV11_FIXED_SIZE {
            return Err(PeerCardReadBytesError::TooShort(String::from(
                "min_fixed_size",
            )));
        }
        // read endpoints_count
        let endpoints_count = binary_peer_card[index];
        index += 1;
        // read currency_code
        let currency_code = u16::read_u16_be(&binary_peer_card[index..index + 2])?;
        index += 2;
        // read node_id
        let node_id = NodeId(u32::read_u32_be(&binary_peer_card[index..index + 4])?);
        index += 4;
        // read issuer_size
        let issuer_size = u16::read_u16_be(&binary_peer_card[index..index + 2])? as usize;
        index += 2;
        // read issuer
        let (issuer, key_algo) = if binary_peer_card.len() > index + issuer_size {
            index += issuer_size;
            pubkey_box::read_pubkey_box(&binary_peer_card[index - issuer_size..index])?
        } else {
            return Err(PeerCardReadBytesError::TooShort(String::from("issuer")));
        };
        // read blockstamp
        let blockstamp = if binary_peer_card.len() > index + 36 {
            index += 36;
            Blockstamp::from_bytes_slice(&binary_peer_card[index - 36..index])?
        } else {
            return Err(PeerCardReadBytesError::TooShort(String::from("blockstamp")));
        };
        // read endpoints_datas
        println!("DEBUG: index={}", index);
        let mut endpoints = Vec::with_capacity(endpoints_count as usize);
        for _ in 0..endpoints_count {
            // read endpoint_size
            if binary_peer_card.len() < index + 2 {
                return Err(PeerCardReadBytesError::TooShort(String::from(
                    "endpoint_size",
                )));
            }

            let mut endpoint_size_bytes = Cursor::new(binary_peer_card[index..index + 2].to_vec());
            index += 2;
            let endpoint_size = endpoint_size_bytes.read_u16::<BigEndian>()?;
            // read endpoint_datas
            if binary_peer_card.len() < index + endpoint_size as usize {
                return Err(PeerCardReadBytesError::TooShort(format!(
                    "endpoint_datas (expected >= {} found {})",
                    index + endpoint_size as usize,
                    binary_peer_card.len()
                )));
            }
            endpoints.push(EndpointV11::from_bytes(
                &binary_peer_card[index..index + endpoint_size as usize],
            )?);
            index += endpoint_size as usize;
        }
        println!("DEBUG: index={}", index);
        // read signature_size
        let signature_size = if binary_peer_card.len() > index + 2 {
            index += 2;
            u16::read_u16_be(&binary_peer_card[index - 2..index])? as usize
        } else {
            return Err(PeerCardReadBytesError::TooShort(String::from(
                "signature_size",
            )));
        };
        println!("DEBUG: index={}", index);
        println!("DEBUG: signature_size={}", signature_size);
        // read signature
        let sig = if binary_peer_card.len() > index + signature_size {
            return Err(PeerCardReadBytesError::TooLong());
        } else if binary_peer_card.len() == index + signature_size {
            index += signature_size;
            Some(sig_box::read_sig_box(
                &binary_peer_card[index - signature_size..index],
                key_algo,
            )?)
        } else if binary_peer_card.len() > index {
            return Err(PeerCardReadBytesError::TooLong());
        } else if binary_peer_card.len() == index {
            None
        } else {
            return Err(PeerCardReadBytesError::TooShort(String::from("end")));
        };

        match version {
            tmp if tmp > 11 => Err(PeerCardReadBytesError::VersionNotYetSupported()),
            11 => Ok(PeerCardV11 {
                currency_code,
                issuer,
                node_id,
                blockstamp,
                endpoints,
                sig,
            }),
            _ => Err(PeerCardReadBytesError::TooEarlyVersion()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Peer card
pub enum PeerCard {
    /// Peer card V10
    V10(PeerCardV10),
    /// Peer card V11
    V11(PeerCardV11),
}

impl PeerCard {
    /// Get peer card version
    pub fn version(&self) -> u32 {
        match *self {
            PeerCard::V10(ref _peer_v10) => 10,
            PeerCard::V11(ref _peer_v11) => 11,
        }
    }
    /// Get peer card blockstamp
    pub fn blockstamp(&self) -> Blockstamp {
        match *self {
            PeerCard::V10(ref peer_v10) => peer_v10.blockstamp,
            _ => panic!("Peer version is not supported !"),
        }
    }
    /// Get peer card issuer
    pub fn issuer(&self) -> PubKey {
        match *self {
            PeerCard::V10(ref peer_v10) => peer_v10.issuer,
            _ => panic!("Peer version is not supported !"),
        }
    }
    /// Verify validity of peer card signature
    pub fn verify(&self) -> bool {
        false
    }
    /// Get peer card endpoint
    pub fn get_endpoints(&self) -> Vec<NetworkEndpoint> {
        Vec::with_capacity(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    use std::str::FromStr;

    fn keypair1() -> ed25519::KeyPair {
        ed25519::KeyPairFromSaltedPasswordGenerator::with_default_parameters().generate(
            "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV".as_bytes(),
            "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV_".as_bytes(),
        )
    }
    fn create_endpoint_v11() -> EndpointV11 {
        EndpointV11 {
            api: EndpointV11Api::Bin(ApiKnownByDuniter::WS2P()),
            api_version: 2,
            network_features: EndpointV11NetworkFeatures(vec![4u8]),
            api_features: vec![7u8],
            ip_v4: None,
            ip_v6: None,
            host: Some(String::from("g1.durs.ifee.fr")),
            port: 443u16,
            path: Some(String::from("ws2p")),
            status: 0,
            last_check: 0,
        }
    }
    fn create_second_endpoint_v11() -> EndpointV11 {
        EndpointV11 {
            api: EndpointV11Api::Bin(ApiKnownByDuniter::WS2P()),
            api_version: 2,
            network_features: EndpointV11NetworkFeatures(vec![5u8]),
            api_features: vec![7u8],
            ip_v4: Some(Ipv4Addr::from_str("84.16.72.210").unwrap()),
            ip_v6: None,
            host: None,
            port: 443u16,
            path: Some(String::from("ws2p")),
            status: 0,
            last_check: 0,
        }
    }

    #[test]
    fn test_convert_peer_card_v11_into_bytes_vector() {
        let keypair1 = keypair1();
        let mut peer_card_v11 = PeerCardV11 {
            currency_code: 1u16,
            issuer: PubKey::Ed25519(keypair1.public_key()),
            node_id: NodeId(0),
            blockstamp: Blockstamp::from_string(
                "50-000005B1CEB4EC5245EF7E33101A330A1C9A358EC45A25FC13F78BB58C9E7370",
            ).unwrap(),
            endpoints: vec![create_endpoint_v11(), create_second_endpoint_v11()],
            sig: None,
        };
        // Sign
        let sign_result = peer_card_v11.sign(PrivKey::Ed25519(keypair1.private_key()));
        if let Ok(peer_card_v11_bytes) = sign_result {
            // Check peer_size
            let mut peer_size_bytes = Cursor::new(peer_card_v11_bytes[0..2].to_vec());
            let peer_size = peer_size_bytes
                .read_u16::<BigEndian>()
                .expect("Fail to read peer_size !") as usize;
            assert_eq!(peer_size, peer_card_v11_bytes.len() - 2);
            // Check bytes content
            assert_eq!(
                peer_card_v11_bytes,
                vec![
                    // peer_size
                    0, 200, // peer_version
                    11,  // endpoints_count
                    2,   // currency_code
                    0, 1, // node_id
                    0, 0, 0, 0, // PubkeyBox.size
                    0, 33, // PubkeyBox.algo
                    0,  // PubkeyBox.content
                    99, 190, 24, 116, 151, 100, 11, 15, 205, 240, 34, 56, 191, 90, 136, 254, 98,
                    146, 159, 126, 97, 198, 39, 32, 72, 232, 66, 145, 92, 202, 215, 50,
                    // Blockstamp.id
                    0, 0, 0, 50, // Blockstamp.hash
                    0, 0, 5, 177, 206, 180, 236, 82, 69, 239, 126, 51, 16, 26, 51, 10, 28, 154, 53,
                    142, 196, 90, 37, 252, 19, 247, 139, 181, 140, // endpoints_datas
                    158, 115, 112, 0, 31, 0, 15, 4, 1, 0, 2, 1, 4, 1, 7, 103, 49, 46, 100, 117,
                    114, 115, 46, 105, 102, 101, 101, 46, 102, 114, 1, 187, 119, 115, 50, 112, 0,
                    20, 0, 0, 4, 1, 0, 2, 1, 5, 1, 7, 84, 16, 72, 210, 1, 187, 119, 115, 50, 112,
                    // SigBox.size
                    0, 64, // SigBox.content
                    241, 163, 69, 32, 232, 222, 69, 171, 55, 125, 150, 221, 66, 16, 249, 22, 219,
                    19, 64, 106, 171, 95, 79, 6, 103, 207, 10, 93, 177, 241, 160, 106, 33, 140,
                    120, 146, 145, 223, 96, 138, 156, 69, 246, 165, 51, 254, 95, 54, 189, 125, 249,
                    228, 145, 159, 106, 16, 197, 23, 230, 194, 232, 125, 235, 1
                ]
            );
            assert_eq!(
                peer_card_v11,
                PeerCardV11::from_bytes(&peer_card_v11_bytes[2..])
                    .expect("Fail to parse peer card bytes vector"),
            )
        } else {
            panic!("failt to sign peer card : {:?}", sign_result.err().unwrap())
        }
    }
}
