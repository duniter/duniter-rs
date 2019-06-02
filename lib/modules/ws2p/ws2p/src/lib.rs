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

mod constants;
pub mod controllers;
mod errors;
mod generate_peer;
pub mod services;

use crate::errors::WS2PError;
use dup_currency_params::CurrencyName;
use durs_common_tools::fatal_error;
use durs_common_tools::traits::merge::Merge;
use durs_conf::DuRsConf;
use durs_message::DursMsg;
use durs_module::*;
use durs_network::cli::sync::SyncOpt;
use durs_network::*;
use durs_network_documents::network_endpoint::*;
use maplit::hashset;
use std::sync::mpsc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// WS2P Configuration
pub struct WS2PConf {
    /// Limit of outcoming connections
    pub outcoming_quota: usize,
    /// Default WS2P endpoints provides by configuration file
    pub sync_endpoints: Vec<EndpointEnum>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// WS2P Configuration
pub struct WS2PUserConf {
    /// Limit of outcoming connections
    pub outcoming_quota: Option<usize>,
    /// Default WS2P endpoints provides by configuration file
    pub sync_endpoints: Option<Vec<EndpointEnum>>,
}

impl Merge for WS2PUserConf {
    fn merge(self, other: Self) -> Self {
        WS2PUserConf {
            outcoming_quota: self.outcoming_quota.or(other.outcoming_quota),
            sync_endpoints: self.sync_endpoints.or(other.sync_endpoints),
        }
    }
}

impl Default for WS2PConf {
    fn default() -> Self {
        WS2PConf {
            outcoming_quota: *constants::WS2P_DEFAULT_OUTCOMING_QUOTA,
            sync_endpoints: vec![
                EndpointV2::parse_from_raw("WS2P 2 g1.durs.info 443 ws2p").unwrap(),
                EndpointV2::parse_from_raw("WS2P 2 rs.g1.librelois.fr 443 ws2p").unwrap(),
            ],
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// WS2Pv2 Module
pub struct WS2PModule {}

impl Default for WS2PModule {
    fn default() -> WS2PModule {
        WS2PModule {}
    }
}

#[derive(Debug)]
/// WS2PFeaturesParseError
pub enum WS2PFeaturesParseError {
    /// UnknowApiFeature
    UnknowApiFeature(String),
}

impl ApiModule<DuRsConf, DursMsg> for WS2PModule {
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

impl NetworkModule<DuRsConf, DursMsg> for WS2PModule {
    fn sync(
        _soft_meta_datas: &SoftwareMetaDatas<DuRsConf>,
        _keys: RequiredKeysContent,
        _conf: WS2PConf,
        _main_sender: mpsc::Sender<RouterThreadMessage<DursMsg>>,
        _sync_params: SyncOpt,
    ) -> Result<(), SyncError> {
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

impl DursModule<DuRsConf, DursMsg> for WS2PModule {
    type ModuleUserConf = WS2PUserConf;
    type ModuleConf = WS2PConf;
    type ModuleOpt = WS2POpt;

    fn name() -> ModuleStaticName {
        ModuleStaticName(constants::MODULE_NAME)
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
    fn generate_module_conf(
        _currency_name: Option<&CurrencyName>,
        _global_conf: &<DuRsConf as DursConfTrait>::GlobalConf,
        module_user_conf: Option<Self::ModuleUserConf>,
    ) -> Result<(Self::ModuleConf, Option<Self::ModuleUserConf>), ModuleConfError> {
        let mut conf = WS2PConf::default();

        if let Some(module_user_conf) = module_user_conf.clone() {
            if let Some(outcoming_quota) = module_user_conf.outcoming_quota {
                conf.outcoming_quota = outcoming_quota;
            }
            if let Some(sync_endpoints) = module_user_conf.sync_endpoints {
                conf.sync_endpoints = sync_endpoints;
            }
        }

        Ok((conf, module_user_conf))
    }
    fn exec_subcommand(
        _soft_meta_datas: &SoftwareMetaDatas<DuRsConf>,
        _keys: RequiredKeysContent,
        _module_conf: Self::ModuleConf,
        _module_user_conf: Option<Self::ModuleUserConf>,
        _subcommand_args: WS2POpt,
    ) -> Option<Self::ModuleUserConf> {
        println!("Succesfully exec ws2p subcommand !");
        None
    }
    fn start(
        _soft_meta_datas: &SoftwareMetaDatas<DuRsConf>,
        keys: RequiredKeysContent,
        _conf: WS2PConf,
        router_sender: mpsc::Sender<RouterThreadMessage<DursMsg>>,
    ) -> Result<(), failure::Error> {
        // Get key_pair
        let _key_pair = if let RequiredKeysContent::NetworkKeyPair(key_pair) = keys {
            key_pair
        } else {
            return Err(WS2PError::UnexpectedKeys.into());
        };

        // Create module channel
        let (module_sender, module_receiver) = mpsc::channel();

        // Registration with the rooter
        if router_sender
            .send(RouterThreadMessage::ModuleRegistration {
                static_name: ModuleStaticName(constants::MODULE_NAME),
                sender: module_sender,
                roles: vec![ModuleRole::InterNodesNetwork],
                events_subscription: vec![
                    ModuleEvent::NewValidBlock,
                    ModuleEvent::NewWotDocInPool,
                    ModuleEvent::NewTxinPool,
                ],
                reserved_apis_parts: vec![ApiPart {
                    name: ApiName(constants::API_NAME.to_owned()),
                    versions: hashset![ApiVersion(2)],
                }],
                endpoints: vec![],
            })
            .is_err()
        {
            fatal_error!("WS2P module fail to send registration to router !")
        }

        while let Ok(msg) = module_receiver.recv() {
            if let DursMsg::Stop = msg {
                break;
            }
        }

        Ok(())
    }
}
