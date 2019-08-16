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

//! Sub-module managing the WS2Pv1 responses sent and received.

pub mod received;
pub mod sent;

use crate::serializers::IntoWS2Pv1Json;
use crate::ws_connections::requests::WS2Pv1ReqId;
use dubp_block_doc::BlockDocument;
use dubp_common_doc::traits::ToStringObject;
use dup_crypto::keys::PubKey;

/// WS2Pv1 request response
#[derive(Clone, Debug)]
pub struct WS2Pv1ReqRes {
    /// WS2Pv1 request id
    pub req_id: WS2Pv1ReqId,
    /// WS2Pv1 request response body
    pub body: WS2Pv1ReqResBody,
}

impl Into<serde_json::Value> for WS2Pv1ReqRes {
    fn into(self) -> serde_json::Value {
        let mut map = serde_json::map::Map::with_capacity(2);
        map.insert(
            "resId".to_owned(),
            self.req_id.to_hyphenated_string().into(),
        );
        map.insert("body".to_owned(), self.body.into());
        serde_json::Value::Object(map)
    }
}

/// WS2Pv1 request response body
#[derive(Clone, Debug)]
pub enum WS2Pv1ReqResBody {
    /// Response to request getCurrent
    GetCurrent(BlockDocument),
    // Response to request getBlock
    GetBlock(BlockDocument),
    // Response to request getBlocks
    GetBlocks(Vec<BlockDocument>),
    // Response to request getRequirementsPending
    GetRequirementsPending {
        identities: Vec<WS2Pv1IdentityRequirementsPending>,
    },
}

impl Into<serde_json::Value> for WS2Pv1ReqResBody {
    fn into(self) -> serde_json::Value {
        match self {
            WS2Pv1ReqResBody::GetCurrent(block_doc) => {
                block_doc.to_string_object().into_ws2p_v1_json()
            }
            WS2Pv1ReqResBody::GetBlock(block_doc) => {
                block_doc.to_string_object().into_ws2p_v1_json()
            }
            WS2Pv1ReqResBody::GetBlocks(blocks) => serde_json::Value::Array(
                blocks
                    .iter()
                    .map(ToStringObject::to_string_object)
                    .map(IntoWS2Pv1Json::into_ws2p_v1_json)
                    .collect(),
            ),
            WS2Pv1ReqResBody::GetRequirementsPending { .. } => {
                let mut map = serde_json::map::Map::with_capacity(1);
                map.insert("identities".to_owned(), serde_json::Value::Array(vec![]));
                serde_json::Value::Object(map)
            }
        }
    }
}

/// WS2Pv1 Identity requirements pending
#[derive(Clone, Debug)]
pub struct WS2Pv1IdentityRequirementsPending {
    pub certifications: Vec<WS2pv1CertificationPending>,
    pub expired: bool,
    pub is_sentry: bool,
    pub membership_expires_in: u64,
    pub membership_pending_expires_in: u64,
    // Some fields missing ...
}

/// WS2Pv1 Certification pending
#[derive(Copy, Clone, Debug)]
pub struct WS2pv1CertificationPending {
    /// Expires in
    pub expires_in: u64,
    /// From
    pub from: PubKey,
    /// Timestamp
    pub timestamp: u64,
    /// To
    pub to: PubKey,
}
