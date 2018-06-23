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

#[macro_use]
extern crate serde_derive;

extern crate duniter_crypto;
extern crate serde;
extern crate serde_json;

use duniter_crypto::keys::{KeyPair, KeyPairEnum};
use serde::de::DeserializeOwned;
use serde::ser::{Serialize, Serializer};
use std::fmt::Debug;
use std::sync::mpsc;

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash, Serialize)]
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

#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Hash, Serialize)]
/// Store module identifier
pub struct ModuleId(pub String);

impl ToString for ModuleId {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Several modules can simultaneously send requests with the same identifier.
/// To identify each request in a unique way, we must therefore also take into account the identifier of the module performing the request.
pub struct ModuleReqFullId(pub ModuleId, pub ModuleReqId);

impl ToString for ModuleReqFullId {
    fn to_string(&self) -> String {
        format!("{}-{}", self.0.to_string(), (self.1).0)
    }
}

/// Duniter configuration
pub trait DuniterConf: Clone + Debug + Default + PartialEq + Serialize + DeserializeOwned {
    /// Get conf version profile
    fn version(&self) -> usize;
    /// Get currency
    fn currency(&self) -> Currency;
    /// Set currency
    fn set_currency(&mut self, new_currency: Currency);
    /// Get node id
    fn my_node_id(&self) -> u32;
    /// Get disabled modules
    fn disabled_modules(&self) -> Vec<ModuleId>;
    /// Get enabled modules
    fn enabled_modules(&self) -> Vec<ModuleId>;
    /// Get modules conf
    fn modules(&self) -> serde_json::Value;
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

/// All Duniter-rs modules must implement this trait.
pub trait DuniterModule<DC: DuniterConf, M: ModuleMessage> {
    /// Module configuration
    type ModuleConf: Clone + Debug + Default + DeserializeOwned + Send + Serialize + Sync;

    /// Returns the module identifier
    fn id() -> ModuleId;
    /// Returns the module priority
    fn priority() -> ModulePriority;
    /// Indicates which keys the module needs
    fn ask_required_keys() -> RequiredKeys;
    /// Launch the module
    fn start(
        soft_meta_datas: &SoftwareMetaDatas<DC>,
        keys: RequiredKeysContent,
        module_conf: Self::ModuleConf,
        main_sender: mpsc::Sender<RooterThreadMessage<M>>,
        load_conf_only: bool,
    ) -> Result<(), ModuleInitError>;
}
