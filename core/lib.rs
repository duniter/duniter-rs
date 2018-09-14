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
#![cfg_attr(feature = "cargo-clippy", allow(implicit_hasher))]
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
extern crate clap;

#[macro_use]
extern crate log;

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
extern crate sqlite;
extern crate threadpool;

pub mod change_conf;

use clap::{App, ArgMatches};
use duniter_blockchain::{BlockchainModule, DBExQuery, DBExTxQuery, DBExWotQuery};
pub use duniter_conf::{ChangeGlobalConf, DuRsConf, DuniterKeyPairs};
use duniter_message::DuniterMessage;
use duniter_module::*;
use duniter_network::{NetworkModule, SyncEndpoint};
use log::Level;
use simplelog::*;
use std::collections::HashSet;
use std::fs;
use std::fs::{File, OpenOptions};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use threadpool::ThreadPool;

#[derive(Debug, Clone)]
/// User command
pub enum UserCommand {
    /// Start
    Start(),
    /// Sync (SyncEndpoint)
    Sync(SyncEndpoint),
    /// List modules
    ListModules(HashSet<ModulesFilter>),
    /// Other command
    Other(),
}

#[derive(Debug)]
/// Duniter Core Datas
pub struct DuniterCore<DC: DuniterConf> {
    /// Does the entered command require to launch server ?
    pub user_command: UserCommand,
    /// Software meta datas
    pub soft_meta_datas: SoftwareMetaDatas<DC>,
    /// Keypairs
    pub keypairs: DuniterKeyPairs,
    /// Run duration. Zero = infinite duration.
    pub run_duration_in_secs: u64,
    /// Sender channel of rooter thread
    pub rooter_sender: mpsc::Sender<RooterThreadMessage<DuniterMessage>>,
    ///  Count the number of plugged modules
    pub modules_count: usize,
    ///  Count the number of plugged network modules
    pub network_modules_count: usize,
    /// ThreadPool that execute plugged modules
    pub thread_pool: ThreadPool,
}

impl DuniterCore<DuRsConf> {
    /// Instantiate Duniter classic node
    pub fn new(
        soft_name: &'static str,
        soft_version: &'static str,
    ) -> Option<DuniterCore<DuRsConf>> {
        DuniterCore::new_specialized_node(soft_name, soft_version, 0, vec![], vec![], None)
    }
    /// Instantiate Duniter specialize node
    pub fn new_specialized_node<'a, 'b>(
        soft_name: &'static str,
        soft_version: &'static str,
        run_duration_in_secs: u64,
        external_followers: Vec<mpsc::Sender<DuniterMessage>>,
        sup_apps: Vec<App<'a, 'b>>,
        sup_apps_fn: Option<&Fn(&str, &ArgMatches) -> ()>,
    ) -> Option<DuniterCore<DuRsConf>> {
        // Get cli conf
        let yaml = load_yaml!("./cli/en.yml");
        let cli_conf = App::from_yaml(yaml);

        // Math command line arguments
        let cli_args = if !sup_apps.is_empty() {
            cli_conf.subcommands(sup_apps).get_matches()
        } else {
            cli_conf.get_matches()
        };

        // Get datas profile name
        let profile = match_profile(&cli_args);

        // Init logger
        init_logger(profile.as_str(), soft_name, &cli_args);

        // Print panic! in logs
        log_panics::init();

        // Load global conf
        let (conf, keypairs) = duniter_conf::load_conf(profile.as_str());
        info!("Success to load global conf.");

        // Define SoftwareMetaDatas
        let soft_meta_datas = SoftwareMetaDatas {
            soft_name,
            soft_version,
            profile: profile.clone(),
            conf: conf.clone(),
        };

        /*
         * COMMAND LINE PROCESSING
         */
        if let Some(matches) = cli_args.subcommand_matches("disable") {
            let module_name = matches
                .value_of("MODULE_NAME")
                .expect("disable: you must enter a module name !")
                .to_string();
            change_conf::change_global_conf(
                &profile,
                conf,
                ChangeGlobalConf::DisableModule(ModuleId(module_name)),
            );
            None
        } else if let Some(matches) = cli_args.subcommand_matches("enable") {
            let module_name = matches
                .value_of("MODULE_NAME")
                .expect("enable: you must enter a module name !")
                .to_string();
            change_conf::change_global_conf(
                &profile,
                conf,
                ChangeGlobalConf::EnableModule(ModuleId(module_name)),
            );
            None
        } else if let Some(matches) = cli_args.subcommand_matches("modules") {
            let mut filters = HashSet::new();
            if matches.is_present("disabled") {
                filters.insert(ModulesFilter::Enabled(false));
            } else if matches.is_present("enabled") {
                filters.insert(ModulesFilter::Enabled(true));
            }
            if matches.is_present("network") {
                filters.insert(ModulesFilter::Network());
            }
            if matches.is_present("secret") {
                filters.insert(ModulesFilter::RequireMemberPrivKey());
            }
            Some(list_modules(soft_meta_datas, keypairs, filters))
        } else if let Some(_matches) = cli_args.subcommand_matches("start") {
            Some(start(
                soft_meta_datas,
                keypairs,
                run_duration_in_secs,
                external_followers,
            ))
        } else if let Some(matches) = cli_args.subcommand_matches("sync") {
            let domain_or_ip = matches
                .value_of("DOMAIN_OR_IP")
                .expect("sync: you must enter a domain name or ip address !")
                .to_string();
            let port: u16 = matches
                .value_of("PORT")
                .expect("sync: you must enter a port number !")
                .parse()
                .expect("sync: port : you must enter an integer value !");
            let path = if let Some(path) = matches.value_of("PATH") {
                Some(path.to_string())
            } else {
                None
            };
            let sync_endpoint = SyncEndpoint {
                domain_or_ip,
                port,
                path,
                tls: false,
            };
            Some(sync(
                soft_meta_datas,
                keypairs,
                sync_endpoint,
                matches.is_present("cautious"),
                !matches.is_present("unsafe"),
            ))
        } else if let Some(matches) = cli_args.subcommand_matches("sync_ts") {
            let ts_profile = matches.value_of("TS_PROFILE").unwrap_or("duniter_default");
            sync_ts(
                profile.as_str(),
                &conf,
                ts_profile,
                matches.is_present("cautious"),
                !matches.is_present("unsafe"),
            );
            None
        } else if let Some(matches) = cli_args.subcommand_matches("dbex") {
            let csv = matches.is_present("csv");
            if let Some(distances_matches) = matches.subcommand_matches("distances") {
                dbex(
                    profile.as_str(),
                    &conf,
                    csv,
                    &DBExQuery::WotQuery(DBExWotQuery::AllDistances(
                        distances_matches.is_present("reverse"),
                    )),
                );
            } else if let Some(member_matches) = matches.subcommand_matches("member") {
                let uid = member_matches.value_of("UID").unwrap_or("");
                dbex(
                    profile.as_str(),
                    &conf,
                    csv,
                    &DBExQuery::WotQuery(DBExWotQuery::MemberDatas(String::from(uid))),
                );
            } else if let Some(members_matches) = matches.subcommand_matches("members") {
                if members_matches.is_present("expire") {
                    dbex(
                        profile.as_str(),
                        &conf,
                        csv,
                        &DBExQuery::WotQuery(DBExWotQuery::ExpireMembers(
                            members_matches.is_present("reverse"),
                        )),
                    );
                } else {
                    dbex(
                        profile.as_str(),
                        &conf,
                        csv,
                        &DBExQuery::WotQuery(DBExWotQuery::ListMembers(
                            members_matches.is_present("reverse"),
                        )),
                    );
                }
            } else if let Some(balance_matches) = matches.subcommand_matches("balance") {
                let address = balance_matches.value_of("ADDRESS").unwrap_or("");
                dbex(
                    &profile,
                    &conf,
                    csv,
                    &DBExQuery::TxQuery(DBExTxQuery::Balance(String::from(address))),
                );
            }
            None
        } else if let Some(matches) = cli_args.subcommand_matches("reset") {
            let mut profile_path = match dirs::config_dir() {
                Some(path) => path,
                None => panic!("Impossible to get user config directory !"),
            };
            profile_path.push(".config");
            profile_path.push(duniter_conf::get_user_datas_folder());
            profile_path.push(profile.clone());
            if !profile_path.as_path().exists() {
                panic!(format!("Error : {} profile don't exist !", profile));
            }
            match matches
                .value_of("DATAS_TYPE")
                .expect("cli param DATAS_TYPE is missing !")
            {
                "data" => {
                    let mut currency_datas_path = profile_path.clone();
                    currency_datas_path.push("g1");
                    fs::remove_dir_all(currency_datas_path.as_path())
                        .expect("Fail to remove all currency datas !");
                }
                "conf" => {
                    let mut conf_file_path = profile_path.clone();
                    conf_file_path.push("conf.json");
                    fs::remove_file(conf_file_path.as_path()).expect("Fail to remove conf file !");
                    let mut conf_keys_path = profile_path.clone();
                    conf_keys_path.push("keypairs.json");
                    fs::remove_file(conf_keys_path.as_path())
                        .expect("Fail to remove keypairs file !");
                }
                "all" => {
                    fs::remove_dir_all(profile_path.as_path())
                        .expect("Fail to remove all profile datas !");
                }
                _ => {}
            }
            None
        } else if let Some(sup_apps_fn) = sup_apps_fn {
            sup_apps_fn(profile.as_str(), &cli_args);
            None
        } else {
            panic!("unknow sub-command !")
        }
    }
    /// Start blockchain module
    pub fn start_blockchain(&self) {
        if self.network_modules_count == 0 {
            panic!("You must plug at least one network layer !");
        }
        if let UserCommand::Start() = self.user_command {
            thread::sleep(Duration::from_secs(2));
            // Create blockchain module channel
            let (blockchain_sender, blockchain_receiver): (
                mpsc::Sender<DuniterMessage>,
                mpsc::Receiver<DuniterMessage>,
            ) = mpsc::channel();

            // Send blockchain sender to rooter thread
            self.rooter_sender
                .send(RooterThreadMessage::ModuleSender(blockchain_sender))
                .expect("Fatal error: fail to send blockchain sender to rooter thread !");

            // Send modules_count to rooter thread
            self.rooter_sender
                .send(RooterThreadMessage::ModulesCount(self.modules_count + 1))
                .expect("Fatal error: fail to send modules count to rooter thread !");

            // Instantiate blockchain module and load is conf
            let mut blockchain_module = BlockchainModule::load_blockchain_conf(
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
    pub fn plug_network<NM: NetworkModule<DuRsConf, DuniterMessage>>(&mut self) {
        let enabled = enabled::<DuRsConf, DuniterMessage, NM>(&self.soft_meta_datas.conf);
        if enabled {
            if let UserCommand::Start() = self.user_command {
                self.network_modules_count += 1;
                self.plug::<NM>();
            } else if let UserCommand::Sync(ref sync_endpoint) = self.user_command {
                self.network_modules_count += 1;
                // Start module in a new thread
                let rooter_sender = self.rooter_sender.clone();
                let soft_meta_datas = self.soft_meta_datas.clone();
                let module_conf_json = self
                    .soft_meta_datas
                    .conf
                    .clone()
                    .modules()
                    .get(&NM::id().to_string().as_str())
                    .cloned();
                let keypairs = self.keypairs;
                let sync_endpoint = sync_endpoint.clone();
                self.thread_pool.execute(move || {
                    // Load module conf and keys
                    let (module_conf, required_keys) =
                        load_module_conf_and_keys::<NM>(module_conf_json, keypairs);
                    NM::sync(
                        &soft_meta_datas,
                        required_keys,
                        module_conf,
                        rooter_sender,
                        sync_endpoint,
                    ).unwrap_or_else(|_| {
                        panic!(
                            "Fatal error : fail to load {} Module !",
                            NM::id().to_string()
                        )
                    });
                });
                self.modules_count += 1;
                info!("Success to load {} module.", NM::id().to_string());
            }
        }
        if let UserCommand::ListModules(ref filters) = self.user_command {
            if module_valid_filters::<DuRsConf, DuniterMessage, NM>(
                &self.soft_meta_datas.conf,
                filters,
                true,
            ) {
                if enabled {
                    println!("{}", NM::id().to_string());
                } else {
                    println!("{} (disabled)", NM::id().to_string());
                }
            }
        }
    }
    /// Plug a module
    pub fn plug<M: DuniterModule<DuRsConf, DuniterMessage>>(&mut self) {
        let enabled = enabled::<DuRsConf, DuniterMessage, M>(&self.soft_meta_datas.conf);
        if enabled {
            if let UserCommand::Start() = self.user_command {
                // Start module in a new thread
                let rooter_sender_clone = self.rooter_sender.clone();
                let soft_meta_datas = self.soft_meta_datas.clone();
                let module_conf_json = self
                    .soft_meta_datas
                    .conf
                    .clone()
                    .modules()
                    .get(&M::id().to_string().as_str())
                    .cloned();
                let keypairs = self.keypairs;
                self.thread_pool.execute(move || {
                    // Load module conf and keys
                    let (module_conf, required_keys) =
                        load_module_conf_and_keys::<M>(module_conf_json, keypairs);
                    M::start(
                        &soft_meta_datas,
                        required_keys,
                        module_conf,
                        rooter_sender_clone,
                        false,
                    ).unwrap_or_else(|_| {
                        panic!(
                            "Fatal error : fail to load {} Module !",
                            M::id().to_string()
                        )
                    });
                });
                self.modules_count += 1;
                info!("Success to load {} module.", M::id().to_string());
            }
        }
        if let UserCommand::ListModules(ref filters) = self.user_command {
            if module_valid_filters::<DuRsConf, DuniterMessage, M>(
                &self.soft_meta_datas.conf,
                filters,
                false,
            ) {
                if enabled {
                    println!("{}", M::id().to_string());
                } else {
                    println!("{} (disabled)", M::id().to_string());
                }
            }
        }
    }
}

/// Load module conf and keys
pub fn load_module_conf_and_keys<M: DuniterModule<DuRsConf, DuniterMessage>>(
    module_conf_json: Option<serde_json::Value>,
    keypairs: DuniterKeyPairs,
) -> (M::ModuleConf, RequiredKeysContent) {
    let module_conf = if let Some(module_conf_json) = module_conf_json {
        serde_json::from_str(module_conf_json.to_string().as_str())
            .unwrap_or_else(|_| panic!("Fail to parse conf of module {}", M::id().to_string()))
    } else {
        M::ModuleConf::default()
    };
    let required_keys =
        DuniterKeyPairs::get_required_keys_content(M::ask_required_keys(), keypairs);

    (module_conf, required_keys)
}

/// Match cli option --profile
pub fn match_profile(cli_args: &ArgMatches) -> String {
    String::from(cli_args.value_of("profile").unwrap_or("default"))
}

/// List modules
pub fn list_modules<DC: DuniterConf>(
    soft_meta_datas: SoftwareMetaDatas<DC>,
    keypairs: DuniterKeyPairs,
    modules_filter: HashSet<ModulesFilter>,
) -> DuniterCore<DC> {
    // Start rooter thread
    let rooter_sender = start_rooter::<DC>(0, vec![]);

    // Instanciate DuniterCore
    DuniterCore {
        user_command: UserCommand::ListModules(modules_filter),
        soft_meta_datas,
        keypairs,
        run_duration_in_secs: 0,
        rooter_sender,
        modules_count: 0,
        network_modules_count: 0,
        thread_pool: ThreadPool::new(2),
    }
}

/// Start rooter thread
pub fn start_rooter<DC: DuniterConf>(
    run_duration_in_secs: u64,
    external_followers: Vec<mpsc::Sender<DuniterMessage>>,
) -> mpsc::Sender<RooterThreadMessage<DuniterMessage>> {
    // Create senders channel
    let (rooter_sender, main_receiver): (
        mpsc::Sender<RooterThreadMessage<DuniterMessage>>,
        mpsc::Receiver<RooterThreadMessage<DuniterMessage>>,
    ) = mpsc::channel();

    // Create rooter thread
    thread::spawn(move || {
        // Wait to receiver modules senders
        let mut modules_senders: Vec<mpsc::Sender<DuniterMessage>> = Vec::new();
        let mut modules_count_expected = None;
        while modules_count_expected.is_none()
            || modules_senders.len() < modules_count_expected.expect("safe unwrap") + 1
        {
            match main_receiver.recv_timeout(Duration::from_secs(20)) {
                Ok(mess) => {
                    match mess {
                        RooterThreadMessage::ModuleSender(module_sender) => {
                            // Subscribe this module to all others modules
                            for other_module in modules_senders.clone() {
                                if other_module
                                    .send(DuniterMessage::Followers(vec![module_sender.clone()]))
                                    .is_err()
                                {
                                    panic!("Fatal error : fail to send all modules senders to all modules !");
                                }
                            }
                            // Subcribe this module to all external_followers
                            for external_follower in external_followers.clone() {
                                if external_follower
                                    .send(DuniterMessage::Followers(vec![module_sender.clone()]))
                                    .is_err()
                                {
                                    panic!("Fatal error : fail to send all modules senders to all external_followers !");
                                }
                            }
                            // Subscribe all other modules to this module
                            if module_sender
                                .send(DuniterMessage::Followers(modules_senders.clone()))
                                .is_err()
                            {
                                panic!("Fatal error : fail to send all modules senders to all modules !");
                            }
                            // Subcribe all external_followers to this module
                            if module_sender
                                .send(DuniterMessage::Followers(external_followers.clone()))
                                .is_err()
                            {
                                panic!("Fatal error : fail to send all external_followers to all modules !");
                            }
                            // Push this module to modules_senders list
                            modules_senders.push(module_sender);
                            // Log the number of modules_senders received
                            info!(
                                "Rooter thread receive {} module senders",
                                modules_senders.len()
                            );
                        }
                        RooterThreadMessage::ModulesCount(modules_count) => {
                            info!("Rooter thread receive ModulesCount({})", modules_count);
                            if modules_senders.len() == modules_count {
                                break;
                            } else if modules_senders.len() < modules_count {
                                modules_count_expected = Some(modules_count);
                            } else {
                                panic!("Fatal error : Receive more modules_sender than expected !")
                            }
                        }
                    }
                }
                Err(e) => match e {
                    mpsc::RecvTimeoutError::Timeout => {
                        panic!("Fatal error : not receive all modules_senders after 20 secs !")
                    }
                    mpsc::RecvTimeoutError::Disconnected => {
                        panic!("Fatal error : rooter thread disconnnected !")
                    }
                },
            }
        }
        info!("Receive all modules senders.");
        if run_duration_in_secs > 0 {
            thread::sleep(Duration::from_secs(run_duration_in_secs));
            // Send DuniterMessage::Stop() to all modules
            for sender in modules_senders {
                if sender.send(DuniterMessage::Stop()).is_err() {
                    panic!("Fail to send Stop() message to one module !")
                }
            }
            thread::sleep(Duration::from_secs(2));
        }
    });

    rooter_sender
}

/// Launch duniter server
pub fn start<DC: DuniterConf>(
    soft_meta_datas: SoftwareMetaDatas<DC>,
    keypairs: DuniterKeyPairs,
    run_duration_in_secs: u64,
    external_followers: Vec<mpsc::Sender<DuniterMessage>>,
) -> DuniterCore<DC> {
    info!("Starting Duniter-rs...");

    // Start rooter thread
    let rooter_sender = start_rooter::<DC>(run_duration_in_secs, external_followers);

    // Instanciate DuniterCore
    DuniterCore {
        user_command: UserCommand::Start(),
        soft_meta_datas,
        keypairs,
        run_duration_in_secs,
        rooter_sender,
        modules_count: 0,
        network_modules_count: 0,
        thread_pool: ThreadPool::new(2),
    }
}

/// Launch synchronisation from network
pub fn sync<DC: DuniterConf>(
    soft_meta_datas: SoftwareMetaDatas<DC>,
    keypairs: DuniterKeyPairs,
    sync_endpoint: SyncEndpoint,
    _cautious: bool,
    _verif_hashs: bool,
) -> DuniterCore<DC> {
    // Start rooter thread
    let rooter_sender = start_rooter::<DC>(0, vec![]);

    // Instanciate DuniterCore
    DuniterCore {
        user_command: UserCommand::Sync(sync_endpoint),
        soft_meta_datas,
        keypairs,
        run_duration_in_secs: 0,
        rooter_sender,
        modules_count: 0,
        network_modules_count: 0,
        thread_pool: ThreadPool::new(2),
    }
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
    log_file_path.push(".config");
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
    let log_level = match cli_args.value_of("logs").unwrap_or("i") {
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
        ).expect("Fatal error : fail to create log file path !");
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
            ).expect("Fatal error : fail to open log file !"),
    )]).expect("Fatal error : fail to init logger !");
}
