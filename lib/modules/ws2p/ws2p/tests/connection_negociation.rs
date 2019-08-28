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

use dubp_currency_params::CurrencyName;
use dup_crypto::keys::KeyPair;
use dup_crypto::keys::*;
//use durs_common_tests_tools::logger::init_logger_stdout;
use durs_message::DursMsg;
use durs_network_documents::network_endpoint::*;
use durs_network_documents::*;
use durs_ws2p::controllers::incoming_connections::*;
use durs_ws2p::controllers::outgoing_connections::*;
use durs_ws2p_messages::v2::api_features::*;
use durs_ws2p_messages::v2::connect::WS2Pv2ConnectType;
use durs_ws2p_protocol::connection_state::WS2PConnectionState;
use durs_ws2p_protocol::controller::{WS2PControllerEvent, WebsocketActionOrder};
use durs_ws2p_protocol::orchestrator::OrchestratorMsg;
use durs_ws2p_protocol::MySelfWs2pNode;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub static TIMEOUT_IN_MS: &'static u64 = &20_000;
pub static PORT: &'static u16 = &10899;

pub fn currency() -> CurrencyName {
    CurrencyName(String::from("g1"))
}

pub fn keypair1() -> ed25519::Ed25519KeyPair {
    let seed = Seed32::new([
        61u8, 245, 136, 162, 155, 50, 205, 43, 116, 15, 45, 84, 138, 54, 114, 214, 71, 213, 11,
        251, 135, 182, 202, 131, 48, 91, 166, 226, 40, 255, 251, 172,
    ]);
    ed25519::KeyPairFromSeed32Generator::generate(seed)
}

pub fn keypair2() -> ed25519::Ed25519KeyPair {
    let seed = Seed32::new([
        228, 125, 124, 120, 57, 212, 246, 250, 139, 246, 62, 26, 56, 241, 175, 123, 151, 209, 5,
        106, 2, 148, 43, 101, 118, 160, 233, 7, 112, 222, 0, 169,
    ]);
    ed25519::KeyPairFromSeed32Generator::generate(seed)
}

fn server_node() -> MySelfWs2pNode {
    MySelfWs2pNode {
        my_node_id: NodeId(0),
        my_key_pair: KeyPairEnum::Ed25519(keypair1()),
        my_features: WS2PFeatures([5u8, 0, 0, 0]),
    }
}

fn client_node() -> MySelfWs2pNode {
    MySelfWs2pNode {
        my_node_id: NodeId(1),
        my_key_pair: KeyPairEnum::Ed25519(keypair2()),
        my_features: WS2PFeatures([5u8, 0, 0, 0]),
    }
}

//#[ignore]
#[test]
#[cfg(unix)]
fn test_connection_negociation_denial() {
    //init_logger_stdout();

    // ===== initialization =====
    // client and server are initialized and launched in separate threads

    let server_node = server_node();
    let client_node = client_node();

    // Create server service channel
    let server_service_channel = mpsc::channel();

    // Launch server controller
    let server_node_clone = server_node.clone();
    let server_service_sender = server_service_channel.0.clone();
    thread::spawn(move || {
        listen_on_ws2p_v2_endpoint(
            &currency(),
            &server_service_sender,
            &server_node_clone,
            format!("localhost:{}", *PORT + 1),
        )
    });

    // Create client service channel
    let client_service_channel = mpsc::channel();

    // launch client controller
    let server_node_clone = server_node.clone();
    let client_service_sender = client_service_channel.0.clone();
    thread::spawn(move || {
        connect_to_ws2p_v2_endpoint(
            &currency(),
            &client_service_sender,
            &client_node,
            Some(NodeFullId(
                NodeId(2),
                server_node_clone.my_key_pair.public_key(),
            )),
            &EndpointV2::parse_from_raw(&format!("WS2P V2 localhost {}", *PORT + 1))
                .expect("Fail to parse endpoint"),
        )
    });

    // ===== opening connection =====
    // we must get Ws2pServiceSender::ControllerSender from the client and server threads (but we ignore them)
    // we also test that the statuses match expected ones

    let client_controller = get_controller(&client_service_channel.1);
    let server_controller = get_controller(&server_service_channel.1);

    // TryToSendConnectMsg
    let state = get_state(&server_service_channel.1); // server
    assert_eq!(WS2PConnectionState::TryToSendConnectMsg, state);
    let state = get_state(&client_service_channel.1); // client
    assert_eq!(WS2PConnectionState::TryToSendConnectMsg, state);

    // WaitingConnectMsg
    let state = get_state(&server_service_channel.1); // server
    assert_eq!(WS2PConnectionState::WaitingConnectMsg, state);
    let state = get_state(&client_service_channel.1); // client
    assert_eq!(WS2PConnectionState::WaitingConnectMsg, state);

    // ConnectMessOk & Denial
    let state = get_state(&server_service_channel.1); // server
    assert_eq!(WS2PConnectionState::ConnectMessOk, state);
    let state = get_state(&client_service_channel.1); // client
    assert_eq!(WS2PConnectionState::Denial, state);

    // Stop
    let _ = client_controller.send(WebsocketActionOrder::close());
    let _ = server_controller.send(WebsocketActionOrder::close());
}

//#[ignore]
#[test]
#[cfg(unix)]
fn test_connection_negociation_success() {
    //init_logger_stdout();

    // ===== initialization =====
    // client and server are initialized and launched in separate threads

    let server_node = server_node();
    let client_node = client_node();

    // Create server service channel
    let server_service_channel = mpsc::channel();

    // Launch server controller
    let server_node_clone = server_node.clone();
    let server_service_sender = server_service_channel.0.clone();
    thread::spawn(move || {
        listen_on_ws2p_v2_endpoint(
            &currency(),
            &server_service_sender,
            &server_node_clone,
            format!("localhost:{}", *PORT),
        )
    });

    // Wait server ready...
    //thread::sleep(Duration::from_millis(500));

    // Create client service channel
    let client_service_channel = mpsc::channel();

    // launch client controller
    let client_node_clone = client_node.clone();
    let server_node_clone = server_node.clone();
    let client_service_sender = client_service_channel.0.clone();
    thread::spawn(move || {
        connect_to_ws2p_v2_endpoint(
            &currency(),
            &client_service_sender,
            &client_node_clone,
            Some(server_node_clone.get_full_id()),
            &EndpointV2::parse_from_raw(&format!("WS2P V2 localhost {}", *PORT))
                .expect("Fail to parse endpoint"),
        )
    });

    // ===== opening connection =====
    // we must get Ws2pServiceSender::ControllerSender from the client and server threads (but we ignore them)
    // we also test that the statuses match expected ones

    let _client_controller = get_controller(&client_service_channel.1);
    let _server_controller = get_controller(&server_service_channel.1);

    // TryToSendConnectMsg
    let state = get_state(&client_service_channel.1); // client
    assert_eq!(WS2PConnectionState::TryToSendConnectMsg, state);
    let state = get_state(&server_service_channel.1); // server
    assert_eq!(WS2PConnectionState::TryToSendConnectMsg, state);

    // WaitingConnectMsg
    let state = get_state(&client_service_channel.1); // client
    assert_eq!(WS2PConnectionState::WaitingConnectMsg, state);
    let state = get_state(&server_service_channel.1); // server
    assert_eq!(WS2PConnectionState::WaitingConnectMsg, state);

    // ConnectMessOk
    let state = get_state(&client_service_channel.1); // client
    assert_eq!(WS2PConnectionState::ConnectMessOk, state);
    let state = get_state(&server_service_channel.1); // server
    assert_eq!(WS2PConnectionState::ConnectMessOk, state);

    // Ack message
    let state_1 = get_state(&client_service_channel.1); // client
    let state_2 = get_state(&server_service_channel.1); // server

    println!("state_1: {:?}", &state_1);
    println!("state_2: {:?}", &state_2);

    assert!(
        // client faster
        ( state_1 == WS2PConnectionState::OkMsgOkWaitingAckMsg &&
        state_2 == WS2PConnectionState::AckMsgOk ) ||
        // server faster
        ( state_1 == WS2PConnectionState::AckMsgOk &&
        state_2 == WS2PConnectionState::OkMsgOkWaitingAckMsg ) ||
        // ack messages received at the same time
        ( state_1 == WS2PConnectionState::AckMsgOk &&
        state_2 == WS2PConnectionState::AckMsgOk )
    );

    // Established for client
    expected_event(
        &client_service_channel.1,
        WS2PControllerEvent::NewConnEstablished {
            conn_type: WS2Pv2ConnectType::OutgoingServer,
            remote_full_id: server_node.get_full_id(),
        },
    );
    // Established for server
    expected_event(
        &server_service_channel.1,
        WS2PControllerEvent::NewConnEstablished {
            conn_type: WS2Pv2ConnectType::OutgoingServer,
            remote_full_id: client_node.get_full_id(),
        },
    );
}

// === functions used in above test ===

// Get established event in a receiver
fn expected_event(
    orchestrator_receiver: &mpsc::Receiver<OrchestratorMsg<DursMsg>>,
    expected_event: WS2PControllerEvent,
) {
    match orchestrator_receiver
        .recv_timeout(Duration::from_millis(*TIMEOUT_IN_MS))
        .expect("Receive nothing from controller :")
    {
        OrchestratorMsg::ControllerEvent { event, .. } => assert_eq!(expected_event, event),
        other => panic!("Expect signal ControllerEvent, receive '{:?}' !", other),
    }
}

// get the state in a receiver
fn get_state(
    orchestrator_receiver: &mpsc::Receiver<OrchestratorMsg<DursMsg>>,
) -> WS2PConnectionState {
    match orchestrator_receiver
        .recv_timeout(Duration::from_millis(*TIMEOUT_IN_MS))
        .expect("Receive nothing from controller :")
    {
        OrchestratorMsg::ControllerEvent {
            event: WS2PControllerEvent::StateChange { new_state },
            ..
        } => new_state,
        other => panic!(
            "Expect signal ChangeConnectionState, receive '{:?}' !",
            other
        ),
    }
}

// get the controller from the thread
fn get_controller(
    orchestrator_receiver: &mpsc::Receiver<OrchestratorMsg<DursMsg>>,
) -> mpsc::Sender<WebsocketActionOrder> {
    // we must receive controller sender
    if let Ok(OrchestratorMsg::ControllerSender(controller_sender)) =
        orchestrator_receiver.recv_timeout(Duration::from_millis(*TIMEOUT_IN_MS))
    {
        return controller_sender;
    } else {
        panic!("Not receive client controller sender");
    }
}
