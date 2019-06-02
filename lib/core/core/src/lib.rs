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

//! Crate containing Durs core.

//#![cfg_attr(feature = "cargo-clippy", allow(implicit_hasher))]
#![deny(
    missing_docs,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate structopt;

mod change_conf;
pub mod commands;
mod constants;
pub mod errors;
mod logger;
mod router;

use crate::commands::*;
use crate::constants::DEFAULT_USER_PROFILE;
use crate::errors::DursCoreError;
use dup_currency_params::CurrencyName;
use durs_blockchain::{BlockchainModule, DBExQuery};
use durs_common_tools::fatal_error;
pub use durs_conf::{
    constants::KEYPAIRS_FILENAME, keys::*, ChangeGlobalConf, DuRsConf, DuniterKeyPairs,
};
use durs_message::*;
use durs_module::*;
use durs_network::cli::sync::*;
use durs_network::NetworkModule;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use unwrap::unwrap;

#[macro_export]
/// Plug modules in durs core
macro_rules! durs_plug {
    ( [ $( $NetworkModule:ty ),* ], [ $( $Module:ty ),* ] ) => {
        {
            |core| {
                $(core.plug::<$Module>()?;)*
                $(core.plug_network::<$NetworkModule>()?;)*
                Ok(())
            }
        }
    };
}

/// Durs Core Datas
pub struct DursCore<DC: DursConfTrait> {
    /// Currency name
    pub currency_name: Option<CurrencyName>,
    /// Durs core options
    pub options: DursCoreOptions,
    /// Does the entered command require to launch server ?
    server_command: Option<ServerMode>,
    /// Software meta datas
    pub soft_meta_datas: SoftwareMetaDatas<DC>,
    /// Keypairs
    pub keypairs: DuniterKeyPairs,
    /// Run duration. Zero = infinite duration.
    pub run_duration_in_secs: u64,
    /// Sender channel of router thread
    pub router_sender: Option<mpsc::Sender<RouterThreadMessage<DursMsg>>>,
    ///  Count the number of plugged network modules
    pub network_modules_count: usize,
    /// Modules names
    pub modules_names: Vec<ModuleStaticName>,
    /// Threads handlers that execute plugged modules
    pub threads: HashMap<ModuleStaticName, thread::JoinHandle<()>>,
}

#[derive(Debug, Clone)]
/// Server command
enum ServerMode {
    /// Start
    Start(),
    /// Sync (SyncEndpoint)
    Sync(SyncOpt),
    /// List modules
    ListModules(ListModulesOpt),
}

impl DursCore<DuRsConf> {
    /// Execute module command
    pub fn execute_module_command<M: DursModule<DuRsConf, DursMsg>>(
        durs_core_opts: DursCoreOptions,
        module_command: M::ModuleOpt,
        soft_name: &'static str,
        soft_version: &'static str,
    ) -> Result<(), DursCoreError> {
        let mut durs_core = DursCore::<DuRsConf>::init(soft_name, soft_version, durs_core_opts, 0)?;
        // Load module conf and keys
        let module_conf_json = durs_core
            .soft_meta_datas
            .conf
            .clone()
            .modules()
            .get(&M::name().to_string().as_str())
            .cloned();

        let ((module_conf, module_user_conf), required_keys) = get_module_conf_and_keys::<M>(
            durs_core.currency_name.as_ref(),
            &durs_core.soft_meta_datas.conf.get_global_conf(),
            module_conf_json,
            durs_core.keypairs,
        )
        .map_err(|e| DursCoreError::PlugModuleError {
            module_name: M::name(),
            error: e.into(),
        })?;
        // Execute module subcommand
        let new_module_conf = M::exec_subcommand(
            &durs_core.soft_meta_datas,
            required_keys,
            module_conf,
            module_user_conf,
            module_command,
        );

        durs_conf::write_new_module_conf(
            &mut durs_core.soft_meta_datas.conf,
            durs_core.soft_meta_datas.profile_path.clone(),
            M::name().into(),
            unwrap!(serde_json::value::to_value(new_module_conf)),
        );

        Ok(())
    }

    /// Execute core command
    pub fn execute_core_command<PlugFunc>(
        core_command: DursCoreCommand,
        durs_core_opts: DursCoreOptions,
        external_followers: Vec<mpsc::Sender<DursMsg>>,
        mut plug_modules: PlugFunc,
        soft_name: &'static str,
        soft_version: &'static str,
    ) -> Result<(), DursCoreError>
    where
        PlugFunc: FnMut(&mut DursCore<DuRsConf>) -> Result<(), DursCoreError>,
    {
        // Instantiate durs core
        let mut durs_core = DursCore::<DuRsConf>::init(soft_name, soft_version, durs_core_opts, 0)?;

        let profile_path = durs_core.soft_meta_datas.profile_path.clone();

        /*
         * CORE COMMAND PROCESSING
         */
        match core_command {
            DursCoreCommand::DisableOpt(opts) => opts.execute(durs_core),
            DursCoreCommand::EnableOpt(opts) => opts.execute(durs_core),
            DursCoreCommand::ListModulesOpt(opts) => {
                durs_core.server_command = Some(ServerMode::ListModules(opts));

                durs_core.router_sender = Some(router::start_router(
                    0,
                    profile_path.clone(),
                    durs_core.soft_meta_datas.conf.clone(),
                    vec![],
                ));
                plug_modules(&mut durs_core)
            }
            DursCoreCommand::StartOpt(_opts) => {
                durs_core.server_command = Some(ServerMode::Start());

                durs_core.router_sender = Some(router::start_router(
                    durs_core.run_duration_in_secs,
                    profile_path.clone(),
                    durs_core.soft_meta_datas.conf.clone(),
                    external_followers,
                ));
                plug_modules(&mut durs_core)?;
                durs_core.start()
            }
            DursCoreCommand::SyncOpt(opts) => match opts.source_type {
                SyncSourceType::Network => {
                    durs_core.server_command = Some(ServerMode::Sync(opts));

                    durs_core.router_sender = Some(router::start_router(
                        durs_core.run_duration_in_secs,
                        profile_path.clone(),
                        durs_core.soft_meta_datas.conf.clone(),
                        external_followers,
                    ));
                    plug_modules(&mut durs_core)?;
                    durs_core.start()
                }
                SyncSourceType::LocalDuniter => {
                    sync_ts(profile_path.clone(), &durs_core.soft_meta_datas.conf, opts);
                    Ok(())
                }
            },
            DursCoreCommand::DbExOpt(opts) => opts.execute(durs_core),
            DursCoreCommand::ResetOpt(opts) => opts.execute(durs_core),
            DursCoreCommand::KeysOpt(opts) => opts.execute(durs_core),
        }
    }
    /// Initialize Durs core
    fn init(
        soft_name: &'static str,
        soft_version: &'static str,
        durs_core_opts: DursCoreOptions,
        run_duration_in_secs: u64,
    ) -> Result<DursCore<DuRsConf>, DursCoreError> {
        // get profile path
        let profile_path = durs_conf::get_profile_path(
            &durs_core_opts.profiles_path,
            &durs_core_opts
                .profile_name
                .clone()
                .unwrap_or_else(|| DEFAULT_USER_PROFILE.to_owned()),
        );

        // Init logger
        logger::init(
            profile_path.clone(),
            soft_name,
            soft_version,
            &durs_core_opts,
        )?;

        // Load global conf
        let (conf, keypairs) =
            durs_conf::load_conf(profile_path.clone(), &durs_core_opts.keypairs_file)
                .map_err(DursCoreError::ConfFileError)?;
        info!("Success to load global conf.");

        // Get currency name
        let currency_name = dup_currency_params::db::get_currency_name(durs_conf::get_datas_path(
            profile_path.clone(),
        ))
        .map_err(DursCoreError::FailReadCurrencyParamsDb)?;

        // Instanciate durs core
        Ok(DursCore {
            currency_name,
            keypairs,
            options: durs_core_opts,
            modules_names: Vec::new(),
            network_modules_count: 0,
            router_sender: None,
            run_duration_in_secs,
            server_command: None,
            soft_meta_datas: SoftwareMetaDatas {
                conf,
                profile_path,
                soft_name,
                soft_version,
            },
            threads: HashMap::new(),
        })
    }
    /// Start durs server
    pub fn start(mut self) -> Result<(), DursCoreError> {
        if self.network_modules_count == 0 {
            fatal_error!(
                "Dev error: no network module found: you must plug at least one network module !"
            );
        }

        // Create blockchain module channel
        let (blockchain_sender, blockchain_receiver): (
            mpsc::Sender<DursMsg>,
            mpsc::Receiver<DursMsg>,
        ) = mpsc::channel();

        let router_sender = if let Some(ref router_sender) = self.router_sender {
            router_sender
        } else {
            fatal_error!("Dev error: try to start core without router_sender !");
        };

        // Send expected modules count to router thread
        router_sender
            .send(RouterThreadMessage::ModulesCount(
                self.modules_names.len() + 1,
            ))
            .expect("Fatal error: fail to send expected modules count to router thread !");

        // Send blockchain module registration to router thread
        router_sender
            .send(RouterThreadMessage::ModuleRegistration {
                static_name: BlockchainModule::name(),
                sender: blockchain_sender,
                roles: vec![ModuleRole::BlockchainDatas, ModuleRole::BlockValidation],
                events_subscription: vec![ModuleEvent::NewBlockFromNetwork, ModuleEvent::SyncEvent],
                reserved_apis_parts: vec![],
                endpoints: vec![],
            })
            .expect("Fatal error: fail to send blockchain registration to router thread !");

        // Get profile path
        let profile_path = self.soft_meta_datas.profile_path;

        // Instantiate blockchain module and load is conf
        let mut blockchain_module = BlockchainModule::load_blockchain_conf(
            router_sender.clone(),
            profile_path,
            RequiredKeysContent::MemberKeyPair(None),
        );
        info!("Success to load Blockchain module.");

        // Start blockchain module in thread
        let thread_builder = thread::Builder::new().name(BlockchainModule::name().0.into());
        let sync_opts = if let Some(ServerMode::Sync(opts)) = self.server_command {
            Some(opts)
        } else {
            None
        };
        let blockchain_thread_handler = thread_builder
            .spawn(move || blockchain_module.start_blockchain(&blockchain_receiver, sync_opts))
            .expect("Fatal error: fail to spawn module main thread !");

        // Wait until all modules threads are finished
        for module_static_name in &self.modules_names {
            if let Some(module_thread_handler) = self.threads.remove(module_static_name) {
                if let Err(err) = module_thread_handler.join() {
                    error!("'{}' module thread panic : {:?}", module_static_name.0, err);
                }
            }
        }

        // Wait until blockchain main thread finished
        if let Err(err) = blockchain_thread_handler.join() {
            error!("'blockchain' thread panic : {:?}", err);
        }

        Ok(())
    }
    #[inline]
    /// Plug a network module
    pub fn plug_network<NM: NetworkModule<DuRsConf, DursMsg>>(
        &mut self,
    ) -> Result<(), DursCoreError> {
        self.plug_network_::<NM>()
            .map_err(|error| DursCoreError::PlugModuleError {
                module_name: NM::name(),
                error,
            })
    }
    fn plug_network_<NM: NetworkModule<DuRsConf, DursMsg>>(
        &mut self,
    ) -> Result<(), PlugModuleError> {
        let enabled = enabled::<DuRsConf, DursMsg, NM>(&self.soft_meta_datas.conf);
        if enabled {
            self.network_modules_count += 1;
            if let Some(ServerMode::Sync(ref network_sync)) = self.server_command {
                let sync_module_name =
                    if let Some(ref sync_module_name) = network_sync.sync_module_name {
                        sync_module_name.clone()
                    } else {
                        self.soft_meta_datas
                            .conf
                            .get_global_conf()
                            .default_sync_module()
                            .0
                    };

                if NM::name().0 == sync_module_name {
                    // Start module in a new thread
                    let router_sender = self
                        .router_sender
                        .clone()
                        .expect("Try to start a core without router_sender !");
                    let soft_meta_datas = self.soft_meta_datas.clone();
                    let module_conf_json = self
                        .soft_meta_datas
                        .conf
                        .clone()
                        .modules()
                        .get(&NM::name().to_string().as_str())
                        .cloned();
                    let keypairs = self.keypairs;

                    // Load module conf and keys
                    let ((module_conf, _), required_keys) = get_module_conf_and_keys::<NM>(
                        self.currency_name.as_ref(),
                        &soft_meta_datas.conf.get_global_conf(),
                        module_conf_json,
                        keypairs,
                    )?;

                    let sync_params = network_sync.clone();
                    let thread_builder = thread::Builder::new().name(NM::name().0.into());
                    self.threads.insert(
                        NM::name(),
                        thread_builder
                            .spawn(move || {
                                NM::sync(
                                    &soft_meta_datas,
                                    required_keys,
                                    module_conf,
                                    router_sender,
                                    sync_params,
                                )
                                .unwrap_or_else(|_| {
                                    fatal_error!(
                                        "Fatal error : fail to load module '{}' !",
                                        NM::name().to_string()
                                    )
                                });
                            })
                            .map_err(|e| PlugModuleError::FailSpawnModuleThread {
                                module_name: NM::name(),
                                error: e,
                            })?,
                    );
                    self.modules_names.push(NM::name());
                    info!("Success to load {} module.", NM::name().to_string());
                    Ok(())
                } else {
                    debug!("Module '{}' not used for sync.", NM::name());
                    Ok(())
                }
            } else {
                self.plug_::<NM>(true)
            }
        } else {
            self.plug_::<NM>(true)
        }
    }
    #[inline]
    /// Plug a module
    pub fn plug<M: DursModule<DuRsConf, DursMsg>>(&mut self) -> Result<(), DursCoreError> {
        self.plug_::<M>(false)
            .map_err(|error| DursCoreError::PlugModuleError {
                module_name: M::name(),
                error,
            })
    }

    /// Plug a module
    pub fn plug_<M: DursModule<DuRsConf, DursMsg>>(
        &mut self,
        is_network_module: bool,
    ) -> Result<(), PlugModuleError> {
        let enabled = enabled::<DuRsConf, DursMsg, M>(&self.soft_meta_datas.conf);
        if enabled {
            let (launch_module, sync_opts) = match self.server_command {
                Some(ServerMode::Start()) => (true, None),
                Some(ServerMode::Sync(ref opts)) => (M::launchable_as_sync(), Some(opts.clone())),
                Some(_) | None => (false, None),
            };

            if launch_module {
                // Start module in a new thread
                let router_sender_clone = self
                    .router_sender
                    .clone()
                    .expect("Try to start a core without router_sender !");
                let soft_meta_datas = self.soft_meta_datas.clone();
                let module_conf_json = self
                    .soft_meta_datas
                    .conf
                    .clone()
                    .modules()
                    .get(&M::name().to_string().as_str())
                    .cloned();
                let keypairs = self.keypairs;
                // Load module conf and keys
                let ((module_conf, _), required_keys) = get_module_conf_and_keys::<M>(
                    self.currency_name.as_ref(),
                    &soft_meta_datas.conf.get_global_conf(),
                    module_conf_json,
                    keypairs,
                )?;

                let thread_builder = thread::Builder::new().name(M::name().0.into());
                self.threads.insert(
                    M::name(),
                    thread_builder
                        .spawn(move || {
                            if let Some(sync_opts) = sync_opts {
                                M::start_at_sync(
                                    &soft_meta_datas,
                                    required_keys,
                                    module_conf,
                                    router_sender_clone,
                                    sync_opts.cautious_mode,
                                    sync_opts.unsafe_mode,
                                )
                                .unwrap_or_else(|e| fatal_error!("Module '{}': {}", M::name(), e));
                            } else {
                                M::start(
                                    &soft_meta_datas,
                                    required_keys,
                                    module_conf,
                                    router_sender_clone,
                                )
                                .unwrap_or_else(|e| fatal_error!("Module '{}': {}", M::name(), e));
                            }
                        })
                        .map_err(|e| PlugModuleError::FailSpawnModuleThread {
                            module_name: M::name(),
                            error: e,
                        })?,
                );
                self.modules_names.push(M::name());
                info!("Success to load {} module.", M::name().to_string());
            }
        }
        if let Some(ServerMode::ListModules(ref options)) = self.server_command {
            if module_valid_filters::<DuRsConf, DursMsg, M, std::collections::hash_map::RandomState>(
                &self.soft_meta_datas.conf,
                &options.get_filters(),
                is_network_module,
            ) {
                if enabled {
                    println!("{}", M::name().to_string());
                } else {
                    println!("{} (disabled)", M::name().to_string());
                }
            }
        }
        Ok(())
    }
}

/// Module configurations and required keys
pub type ModuleConfsAndKeys<M> = (
    (
        <M as DursModule<DuRsConf, DursMsg>>::ModuleConf,
        Option<<M as DursModule<DuRsConf, DursMsg>>::ModuleUserConf>,
    ),
    RequiredKeysContent,
);

/// Get module conf and keys
pub fn get_module_conf_and_keys<M: DursModule<DuRsConf, DursMsg>>(
    currency_name: Option<&CurrencyName>,
    global_conf: &<DuRsConf as DursConfTrait>::GlobalConf,
    module_conf_json: Option<serde_json::Value>,
    keypairs: DuniterKeyPairs,
) -> Result<ModuleConfsAndKeys<M>, ModuleConfError> {
    Ok((
        get_module_conf::<M>(currency_name, global_conf, module_conf_json)?,
        DuniterKeyPairs::get_required_keys_content(M::ask_required_keys(), keypairs),
    ))
}

/// get module conf
pub fn get_module_conf<M: DursModule<DuRsConf, DursMsg>>(
    currency_name: Option<&CurrencyName>,
    global_conf: &<DuRsConf as DursConfTrait>::GlobalConf,
    module_conf_json: Option<serde_json::Value>,
) -> Result<(M::ModuleConf, Option<M::ModuleUserConf>), ModuleConfError> {
    if let Some(module_conf_json) = module_conf_json {
        let module_user_conf: Option<M::ModuleUserConf> =
            serde_json::from_str(module_conf_json.to_string().as_str())?;
        M::generate_module_conf(currency_name, global_conf, module_user_conf)
    } else {
        M::generate_module_conf(currency_name, global_conf, None)
    }
}

/// Launch synchronisation from a duniter-ts database
pub fn sync_ts<DC: DursConfTrait>(profile_path: PathBuf, conf: &DC, sync_opts: SyncOpt) {
    // Launch sync-ts
    BlockchainModule::sync_ts(profile_path, conf, sync_opts);
}

/// Launch databases explorer
pub fn dbex(profile_path: PathBuf, csv: bool, query: &DBExQuery) {
    // Launch databases explorer
    BlockchainModule::dbex(profile_path, csv, query);
}

#[inline]
/// Get sofware informations
pub fn get_software_infos(soft_name: &'static str, soft_version: &'static str) -> String {
    if let Some(last_commit_hash) = get_last_commit_hash() {
        format!(
            "{} v{}-dev (commit {})",
            soft_name, soft_version, last_commit_hash
        )
    } else {
        format!("{} v{}", soft_name, soft_version)
    }
}

#[inline]
/// Get last commit hash
pub fn get_last_commit_hash() -> Option<&'static str> {
    option_env!("LAST_COMMIT_HASH")
}
