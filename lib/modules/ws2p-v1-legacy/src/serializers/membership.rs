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

//! Sub-module that serialize MembershipDocumentV10Stringified into WS2Pv1 json format

use super::IntoWS2Pv1Json;
use dubp_user_docs::documents::membership::MembershipDocumentV10Stringified;

impl IntoWS2Pv1Json for MembershipDocumentV10Stringified {
    fn into_ws2p_v1_json(self) -> serde_json::Value {
        format!(
            "{issuer}:{signature}:{blockstamp}:{idty_blockstamp}:{username}",
            issuer = self.issuer,
            signature = self.signature,
            blockstamp = self.blockstamp,
            idty_blockstamp = self.identity_blockstamp,
            username = self.username,
        )
        .into()
    }
}
