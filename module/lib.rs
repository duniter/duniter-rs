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
    missing_docs, missing_debug_implementations, missing_copy_implementations, trivial_casts,
    trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces,
    unused_qualifications
)]

extern crate duniter_crypto;
extern crate serde;
extern crate serde_json;

use duniter_crypto::keys::KeyPair;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::fmt::Debug;
use std::sync::mpsc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Store Currency
pub enum Currency {
    /// Currency in string format
    Str(String),
    /// Currency in binary format
    Bin([u8; 2]),
}

impl ToString for Currency {
    fn to_string(&self) -> String {
        match *self {
            Currency::Str(ref currency_str) => currency_str.clone(),
            Currency::Bin(_) => panic!("Currency binary format is not implemented !"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// Store module identifier
pub enum ModuleId {
    /// Module in static str format because module name must be know at compile time
    Str(&'static str),
    /// Module in binary format
    Bin([u8; 2]),
}

impl ToString for ModuleId {
    fn to_string(&self) -> String {
        match *self {
            ModuleId::Str(module_id_str) => String::from(module_id_str),
            ModuleId::Bin(_) => panic!("ModuleId binary format is not implemented !"),
        }
    }
}

impl Serialize for ModuleId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let module_id_string = match *self {
            ModuleId::Str(module_id_str) => String::from(module_id_str),
            ModuleId::Bin(_) => panic!("ModuleId binary format is not implemented !"),
        };
        serializer.serialize_str(module_id_string.as_str())
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
pub struct ModuleReqFullId(pub ModuleId, pub ModuleReqId);

impl ToString for ModuleReqFullId {
    fn to_string(&self) -> String {
        format!("{}-{}", self.0.to_string(), (self.1).0)
    }
}

/*impl Serialize for ModuleReqFullId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}-{}",  self.0.to_string(), (self.1).0))
    }
}*/

#[derive(Debug, Clone, PartialEq)]
/// Duniter configuration v1
pub struct DuniterConfV1 {
    /// Name of datas folder in ~/.config/durs/
    pub profile: String,
    /// Currency
    pub currency: Currency,
    /// Duniter node unique identifier
    pub my_node_id: u32,
    /// Configuration of modules in json format (obtained from the conf.json file)
    pub modules: serde_json::Value,
}

impl Serialize for DuniterConfV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("DuniterConfV1", 3)?;

        // Currency
        state.serialize_field("currency", self.currency.to_string().as_str())?;

        // Node id
        state.serialize_field("node_id", &format!("{:x}", self.my_node_id))?;

        // Modules
        state.serialize_field("modules", &self.modules)?;

        // End
        state.end()
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Duniter node configuration
pub enum DuniterConf {
    /// Duniter node configuration v1
    V1(DuniterConfV1),
    /// Duniter node configuration v2
    V2(),
}

impl DuniterConf {
    /// Get profile
    pub fn profile(&self) -> String {
        match *self {
            DuniterConf::V1(ref conf_v1) => conf_v1.profile.clone(),
            _ => panic!("Fail to load duniter conf : conf version not supported !"),
        }
    }
    /// Get currency
    pub fn currency(&self) -> Currency {
        match *self {
            DuniterConf::V1(ref conf_v1) => conf_v1.currency.clone(),
            _ => panic!("Fail to load duniter conf : conf version not supported !"),
        }
    }
    /// Get node id
    pub fn my_node_id(&self) -> u32 {
        match *self {
            DuniterConf::V1(ref conf_v1) => conf_v1.my_node_id,
            _ => panic!("Fail to load duniter conf : conf version not supported !"),
        }
    }
    /// Get modules conf
    pub fn modules(&self) -> serde_json::Value {
        match *self {
            DuniterConf::V1(ref conf_v1) => conf_v1.modules.clone(),
            _ => panic!("Fail to load duniter conf : conf version not supported !"),
        }
    }
}

/// The different modules of Duniter-rs can exchange messages with the type of their choice,
/// provided that this type implements the ModuleMessage trait.
pub trait ModuleMessage: Debug + Clone {}

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
    ModuleSender(mpsc::Sender<M>),
    /// When the number of plugged modules is known, the rooter thread must be informed of the number of modules it must connect between them.
    ModulesCount(usize),
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// Contains the keys the module needs
pub enum RequiredKeysContent<K: KeyPair> {
    /// Contains the member keypair (private key included).
    MemberKeyPair(Option<K>),
    /// Contains the member public key.
    MemberPublicKey(Option<K::PublicKey>),
    /// Contains the network keypair (private key included).
    NetworkKeyPair(K),
    /// Contains the network public key.
    NetworkPublicKey(K::PublicKey),
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

/// All Duniter-rs modules must implement this trait.
pub trait DuniterModule<K: KeyPair, M: ModuleMessage> {
    /// Returns the module identifier
    fn id() -> ModuleId;
    /// Returns the module priority
    fn priority() -> ModulePriority;
    /// Indicates which keys the module needs
    fn ask_required_keys() -> RequiredKeys;
    /// Provides the default module configuration
    fn default_conf() -> serde_json::Value;
    /// Launch the module
    fn start(
        soft_name: &str,
        soft_version: &str,
        keys: RequiredKeysContent<K>,
        conf: &DuniterConf,
        module_conf: &serde_json::Value,
        main_sender: mpsc::Sender<RooterThreadMessage<M>>,
        load_conf_only: bool,
    ) -> Result<(), ModuleInitError>;
}