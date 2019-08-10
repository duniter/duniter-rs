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

//! Sub-module managing the WS2Pv1 requests sent and received.

pub mod received;
pub mod sent;

use dubp_documents::BlockNumber;
use durs_network_documents::NodeFullId;
use serde::Serialize;
use std::convert::TryFrom;
use uuid::Uuid;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct WS2Pv1ReqId(pub Uuid);

impl WS2Pv1ReqId {
    #[inline]
    pub fn random() -> Self {
        WS2Pv1ReqId(Uuid::new_v4())
    }
    #[inline]
    pub fn to_hyphenated_string(&self) -> String {
        self.0.to_hyphenated().to_string()
    }
}

impl std::str::FromStr for WS2Pv1ReqId {
    type Err = uuid::parser::ParseError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        Ok(WS2Pv1ReqId(Uuid::parse_str(source)?))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct WS2Pv1ReqFullId {
    pub from: NodeFullId,
    pub req_id: WS2Pv1ReqId,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// WS2Pv1 requet
pub struct WS2Pv1Request {
    pub id: WS2Pv1ReqId,
    pub body: WS2Pv1ReqBody,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// WS2Pv1 requets body
pub enum WS2Pv1ReqBody {
    /// get current block
    GetCurrent,
    /// Get one block
    GetBlock {
        /// Block number
        number: BlockNumber,
    },
    /// Get a chunk of blocks
    GetBlocks {
        /// Number of blocks
        count: u32,
        /// First block number
        from_number: BlockNumber,
    },
    /// Get wot mempool
    GetRequirementsPending {
        /// The identities transmitted must have at least `minCert` certifications
        min_cert: usize,
    },
}

#[derive(Copy, Clone, Debug)]
pub struct WS2Pv1InvalidReqError;

impl TryFrom<&serde_json::Value> for WS2Pv1ReqBody {
    type Error = WS2Pv1InvalidReqError;

    fn try_from(json: &serde_json::Value) -> Result<WS2Pv1ReqBody, WS2Pv1InvalidReqError> {
        let req_name = json.get("name").ok_or(WS2Pv1InvalidReqError)?;
        match req_name.as_str().ok_or(WS2Pv1InvalidReqError)? {
            "CURRENT" => Ok(WS2Pv1ReqBody::GetCurrent),
            "BLOCK_BY_NUMBER" => {
                let params = json
                    .get("params")
                    .ok_or(WS2Pv1InvalidReqError)?
                    .as_object()
                    .ok_or(WS2Pv1InvalidReqError)?;
                let number = params
                    .get("number")
                    .ok_or(WS2Pv1InvalidReqError)?
                    .as_u64()
                    .ok_or(WS2Pv1InvalidReqError)?;
                Ok(WS2Pv1ReqBody::GetBlock {
                    number: BlockNumber(u32::try_from(number).map_err(|_| WS2Pv1InvalidReqError)?),
                })
            }
            "BLOCKS_CHUNK" => {
                let params = json
                    .get("params")
                    .ok_or(WS2Pv1InvalidReqError)?
                    .as_object()
                    .ok_or(WS2Pv1InvalidReqError)?;
                let count = params
                    .get("count")
                    .ok_or(WS2Pv1InvalidReqError)?
                    .as_u64()
                    .ok_or(WS2Pv1InvalidReqError)?;
                let from_number = params
                    .get("fromNumber")
                    .ok_or(WS2Pv1InvalidReqError)?
                    .as_u64()
                    .ok_or(WS2Pv1InvalidReqError)?;
                Ok(WS2Pv1ReqBody::GetBlocks {
                    count: u32::try_from(count).map_err(|_| WS2Pv1InvalidReqError)?,
                    from_number: BlockNumber(
                        u32::try_from(from_number).map_err(|_| WS2Pv1InvalidReqError)?,
                    ),
                })
            }
            "WOT_REQUIREMENTS_OF_PENDING" => {
                let params = json
                    .get("params")
                    .ok_or(WS2Pv1InvalidReqError)?
                    .as_object()
                    .ok_or(WS2Pv1InvalidReqError)?;
                let min_cert = params
                    .get("minCert")
                    .ok_or(WS2Pv1InvalidReqError)?
                    .as_u64()
                    .ok_or(WS2Pv1InvalidReqError)?;
                Ok(WS2Pv1ReqBody::GetRequirementsPending {
                    min_cert: usize::try_from(min_cert).map_err(|_| WS2Pv1InvalidReqError)?,
                })
            }
            _ => Err(WS2Pv1InvalidReqError),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use serde_json::json;

    #[test]
    fn parse_ws2p_v1_req_get_current() -> Result<(), WS2Pv1InvalidReqError> {
        let json_req_body = json!({
            "name": "CURRENT",
            "params": {}
        });

        let parsed_req = WS2Pv1ReqBody::try_from(&json_req_body)?;

        assert_eq!(WS2Pv1ReqBody::GetCurrent, parsed_req);

        Ok(())
    }

    #[test]
    fn parse_ws2p_v1_req_get_block() -> Result<(), WS2Pv1InvalidReqError> {
        let json_req_body = json!({
            "name": "BLOCK_BY_NUMBER",
            "params": {
                "number": 42,
            }
        });

        let parsed_req = WS2Pv1ReqBody::try_from(&json_req_body)?;

        assert_eq!(
            WS2Pv1ReqBody::GetBlock {
                number: BlockNumber(42),
            },
            parsed_req
        );

        Ok(())
    }

    #[test]
    fn parse_ws2p_v1_req_get_blocks() -> Result<(), WS2Pv1InvalidReqError> {
        let json_req_body = json!({
            "name": "BLOCKS_CHUNK",
            "params": {
                "count": 50,
                "fromNumber": 100,
            }
        });

        let parsed_req = WS2Pv1ReqBody::try_from(&json_req_body)?;

        assert_eq!(
            WS2Pv1ReqBody::GetBlocks {
                count: 50,
                from_number: BlockNumber(100),
            },
            parsed_req
        );

        Ok(())
    }

    #[test]
    fn parse_ws2p_v1_req_get_requirements_pending() -> Result<(), WS2Pv1InvalidReqError> {
        let json_req_body = json!({
            "name": "WOT_REQUIREMENTS_OF_PENDING",
            "params": {
                "minCert": 3,
            }
        });

        let parsed_req = WS2Pv1ReqBody::try_from(&json_req_body)?;

        assert_eq!(
            WS2Pv1ReqBody::GetRequirementsPending { min_cert: 3 },
            parsed_req
        );

        Ok(())
    }
}
