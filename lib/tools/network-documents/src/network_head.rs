//  Copyright (C) 2017  The Dunitrust Project Developers.
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

//! Module defining the format of network heads and how to handle them.

use crate::network_head_v2::*;
use crate::network_head_v3::*;
use crate::{NodeFullId, NodeId};
use dubp_documents::blockstamp::*;
use dup_crypto::bases::BaseConvertionError;
use dup_crypto::keys::*;
use durs_common_tools::fatal_error;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::num::ParseIntError;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, Ord, PartialOrd, PartialEq, Serialize, Deserialize)]
/// Network Head : Set of information on the current state of a node, the central information being the blockstamp of its current block (= the head of its blockchain).
pub enum NetworkHead {
    /// Head V2
    V2(Box<NetworkHeadV2>),
    /// head V3
    V3(Box<NetworkHeadV3>),
}

impl ToString for NetworkHead {
    fn to_string(&self) -> String {
        match *self {
            NetworkHead::V2(ref head_v2) => head_v2.deref().to_string(),
            _ => fatal_error!("NetworkHead version not supported !"),
        }
    }
}

/// NetworkHeadParseErr parse error
#[derive(Debug)]
pub enum NetworkHeadParseErr {
    /// BaseConvertionError
    BaseConvertionError(BaseConvertionError),
    /// ParseIntError
    ParseIntError(ParseIntError),
    /// BlockstampParseError
    BlockstampParseError(BlockstampParseError),
    /// NetworkHeadMessageParseErr
    NetworkHeadMessageParseErr(NetworkHeadMessageParseErr),
    /// InvalidMessageVersion
    InvalidMessageVersion(),
    /// InvalidStep
    InvalidStep(),
    /// InvalidStr
    InvalidStr(&'static str),
    /// MissingField
    MissingField(&'static str),
}

impl From<NetworkHeadMessageParseErr> for NetworkHeadParseErr {
    fn from(e: NetworkHeadMessageParseErr) -> Self {
        NetworkHeadParseErr::NetworkHeadMessageParseErr(e)
    }
}

impl From<BaseConvertionError> for NetworkHeadParseErr {
    fn from(e: BaseConvertionError) -> Self {
        NetworkHeadParseErr::BaseConvertionError(e)
    }
}

impl From<BlockstampParseError> for NetworkHeadParseErr {
    fn from(e: BlockstampParseError) -> Self {
        NetworkHeadParseErr::BlockstampParseError(e)
    }
}

impl From<ParseIntError> for NetworkHeadParseErr {
    fn from(e: ParseIntError) -> Self {
        NetworkHeadParseErr::ParseIntError(e)
    }
}

impl NetworkHead {
    /// Get HEAD version
    pub fn version(&self) -> u32 {
        match *self {
            NetworkHead::V2(_) => 2,
            _ => fatal_error!("This HEAD version is not supported !"),
        }
    }
    /// Get HEAD blockstamp
    pub fn blockstamp(&self) -> Blockstamp {
        match *self {
            NetworkHead::V2(ref head_v2) => head_v2.message_v2.blockstamp(),
            _ => fatal_error!("This HEAD version is not supported !"),
        }
    }
    /// Get pubkey of head issuer
    pub fn pubkey(&self) -> PubKey {
        match *self {
            NetworkHead::V2(ref head_v2) => match head_v2.message_v2 {
                NetworkHeadMessage::V2(ref head_message_v2) => head_message_v2.pubkey,
                _ => fatal_error!("This HEAD message version is not supported !"),
            },
            _ => fatal_error!("This HEAD version is not supported !"),
        }
    }
    /// Get uid of head issuer
    pub fn uid(&self) -> Option<String> {
        match *self {
            NetworkHead::V2(ref head_v2) => head_v2.uid(),
            _ => fatal_error!("This HEAD version is not supported !"),
        }
    }
    /// Change uid of head issuer
    pub fn set_uid(&mut self, uid: &str) {
        match *self {
            NetworkHead::V2(ref mut head_v2) => head_v2.uid = Some(String::from(uid)),
            _ => fatal_error!("This HEAD version is not supported !"),
        }
    }
    /// return the HEAD Step
    pub fn step(&self) -> u32 {
        match *self {
            NetworkHead::V2(ref head_v2) => head_v2.step,
            _ => fatal_error!("This HEAD version is not supported !"),
        }
    }
    /// Checks the validity of all head signatures
    pub fn verify(&self) -> bool {
        match *self {
            NetworkHead::V2(ref head_v2) => {
                self.pubkey()
                    .verify(head_v2.message.to_string().as_bytes(), &head_v2.sig)
                    && self
                        .pubkey()
                        .verify(head_v2.message_v2.to_string().as_bytes(), &head_v2.sig_v2)
            }
            _ => fatal_error!("This HEAD version is not supported !"),
        }
    }
    /// Returns issuer node id
    pub fn node_uuid(&self) -> NodeId {
        match *self {
            NetworkHead::V2(ref head_v2) => head_v2.message_v2.node_uuid(),
            _ => fatal_error!("This HEAD version is not supported !"),
        }
    }
    /// Returns issuer node full identifier
    pub fn node_full_id(&self) -> NodeFullId {
        NodeFullId(self.node_uuid(), self.pubkey())
    }
    /// Returns true only if this head is to replace the old head of the same issuer in the head cache (or if it's the 1st head of this issuer)
    pub fn apply(&self, heads_cache: &mut HashMap<NodeFullId, NetworkHead>) -> bool {
        let heads_cache_copy = heads_cache.clone();
        if let Some(head) = heads_cache_copy.get(&self.node_full_id()) {
            if self.blockstamp().id.0 > head.blockstamp().id.0
                || (self.blockstamp().id.0 == head.blockstamp().id.0
                    && self.version() >= head.version()
                    && self.step() < head.step())
            {
                if let Some(head_mut) = heads_cache.get_mut(&self.node_full_id()) {
                    *head_mut = self.clone();
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            heads_cache.insert(self.node_full_id(), self.clone());
            true
        }
    }
    /// Parse Json Head
    pub fn from_json_value(source: &serde_json::Value) -> Result<NetworkHead, NetworkHeadParseErr> {
        let message = NetworkHeadMessage::from_str(if let Some(str_msg) = source.get("message") {
            if let Some(str_msg) = str_msg.as_str() {
                str_msg
            } else {
                return Err(NetworkHeadParseErr::InvalidStr("message"));
            }
        } else {
            return Err(NetworkHeadParseErr::MissingField("message"));
        })?;
        match message {
            NetworkHeadMessage::V2(_) => Ok(NetworkHead::V2(Box::new(NetworkHeadV2 {
                message,
                sig: Sig::Ed25519(ed25519::Signature::from_base64(
                    if let Some(str_sig) = source.get("sig") {
                        if let Some(str_sig) = str_sig.as_str() {
                            str_sig
                        } else {
                            return Err(NetworkHeadParseErr::InvalidStr("sig"));
                        }
                    } else {
                        return Err(NetworkHeadParseErr::MissingField("sigV2"));
                    },
                )?),
                message_v2: NetworkHeadMessage::from_str(
                    if let Some(str_msg) = source.get("messageV2") {
                        if let Some(str_msg) = str_msg.as_str() {
                            str_msg
                        } else {
                            return Err(NetworkHeadParseErr::InvalidStr("messageV2"));
                        }
                    } else {
                        return Err(NetworkHeadParseErr::MissingField("messageV2"));
                    },
                )?,
                sig_v2: Sig::Ed25519(ed25519::Signature::from_base64(
                    if let Some(str_sig) = source.get("sigV2") {
                        if let Some(str_sig) = str_sig.as_str() {
                            str_sig
                        } else {
                            return Err(NetworkHeadParseErr::InvalidStr("sigV2"));
                        }
                    } else {
                        return Err(NetworkHeadParseErr::MissingField("sigV2"));
                    },
                )?),
                step: if let Some(step) = source.get("step") {
                    if let Some(step) = step.as_u64() {
                        step as u32
                    } else {
                        return Err(NetworkHeadParseErr::InvalidStep());
                    }
                } else {
                    return Err(NetworkHeadParseErr::MissingField("step"));
                },
                uid: None,
            }))),
            _ => Err(NetworkHeadParseErr::InvalidMessageVersion()),
        }
    }
    /// To human readable string
    pub fn to_human_string(&self, max_len: usize) -> String {
        match *self {
            NetworkHead::V2(ref head_v2) => head_v2.deref().to_human_string(max_len),
            _ => fatal_error!("NetworkHead version not supported !"),
        }
    }
}
