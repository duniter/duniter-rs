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

#![cfg_attr(feature = "strict", deny(warnings))]
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

extern crate dirs;
extern crate duniter_blockchain;
extern crate duniter_conf;
extern crate duniter_crypto;
extern crate duniter_message;
extern crate duniter_module;
extern crate duniter_network;
extern crate log_panics;
extern crate serde_json;
extern crate simplelog;
extern crate threadpool;

pub mod change_conf;
pub mod cli;
pub mod rooter;

use duniter_blockchain::{BlockchainModule, DBExQuery, DBExTxQuery, DBExWotQuery};
pub use duniter_conf::{ChangeGlobalConf, DuRsConf, DuniterKeyPairs};
use duniter_message::*;
use duniter_module::*;
use duniter_network::{NetworkModule, SyncEndpoint, SyncParams};
use log::Level;
use simplelog::*;
//use std::error::Error;
//use std::fmt::{Debug, Formatter};
use cli::*;
use std::fs;
use std::fs::{File, OpenOptions};
use std::sync::mpsc;
use structopt::clap::{App, ArgMatches};
use structopt::StructOpt;
use threadpool::ThreadPool;

/// Number of thread in plugins ThreadPool
pub static THREAD_POOL_SIZE: &'static usize = &2;

/// Durs main function
pub fn main<'b, 'a: 'b, CliFunc, PlugFunc>(
    soft_name: &'static str,
    soft_version: &'static str,
    clap_app: &'a App<'b, 'a>,
    mut inject_modules_subcommands: CliFunc,
    mut plug_modules: PlugFunc,
) where
    'b: 'a,
    CliFunc: FnMut(&mut DuniterCore<'a, 'b, DuRsConf>) -> (),
    PlugFunc: FnMut(&mut DuniterCore<'a, 'b, DuRsConf>) -> (),
{
    // Instantiate duniter core
    let mut duniter_core = DuniterCore::<DuRsConf>::new(soft_name, soft_version, clap_app, 0);

    // Inject modules subcommands
    inject_modules_subcommands(&mut duniter_core);

    // Match user command
    if duniter_core.match_user_command() {
        // Plug all plugins
        plug_modules(&mut duniter_core);
        duniter_core.start_core();
    }
}

#[derive(Debug, Clone)]
/// User command
pub enum UserCommand {
    /// Start
    Start(),
    /// Sync (SyncEndpoint)
    Sync(SyncParams),
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

#[derive(Clone)]
/// Duniter Core Datas
pub struct DuniterCore<'a, 'b: 'a, DC: DuniterConf> {
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
    /// Sender channel of rooter thread
    pub rooter_sender: Option<mpsc::Sender<RooterThreadMessage<DursMsg>>>,
    ///  Count the number of plugged modules
    pub modules_count: usize,
    ///  Count the number of plugged network modules
    pub network_modules_count: usize,
    /// ThreadPool that execute plugged modules
    pub thread_pool: ThreadPool,
}

impl<'a, 'b: 'a> DuniterCore<'b, 'a, DuRsConf> {
    /// Instantiate Duniter node
    pub fn new(
        soft_name: &'static str,
        soft_version: &'static str,
        cli_conf: &'a App<'b, 'a>,
        run_duration_in_secs: u64,
    ) -> DuniterCore<'b, 'a, DuRsConf> {
        // Get cli conf
        //let yaml = load_yaml!("./cli/en.yml");
        //let cli_conf = TupleApp(App::from_yaml(yaml));
        DuniterCore {
            cli_conf: TupleApp(cli_conf),
            cli_args: None,
            plugins_cli_conf: vec![],
            user_command: None,
            soft_meta_datas: SoftwareMetaDatas {
                soft_name,
                soft_version,
                profile: String::from("default"),
                conf: DuRsConf::default(),
            },
            keypairs: None,
            run_duration_in_secs,
            rooter_sender: None,
            modules_count: 0,
            network_modules_count: 0,
            thread_pool: ThreadPool::new(*THREAD_POOL_SIZE),
        }
    }
    /// Inject cli subcommand
    pub fn inject_cli_subcommand<M: DuniterModule<DuRsConf, DursMsg>>(&mut self) {
        //self.cli_conf = TupleApp(&self.cli_conf.0.clone().subcommand(M::ModuleOpt::clap()));
        self.plugins_cli_conf.push(M::ModuleOpt::clap());
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
        external_followers: Vec<mpsc::Sender<DursMsgContent>>,
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
        // Get datas profile name
        let profile = match_profile(&cli_args);

        // Init logger
        init_logger(profile.as_str(), self.soft_meta_datas.soft_name, &cli_args);

        // Print panic! in logs
        //log_panics::init();

        // Load global conf
        let (conf, keypairs) = duniter_conf::load_conf(profile.as_str());
        info!("Success to load global conf.");

        // save profile and conf
        self.soft_meta_datas.profile = profile.clone();
        self.soft_meta_datas.conf = conf.clone();

        // Save keypairs
        self.keypairs = Some(keypairs);

        /*
         * COMMAND LINE PROCESSING
         */
        if let Some(matches) = cli_args.subcommand_matches("disable") {
            let opts = DisableOpt::from_clap(matches);
            change_conf::change_global_conf(
                &profile,
                conf,
                ChangeGlobalConf::DisableModule(opts.module_name),
            );
            false
        } else if let Some(matches) = cli_args.subcommand_matches("enable") {
            let opts = EnableOpt::from_clap(matches);
            change_conf::change_global_conf(
                &profile,
                conf,
                ChangeGlobalConf::EnableModule(opts.module_name),
            );
            false
        } else if let Some(matches) = cli_args.subcommand_matches("modules") {
            // Store user command
            self.user_command = Some(UserCommand::ListModules(ListModulesOpt::from_clap(matches)));

            // Start rooter thread
            self.rooter_sender = Some(rooter::start_rooter::<DuRsConf>(0, vec![]));
            true
        } else if let Some(_matches) = cli_args.subcommand_matches("start") {
            // Store user command
            self.user_command = Some(UserCommand::Start());

            // Start rooter thread
            self.rooter_sender = Some(rooter::start_rooter::<DuRsConf>(
                self.run_duration_in_secs,
                external_followers,
            ));
            true
        } else if let Some(matches) = cli_args.subcommand_matches("sync") {
            let opts = SyncOpt::from_clap(matches);
            let sync_endpoint = SyncEndpoint {
                domain_or_ip: opts.host,
                port: opts.port,
                path: opts.path,
                tls: false,
            };
            // Store sync command parameters
            self.user_command = Some(UserCommand::Sync(SyncParams {
                sync_endpoint,
                cautious: opts.cautious_mode,
                verif_hashs: opts.unsafe_mode,
            }));
            // Start rooter thread
            self.rooter_sender = Some(rooter::start_rooter::<DuRsConf>(0, vec![]));
            true
        } else if let Some(matches) = cli_args.subcommand_matches("sync_ts") {
            let opts = SyncTsOpt::from_clap(matches);
            let ts_profile = opts
                .ts_profile
                .unwrap_or_else(|| String::from("duniter_default"));
            sync_ts(
                profile.as_str(),
                &conf,
                &ts_profile,
                opts.cautious_mode,
                opts.unsafe_mode,
            );
            false
        } else if let Some(matches) = cli_args.subcommand_matches("dbex") {
            let opts = DbExOpt::from_clap(matches);
            match opts.subcommand {
                DbExSubCommand::DistanceOpt(distance_opts) => dbex(
                    profile.as_str(),
                    &conf,
                    opts.csv,
                    &DBExQuery::WotQuery(DBExWotQuery::AllDistances(distance_opts.reverse)),
                ),
                DbExSubCommand::MemberOpt(member_opts) => dbex(
                    profile.as_str(),
                    &conf,
                    opts.csv,
                    &DBExQuery::WotQuery(DBExWotQuery::MemberDatas(member_opts.uid)),
                ),
                DbExSubCommand::MembersOpt(members_opts) => {
                    if members_opts.expire {
                        dbex(
                            profile.as_str(),
                            &conf,
                            opts.csv,
                            &DBExQuery::WotQuery(DBExWotQuery::ExpireMembers(members_opts.reverse)),
                        );
                    } else {
                        dbex(
                            profile.as_str(),
                            &conf,
                            opts.csv,
                            &DBExQuery::WotQuery(DBExWotQuery::ListMembers(members_opts.reverse)),
                        );
                    }
                }
                DbExSubCommand::BalanceOpt(balance_opts) => dbex(
                    &profile,
                    &conf,
                    opts.csv,
                    &DBExQuery::TxQuery(DBExTxQuery::Balance(balance_opts.address)),
                ),
            }
            false
        } else if let Some(matches) = cli_args.subcommand_matches("reset") {
            let opts = ResetOpt::from_clap(matches);
            let mut profile_path = match dirs::config_dir() {
                Some(path) => path,
                None => panic!("Impossible to get user config directory !"),
            };
            profile_path.push(duniter_conf::get_user_datas_folder());
            profile_path.push(profile.clone());
            if !profile_path.as_path().exists() {
                panic!(format!("Error : {} profile don't exist !", profile));
            }
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
                    conf_keys_path.push("keypairs.json");
                    fs::remove_file(conf_keys_path.as_path())
                        .expect("Fail to remove keypairs file !");
                }
                ResetType::All => {
                    fs::remove_dir_all(profile_path.as_path())
                        .expect("Fail to remove all profile datas !");
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
    pub fn start_core(&self) {
        if self.network_modules_count == 0 {
            panic!("You must plug at least one network layer !");
        }
        if let Some(UserCommand::Start()) = self.user_command {
            // Create blockchain module channel
            let (blockchain_sender, blockchain_receiver): (
                mpsc::Sender<DursMsg>,
                mpsc::Receiver<DursMsg>,
            ) = mpsc::channel();

            let rooter_sender = if let Some(ref rooter_sender) = self.rooter_sender {
                rooter_sender
            } else {
                panic!("Try to start core without rooter_sender !");
            };

            // Send blockchain sender to rooter thread
            rooter_sender
                .send(RooterThreadMessage::ModuleSender(
                    BlockchainModule::name(),
                    blockchain_sender,
                    vec![ModuleRole::BlockchainDatas, ModuleRole::BlockValidation],
                    vec![ModuleEvent::NewBlockFromNetwork],
                ))
                .expect("Fatal error: fail to send blockchain sender to rooter thread !");

            // Instantiate blockchain module and load is conf
            let mut blockchain_module = BlockchainModule::load_blockchain_conf(
                rooter_sender.clone(),
                &self.soft_meta_datas.profile,
                &self.soft_meta_datas.conf,
                RequiredKeysContent::MemberKeyPair(None),
            );
            info!("Success to load Blockchain module.");

            // Start blockchain module in main thread
            blockchain_module.start_blockchain(&blockchain_receiver);
        }
    }
    /// Plug a network module
    pub fn plug_network<NM: NetworkModule<DuRsConf, DursMsg>>(&mut self) {
        let enabled = enabled::<DuRsConf, DursMsg, NM>(&self.soft_meta_datas.conf);
        if enabled {
            self.network_modules_count += 1;
            if let Some(UserCommand::Sync(ref network_sync)) = self.user_command {
                // Start module in a new thread
                let rooter_sender = self
                    .rooter_sender
                    .clone()
                    .expect("Try to start a core without rooter_sender !");
                let soft_meta_datas = self.soft_meta_datas.clone();
                let module_conf_json = self
                    .soft_meta_datas
                    .conf
                    .clone()
                    .modules()
                    .get(&NM::name().to_string().as_str())
                    .cloned();
                let keypairs = self.keypairs;
                let sync_params = network_sync.clone();
                self.thread_pool.execute(move || {
                    // Load module conf and keys
                    let (module_conf, required_keys) = get_module_conf_and_keys::<NM>(
                        module_conf_json,
                        keypairs.expect("Try to plug addon into a core without keypair !"),
                    );
                    NM::sync(
                        &soft_meta_datas,
                        required_keys,
                        module_conf,
                        rooter_sender,
                        sync_params,
                    )
                    .unwrap_or_else(|_| {
                        panic!(
                            "Fatal error : fail to load {} Module !",
                            NM::name().to_string()
                        )
                    });
                });
                self.modules_count += 1;
                info!("Success to load {} module.", NM::name().to_string());
            } else {
                self.plug_::<NM>(true);
            }
        } else {
            self.plug_::<NM>(true);
        }
    }

    /// Plug a module
    pub fn plug<M: DuniterModule<DuRsConf, DursMsg>>(&mut self) {
        self.plug_::<M>(false);
    }

    /// Plug a module
    pub fn plug_<M: DuniterModule<DuRsConf, DursMsg>>(&mut self, is_network_module: bool) {
        let enabled = enabled::<DuRsConf, DursMsg, M>(&self.soft_meta_datas.conf);
        if enabled {
            if let Some(UserCommand::Start()) = self.user_command {
                // Start module in a new thread
                let rooter_sender_clone = self
                    .rooter_sender
                    .clone()
                    .expect("Try to start a core without rooter_sender !");
                let soft_meta_datas = self.soft_meta_datas.clone();
                let module_conf_json = self
                    .soft_meta_datas
                    .conf
                    .clone()
                    .modules()
                    .get(&M::name().to_string().as_str())
                    .cloned();
                let keypairs = self.keypairs;
                self.thread_pool.execute(move || {
                    // Load module conf and keys
                    let (module_conf, required_keys) = get_module_conf_and_keys::<M>(
                        module_conf_json,
                        keypairs.expect("Try to plug addon into a core without keypair !"),
                    );
                    M::start(
                        &soft_meta_datas,
                        required_keys,
                        module_conf,
                        rooter_sender_clone,
                        false,
                    )
                    .unwrap_or_else(|_| {
                        panic!(
                            "Fatal error : fail to load {} Module !",
                            M::name().to_string()
                        )
                    });
                });
                self.modules_count += 1;
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
                        let (_conf, keypairs) =
                            duniter_conf::load_conf(self.soft_meta_datas.profile.as_str());
                        let (module_conf, required_keys) =
                            get_module_conf_and_keys::<M>(module_conf_json, keypairs);
                        // Execute module subcommand
                        M::exec_subcommand(
                            &self.soft_meta_datas,
                            required_keys, //required_keys,
                            module_conf,   //module_conf,
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
    }
}

/// Get module conf and keys
pub fn get_module_conf_and_keys<M: DuniterModule<DuRsConf, DursMsg>>(
    module_conf_json: Option<serde_json::Value>,
    keypairs: DuniterKeyPairs,
) -> (M::ModuleConf, RequiredKeysContent) {
    (
        get_module_conf::<M>(module_conf_json),
        DuniterKeyPairs::get_required_keys_content(M::ask_required_keys(), keypairs),
    )
}

/// get module conf
pub fn get_module_conf<M: DuniterModule<DuRsConf, DursMsg>>(
    module_conf_json: Option<serde_json::Value>,
) -> M::ModuleConf {
    if let Some(module_conf_json) = module_conf_json {
        serde_json::from_str(module_conf_json.to_string().as_str())
            .unwrap_or_else(|_| panic!("Fail to parse conf of module {}", M::name().to_string()))
    } else {
        M::ModuleConf::default()
    }
}

/// Match cli option --profile
pub fn match_profile(cli_args: &ArgMatches) -> String {
    String::from(cli_args.value_of("profile_name").unwrap_or("default"))
}

/// Launch synchronisation from a duniter-ts database
pub fn sync_ts<DC: DuniterConf>(
    profile: &str,
    conf: &DC,
    ts_profile: &str,
    cautious: bool,
    verif_inner_hash: bool,
) {
    // Launch sync-ts
    BlockchainModule::sync_ts(profile, conf, ts_profile, cautious, verif_inner_hash);
}

/// Launch databases explorer
pub fn dbex<DC: DuniterConf>(profile: &str, conf: &DC, csv: bool, query: &DBExQuery) {
    // Launch databases explorer
    BlockchainModule::dbex(profile, conf, csv, query);
}

/// Initialize logger
pub fn init_logger(profile: &str, soft_name: &'static str, cli_args: &ArgMatches) {
    // Get datas folder path
    let mut log_file_path = match dirs::config_dir() {
        Some(path) => path,
        None => panic!("Fatal error : Impossible to get user config directory"),
    };
    if !log_file_path.as_path().exists() {
        fs::create_dir(log_file_path.as_path()).expect("Impossible to create ~/.config dir !");
    }
    log_file_path.push(duniter_conf::get_user_datas_folder());
    if !log_file_path.as_path().exists() {
        fs::create_dir(log_file_path.as_path()).expect("Impossible to create ~/.config/durs dir !");
    }
    log_file_path.push(profile);
    // Create datas folder if not exist
    if !log_file_path.as_path().exists() {
        fs::create_dir(log_file_path.as_path()).expect("Impossible to create your profile dir !");
    }

    // Get log_file_path
    log_file_path.push(format!("{}.log", soft_name));

    // Get log level
    let log_level = match cli_args.value_of("logs_level").unwrap_or("i") {
        "e" | "error" => Level::Error,
        "w" | "warn" => Level::Warn,
        "i" | "info" => Level::Info,
        "d" | "debug" => Level::Debug,
        "t" | "trace" => Level::Trace,
        _ => panic!("Fatal error : unknow log level !"),
    };

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

    CombinedLogger::init(vec![WriteLogger::new(
        log_level.to_level_filter(),
        logger_config,
        OpenOptions::new()
            .write(true)
            .append(true)
            .open(
                log_file_path
                    .to_str()
                    .expect("Fatal error : fail to get log file path !"),
            )
            .expect("Fatal error : fail to open log file !"),
    )])
    .expect("Fatal error : fail to init logger !");

    info!("Successfully init logger");
}
