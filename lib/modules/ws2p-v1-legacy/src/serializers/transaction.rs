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

//! Sub-module that serialize TransactionDocument into WS2Pv1 json format

use super::IntoWS2Pv1Json;
use dubp_user_docs::documents::transaction::v10::TransactionDocumentV10Stringified;
use dubp_user_docs::documents::transaction::TransactionDocumentStringified;

impl IntoWS2Pv1Json for TransactionDocumentStringified {
    fn into_ws2p_v1_json(self) -> serde_json::Value {
        match self {
            TransactionDocumentStringified::V10(tx_doc_v10) => tx_doc_v10.into_ws2p_v1_json(),
        }
    }
}

impl IntoWS2Pv1Json for TransactionDocumentV10Stringified {
    fn into_ws2p_v1_json(self) -> serde_json::Value {
        json!( {
            "blockstamp": self.blockstamp,
            "blockstampTime": 0,
            "comment": self.comment,
            "currency": self.currency,
            "hash": self.hash,
            "inputs": self.inputs,
            "issuers": self.issuers,
            "locktime": self.locktime,
            "outputs": self.outputs,
            "signatures": self.signatures,
            "unlocks": self.unlocks,
            "version": 10
        })
    }
}
