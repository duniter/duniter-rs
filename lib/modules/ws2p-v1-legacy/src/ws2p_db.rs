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

//! Manage WS2Pv1 storage.

use crate::ws_connections::states::WS2PConnectionState;
use durs_network_documents::network_endpoint::EndpointV1;
use durs_network_documents::NodeFullId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EndpointApi {
    WS2P,
    //WS2PS,
    //WS2PTOR,
    //DASA,
    //BMA,
    //BMAS,
}

pub fn string_to_api(api: &str) -> Option<EndpointApi> {
    match api {
        "WS2P" => Some(EndpointApi::WS2P),
        //"WS2PS" => Some(EndpointApi::WS2PS),
        //"WS2PTOR" => Some(EndpointApi::WS2PTOR),
        //"DASA" => Some(EndpointApi::DASA),
        //"BASIC_MERKLED_API" => Some(EndpointApi::BMA),
        //"BMAS" => Some(EndpointApi::BMAS),
        &_ => None,
    }
}

#[derive(Debug)]
pub enum Ws2pPeersDbError {
    IoErr(std::io::Error),
    SerdeErr(bincode::Error),
}

impl From<std::io::Error> for Ws2pPeersDbError {
    fn from(e: std::io::Error) -> Self {
        Ws2pPeersDbError::IoErr(e)
    }
}

impl From<bincode::Error> for Ws2pPeersDbError {
    fn from(e: bincode::Error) -> Self {
        Ws2pPeersDbError::SerdeErr(e)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DbEndpoint {
    pub ep: EndpointV1,
    pub state: WS2PConnectionState,
    pub last_check: u64,
}

pub fn get_endpoints(
    file_path: &Path,
) -> Result<HashMap<NodeFullId, DbEndpoint>, Ws2pPeersDbError> {
    if file_path.exists() {
        let bin_endpoints = durs_common_tools::fns::bin_file::read_bin_file(file_path)?;
        if bin_endpoints.is_empty() {
            Ok(HashMap::new())
        } else {
            Ok(bincode::deserialize(&bin_endpoints[..])?)
        }
    } else {
        File::create(file_path)?;
        Ok(HashMap::new())
    }
}

pub fn write_endpoints<S: std::hash::BuildHasher>(
    file_path: &Path,
    endpoints: &HashMap<NodeFullId, DbEndpoint, S>,
) -> Result<(), Ws2pPeersDbError> {
    let bin_endpoints = bincode::serialize(&endpoints)?;
    durs_common_tools::fns::bin_file::write_bin_file(file_path, &bin_endpoints)?;

    Ok(())
}
