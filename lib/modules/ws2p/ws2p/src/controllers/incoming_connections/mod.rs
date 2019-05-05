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

//! WS2P incoming connections controllers.

use dubp_documents::CurrencyName;
//use durs_module::ModuleReqId;
use crate::controllers::handler::Ws2pConnectionHandler;
use crate::controllers::*;
use crate::services::*;
use ws::deflate::DeflateBuilder;
use ws::listen;
//use durs_network::*;
use durs_ws2p_messages::v2::connect::WS2Pv2ConnectType;
use std::sync::mpsc;

/// Listen on WSPv2 host:port
pub fn listen_on_ws2p_v2_endpoint(
    currency: &CurrencyName,
    service_sender: mpsc::Sender<Ws2pServiceSender>,
    self_node: MySelfWs2pNode,
    host: &str,
    port: u16,
) -> ws::Result<()> {
    // Get endpoint url
    let ws_url = format!("{}:{}", host, port);

    // Log
    info!("Listen on {} ...", ws_url);
    println!("DEBUG: call function listen({}) ...", ws_url);

    // Connect to websocket
    listen(ws_url, move |ws| {
        println!("DEBUG: Listen on host:port...");
        DeflateBuilder::new().build(
            Ws2pConnectionHandler::try_new(
                WsSender(ws),
                service_sender.clone(),
                currency.clone(),
                self_node.clone(),
                Ws2pConnectionDatas::new(WS2Pv2ConnectType::Incoming),
            )
            .expect("WS2P Service unrechable"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use dup_crypto::keys::*;
    use std::thread;
    use std::time::Duration;

    pub fn _keypair1() -> ed25519::KeyPair {
        ed25519::KeyPairFromSaltedPasswordGenerator::with_default_parameters().generate(
            "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV".as_bytes(),
            "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV_".as_bytes(),
        )
    }

    //#[test]
    fn _listen_on_localhost() {
        // create service channel
        let (service_sender, _service_receiver): (
            mpsc::Sender<Ws2pServiceSender>,
            mpsc::Receiver<Ws2pServiceSender>,
        ) = mpsc::channel();

        thread::spawn(move || {
            let result = listen_on_ws2p_v2_endpoint(
                &CurrencyName(String::from("default")),
                service_sender,
                MySelfWs2pNode {
                    my_node_id: NodeId(1),
                    my_key_pair: KeyPairEnum::Ed25519(_keypair1()),
                    my_features: WS2PFeatures(vec![5u8]),
                },
                "localhost",
                10899,
            );
            if let Err(e) = result {
                panic!("Listen error: {}", e);
            }
        });

        thread::sleep(Duration::from_secs(10));

        // Force to print stdout
        assert!(false);
    }
}
