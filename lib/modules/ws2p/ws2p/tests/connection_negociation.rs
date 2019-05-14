//  Copyright (C) 2018  The Durs Project Developers.
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

use dubp_documents::CurrencyName;
use dup_crypto::keys::KeyPair;
use dup_crypto::keys::*;
use durs_common_tests_tools::logger::init_logger_stdout;
use durs_network_documents::network_endpoint::*;
use durs_network_documents::*;
use durs_ws2p::controllers::incoming_connections::*;
use durs_ws2p::controllers::outgoing_connections::*;
use durs_ws2p::controllers::*;
use durs_ws2p::services::*;
use durs_ws2p_messages::v2::api_features::*;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub static TIMEOUT_IN_MS: &'static u64 = &20_000;
pub static PORT: &'static u16 = &10899;

pub fn currency() -> CurrencyName {
    CurrencyName(String::from("g1"))
}

pub fn keypair1() -> ed25519::KeyPair {
    ed25519::KeyPairFromSaltedPasswordGenerator::with_default_parameters().generate(
        "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV1".as_bytes(),
        "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV1_".as_bytes(),
    )
}

pub fn keypair2() -> ed25519::KeyPair {
    ed25519::KeyPairFromSaltedPasswordGenerator::with_default_parameters().generate(
        "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWdLkjrUhHV2".as_bytes(),
        "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWdLkjrUhHV2_".as_bytes(),
    )
}

fn server_node() -> MySelfWs2pNode {
    MySelfWs2pNode {
        my_node_id: NodeId(0),
        my_key_pair: KeyPairEnum::Ed25519(keypair1()),
        my_features: WS2PFeatures(vec![5u8]),
    }
}

fn client_node() -> MySelfWs2pNode {
    MySelfWs2pNode {
        my_node_id: NodeId(1),
        my_key_pair: KeyPairEnum::Ed25519(keypair2()),
        my_features: WS2PFeatures(vec![5u8]),
    }
}

#[ignore]
#[test]
#[cfg(unix)]
fn test_connection_negociation() {
    init_logger_stdout();

    // ===== initialization =====
    // client and server are initialized and launched in separate threads

    let server_node = server_node();
    let client_node = client_node();

    // Create server service channel
    let server_service_channel: (
        mpsc::Sender<Ws2pServiceSender>,
        mpsc::Receiver<Ws2pServiceSender>,
    ) = mpsc::channel();

    // Launch server controller
    let server_node_clone = server_node.clone();
    let server_service_sender = server_service_channel.0.clone();
    thread::spawn(move || {
        listen_on_ws2p_v2_endpoint(
            &currency(),
            server_service_sender,
            server_node_clone,
            "localhost",
            *PORT,
        )
    });

    // Wait server ready...
    //thread::sleep(Duration::from_millis(500));

    // Create client service channel
    let client_service_channel: (
        mpsc::Sender<Ws2pServiceSender>,
        mpsc::Receiver<Ws2pServiceSender>,
    ) = mpsc::channel();

    // launch client controller
    let server_node_clone = server_node.clone();
    let client_service_sender = client_service_channel.0.clone();
    thread::spawn(move || {
        connect_to_ws2p_v2_endpoint(
            &currency(),
            &client_service_sender,
            &client_node,
            Some(NodeFullId(
                server_node_clone.my_node_id,
                server_node_clone.my_key_pair.public_key(),
            )),
            &EndpointV2::parse_from_raw(&format!("WS2P 2 localhost {}", *PORT))
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
    assert!(state == WS2PConnectionState::TryToSendConnectMsg);
    let state = get_state(&server_service_channel.1); // server
    assert!(state == WS2PConnectionState::TryToSendConnectMsg);

    // WaitingConnectMsg
    let state = get_state(&client_service_channel.1); // client
    assert!(state == WS2PConnectionState::WaitingConnectMsg);
    let state = get_state(&server_service_channel.1); // server
    assert!(state == WS2PConnectionState::WaitingConnectMsg);

    // ConnectMessOk
    let state = get_state(&client_service_channel.1); // client
    assert!(state == WS2PConnectionState::ConnectMessOk);
    let state = get_state(&server_service_channel.1); // server
    assert!(state == WS2PConnectionState::ConnectMessOk);

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

    // Established
    let state = get_state(&client_service_channel.1); // client
    assert!(state == WS2PConnectionState::Established);
    let state = get_state(&server_service_channel.1); // server
    assert!(state == WS2PConnectionState::Established);
}

// === functions used in above test ===

// get the state in a receiver
fn get_state(service_receiver: &mpsc::Receiver<Ws2pServiceSender>) -> WS2PConnectionState {
    if let Ws2pServiceSender::ChangeConnectionState(_, new_state) = service_receiver
        .recv_timeout(Duration::from_millis(*TIMEOUT_IN_MS))
        .expect("Receive nothing from controller :")
    {
        return new_state;
    } else {
        panic!("Expect signal ChangeConnectionState, receive other !");
    }
}

// get the controller from the thread
fn get_controller(
    service_receiver: &mpsc::Receiver<Ws2pServiceSender>,
) -> mpsc::Sender<Ws2pControllerOrder> {
    // we must receive controller sender
    if let Ok(Ws2pServiceSender::ControllerSender(controller)) =
        service_receiver.recv_timeout(Duration::from_millis(*TIMEOUT_IN_MS))
    {
        return controller;
    } else {
        panic!("Not receive client controller sender");
    }
}
