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

//! Crate containing Duniter-rust core.

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

pub mod change_conf;
pub mod cli;
pub mod router;

use duniter_network::cli::sync::*;
use duniter_network::NetworkModule;
use durs_blockchain::{BlockchainModule, DBExQuery, DBExTxQuery, DBExWotQuery};
pub use durs_conf::{
    constants::KEYPAIRS_FILENAME, keys::*, ChangeGlobalConf, DuRsConf, DuniterKeyPairs,
};
use durs_message::*;
use durs_module::*;
use failure::Fail;
use log::Level;
use simplelog::*;
//use std::error::Error;
//use std::fmt::{Debug, Formatter};
use crate::cli::keys::*;
use crate::cli::*;
use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use structopt::clap::{App, ArgMatches};
use structopt::StructOpt;

#[macro_export]
/// Launch durs core server
macro_rules! durs_core_server {
    ( $closure_inject_cli:expr, $closure_plug:expr ) => {{
        if let Err(err) = duniter_core::main(
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            &DursOpt::clap(),
            $closure_inject_cli,
            $closure_plug,
        ) {
            println!("{}", err);
            error!("{}", err);
        }
    }};
}

#[macro_export]
/// Inject module subcommand in durs command line
macro_rules! durs_inject_cli {
    ( $( $Module:ty ),* ) => {
        {
            |core| {
                $(core.inject_cli_subcommand::<$Module>();)*
            }
        }
    };
}

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

#[derive(Debug, Fail)]
/// Durs server error
pub enum DursServerError {
    /// Plug module error
    #[fail(display = "Error on loading module '{}': {}", module_name, error)]
    PlugModuleError {
        /// Module name
        module_name: ModuleStaticName,
        /// Error details
        error: PlugModuleError,
    },
}

/// Durs main function
pub fn main<'b, 'a: 'b, CliFunc, PlugFunc>(
    soft_name: &'static str,
    soft_version: &'static str,
    clap_app: &'a App<'b, 'a>,
    mut inject_modules_subcommands: CliFunc,
    mut plug_modules: PlugFunc,
) -> Result<(), DursServerError>
where
    'b: 'a,
    CliFunc: FnMut(&mut DuniterCore<'a, 'b, DuRsConf>),
    PlugFunc: FnMut(&mut DuniterCore<'a, 'b, DuRsConf>) -> Result<(), DursServerError>,
{
    // Instantiate duniter core
    let mut duniter_core = DuniterCore::<DuRsConf>::new(soft_name, soft_version, clap_app, 0);

    // Inject modules subcommands
    inject_modules_subcommands(&mut duniter_core);

    // Match user command
    if duniter_core.match_user_command() {
        // Plug all plugins
        plug_modules(&mut duniter_core)?;
        duniter_core.start_core();
    }

    Ok(())
}

#[derive(Debug, Clone)]
/// User command
pub enum UserCommand {
    /// Start
    Start(),
    /// Sync (SyncEndpoint)
    Sync(SyncOpt),
    /// List modules
    ListModules(ListModulesOpt),
    /// Unknow command
    UnknowCommand(String),
}

/// TupleApp
#[derive(Clone)]
pub struct TupleApp<'b, 'a: 'b>(&'b App<'a, 'b>);

/*impl<'b, 'a: 'b> Debug for TupleApp<'a, 'b> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "TupleApp()")
    }
}*/

/// Duniter Core Datas
pub struct DuniterCore<'a, 'b: 'a, DC: DursConfTrait> {
    /// Command line configuration
    pub cli_conf: TupleApp<'a, 'b>,
    /// Command line arguments parsing by clap
    pub cli_args: Option<ArgMatches<'a>>,
    /// Plugins command line configuration
    pub plugins_cli_conf: Vec<App<'b, 'a>>,
    /// Does the entered command require to launch server ?
    pub user_command: Option<UserCommand>,
    /// Software meta datas
    pub soft_meta_datas: SoftwareMetaDatas<DC>,
    /// Keypairs
    pub keypairs: Option<DuniterKeyPairs>,
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

impl<'a, 'b: 'a> DuniterCore<'b, 'a, DuRsConf> {
    /// Instantiate Duniter node
    pub fn new(
        soft_name: &'static str,
        soft_version: &'static str,
        cli_conf: &'a App<'b, 'a>,
        run_duration_in_secs: u64,
    ) -> DuniterCore<'b, 'a, DuRsConf> {
        DuniterCore {
            cli_conf: TupleApp(cli_conf),
            cli_args: None,
            plugins_cli_conf: vec![],
            user_command: None,
            soft_meta_datas: SoftwareMetaDatas {
                conf: DuRsConf::default(),
                profiles_path: None,
                keypairs_file_path: None,
                profile: String::from("default"),
                soft_name,
                soft_version,
            },
            keypairs: None,
            run_duration_in_secs,
            router_sender: None,
            network_modules_count: 0,
            modules_names: Vec::new(),
            threads: HashMap::new(),
        }
    }
    /// Inject cli subcommand
    pub fn inject_cli_subcommand<M: DursModule<DuRsConf, DursMsg>>(&mut self) {
        if M::have_subcommand() {
            self.plugins_cli_conf.push(M::ModuleOpt::clap());
        }
    }
    /// Execute user command
    pub fn match_user_command(&mut self) -> bool {
        self.match_specialize_user_command(vec![], None, vec![])
    }
    /// Execute specialize user command
    pub fn match_specialize_user_command(
        &mut self,
        sup_apps: Vec<App<'a, 'b>>,
        sup_apps_fn: Option<&Fn(&str, &ArgMatches) -> bool>,
        external_followers: Vec<mpsc::Sender<DursMsg>>,
    ) -> bool {
        // Inject core subcommands
        //let core_cli_conf = inject_core_subcommands(self.cli_conf.0.clone());
        let core_cli_conf = self.cli_conf.0.clone();
        // Inject plugins subcommands
        let cli_conf = if !self.plugins_cli_conf.is_empty() {
            core_cli_conf.subcommands(self.plugins_cli_conf.clone())
        } else {
            core_cli_conf
        };
        // Inject specialize node subcommands a Math command line arguments
        self.cli_args = Some(if !sup_apps.is_empty() {
            cli_conf.subcommands(sup_apps).get_matches()
        } else {
            cli_conf.get_matches()
        });
        let cli_args = self.cli_args.clone().expect("cli_args must be Some !");

        // Get profile name
        let profile = match_profile(&cli_args);

        // Get profile path
        let profiles_path = match_profiles_path(&cli_args);

        // Get keypairs file path
        let keypairs_file_path = match_keypairs_file(&cli_args);

        // Compute user profile path
        let profile_path = durs_conf::get_profile_path(&profiles_path, &profile);

        // Init logger
        init_logger(
            profile_path.clone(),
            self.soft_meta_datas.soft_name,
            self.soft_meta_datas.soft_version,
            &cli_args,
        );

        // Load global conf
        let (conf, keypairs) =
            durs_conf::load_conf(profile.as_str(), &profiles_path, &keypairs_file_path);
        info!("Success to load global conf.");

        // Save conf and profile and keypairs file path
        self.soft_meta_datas.conf = conf;
        self.soft_meta_datas.profiles_path = profiles_path.clone();
        self.soft_meta_datas.keypairs_file_path = keypairs_file_path;
        self.soft_meta_datas.profile = profile.clone();

        // Save keypairs
        self.keypairs = Some(keypairs);

        /*
         * COMMAND LINE PROCESSING
         */
        if let Some(matches) = cli_args.subcommand_matches("disable") {
            let opts = DisableOpt::from_clap(matches);
            change_conf::change_global_conf(
                &profile_path,
                &mut self.soft_meta_datas.conf,
                ChangeGlobalConf::DisableModule(opts.module_name),
            );
            false
        } else if let Some(matches) = cli_args.subcommand_matches("enable") {
            let opts = EnableOpt::from_clap(matches);
            change_conf::change_global_conf(
                &profile_path,
                &mut self.soft_meta_datas.conf,
                ChangeGlobalConf::EnableModule(opts.module_name),
            );
            false
        } else if let Some(matches) = cli_args.subcommand_matches("modules") {
            // Store user command
            self.user_command = Some(UserCommand::ListModules(ListModulesOpt::from_clap(matches)));

            // Start router thread
            self.router_sender = Some(router::start_router(
                0,
                profile_path.clone(),
                self.soft_meta_datas.conf.clone(),
                vec![],
            ));
            true
        } else if let Some(_matches) = cli_args.subcommand_matches("start") {
            // Store user command
            self.user_command = Some(UserCommand::Start());

            // Print panic! in logs
            log_panics::init();

            // Start router thread
            self.router_sender = Some(router::start_router(
                self.run_duration_in_secs,
                profile_path.clone(),
                self.soft_meta_datas.conf.clone(),
                external_followers,
            ));
            true
        } else if let Some(matches) = cli_args.subcommand_matches("sync") {
            let opts = SyncOpt::from_clap(matches);
            match opts.source_type {
                SyncSourceType::Network => unimplemented!(),
                SyncSourceType::LocalDuniter => {
                    sync_ts(profile_path.clone(), &self.soft_meta_datas.conf, opts);
                }
            }

            false
        } else if let Some(matches) = cli_args.subcommand_matches("dbex") {
            let opts = DbExOpt::from_clap(matches);
            match opts.subcommand {
                DbExSubCommand::DistanceOpt(distance_opts) => dbex(
                    profile_path.clone(),
                    &self.soft_meta_datas.conf,
                    opts.csv,
                    &DBExQuery::WotQuery(DBExWotQuery::AllDistances(distance_opts.reverse)),
                ),
                DbExSubCommand::MemberOpt(member_opts) => dbex(
                    profile_path.clone(),
                    &self.soft_meta_datas.conf,
                    opts.csv,
                    &DBExQuery::WotQuery(DBExWotQuery::MemberDatas(member_opts.uid)),
                ),
                DbExSubCommand::MembersOpt(members_opts) => {
                    if members_opts.expire {
                        dbex(
                            profile_path.clone(),
                            &self.soft_meta_datas.conf,
                            opts.csv,
                            &DBExQuery::WotQuery(DBExWotQuery::ExpireMembers(members_opts.reverse)),
                        );
                    } else {
                        dbex(
                            profile_path.clone(),
                            &self.soft_meta_datas.conf,
                            opts.csv,
                            &DBExQuery::WotQuery(DBExWotQuery::ListMembers(members_opts.reverse)),
                        );
                    }
                }
                DbExSubCommand::BalanceOpt(balance_opts) => dbex(
                    profile_path.clone(),
                    &self.soft_meta_datas.conf,
                    opts.csv,
                    &DBExQuery::TxQuery(DBExTxQuery::Balance(balance_opts.address)),
                ),
            }
            false
        } else if let Some(matches) = cli_args.subcommand_matches("reset") {
            let opts = ResetOpt::from_clap(matches);

            match opts.reset_type {
                ResetType::Datas => {
                    let mut currency_datas_path = profile_path.clone();
                    currency_datas_path.push("g1");
                    fs::remove_dir_all(currency_datas_path.as_path())
                        .expect("Fail to remove all currency datas !");
                }
                ResetType::Conf => {
                    let mut conf_file_path = profile_path.clone();
                    conf_file_path.push("conf.json");
                    fs::remove_file(conf_file_path.as_path()).expect("Fail to remove conf file !");
                    let mut conf_keys_path = profile_path.clone();
                    conf_keys_path.push(KEYPAIRS_FILENAME);
                    fs::remove_file(conf_keys_path.as_path())
                        .expect("Fail to remove keypairs file !");
                }
                ResetType::All => {
                    fs::remove_dir_all(profile_path.as_path())
                        .expect("Fail to remove all profile datas !");
                }
            }
            false
        } else if let Some(matches) = cli_args.subcommand_matches("keys") {
            let opts = KeysOpt::from_clap(matches);
            match opts.subcommand {
                KeysSubCommand::Wizard(_wizardopt) => {
                    let new_keypairs = key_wizard(keypairs).unwrap();
                    save_keypairs(&profiles_path, profile.as_str(), new_keypairs);
                }
                KeysSubCommand::Modify(modifyopt) => match modifyopt.subcommand {
                    ModifySubCommand::NetworkSaltPassword(networkopt) => {
                        let new_keypairs =
                            modify_network_keys(&networkopt.salt, &networkopt.password, keypairs);
                        save_keypairs(&profiles_path, profile.as_str(), new_keypairs);
                    }
                    ModifySubCommand::MemberSaltPassword(memberopt) => {
                        let new_keypairs =
                            modify_member_keys(&memberopt.salt, &memberopt.password, keypairs);
                        save_keypairs(&profiles_path, profile.as_str(), new_keypairs);
                    }
                },
                KeysSubCommand::Clear(clearopt) => {
                    let new_keypairs = clear_keys(
                        clearopt.network || clearopt.all,
                        clearopt.member || clearopt.all,
                        keypairs,
                    );
                    save_keypairs(&profiles_path, profile.as_str(), new_keypairs);
                }
                KeysSubCommand::Show(_showopt) => {
                    show_keys(keypairs);
                }
            }
            false
        } else if let Some(unknow_subcommand) = cli_args.subcommand_name() {
            let mut module_subcommand = true;
            if let Some(sup_apps_fn) = sup_apps_fn {
                if sup_apps_fn(profile.as_str(), &cli_args) {
                    module_subcommand = false;
                }
            }
            if module_subcommand {
                self.user_command =
                    Some(UserCommand::UnknowCommand(String::from(unknow_subcommand)));
                true
            } else {
                false
            }
        } else {
            println!("Please use a subcommand. -h for help.");
            false
        }
    }
    /// Start core (=blockchain module)
    pub fn start_core(&mut self) {
        if self.network_modules_count == 0 {
            panic!("You must plug at least one network layer !");
        }
        if let Some(UserCommand::Start()) = self.user_command {
            // Create blockchain module channel
            let (blockchain_sender, blockchain_receiver): (
                mpsc::Sender<DursMsg>,
                mpsc::Receiver<DursMsg>,
            ) = mpsc::channel();

            let router_sender = if let Some(ref router_sender) = self.router_sender {
                router_sender
            } else {
                panic!("Try to start core without router_sender !");
            };

            // Send expected modules count to router thread
            router_sender
                .send(RouterThreadMessage::ModulesCount(
                    self.modules_names.len() + 1,
                ))
                .expect("Fatal error: fail to send expected modules count to router thread !");

            // Send blockchain module registration to router thread
            router_sender
                .send(RouterThreadMessage::ModuleRegistration(
                    BlockchainModule::name(),
                    blockchain_sender,
                    vec![ModuleRole::BlockchainDatas, ModuleRole::BlockValidation],
                    vec![ModuleEvent::NewBlockFromNetwork],
                    vec![],
                    vec![],
                ))
                .expect("Fatal error: fail to send blockchain registration to router thread !");

            // Get profile path
            let profile_path = durs_conf::get_profile_path(
                &self.soft_meta_datas.profiles_path,
                &self.soft_meta_datas.profile,
            );

            // Instantiate blockchain module and load is conf
            let mut blockchain_module = BlockchainModule::load_blockchain_conf(
                router_sender.clone(),
                profile_path,
                &self.soft_meta_datas.conf,
                RequiredKeysContent::MemberKeyPair(None),
            );
            info!("Success to load Blockchain module.");

            // Start blockchain module in thread
            let thread_builder = thread::Builder::new().name(BlockchainModule::name().0.into());
            let blockchain_thread_handler = thread_builder
                .spawn(move || blockchain_module.start_blockchain(&blockchain_receiver))
                .expect("Fatal error: fail to spawn module main thread !");

            // Wait until all modules threads are finished
            for module_static_name in &self.modules_names {
                if let Some(module_thread_handler) = self.threads.remove(module_static_name) {
                    if let Err(err) = module_thread_handler.join() {
                        error!("'{}' module thread panic : {:?}", module_static_name.0, err);
                    }
                }
            }

            // Wait until blockchain main thread are finished
            if let Err(err) = blockchain_thread_handler.join() {
                error!("'blockchain' thread panic : {:?}", err);
            }
        }
    }
    #[inline]
    /// Plug a network module
    pub fn plug_network<NM: NetworkModule<DuRsConf, DursMsg>>(
        &mut self,
    ) -> Result<(), DursServerError> {
        self.plug_network_::<NM>()
            .map_err(|error| DursServerError::PlugModuleError {
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
            if let Some(UserCommand::Sync(ref network_sync)) = self.user_command {
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
                let (module_conf, required_keys) = get_module_conf_and_keys::<NM>(
                    &soft_meta_datas.conf.get_global_conf(),
                    module_conf_json,
                    keypairs.expect("Try to plug addon into a core without keypair !"),
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
                                panic!(
                                    "Fatal error : fail to load {} Module !",
                                    NM::name().to_string()
                                )
                            });
                        })
                        .expect("Fatail error: fail to spawn network module main thread !"),
                );
                self.modules_names.push(NM::name());
                info!("Success to load {} module.", NM::name().to_string());
                Ok(())
            } else {
                self.plug_::<NM>(true)
            }
        } else {
            self.plug_::<NM>(true)
        }
    }
    #[inline]
    /// Plug a module
    pub fn plug<M: DursModule<DuRsConf, DursMsg>>(&mut self) -> Result<(), DursServerError> {
        self.plug_::<M>(false)
            .map_err(|error| DursServerError::PlugModuleError {
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
            if let Some(UserCommand::Start()) = self.user_command {
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
                let (module_conf, required_keys) = get_module_conf_and_keys::<M>(
                    &soft_meta_datas.conf.get_global_conf(),
                    module_conf_json,
                    keypairs.expect("Try to plug addon into a core without keypair !"),
                )?;

                let thread_builder = thread::Builder::new().name(M::name().0.into());
                self.threads.insert(
                    M::name(),
                    thread_builder
                        .spawn(move || {
                            M::start(
                                &soft_meta_datas,
                                required_keys,
                                module_conf,
                                router_sender_clone,
                                false,
                            )
                            .unwrap_or_else(|_| {
                                panic!(
                                    "Fatal error : fail to load {} Module !",
                                    M::name().to_string()
                                )
                            });
                        })
                        .expect("Fatail error: fail to spawn module main thread !"),
                );
                self.modules_names.push(M::name());
                info!("Success to load {} module.", M::name().to_string());
            } else if let Some(UserCommand::UnknowCommand(ref subcommand)) = self.user_command {
                if M::have_subcommand() && *subcommand == M::name().to_string() {
                    // Math command line arguments
                    if let Some(subcommand_args) = self
                        .cli_args
                        .clone()
                        .expect("cli_args must be Some !")
                        .subcommand_matches(M::name().to_string())
                    {
                        // Load module conf and keys
                        let module_conf_json = self
                            .soft_meta_datas
                            .conf
                            .clone()
                            .modules()
                            .get(&M::name().to_string().as_str())
                            .cloned();
                        let (conf, keypairs) = durs_conf::load_conf(
                            self.soft_meta_datas.profile.as_str(),
                            &self.soft_meta_datas.profiles_path,
                            &self.soft_meta_datas.keypairs_file_path,
                        );
                        let (module_conf, required_keys) = get_module_conf_and_keys::<M>(
                            &conf.get_global_conf(),
                            module_conf_json,
                            keypairs,
                        )?;
                        // Execute module subcommand
                        M::exec_subcommand(
                            &self.soft_meta_datas,
                            required_keys,
                            module_conf,
                            M::ModuleOpt::from_clap(subcommand_args),
                        );
                    }
                }
            }
        }
        if let Some(UserCommand::ListModules(ref options)) = self.user_command {
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

/// Get module conf and keys
pub fn get_module_conf_and_keys<M: DursModule<DuRsConf, DursMsg>>(
    global_conf: &<DuRsConf as DursConfTrait>::GlobalConf,
    module_conf_json: Option<serde_json::Value>,
    keypairs: DuniterKeyPairs,
) -> Result<(M::ModuleConf, RequiredKeysContent), ModuleConfError> {
    Ok((
        get_module_conf::<M>(global_conf, module_conf_json)?,
        DuniterKeyPairs::get_required_keys_content(M::ask_required_keys(), keypairs),
    ))
}

/// get module conf
pub fn get_module_conf<M: DursModule<DuRsConf, DursMsg>>(
    global_conf: &<DuRsConf as DursConfTrait>::GlobalConf,
    module_conf_json: Option<serde_json::Value>,
) -> Result<M::ModuleConf, ModuleConfError> {
    if let Some(module_conf_json) = module_conf_json {
        let module_user_conf: M::ModuleUserConf =
            serde_json::from_str(module_conf_json.to_string().as_str())?;
        M::generate_module_conf(global_conf, module_user_conf)
    } else {
        Ok(M::ModuleConf::default())
    }
}

/// Match cli option --profile
#[inline]
pub fn match_profile(cli_args: &ArgMatches) -> String {
    String::from(cli_args.value_of("profile_name").unwrap_or("default"))
}

/// Match cli option --profiles--path
#[inline]
pub fn match_profiles_path(cli_args: &ArgMatches) -> Option<PathBuf> {
    cli_args.value_of_os("profiles_path").map(PathBuf::from)
}

/// Match cli option --keypairs-file
#[inline]
pub fn match_keypairs_file(cli_args: &ArgMatches) -> Option<PathBuf> {
    cli_args.value_of_os("keypairs_file").map(PathBuf::from)
}

/// Launch synchronisation from a duniter-ts database
pub fn sync_ts<DC: DursConfTrait>(profile_path: PathBuf, conf: &DC, sync_opts: SyncOpt) {
    // Launch sync-ts
    BlockchainModule::sync_ts(profile_path, conf, sync_opts);
}

/// Launch databases explorer
pub fn dbex<DC: DursConfTrait>(profile_path: PathBuf, conf: &DC, csv: bool, query: &DBExQuery) {
    // Launch databases explorer
    BlockchainModule::dbex(profile_path, conf, csv, query);
}

/// Initialize logger
/// Warning: This function cannot use the macro fatal_error! because the logger is not yet initialized, so it must use panic !
pub fn init_logger(
    profile_path: PathBuf,
    soft_name: &'static str,
    soft_version: &'static str,
    cli_args: &ArgMatches,
) {
    let mut log_file_path = profile_path;

    // Get log_file_path
    log_file_path.push(format!("{}.log", soft_name));

    // Get log level
    let log_level = match cli_args.value_of("logs_level").unwrap_or("INFO") {
        "ERROR" => Level::Error,
        "WARN" => Level::Warn,
        "INFO" => Level::Info,
        "DEBUG" => Level::Debug,
        "TRACE" => Level::Trace,
        _ => unreachable!("Structopt guarantees us that the string match necessarily with one of the variants of the enum Level"),
    };

    // Get log-stdout option
    let log_stdout = cli_args.is_present("log_stdout");

    // Config logger
    let logger_config = Config {
        time: Some(Level::Error),
        level: Some(Level::Error),
        target: Some(Level::Debug),
        location: Some(Level::Debug),
        time_format: Some("%Y-%m-%d %H:%M:%S%:z"),
    };

    // Create log file if not exist
    if !log_file_path.as_path().exists() {
        File::create(
            log_file_path
                .to_str()
                .expect("Fatal error : fail to get log file path !"),
        )
        .expect("Fatal error : fail to create log file path !");
    }

    let level_filter = log_level.to_level_filter();
    let file_logger_opts = OpenOptions::new()
        .write(true)
        .append(true)
        .open(
            log_file_path
                .to_str()
                .expect("Fatal error : fail to get log file path !"),
        )
        .expect("Fatal error : fail to open log file !");

    if log_stdout {
        CombinedLogger::init(vec![
            TermLogger::new(level_filter, logger_config)
                .expect("Fatal error : fail to create term logger !"),
            WriteLogger::new(level_filter, logger_config, file_logger_opts),
        ])
        .expect("Fatal error : Fail to init combined logger !");
    } else {
        WriteLogger::init(level_filter, logger_config, file_logger_opts)
            .expect("Fatal error : fail to init file logger !");
    }

    info!("Launching {}", get_software_infos(soft_name, soft_version));
    info!("Successfully init logger");
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
