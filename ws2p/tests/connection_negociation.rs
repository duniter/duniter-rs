extern crate duniter_documents;
extern crate dup_crypto;
extern crate durs_network_documents;
extern crate durs_ws2p;
extern crate durs_ws2p_messages;

use duniter_documents::CurrencyName;
use dup_crypto::keys::KeyPair;
use dup_crypto::keys::*;
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

pub static TIMEOUT_IN_MS: &'static u64 = &5_000;
pub static PORT: &'static u16 = &10899;

pub fn currency() -> CurrencyName {
    CurrencyName(String::from("g1"))
}

pub fn keypair1() -> ed25519::KeyPair {
    ed25519::KeyPairFromSaltedPasswordGenerator::with_default_parameters().generate(
        "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV".as_bytes(),
        "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV_".as_bytes(),
    )
}

pub fn keypair2() -> ed25519::KeyPair {
    ed25519::KeyPairFromSaltedPasswordGenerator::with_default_parameters().generate(
        "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWdLkjrUhHV".as_bytes(),
        "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWdLkjrUhHV_".as_bytes(),
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

//#[ignore]

#[test]
#[cfg(unix)]
fn test_connection_negociation() {
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
            &EndpointV2::parse_from_raw(&format!("WS2P 2 0 0 {} localhost", *PORT), 0, 0)
                .expect("Fail to parse endpoint"),
        )
    });

    // Listen client service channel : we must receive controller sender
    if let Ok(Ws2pServiceSender::ControllerSender(_)) = client_service_channel
        .1
        .recv_timeout(Duration::from_millis(*TIMEOUT_IN_MS))
    {
    } else {
        panic!("Not receive client controller sender");
    }

    // Listen client service channel : we must receive status TryToSendConnectMess
    test_expected_states(
        &client_service_channel.1,
        vec![WS2PConnectionState::TryToSendConnectMess],
    );

    // Listen client service channel : we must receive status WaitingConnectMess
    test_expected_states(
        &client_service_channel.1,
        vec![WS2PConnectionState::WaitingConnectMess],
    );

    // Listen server service channel : we must receive controller sender
    if let Ok(Ws2pServiceSender::ControllerSender(_)) = server_service_channel
        .1
        .recv_timeout(Duration::from_millis(*TIMEOUT_IN_MS))
    {
    } else {
        panic!("Not receive server controller sender");
    }

    // Listen server service channel : we must receive status TryToSendConnectMess
    test_expected_states(
        &server_service_channel.1,
        vec![WS2PConnectionState::TryToSendConnectMess],
    );

    // Listen server service channel : we must receive status WaitingConnectMess
    test_expected_states(
        &server_service_channel.1,
        vec![WS2PConnectionState::WaitingConnectMess],
    );

    // Listen server service channel : we must receive status ConnectMessOk
    test_expected_states(
        &server_service_channel.1,
        vec![WS2PConnectionState::ConnectMessOk],
    );
}

fn test_expected_states(
    service_receiver: &mpsc::Receiver<Ws2pServiceSender>,
    expected_states: Vec<WS2PConnectionState>,
) -> WS2PConnectionState {
    if let Ws2pServiceSender::ChangeConnectionState(_, new_state) = service_receiver
        .recv_timeout(Duration::from_millis(*TIMEOUT_IN_MS))
        .expect("Receive nothing from controller :")
    {
        for expected_state in expected_states {
            if new_state == expected_state {
                return new_state;
            }
        }
        panic!("Receive unexpected state: {:?} !", new_state);
    } else {
        panic!("Expect signal ChangeConnectionState, receive other !");
    }
}
