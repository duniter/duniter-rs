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

//! WebSocketToPeer API for the Duniter project.

#![cfg_attr(feature = "strict", deny(warnings))]
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate structopt;

extern crate bincode;
extern crate dubp_documents;
extern crate duniter_conf;
extern crate durs_message;
extern crate duniter_module;
extern crate duniter_network;
extern crate dup_crypto;
extern crate durs_network_documents;
extern crate durs_ws2p_messages;

mod constants;
mod generate_peer;
pub mod controllers;
pub mod services;

use constants::*;
use duniter_conf::DuRsConf;
use durs_message::*;
use duniter_module::*;
use duniter_network::*;
use durs_network_documents::network_endpoint::*;
use std::sync::mpsc;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// WS2P Configuration
pub struct WS2PConf {
    /// Limit of outcoming connections
    pub outcoming_quota: usize,
    /// Default WS2P endpoints provides by configuration file
    pub sync_endpoints: Vec<EndpointEnum>,
}

impl Default for WS2PConf {
    fn default() -> Self {
        WS2PConf {
            outcoming_quota: *WS2P_DEFAULT_OUTCOMING_QUOTA,
            sync_endpoints: vec![
                EndpointV2::parse_from_raw("WS2P g1-monit.librelois.fr 443 ws2p").unwrap(),
                EndpointV2::parse_from_raw("WS2P g1.monnaielibreoccitanie.org 443 ws2p").unwrap(),
            ],
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// WS2Pv2 Module
pub struct WS2Pv2Module {}

impl Default for WS2Pv2Module {
    fn default() -> WS2Pv2Module {
        WS2Pv2Module {}
    }
}

#[derive(Debug)]
/// WS2PFeaturesParseError
pub enum WS2PFeaturesParseError {
    /// UnknowApiFeature
    UnknowApiFeature(String),
}

impl ApiModule<DuRsConf, DursMsg> for WS2Pv2Module {
    type ParseErr = WS2PFeaturesParseError;
    /// Parse raw api features
    fn parse_raw_api_features(str_features: &str) -> Result<ApiFeatures, Self::ParseErr> {
        let str_features: Vec<&str> = str_features.split(' ').collect();
        let mut api_features = Vec::with_capacity(0);
        for str_feature in str_features {
            match str_feature {
                "DEF" => api_features[0] += 1u8,
                "LOW" => api_features[0] += 2u8,
                "ABF" => api_features[0] += 4u8,
                _ => {
                    debug!(
                        "parse_raw_api_features() = UnknowApiFeature({})",
                        str_feature
                    );
                    return Err(WS2PFeaturesParseError::UnknowApiFeature(String::from(
                        str_feature,
                    )));
                }
            }
        }
        Ok(ApiFeatures(api_features))
    }
}

impl NetworkModule<DuRsConf, DursMsg> for WS2Pv2Module {
    fn sync(
        _soft_meta_datas: &SoftwareMetaDatas<DuRsConf>,
        _keys: RequiredKeysContent,
        _conf: WS2PConf,
        _main_sender: mpsc::Sender<RouterThreadMessage<DursMsg>>,
        _sync_params: SyncParams,
    ) -> Result<(), ModuleInitError> {
        unimplemented!()
    }
}

#[derive(StructOpt, Debug, Copy, Clone)]
#[structopt(
    name = "ws2p",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp")
)]
/// WS2P subcommand options
pub struct WS2POpt {}

impl DuniterModule<DuRsConf, DursMsg> for WS2Pv2Module {
    type ModuleConf = WS2PConf;
    type ModuleOpt = WS2POpt;

    fn name() -> ModuleStaticName {
        ModuleStaticName("ws2p")
    }
    fn priority() -> ModulePriority {
        ModulePriority::Essential()
    }
    fn ask_required_keys() -> RequiredKeys {
        RequiredKeys::NetworkKeyPair()
    }
    fn have_subcommand() -> bool {
        true
    }
    fn exec_subcommand(
        _soft_meta_datas: &SoftwareMetaDatas<DuRsConf>,
        _keys: RequiredKeysContent,
        _module_conf: Self::ModuleConf,
        _subcommand_args: WS2POpt,
    ) {
        println!("Succesfully exec ws2p subcommand !")
    }
    fn start(
        _soft_meta_datas: &SoftwareMetaDatas<DuRsConf>,
        _keys: RequiredKeysContent,
        _conf: WS2PConf,
        _router_sender: mpsc::Sender<RouterThreadMessage<DursMsg>>,
        _load_conf_only: bool,
    ) -> Result<(), ModuleInitError> {
        unimplemented!()
    }
}
