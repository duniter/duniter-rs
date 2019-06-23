//  Copyright (C) 2018  The Durs Project Developers.
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

//! Sub-module that serialize BlockDocument into WS2Pv1 json format

use super::IntoWS2Pv1Json;
use dubp_documents::documents::block::{BlockDocumentStringified, BlockDocumentV10Stringified};

impl IntoWS2Pv1Json for BlockDocumentStringified {
    fn into_ws2p_v1_json(self) -> serde_json::Value {
        match self {
            BlockDocumentStringified::V10(block_str_v10) => block_str_v10.into_ws2p_v1_json(),
        }
    }
}

impl IntoWS2Pv1Json for BlockDocumentV10Stringified {
    fn into_ws2p_v1_json(self) -> serde_json::Value {
        let actives = self
            .actives
            .into_iter()
            .map(IntoWS2Pv1Json::into_ws2p_v1_json)
            .collect::<Vec<serde_json::Value>>();
        let certifications = self
            .certifications
            .into_iter()
            .map(IntoWS2Pv1Json::into_ws2p_v1_json)
            .collect::<Vec<serde_json::Value>>();
        let identities = self
            .identities
            .into_iter()
            .map(IntoWS2Pv1Json::into_ws2p_v1_json)
            .collect::<Vec<serde_json::Value>>();
        let joiners = self
            .joiners
            .into_iter()
            .map(IntoWS2Pv1Json::into_ws2p_v1_json)
            .collect::<Vec<serde_json::Value>>();
        let leavers = self
            .leavers
            .into_iter()
            .map(IntoWS2Pv1Json::into_ws2p_v1_json)
            .collect::<Vec<serde_json::Value>>();
        let revoked = self
            .revoked
            .into_iter()
            .map(IntoWS2Pv1Json::into_ws2p_v1_json)
            .collect::<Vec<serde_json::Value>>();
        let transactions = self
            .transactions
            .into_iter()
            .map(IntoWS2Pv1Json::into_ws2p_v1_json)
            .collect::<Vec<serde_json::Value>>();

        json!( {
            "actives": actives,
            "certifications": certifications,
            "currency": self.currency,
            "dividend": null,
            "excluded": self.excluded,
            "fork": false,
            "hash": self.hash,
            "identities": identities,
            "inner_hash": self.inner_hash,
            "issuer": self.issuers[0],
            "issuersCount": self.issuers_count,
            "issuersFrame": self.issuers_frame,
            "issuersFrameVar": self.issuers_frame_var,
            "joiners": joiners,
            "leavers": leavers,
            "medianTime": self.median_time,
            "membersCount": self.members_count,
            "monetaryMass": self.monetary_mass,
            "nonce": self.nonce,
            "number": self.number,
            "parameters": self.parameters.unwrap_or_else(|| "".to_owned()),
            "powMin": self.pow_min,
            "previousHash": self.previous_hash,
            "previousIssuer": self.previous_issuer,
            "revoked": revoked,
            "signature": self.signatures[0],
            "time": self.time,
            "transactions": transactions,
            "unitbase": self.unit_base,
            "version": 10
        })
    }
}
