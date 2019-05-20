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

//! Sub-module that serialize HEAD into WS2Pv1 json format

use super::IntoWS2Pv1Json;
use durs_common_tools::fatal_error;
use durs_network_documents::network_head::*;
use std::ops::Deref;

impl IntoWS2Pv1Json for NetworkHead {
    fn into_ws2p_v1_json(self) -> serde_json::Value {
        match self {
            NetworkHead::V2(box_head_v2) => {
                let head_v2 = box_head_v2.deref();
                json!({
                    "message": head_v2.message.to_string(),
                    "sig": head_v2.sig.to_string(),
                    "messageV2": head_v2.message_v2.to_string(),
                    "sigV2": head_v2.sig_v2.to_string(),
                    "step": head_v2.step + 1
                })
            }
            _ => fatal_error!("HEAD version not supported !"),
        }
    }
}
