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

use dubp_documents::v10::block::BlockDocument;
use dubp_documents::v10::certification::CompactCertificationDocument;
use dubp_documents::v10::identity::CompactIdentityDocument;
use dubp_documents::v10::membership::CompactPoolMembershipDoc;
use dubp_documents::Blockstamp;
use dup_crypto::hashs::Hash;
use std::str;

/// WS2Pv2 request response
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WS2Pv2ReqRes {
    /// request unique identifier
    pub id: u32,
    /// request body
    pub body: WS2Pv2ReqResBody,
}

/// WS2Pv2 request response body
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WS2Pv2ReqResBody {
    /// Empty response
    None,
    /// BadRequest (reason)
    BadRequest(String),
    /// Current blockstamp
    Current(Blockstamp),
    /// Blocks hashs
    BlocksHashs(Vec<Hash>),
    /// Chunk of blocks.
    Chunk(Vec<BlockDocument>),
    /// Wot pool datas
    WotPool(Vec<CompactCertificationDocument>, Vec<WotPoolFolder>),
}

///WotPoolFolder
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WotPoolFolder {
    /// Pending identity
    pub idty: CompactIdentityDocument,
    /// Pending first membership
    pub membership: CompactPoolMembershipDoc,
    /// Pending certs
    pub certs: Vec<CompactCertificationDocument>,
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;
    use dubp_documents::Blockstamp;
    use tests::*;

    #[test]
    fn test_ws2p_message_req_res_none() {
        let response = WS2Pv2ReqRes {
            id: 27,
            body: WS2Pv2ReqResBody::None,
        };
        test_ws2p_message(WS2Pv0MessagePayload::ReqRes(response));
    }

    #[test]
    fn test_ws2p_message_req_res_bad_request() {
        let reason = String::from("bla bla bla");
        let response = WS2Pv2ReqRes {
            id: 28,
            body: WS2Pv2ReqResBody::BadRequest(reason),
        };
        test_ws2p_message(WS2Pv0MessagePayload::ReqRes(response));
    }

    #[test]
    fn test_ws2p_message_req_res_current() {
        let blockstamp = Blockstamp::from_string(
            "499-000011BABEEE1020B1F6B2627E2BC1C35BCD24375E114349634404D2C266D84F",
        )
        .unwrap();
        let response = WS2Pv2ReqRes {
            id: 28,
            body: WS2Pv2ReqResBody::Current(blockstamp),
        };
        test_ws2p_message(WS2Pv0MessagePayload::ReqRes(response));
    }

    #[test]
    fn test_ws2p_message_req_res_blocks_hashs() {
        let hashs = vec![
            Hash::from_hex("000011BABEEE1020B1F6B2627E2BC1C35BCD24375E114349634404D2C266D84F")
                .unwrap(),
            Hash::from_hex("0000007F8D3CCAF77CB77C5C025C4AED8A82BA2DBD2156FD92C9634DAB59BD7E")
                .unwrap(),
        ];
        let response = WS2Pv2ReqRes {
            id: 29,
            body: WS2Pv2ReqResBody::BlocksHashs(hashs),
        };
        test_ws2p_message(WS2Pv0MessagePayload::ReqRes(response));
    }

    #[test]
    fn test_ws2p_message_req_res_wot_pool() {
        let cert_doc = create_cert_doc();
        let response = WS2Pv2ReqRes {
            id: 29,
            body: WS2Pv2ReqResBody::WotPool(vec![cert_doc.clone(), cert_doc.clone()], vec![]),
        };
        test_ws2p_message(WS2Pv0MessagePayload::ReqRes(response));
    }
}
