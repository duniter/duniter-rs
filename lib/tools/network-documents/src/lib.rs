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

//! Implements the Documents of DUNP (DUniter Network Protocol).

#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces
)]

#[macro_use]
extern crate pest_derive;
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate log;

pub mod host;
pub mod network_endpoint;
pub mod network_head;
pub mod network_head_v2;
pub mod network_head_v3;
pub mod network_peer;
pub mod url;

use crate::network_head::NetworkHead;
use crate::network_head_v3::NetworkHeadV3;
use crate::network_peer::PeerCard;
use crate::network_peer::PeerCardV11;
use dubp_documents::{TextDocumentParseError, TextDocumentParser};
use dup_crypto::hashs::*;
use dup_crypto::keys::*;
use durs_common_tools::fatal_error;
use pest::iterators::Pair;
use pest::Parser;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};
use std::net::AddrParseError;

#[derive(Parser)]
#[grammar = "network_documents.pest"]
/// Parser for network documents
struct NetworkDocsParser;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Network document
pub enum NetworkDocument {
    /// Peer
    Peer(Box<PeerCard>),
    /// Head
    Head(NetworkHead),
}

impl TextDocumentParser<Rule> for NetworkDocument {
    type DocumentType = NetworkDocument;

    fn parse(doc: &str) -> Result<NetworkDocument, TextDocumentParseError> {
        let mut net_doc_pairs = NetworkDocsParser::parse(Rule::network_document, doc)?;
        Ok(NetworkDocument::from_pest_pair(
            net_doc_pairs.next().unwrap().into_inner().next().unwrap(), // get and unwrap the `network_document` rule; never fails
        )?)
    }
    fn from_pest_pair(pair: Pair<Rule>) -> Result<NetworkDocument, TextDocumentParseError> {
        Ok(match pair.as_rule() {
            Rule::peer_v11 => {
                NetworkDocument::Peer(Box::new(PeerCard::V11(PeerCardV11::from_pest_pair(pair)?)))
            }
            Rule::head_v3 => NetworkDocument::Head(NetworkHead::V3(Box::new(
                NetworkHeadV3::from_pest_pair(pair)?,
            ))),
            _ => fatal_error!("unexpected rule: {:?}", pair.as_rule()), // Grammar ensures that we never reach this line
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Random identifier with which several Duniter nodes with the same network keypair can be differentiated
pub struct NodeId(pub u32);

impl Default for NodeId {
    fn default() -> NodeId {
        NodeId(0)
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{:x}", self.0)
    }
}

impl<'a> From<&'a str> for NodeId {
    fn from(source: &'a str) -> NodeId {
        NodeId(u32::from_str_radix(source, 16).expect("Fail to parse NodeId"))
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
/// Complete identifier of a duniter node.
pub struct NodeFullId(pub NodeId, pub PubKey);

impl Default for NodeFullId {
    fn default() -> NodeFullId {
        NodeFullId(
            NodeId::default(),
            PubKey::Ed25519(
                ed25519::PublicKey::from_base58("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
                    .unwrap(),
            ),
        )
    }
}

impl Display for NodeFullId {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}-{}", self.0, self.1)
    }
}

impl NodeFullId {
    /// Compute sha256 hash
    pub fn sha256(&self) -> Hash {
        Hash::compute(format!("{}", self).as_bytes())
    }
    /// To human string
    pub fn to_human_string(&self) -> String {
        let mut pubkey_string = self.1.to_string();
        pubkey_string.truncate(8);
        format!("{:8x}-{:8}", (self.0).0, pubkey_string)
    }
}

#[cfg(test)]
mod tests {
    use super::network_endpoint::*;
    use super::*;

    pub fn keypair1() -> ed25519::KeyPair {
        ed25519::KeyPairFromSaltedPasswordGenerator::with_default_parameters().generate(
            "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV".as_bytes(),
            "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV_".as_bytes(),
        )
    }

    #[test]
    fn parse_endpoint() {
        let issuer = PubKey::Ed25519(
            ed25519::PublicKey::from_base58("D9D2zaJoWYWveii1JRYLVK3J4Z7ZH3QczoKrnQeiM6mx")
                .unwrap(),
        );
        let node_id = NodeId(u32::from_str_radix("c1c39a0a", 16).unwrap());
        let full_id = NodeFullId(node_id, issuer);
        assert_eq!(
            EndpointV1::parse_from_raw("WS2P c1c39a0a i3.ifee.fr 80 /ws2p", issuer, 0, 0),
            Ok(EndpointV1 {
                issuer,
                api: ApiName(String::from("WS2P")),
                node_id: Some(node_id),
                hash_full_id: Some(full_id.sha256()),
                host: String::from("i3.ifee.fr"),
                port: 80,
                path: Some(String::from("ws2p")),
                raw_endpoint: String::from("WS2P c1c39a0a i3.ifee.fr 80 /ws2p"),
                last_check: 0,
                status: 0,
            })
        );
    }

    #[test]
    fn parse_endpoint2() {
        let issuer = PubKey::Ed25519(
            ed25519::PublicKey::from_base58("5gJYnQp8v7bWwk7EWRoL8vCLof1r3y9c6VDdnGSM1GLv")
                .unwrap(),
        );
        let node_id = NodeId(u32::from_str_radix("cb06a19b", 16).unwrap());
        let full_id = NodeFullId(node_id, issuer);
        assert_eq!(
            EndpointV1::parse_from_raw("WS2P cb06a19b g1.imirhil.fr 53012", issuer, 0, 0),
            Ok(EndpointV1 {
                issuer,
                api: ApiName(String::from("WS2P")),
                node_id: Some(node_id),
                hash_full_id: Some(full_id.sha256()),
                host: String::from("g1.imirhil.fr"),
                port: 53012,
                path: None,
                raw_endpoint: String::from("WS2P cb06a19b g1.imirhil.fr 53012"),
                last_check: 0,
                status: 0,
            })
        );
    }
}
