//  Copyright (C) 2018  The Duniter Project Developers.
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

//! Defined the few global types used by all modules,
//! as well as the DuniterModule trait that all modules must implement.

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

#[macro_use]
extern crate serde_derive;

extern crate duniter_crypto;
extern crate duniter_documents;
extern crate serde;
extern crate serde_json;
extern crate structopt;

use duniter_crypto::keys::{KeyPair, KeyPairEnum};
use duniter_documents::CurrencyName;
use serde::de::DeserializeOwned;
use serde::ser::{Serialize, Serializer};
use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::mpsc;
//use structopt::clap::ArgMatches;
use structopt::StructOpt;

#[derive(Copy, Clone, Deserialize, Debug, PartialEq, Eq, Hash, Serialize)]
/// Store module name in static lifetime
pub struct ModuleStaticName(pub &'static str);

impl ToString for ModuleStaticName {
    fn to_string(&self) -> String {
        String::from(self.0)
    }
}

#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Hash, Serialize)]
/// Store module name
pub struct ModuleName(pub String);

impl From<ModuleStaticName> for ModuleName {
    fn from(source: ModuleStaticName) -> Self {
        ModuleName(String::from(source.0))
    }
}

impl<'a> From<&'a str> for ModuleName {
    fn from(source: &str) -> Self {
        ModuleName(String::from(source))
    }
}

impl ToString for ModuleName {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// Identifier of an inter-module request
pub struct ModuleReqId(pub u32);

impl Serialize for ModuleReqId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{:x}", self.0))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// Several modules can simultaneously send requests with the same identifier.
/// To identify each request in a unique way, we must therefore also take into account the identifier of the module performing the request.
pub struct ModuleReqFullId(pub ModuleStaticName, pub ModuleReqId);

impl ToString for ModuleReqFullId {
    fn to_string(&self) -> String {
        format!("{}-{}", self.0.to_string(), (self.1).0)
    }
}

/// Duniter configuration
pub trait DuniterConf:
    Clone + Debug + Default + PartialEq + Serialize + DeserializeOwned + Send + ToOwned
{
    /// Get conf version profile
    fn version(&self) -> usize;
    /// Get currency
    fn currency(&self) -> CurrencyName;
    /// Set currency
    fn set_currency(&mut self, new_currency: CurrencyName);
    /// Get node id
    fn my_node_id(&self) -> u32;
    /// Disable a module
    fn disable(&mut self, module: ModuleName);
    /// Enable a module
    fn enable(&mut self, module: ModuleName);
    /// Get disabled modules
    fn disabled_modules(&self) -> HashSet<ModuleName>;
    /// Get enabled modules
    fn enabled_modules(&self) -> HashSet<ModuleName>;
    /// Get modules conf
    fn modules(&self) -> serde_json::Value;
    /// Change module conf
    fn set_module_conf(&mut self, module_id: String, new_module_conf: serde_json::Value);
}

/// Sofware meta datas
#[derive(Debug, Clone)]
pub struct SoftwareMetaDatas<DC: DuniterConf> {
    /// Software name
    pub soft_name: &'static str,
    /// Software version
    pub soft_version: &'static str,
    /// User profile
    pub profile: String,
    /// User configuration
    pub conf: DC,
}

/// The different modules of Duniter-rs can exchange messages with the type of their choice,
/// provided that this type implements the ModuleMessage trait.
pub trait ModuleMessage: Clone + Debug {}

/// List of the different roles that can be assigned to a module.
/// This role list allows a module to send a message to all modules playing a specific role without knowing their name.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ModuleRole {
    /// Manages the blockchain data (include forks datas)
    BlockchainDatas,
    /// Checks if a block complies with the entire blockchain protocol
    BlockValidation,
    /// Generates the content of the next block
    BlockGeneration,
    /// Change configuration file
    ChangeConf,
    /// Communicates with client software
    ClientsNetwork,
    /// Manage pending data for the currency (transactions and scripts)
    CurrencyPool,
    /// Manages the network between nodes implementing the DUP protocol
    InterNodesNetwork,
    /// Communicates with the node user
    UserInterface,
    /// Manage pending data for the wot
    WotPool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
///List of the different types of events that can be generated by a module.
/// This list allows the different modules to subscribe only to the types of events that interest them
pub enum ModuleEvent {
    /// A new block has been received from the network
    NewBlockFromNetwork,
    /// A new transaction has been received from a client software.
    NewTxFromNetwork,
    /// A new wot document has been received from a network.
    NewWotDocFromNetwork,
    /// A new valid block has been added to the local blockchain
    NewValidBlock,
    /// A new valid block issued by the local node has been added to the local blockchain
    NewValidBlockFromSelf,
    /// A new non-isolated fork is in the local database
    NewFork,
    /// Blockchain rooling back
    RevertBlocks,
    /// A new transaction has been integrated into the local waiting room
    NewTxinPool,
    /// A new wot document has been integrated into the local waiting room
    NewWotDocInPool,
    /// A new valid HEAD has been received from the network
    NewValidHeadFromNetwork,
    /// Change in connections with other nodes (disconnection of a connection or establishment of a new connection)
    ConnectionsChangeNodeNetwork,
    /// A new valid peer record has been received from the network
    NewValidPeerFromNodeNetwork,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// Type returned by module initialization function
pub enum ModuleInitError {
    /// Fail to load configuration
    FailToLoadConf(),
    /// Unknow error
    UnknowError(),
}

#[derive(Debug, Clone)]
/// Type sent by each module to the rooter during initialization
pub enum RooterThreadMessage<M: ModuleMessage> {
    /// Channel on which the module listens
    ModuleSender(
        ModuleStaticName,
        mpsc::Sender<M>,
        Vec<ModuleRole>,
        Vec<ModuleEvent>,
    ),
    /// Module message
    ModuleMessage(M),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// Indicates which keys the module needs to operate
pub enum RequiredKeys {
    /// The module needs the member keypair (private key included).
    MemberKeyPair(),
    /// The module only needs the member public key.
    MemberPublicKey(),
    /// The module needs the network keypair (private key included).
    NetworkKeyPair(),
    /// The module only needs the network public key.
    NetworkPublicKey(),
    /// The module does not need any key
    None(),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Contains the keys the module needs
pub enum RequiredKeysContent {
    /// Contains the member keypair (private key included).
    MemberKeyPair(Option<KeyPairEnum>),
    /// Contains the member public key.
    MemberPublicKey(Option<<KeyPairEnum as KeyPair>::PublicKey>),
    /// Contains the network keypair (private key included).
    NetworkKeyPair(KeyPairEnum),
    /// Contains the network public key.
    NetworkPublicKey(<KeyPairEnum as KeyPair>::PublicKey),
    /// Does not contain any keys
    None(),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// Defined the priority level of the module
pub enum ModulePriority {
    /// This module is necessary for Duniter-Rs to work properly, impossible to disable it.
    Essential(),
    /// This module is recommended but it's not essential, it's enabled by default but can be disabled by the user.
    Recommended(),
    /// This module is disabled by default, it must be explicitly enabled by the user.
    Optional(),
}

/// Determines if a module is activated or not
pub fn enabled<DC: DuniterConf, Mess: ModuleMessage, M: DuniterModule<DC, Mess>>(
    conf: &DC,
) -> bool {
    let disabled_modules = conf.disabled_modules();
    let enabled_modules = conf.enabled_modules();
    match M::priority() {
        ModulePriority::Essential() => true,
        ModulePriority::Recommended() => !disabled_modules.contains(&ModuleName::from(M::name())),
        ModulePriority::Optional() => enabled_modules.contains(&ModuleName::from(M::name())),
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// Modules filter
/// If bool = false, the meaning of the filter is reversed.
pub enum ModulesFilter {
    /// Enabled modules
    Enabled(bool),
    /// Network modules
    Network(),
    /// Modules that require member private key
    RequireMemberPrivKey(),
}

/// Returns true only if the module checks all filters
pub fn module_valid_filters<
    DC: DuniterConf,
    Mess: ModuleMessage,
    M: DuniterModule<DC, Mess>,
    S: ::std::hash::BuildHasher,
>(
    conf: &DC,
    filters: &HashSet<ModulesFilter, S>,
    network_module: bool,
) -> bool {
    if filters.contains(&ModulesFilter::Network()) && !network_module {
        return false;
    }
    if filters.contains(&ModulesFilter::RequireMemberPrivKey())
        && M::ask_required_keys() != RequiredKeys::MemberKeyPair()
    {
        return false;
    }
    if filters.contains(&ModulesFilter::Enabled(true)) && !enabled::<DC, Mess, M>(conf) {
        return false;
    }
    if filters.contains(&ModulesFilter::Enabled(false)) && enabled::<DC, Mess, M>(conf) {
        return false;
    }
    true
}

/// All Duniter-rs modules must implement this trait.
pub trait DuniterModule<DC: DuniterConf, M: ModuleMessage> {
    /// Module configuration
    type ModuleConf: Clone + Debug + Default + DeserializeOwned + Send + Serialize + Sync;
    /// Module subcommand options
    type ModuleOpt: StructOpt;

    /// Returns the module name
    fn name() -> ModuleStaticName;
    /// Returns the module priority
    fn priority() -> ModulePriority;
    /// Indicates which keys the module needs
    fn ask_required_keys() -> RequiredKeys;
    /// Define if module have a cli subcommand
    fn have_subcommand() -> bool {
        false
    }
    /// Execute injected subcommand
    fn exec_subcommand(
        soft_meta_datas: &SoftwareMetaDatas<DC>,
        keys: RequiredKeysContent,
        module_conf: Self::ModuleConf,
        subcommand_args: Self::ModuleOpt,
    ) -> ();
    /// Launch the module
    fn start(
        soft_meta_datas: &SoftwareMetaDatas<DC>,
        keys: RequiredKeysContent,
        module_conf: Self::ModuleConf,
        main_sender: mpsc::Sender<RooterThreadMessage<M>>,
        load_conf_only: bool,
    ) -> Result<(), ModuleInitError>;
}
