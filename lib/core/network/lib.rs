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

//! Defined all aspects of the inter-node network that concern all modules and are therefore independent of one implementation or another of this network layer.

#![cfg_attr(feature = "strict", deny(warnings))]
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

extern crate dubp_documents;
extern crate duniter_module;
extern crate dup_crypto;
extern crate durs_network_documents;
extern crate serde;
extern crate serde_json;

use dubp_documents::v10::block::BlockDocument;
use dubp_documents::v10::certification::CertificationDocument;
use dubp_documents::v10::identity::IdentityDocument;
use dubp_documents::v10::membership::MembershipDocument;
use dubp_documents::v10::revocation::RevocationDocument;
use dubp_documents::v10::transaction::TransactionDocument;
use dubp_documents::Document;
use dubp_documents::{blockstamp::Blockstamp, BlockHash, BlockId};
use duniter_module::*;
use durs_network_documents::network_endpoint::ApiFeatures;
use durs_network_documents::network_head::NetworkHead;
use durs_network_documents::*;
use std::fmt::Debug;
use std::sync::mpsc;

pub mod documents;
pub mod events;
pub mod requests;

/// ApiModule
pub trait ApiModule<DC: DuniterConf, M: ModuleMessage>: DuniterModule<DC, M> {
    /// Parsing error
    type ParseErr;
    /// Parse raw api features
    fn parse_raw_api_features(str_features: &str) -> Result<ApiFeatures, Self::ParseErr>;
}

/// NetworkModule
pub trait NetworkModule<DC: DuniterConf, M: ModuleMessage>: ApiModule<DC, M> {
    /// Launch synchronisation
    fn sync(
        soft_meta_datas: &SoftwareMetaDatas<DC>,
        keys: RequiredKeysContent,
        module_conf: <Self as DuniterModule<DC, M>>::ModuleConf,
        main_sender: mpsc::Sender<RouterThreadMessage<M>>,
        sync_params: SyncParams,
    ) -> Result<(), ModuleInitError>;
}

/// SyncParams
#[derive(Debug, Clone)]
pub struct SyncParams {
    /// Synchronisation endpoint
    pub sync_endpoint: SyncEndpoint,
    /// Cautious flag
    pub cautious: bool,
    /// VERIF_HASHS flag
    pub verif_hashs: bool,
}

#[derive(Debug, Clone)]
/// Synchronisation endpoint
pub struct SyncEndpoint {
    /// Domaine name or IP
    pub domain_or_ip: String,
    /// Port number
    pub port: u16,
    /// Optionnal path
    pub path: Option<String>,
    /// Use TLS
    pub tls: bool,
}

/// Trait to be implemented by the configuration object of the module managing the inter-node network.
pub trait NetworkConf: Debug + Copy + Clone + PartialEq {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// Type returned when the network module fails to determine the current network consensus
pub enum NetworkConsensusError {
    /// The network module does not have enough data to determine consensus
    InsufficientData(usize),
    /// The network module does not determine consensus, there is most likely a fork
    Fork(),
}
