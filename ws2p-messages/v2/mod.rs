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

/// WS2P Features
pub mod api_features;
/// WS2P v2 CONNECT Message
pub mod connect;
/// WS2P v2 OK Message
pub mod ok;
/// Message Payload container
pub mod payload_container;
/// WS2Pv2 requests responses messages
pub mod req_responses;
/// WS2Pv2 requests messages
pub mod requests;
/// WS2P v2 SECRET_FLAGS Message
pub mod secret_flags;

use duniter_documents::CurrencyName;
use dup_crypto::hashs::Hash;
use dup_crypto::keys::bin_signable::BinSignable;
use dup_crypto::keys::*;
use durs_network_documents::NodeId;
use v2::payload_container::*;

/// WS2P v2 message metadata size
pub static WS2P_V2_MESSAGE_METADATA_SIZE: &'static usize = &144;

/// WS2Pv0Message
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct WS2Pv0Message {
    /// Currency name
    pub currency_code: CurrencyName,
    /// Issuer NodeId
    pub issuer_node_id: NodeId,
    /// Issuer plublic key
    pub issuer_pubkey: PubKey,
    /// Message payload
    pub payload: WS2Pv0MessagePayload,
    /// Message hash
    pub message_hash: Option<Hash>,
    /// Signature
    pub signature: Option<Sig>,
}

impl WS2Pv0Message {
    /// WS2P Version number
    pub const WS2P_VERSION: u16 = 0;
}

impl<'de> BinSignable<'de> for WS2Pv0Message {
    fn issuer_pubkey(&self) -> PubKey {
        self.issuer_pubkey
    }
    fn store_hash(&self) -> bool {
        true
    }
    fn hash(&self) -> Option<Hash> {
        self.message_hash
    }
    fn set_hash(&mut self, hash: Hash) {
        self.message_hash = Some(hash)
    }
    fn signature(&self) -> Option<Sig> {
        self.signature
    }
    fn set_signature(&mut self, signature: Sig) {
        self.signature = Some(signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dup_crypto::keys::text_signable::TextSignable;
    use tests::*;

    #[test]
    fn test_ws2p_message_ack() {
        test_ws2p_message(WS2Pv0MessagePayload::Ack(Hash::random()));
    }

    #[test]
    fn test_ws2p_message_peers() {
        let keypair1 = keypair1();
        let mut peer = create_peer_card_v11();
        peer.sign(PrivKey::Ed25519(keypair1.private_key()))
            .expect("Fail to sign peer card !");
        test_ws2p_message(WS2Pv0MessagePayload::Peers(vec![peer]));
    }
}
