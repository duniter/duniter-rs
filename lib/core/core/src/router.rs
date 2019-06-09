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

//! Relay messages between durs modules.

use durs_common_tools::fatal_error;
use durs_conf::DuRsConf;
use durs_message::*;
use durs_module::*;
use durs_network_documents::network_endpoint::{ApiPart, EndpointEnum};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::RecvTimeoutError;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;

static MAX_REGISTRATION_DELAY: &'static u64 = &20;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum DursMsgReceiver {
    Role(ModuleRole),
    Event(ModuleEvent),
    One(ModuleStaticName),
}

/// Start broadcasting thread
fn start_broadcasting_thread(
    start_time: SystemTime,
    receiver: &mpsc::Receiver<RouterThreadMessage<DursMsg>>,
    _external_followers: &[mpsc::Sender<DursMsg>],
) {
    // Define variables
    let mut modules_senders: HashMap<ModuleStaticName, mpsc::Sender<DursMsg>> = HashMap::new();
    let mut pool_msgs: HashMap<DursMsgReceiver, Vec<DursMsg>> = HashMap::new();
    let mut events_subscriptions: HashMap<ModuleEvent, Vec<ModuleStaticName>> = HashMap::new();
    let mut roles: HashMap<ModuleRole, Vec<ModuleStaticName>> = HashMap::new();
    let mut registrations_count = 0;
    let mut expected_registrations_count = None;
    let mut local_node_endpoints: Vec<EndpointEnum> = Vec::new();
    let mut reserved_apis_parts: HashMap<ModuleStaticName, Vec<ApiPart>> = HashMap::new();

    loop {
        match receiver.recv_timeout(Duration::from_secs(1)) {
            Ok(mess) => {
                match mess {
                    RouterThreadMessage::ModulesCount(modules_count) => {
                        expected_registrations_count = Some(modules_count)
                    }
                    RouterThreadMessage::ModuleRegistration {
                        static_name: module_static_name,
                        sender: module_sender,
                        roles: module_roles,
                        events_subscription,
                        reserved_apis_parts: module_reserved_apis_parts,
                        endpoints: mut module_endpoints,
                    } => {
                        registrations_count += 1;
                        // For all events
                        for event in events_subscription {
                            // Send pending message of this event
                            for msg in pool_msgs
                                .get(&DursMsgReceiver::Event(event))
                                .unwrap_or(&Vec::with_capacity(0))
                            {
                                module_sender.send(msg.clone()).unwrap_or_else(|_| {
                                    fatal_error!(
                                        "fail to relay DursMsg to {:?} !",
                                        module_static_name
                                    )
                                });
                            }
                            // Store event subscription
                            events_subscriptions
                                .entry(event)
                                .or_insert_with(Vec::new)
                                .push(module_static_name);
                        }
                        // For all roles
                        for role in module_roles {
                            // Send pending message for this role
                            for msg in pool_msgs
                                .get(&DursMsgReceiver::Role(role))
                                .unwrap_or(&Vec::with_capacity(0))
                            {
                                module_sender.send(msg.clone()).unwrap_or_else(|_| {
                                    fatal_error!(
                                        "fail to relay DursMsg to {:?} !",
                                        module_static_name
                                    )
                                });
                            }
                            // Store sender roles
                            roles
                                .entry(role)
                                .or_insert_with(Vec::new)
                                .push(module_static_name);
                        }
                        // For all reserved apis parts
                        for other_module_reserved_apis_parts in reserved_apis_parts.values() {
                            for other_api_part in other_module_reserved_apis_parts {
                                for api_part in &module_reserved_apis_parts {
                                    if api_part.union_exist(other_api_part) {
                                        fatal_error!(
                                            "two modules try to reserve same api name '{}' with at least 1 version in common '({:?}; {:?})' !",
                                            api_part.name.0,
                                            api_part.versions,
                                            other_api_part.versions,
                                        );
                                    }
                                }
                            }
                        }
                        // For all endpoints
                        for ep in &module_endpoints {
                            let ep_api = ep.api();
                            let ep_version = ep.version();

                            if module_reserved_apis_parts
                                .iter()
                                .filter(|api_part| api_part.contains(&ep_api, ep_version))
                                .count()
                                == 0
                            {
                                fatal_error!(
                                    "Module {} try to declare endpoint with undeclared api part (name: '{}', version: '{}') !",
                                    module_static_name.0,
                                    ep_api.0,
                                    ep_version.0,
                                );
                            }
                            /*for other_module_ep in &local_node_endpoints {
                                if ep_api == other_module_ep.api() && ep_version == other_module_ep.version() {
                                    fatal_error!(
                                        "two modules try to declare endpoint of same api '{}' and same version '{}' !",
                                        ep_api.0,
                                        ep_version.0,
                                    );
                                }
                            }*/
                        }
                        // Store reserved APIs parts
                        reserved_apis_parts.insert(module_static_name, module_reserved_apis_parts);
                        // Add module endpoints to local node endpoints
                        local_node_endpoints.append(&mut module_endpoints);

                        // If all modules registered
                        if expected_registrations_count.is_some()
                            && registrations_count == expected_registrations_count.unwrap()
                        {
                            // Get list of InterNodesNetwork modules
                            let receivers = roles
                                .get(&ModuleRole::InterNodesNetwork)
                                .expect("Fatal error : no module with role InterNodesNetwork !")
                                .to_vec();
                            // Send endpoints to network module
                            send_msg_to_several_receivers(
                                DursMsg::ModulesEndpoints(local_node_endpoints.clone()),
                                &receivers,
                                &modules_senders,
                            );
                        }
                        // Add this sender to modules_senders
                        modules_senders.insert(module_static_name, module_sender);
                    }
                    RouterThreadMessage::ModuleMessage(msg) => match msg {
                        DursMsg::Stop => break,
                        DursMsg::Event {
                            event_from,
                            event_type,
                            ..
                        } => {
                            // the node to be started less than MAX_REGISTRATION_DELAY seconds ago,
                            // keep the message in memory to be able to send it back to modules not yet plugged
                            store_msg_in_pool(start_time, &msg, &mut pool_msgs);
                            // Get list of receivers
                            let receivers = events_subscriptions
                                .get(&event_type)
                                .unwrap_or(&Vec::with_capacity(0))
                                .iter()
                                .filter(|module_static_name| **module_static_name != event_from)
                                .cloned()
                                .collect::<Vec<ModuleStaticName>>();
                            // Send msg to receivers
                            send_msg_to_several_receivers(msg, &receivers, &modules_senders)
                        }
                        DursMsg::Request { req_to: role, .. } => {
                            // If the node to be started less than MAX_REGISTRATION_DELAY seconds ago,
                            // keep the message in memory to be able to send it back to modules not yet plugged
                            store_msg_in_pool(start_time, &msg, &mut pool_msgs);
                            // Get list of receivers
                            let receivers =
                                roles.get(&role).unwrap_or(&Vec::with_capacity(0)).to_vec();
                            // Send msg to receivers
                            send_msg_to_several_receivers(msg, &receivers, &modules_senders)
                        }
                        _ => {} // Others DursMsg variants
                    },
                }
            }
            Err(e) => match e {
                RecvTimeoutError::Timeout => continue,
                RecvTimeoutError::Disconnected => fatal_error!("router thread disconnnected !"),
            },
        }
        if (expected_registrations_count.is_none()
            || registrations_count < expected_registrations_count.unwrap())
            && SystemTime::now()
                .duration_since(start_time)
                .expect("Duration error !")
                .as_secs()
                > *MAX_REGISTRATION_DELAY
        {
            fatal_error!(
                "{} modules have registered, but expected {} !",
                registrations_count,
                expected_registrations_count.unwrap_or(0)
            );
        }
    }
}

/// Start conf thread
fn start_conf_thread(
    profile_path: PathBuf,
    mut conf: DuRsConf,
    receiver: &mpsc::Receiver<DursMsg>,
) {
    let conf_path = durs_conf::get_conf_path(&profile_path);
    loop {
        match receiver.recv() {
            Ok(msg) => {
                if let DursMsg::SaveNewModuleConf(module_static_name, new_json_conf) = msg {
                    conf.set_module_conf(ModuleName(module_static_name.to_string()), new_json_conf);
                    durs_conf::write_conf_file(&conf_path, &conf)
                        .expect("Fail to write new module conf in conf file ! ");
                }
            }
            Err(_) => {
                info!("Conf thread stops.");
                break;
            }
        }
    }
}

/// Send msg to several receivers
fn send_msg_to_several_receivers(
    msg: DursMsg,
    receivers: &[ModuleStaticName],
    modules_senders: &HashMap<ModuleStaticName, mpsc::Sender<DursMsg>>,
) {
    if !receivers.is_empty() {
        // Send message by copy To all modules that subscribed to this event
        for module_static_name in &receivers[1..] {
            if let Some(module_sender) = modules_senders.get(module_static_name) {
                module_sender.send(msg.clone()).unwrap_or_else(|_| {
                    fatal_error!("fail to relay DursMsg to {:?} !", module_static_name)
                });
            }
        }
        // Send message by move to the last module to be receive
        if let Some(module_sender) = modules_senders.get(&receivers[0]) {
            module_sender
                .send(msg)
                .unwrap_or_else(|_| fatal_error!("Fail to relay DursMsg to {:?} !", receivers[0]));
        }
    }
}

/// If the node to be started less than MAX_REGISTRATION_DELAY seconds ago,
/// keep the message in memory to be able to send it back to modules not yet plugged
fn store_msg_in_pool(
    start_time: SystemTime,
    msg: &DursMsg,
    pool_msgs: &mut HashMap<DursMsgReceiver, Vec<DursMsg>>,
) {
    if SystemTime::now()
        .duration_since(start_time)
        .expect("Duration error !")
        .as_secs()
        < *MAX_REGISTRATION_DELAY
    {
        let msg_recv = match msg {
            DursMsg::Event { event_type, .. } => Some(DursMsgReceiver::Event(*event_type)),
            DursMsg::Request { req_to, .. } => Some(DursMsgReceiver::Role(*req_to)),
            DursMsg::Response { res_to, .. } => Some(DursMsgReceiver::One(*res_to)),
            _ => None,
        };
        if let Some(msg_recv) = msg_recv {
            pool_msgs
                .entry(msg_recv)
                .or_insert_with(Vec::new)
                .push(msg.clone());
        }
    } else if !pool_msgs.is_empty() {
        // Clear pool_msgs
        pool_msgs.clear();
    }
}

/// Start router thread
pub fn start_router(
    run_duration_in_secs: u64,
    profile_path: PathBuf,
    conf: DuRsConf,
    external_followers: Vec<mpsc::Sender<DursMsg>>,
) -> mpsc::Sender<RouterThreadMessage<DursMsg>> {
    let start_time = SystemTime::now();

    // Create router channel
    let (router_sender, router_receiver): (
        mpsc::Sender<RouterThreadMessage<DursMsg>>,
        mpsc::Receiver<RouterThreadMessage<DursMsg>>,
    ) = mpsc::channel();

    // Create router thread
    thread::spawn(move || {
        // Create broadcasting thread channel
        let (broadcasting_sender, broadcasting_receiver): (
            mpsc::Sender<RouterThreadMessage<DursMsg>>,
            mpsc::Receiver<RouterThreadMessage<DursMsg>>,
        ) = mpsc::channel();

        // Create broadcasting thread
        thread::spawn(move || {
            start_broadcasting_thread(start_time, &broadcasting_receiver, &external_followers);
        });

        // Create conf thread channel
        let (conf_sender, conf_receiver): (mpsc::Sender<DursMsg>, mpsc::Receiver<DursMsg>) =
            mpsc::channel();

        // Create conf thread
        thread::spawn(move || {
            start_conf_thread(profile_path.clone(), conf, &conf_receiver);
        });

        // Define variables
        let mut modules_senders: HashMap<ModuleStaticName, mpsc::Sender<DursMsg>> = HashMap::new();
        let mut pool_msgs: HashMap<ModuleStaticName, Vec<DursMsg>> = HashMap::new();

        // Wait to receiver modules senders
        loop {
            match router_receiver.recv_timeout(Duration::from_secs(1)) {
                Ok(mess) => {
                    match mess {
                        RouterThreadMessage::ModulesCount(expected_registrations_count) => {
                            // Relay to broadcasting thread
                            broadcasting_sender
                                .send(RouterThreadMessage::ModulesCount(
                                    expected_registrations_count,
                                ))
                                .expect(
                                    "Fail to relay ModulesCount message to broadcasting thread !",
                                );
                        }
                        RouterThreadMessage::ModuleRegistration {
                            static_name: module_static_name,
                            sender: module_sender,
                            events_subscription,
                            roles,
                            reserved_apis_parts,
                            endpoints,
                        } => {
                            // Send pending messages destined specifically to this module
                            if let Some(msgs) = pool_msgs.remove(&module_static_name) {
                                for msg in msgs {
                                    module_sender.send(msg).unwrap_or_else(|_| {
                                        fatal_error!(
                                            "Fail to relay DursMsg to {:?} !",
                                            module_static_name
                                        )
                                    });
                                }
                            }
                            // Add this sender to modules_senders
                            modules_senders.insert(module_static_name, module_sender.clone());
                            // Relay to broadcasting thread
                            broadcasting_sender
                                .send(RouterThreadMessage::ModuleRegistration {
                                    static_name: module_static_name,
                                    sender: module_sender,
                                    events_subscription,
                                    roles,
                                    reserved_apis_parts,
                                    endpoints,
                                })
                                .expect(
                                    "Fail to relay module registration to broadcasting thread !",
                                );
                            // Log the number of modules_senders received
                            info!(
                                "Router thread receive '{}' module registration ({} modules registered).",
                                module_static_name.0,
                                modules_senders.len()
                            );
                        }
                        RouterThreadMessage::ModuleMessage(msg) => {
                            trace!("Router thread receive ModuleMessage({:?})", msg);
                            match msg {
                                DursMsg::Stop => {
                                    info!("TMP: Router: RECEIVE STOP MESSAGE !");
                                    // Relay stop signal to broadcasting thread
                                    broadcasting_sender
                                        .send(RouterThreadMessage::ModuleMessage(msg))
                                        .expect("Fail to relay message to broadcasting thread !");
                                    // Relay stop message to all modules
                                    for module_sender in modules_senders.values() {
                                        if module_sender.send(DursMsg::Stop).is_err() {
                                            warn!("Fail to relay stop to modules !");
                                        }
                                    }
                                    break;
                                }
                                DursMsg::SaveNewModuleConf(_, _) => {
                                    // Forward it to the conf thread
                                    conf_sender
                                        .send(msg)
                                        .expect("Fail to reach conf thread !");
                                }
                                DursMsg::Request{ .. } => {
                                    broadcasting_sender
                                        .send(RouterThreadMessage::ModuleMessage(msg))
                                        .expect(
                                            "Fail to relay specific role message to broadcasting thread !",
                                        );
                                }
                                DursMsg::Event{ .. } => broadcasting_sender
                                    .send(RouterThreadMessage::ModuleMessage(msg))
                                    .expect("Fail to relay specific event message to broadcasting thread !"),
                                DursMsg::Response {
                                    res_to: module_static_name,
                                    ..
                                } => {
                                    if let Some(module_sender) =
                                        modules_senders.get(&module_static_name)
                                    {
                                        module_sender.send(msg).unwrap_or_else(|_| {
                                            fatal_error!(
                                                "Fail to relay DursMsg to {:?} !",
                                                module_static_name
                                            )
                                        });
                                    } else if SystemTime::now()
                                        .duration_since(start_time)
                                        .expect("Duration error !")
                                        .as_secs()
                                        < *MAX_REGISTRATION_DELAY
                                    {
                                        pool_msgs
                                            .entry(module_static_name)
                                            .or_insert_with(Vec::new)
                                            .push(msg);
                                    } else {
                                        if !pool_msgs.is_empty() {
                                            pool_msgs = HashMap::with_capacity(0);
                                        }
                                        warn!(
                                            "Message for unknow receiver : {:?}.",
                                            module_static_name
                                        );
                                    }
                                }
                                DursMsg::ModulesEndpoints(_) => {
                                    warn!("A module try to send reserved router message: ModulesEndpoints.");
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    if let RecvTimeoutError::Disconnected = e {
                        warn!("Router thread disconnnected... break router main loop.");
                        break;
                    }
                }
            }
            if run_duration_in_secs > 0
                && SystemTime::now()
                    .duration_since(start_time)
                    .expect("Duration error !")
                    .as_secs()
                    > run_duration_in_secs
            {
                broadcasting_sender
                    .send(RouterThreadMessage::ModuleMessage(DursMsg::Stop))
                    .expect("Fail to relay stop message to broadcasting thread !");
                break;
            }
        }
        info!("Router thread stop.")
    });

    router_sender
}
