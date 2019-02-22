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

#[macro_use]
extern crate structopt;

use crate::cli::sync::SyncOpt;
use duniter_module::*;
use durs_network_documents::network_endpoint::ApiFeatures;
use durs_network_documents::network_head::NetworkHead;
use durs_network_documents::*;
use std::fmt::Debug;
use std::sync::mpsc;

pub mod cli;
pub mod documents;
pub mod events;
pub mod requests;

/// ApiModule
pub trait ApiModule<DC: DuniterConf, M: ModuleMessage>: DursModule<DC, M> {
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
        module_conf: <Self as DursModule<DC, M>>::ModuleConf,
        main_sender: mpsc::Sender<RouterThreadMessage<M>>,
        sync_params: SyncOpt,
    ) -> Result<(), ModuleInitError>;
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
