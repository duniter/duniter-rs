//  Copyright (C) 2017-2019  The AXIOM TEAM Association.
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

use super::connect::WS2Pv2ConnectMsg;
use super::ok::WS2Pv2OkMsg;
use super::req_responses::WS2Pv2ReqRes;
use super::requests::WS2Pv2Request;
use super::secret_flags::WS2Pv2SecretFlagsMsg;
use dubp_documents::documents::block::BlockDocument;
use dubp_documents::documents::certification::CertificationDocument;
use dubp_documents::documents::identity::IdentityDocument;
use dubp_documents::documents::membership::MembershipDocument;
use dubp_documents::documents::revocation::RevocationDocumentV10;
use dubp_documents::documents::transaction::TransactionDocument;
use dup_crypto::hashs::Hash;
use durs_network_documents::network_head_v2::NetworkHeadV2;
use durs_network_documents::network_head_v3::NetworkHeadV3;
use durs_network_documents::network_peer::PeerCardV11;

/// WS2P v2 message payload metadata size
pub static WS2P_V2_MESSAGE_PAYLOAD_METADATA_SIZE: &'static usize = &8;

/// WS2Pv2MessagePayload
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WS2Pv2MessagePayload {
    /// CONNECT message
    Connect(Box<WS2Pv2ConnectMsg>),
    /// ACK message
    Ack {
        /// Hash previously sent in CONNECT message
        challenge: Hash,
    },
    /// SECRET_FLAGS Message
    SecretFlags(WS2Pv2SecretFlagsMsg),
    /// OK Message
    Ok(WS2Pv2OkMsg),
    /// KO Message
    Ko(u16),
    /// REQUEST Message
    Request(WS2Pv2Request),
    /// REQUEST_RESPONSE Message
    ReqRes(WS2Pv2ReqRes),
    /// PEERS Message
    Peers(Vec<PeerCardV11>),
    /// HEADS_V2 Message
    Headsv2(Vec<NetworkHeadV2>),
    /// HEADS_V3 Message
    Heads3(Vec<NetworkHeadV3>),
    /// BLOCKS Message
    Blocks(Vec<BlockDocument>),
    /// PENDING_IDENTITIES Message
    PendingIdentities(Vec<IdentityDocument>),
    /// PENDING_MEMBERSHIPS Message
    PendingMemberships(Vec<MembershipDocument>),
    /// PENDING_CERTS Message
    PendingCerts(Vec<CertificationDocument>),
    /// PENDING_REVOCATIONS Message
    PendingRevocations(Vec<RevocationDocumentV10>),
    /// PENDING_TXS Message
    PendingTxs(Vec<TransactionDocument>),
}
