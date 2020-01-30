//  Copyright (C) 2017-2019  The AXIOM TEAM Association.
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

//! Gva Module
//! This module provides a graphql API implementation of the 0003 RFC
//!
//! /src/schema.gql contains schema description
//! /src/schema.rs contains model and resolvers implementation
//! /src/webserver.rs contains web server implementaion based on actix-web
//!
//! Graphiql web client is accessible at
//! http://127.0.0.1:10901/graphiql

#![deny(
    clippy::option_unwrap_used,
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
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

extern crate juniper;

mod constants;
mod context;
mod db;
mod errors;
mod graphql;
mod schema;
mod webserver;

use crate::errors::GvaError;
use dubp_currency_params::CurrencyName;
use durs_common_tools::fatal_error;
use durs_common_tools::traits::merge::Merge;
use durs_conf::DuRsConf;
use durs_message::events::{BlockchainEvent, DursEvent};
use durs_message::DursMsg;
use durs_module::{
    DursConfTrait, DursModule, ModuleConfError, ModuleEvent, ModulePriority, ModuleRole,
    ModuleStaticName, RequiredKeys, RequiredKeysContent, RouterThreadMessage, SoftwareMetaDatas,
};

use durs_network::events::NetworkEvent;
use durs_network_documents::host::Host;

use std::ops::Deref;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime};

static MODULE_NAME: &str = "gva";

static DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 10_901;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Gva Module Configuration
pub struct GvaConf {
    host: String,
    port: u16,
}

impl Default for GvaConf {
    fn default() -> Self {
        GvaConf {
            host: DEFAULT_HOST.to_owned(),
            port: DEFAULT_PORT,
        }
    }
}

impl std::fmt::Display for GvaConf {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "host: {}\nport: {}", self.host, self.port,)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Gva user Configuration
pub struct GvaUserConf {
    host: Option<String>,
    port: Option<u16>,
}

impl Merge for GvaUserConf {
    fn merge(self, other: Self) -> Self {
        GvaUserConf {
            host: self.host.or(other.host),
            port: self.port.or(other.port),
        }
    }
}

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "gva", setting(structopt::clap::AppSettings::ColoredHelp))]
/// Gva subcommand options
pub struct GvaOpt {
    /// Change GVA API host listen
    #[structopt(long = "host", parse(try_from_str = Host::parse))]
    pub host: Option<Host>,
    #[structopt(long = "port")]
    /// Change GVA API port listen
    pub port: Option<u16>,
}

#[derive(Debug, Copy, Clone)]
/// Data that the Gva module needs to cache
pub struct GvaModuleDatas {}

#[derive(Debug, Copy, Clone)]
/// Gva module
pub struct GvaModule {}

impl Default for GvaModule {
    fn default() -> GvaModule {
        GvaModule {}
    }
}

impl DursModule<DuRsConf, DursMsg> for GvaModule {
    type ModuleConf = GvaConf;
    type ModuleUserConf = GvaUserConf;
    type ModuleOpt = GvaOpt;

    fn name() -> ModuleStaticName {
        ModuleStaticName(MODULE_NAME)
    }
    fn priority() -> ModulePriority {
        ModulePriority::Recommended
    }
    fn ask_required_keys() -> RequiredKeys {
        RequiredKeys::None
    }
    fn have_subcommand() -> bool {
        false
    }
    fn generate_module_conf(
        _currency_name: Option<&CurrencyName>,
        _global_conf: &<DuRsConf as DursConfTrait>::GlobalConf,
        module_user_conf: Option<Self::ModuleUserConf>,
    ) -> Result<(Self::ModuleConf, Option<Self::ModuleUserConf>), ModuleConfError> {
        let mut conf = GvaConf::default();

        if let Some(ref module_user_conf) = module_user_conf {
            if let Some(ref host) = module_user_conf.host {
                conf.host = host.to_owned();
            }
            if let Some(port) = module_user_conf.port {
                conf.port = port;
            }
        }

        Ok((conf, module_user_conf))
    }
    fn exec_subcommand(
        soft_meta_datas: &SoftwareMetaDatas<DuRsConf>,
        _keys: RequiredKeysContent,
        _module_conf: Self::ModuleConf,
        module_user_conf: Option<Self::ModuleUserConf>,
        subcommand_args: Self::ModuleOpt,
    ) -> Option<Self::ModuleUserConf> {
        let new_gva_user_conf = GvaUserConf {
            host: subcommand_args.host.map(|h| h.to_string()),
            port: subcommand_args.port,
        }
        .merge(module_user_conf.unwrap_or_default());
        match Self::generate_module_conf(
            Some(&soft_meta_datas.conf.get_currency()),
            &soft_meta_datas.conf.get_global_conf(),
            Some(new_gva_user_conf.clone()),
        ) {
            Ok((new_gva_conf, _)) => println!("New GVA configuration:\n{}", new_gva_conf),
            Err(e) => println!("Fail to change GVA confguration : {:?}", e),
        }

        Some(new_gva_user_conf)
    }
    fn start(
        soft_meta_datas: &SoftwareMetaDatas<DuRsConf>,
        _keys: RequiredKeysContent,
        conf: Self::ModuleConf,
        router_sender: mpsc::Sender<RouterThreadMessage<DursMsg>>,
    ) -> Result<(), failure::Error> {
        let _start_time = SystemTime::now();

        // Check conf validity
        let host = Host::parse(&conf.host).map_err(|_| GvaError::InvalidHost)?;

        // Instanciate Gva module datas
        let _datas = GvaModuleDatas {};

        // Create gva main thread channel
        let (gva_sender, gva_receiver): (mpsc::Sender<DursMsg>, mpsc::Receiver<DursMsg>) =
            mpsc::channel();

        // Send gva module registration to router thread
        router_sender
            .send(RouterThreadMessage::ModuleRegistration {
                static_name: ModuleStaticName(MODULE_NAME),
                sender: gva_sender, // Messages sent by the router will be received by your proxy thread
                roles: vec![ModuleRole::UserInterface], // Roles assigned to your module
                events_subscription: vec![ModuleEvent::NewValidBlock], // Events to which your module subscribes
                reserved_apis_parts: vec![],
                endpoints: vec![],
            })
            .expect("Fatal error : gva module fail to register to router !"); // The registration of this module must be successful, in case of failure the program must be interrupted.

        // If we are here it means that this module has successfully registered,
        // we indicate it in the debug level log, it can be helpful.
        debug!("Send gva module registration to router thread.");

        let smd: SoftwareMetaDatas<DuRsConf> = soft_meta_datas.clone();
        let router_sender_clone = router_sender.clone();
        let _webserver_thread = thread::spawn(move || {
            if let Err(e) = webserver::start_web_server(&smd, host, conf.port) {
                error!("GVA http web server error  : {}  ", e);
            } else {
                info!("GVA http web server stop.")
            }
            let _result =
                router_sender_clone.send(RouterThreadMessage::ModuleMessage(DursMsg::Stop));
        });

        /*
         * Main loop of your module
         */
        loop {
            // Get messages
            match gva_receiver.recv_timeout(Duration::from_millis(250)) {
                Ok(durs_message) => match durs_message {
                    DursMsg::Stop => {
                        // Relay stop signal to router
                        let _result =
                            router_sender.send(RouterThreadMessage::ModuleMessage(DursMsg::Stop));
                        // Break main loop
                        break;
                    }
                    DursMsg::Event {
                        ref event_content, ..
                    } => match *event_content {
                        DursEvent::BlockchainEvent(ref blockchain_event) => {
                            match *blockchain_event.deref() {
                                BlockchainEvent::StackUpValidBlock(ref _block) => {
                                    // Do something when the node has stacked a new block at its local blockchain
                                }
                                BlockchainEvent::RevertBlocks(ref _blocks) => {
                                    // Do something when the node has destacked blocks from its local blockchain (roll back)
                                }
                                _ => {} // Do nothing for events that don't concern this module.
                            }
                        }
                        DursEvent::NetworkEvent(ref network_event_box) => {
                            match *network_event_box.deref() {
                                NetworkEvent::ReceivePeers(ref _peers) => {
                                    // Do something when the node receive peers cards from network
                                }
                                NetworkEvent::ReceiveDocuments(ref _bc_documents) => {
                                    // Do something when the node receive blockchain documents from network
                                }
                                _ => {} // Do nothing for events that don't concern this module.
                            }
                        }
                        _ => {} // Do nothing for DursEvent variants that don't concern this module.
                    },
                    _ => {} // Do nothing for DursMsgContent variants that don't concern this module.
                },
                Err(e) => match e {
                    mpsc::RecvTimeoutError::Disconnected => {
                        fatal_error!("Disconnected gva module !");
                    }
                    mpsc::RecvTimeoutError::Timeout => {
                        // If you arrive here it's because this main thread did not receive anything at the end of the timeout.
                        // This is quite normal and happens regularly when there is little activity, there is nothing particular to do.
                    }
                },
            }
        }
        // If we reach this point it means that the module has stopped correctly, so we return OK.
        Ok(())
    }
}
