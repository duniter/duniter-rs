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

//! Generate self peer card

use bincode;
use dubp_documents::{Blockstamp, CurrencyName};
use dup_crypto::keys::PubKey;
use durs_common_tools::fatal_error;
use durs_network_documents::network_endpoint::*;
use durs_network_documents::network_peer::*;
use durs_network_documents::*;

pub fn _self_peer_update_endpoints(
    self_peer: PeerCardV11,
    blockstamp: Blockstamp,
    new_endpoints: Vec<EndpointEnum>,
) -> PeerCardV11 {
    let max_eps = self_peer.endpoints.len() + self_peer.endpoints_str.len() + new_endpoints.len();
    let apis: Vec<NetworkEndpointApi> = new_endpoints
        .iter()
        .filter(|ep| {
            if let EndpointEnum::V2(_) = ep {
                true
            } else {
                false
            }
        })
        .map(EndpointEnum::api)
        .collect();
    let mut new_endpoints_bin = Vec::with_capacity(max_eps);
    let mut new_endpoints_str = Vec::with_capacity(max_eps);
    for ep in self_peer.endpoints {
        if !apis.contains(&ep.api) {
            new_endpoints_bin.push(ep);
        }
    }
    for ep in self_peer.endpoints_str {
        let ep_clone = ep.clone();
        let ep_fields: Vec<&str> = ep_clone.split(' ').collect();
        if !apis.contains(&NetworkEndpointApi(ep_fields[0].to_owned())) {
            new_endpoints_str.push(ep);
        }
    }
    for ep in new_endpoints {
        if let EndpointEnum::V2(ep_v2) = ep {
            let bin_len = bincode::serialize(&ep_v2)
                .unwrap_or_else(|_| {
                    fatal_error!(
                        "Fail to update self peer : invalid endpoint : {:?} !",
                        ep_v2
                    )
                })
                .len();
            let str_ep = ep_v2.to_string();
            if str_ep.len() < bin_len {
                new_endpoints_str.push(str_ep);
            } else {
                new_endpoints_bin.push(ep_v2);
            }
        }
    }

    PeerCardV11 {
        currency_name: self_peer.currency_name,
        issuer: self_peer.issuer,
        node_id: self_peer.node_id,
        blockstamp,
        endpoints: new_endpoints_bin,
        endpoints_str: new_endpoints_str,
        sig: None,
    }
}

pub fn _generate_self_peer(
    currency_name: CurrencyName,
    issuer: PubKey,
    node_id: NodeId,
    blockstamp: Blockstamp,
    endpoints: Vec<EndpointEnum>,
) -> PeerCardV11 {
    let mut endpoints_bin = Vec::with_capacity(endpoints.len());
    let mut endpoints_str = Vec::with_capacity(endpoints.len());

    for ep in endpoints {
        if let EndpointEnum::V2(ep_v2) = ep {
            let bin_len = bincode::serialize(&ep_v2)
                .unwrap_or_else(|_| {
                    fatal_error!(
                        "Fail to generate self peer : invalid endpoint : {:?} !",
                        ep_v2
                    )
                })
                .len();
            let str_ep = ep_v2.to_string();
            if str_ep.len() < bin_len {
                endpoints_str.push(str_ep);
            } else {
                endpoints_bin.push(ep_v2);
            }
        }
    }

    PeerCardV11 {
        currency_name,
        issuer,
        node_id,
        blockstamp,
        endpoints: endpoints_bin,
        endpoints_str,
        sig: None,
    }
}
