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

mod common;

use crate::common::*;
use dubp_block_doc::{block::BlockDocumentTrait, BlockDocument};
use dubp_common_doc::BlockNumber;
use dubp_currency_params::genesis_block_params::v10::BlockV10Parameters;
use dup_crypto::keys::{KeyPair, Signator, SignatorEnum};
use durs_blockchain::BlockchainModule;
use durs_message::events::{BlockchainEvent, DursEvent};
use durs_message::requests::DursReqContent;
use durs_message::DursMsg;
use durs_module::{
    ModuleEvent, ModuleReqFullId, ModuleReqId, ModuleRole, ModuleStaticName, RouterThreadMessage,
};
use durs_network::events::NetworkEvent;
use durs_network::requests::OldNetworkRequest;
use pretty_assertions::assert_eq;
use std::sync::mpsc::{channel, Receiver, Sender};

#[cfg(unix)]
#[test]
fn test_revert_block() {
    // Init test
    let tmp_profile_path = common::init();

    // Router channel
    let (router_sender, router_receiver) = channel(); // RouterThreadMessage<DursMsg>

    let genesis_params = BlockV10Parameters::default();

    let mut bc = init_bc_module(router_sender, genesis_params, tmp_profile_path.as_path());

    // Create blockchain module channel
    let (bc_sender, bc_receiver): (Sender<DursMsg>, Receiver<DursMsg>) = channel();

    let handle = std::thread::spawn(move || {
        bc.start_blockchain(&bc_receiver, None);
    });

    // Receive 11 requests GetBlocks
    for i in 0..11 {
        let msg = router_receiver
            .recv()
            .expect("blockchain module disconnected.");
        if let RouterThreadMessage::ModuleMessage(durs_msg) = msg {
            assert_eq!(
                DursMsg::Request {
                    req_from: BlockchainModule::name(),
                    req_to: ModuleRole::InterNodesNetwork,
                    req_id: ModuleReqId(i),
                    req_content: DursReqContent::OldNetworkRequest(OldNetworkRequest::GetBlocks(
                        ModuleReqFullId(BlockchainModule::name(), ModuleReqId(i)),
                        50,
                        i * 50
                    )),
                },
                durs_msg
            );
            log::info!("Router receive: {:?}", durs_msg);
        } else {
            panic!("Expect ModuleMesage, found: {:?}", msg)
        }
    }

    // Receive first g1-test chunk
    let gt_chunk_0 = dubp_blocks_tests_tools::gt::get_gt_chunk(0);
    receive_valid_blocks(&bc_sender, &router_receiver, gt_chunk_0);

    // Receive second g1-test chunk
    let gt_chunk_1 = dubp_blocks_tests_tools::gt::get_gt_chunk(1);
    receive_valid_blocks(&bc_sender, &router_receiver, gt_chunk_1);

    // Receive third g1-test chunk
    let mut gt_chunk_2 = dubp_blocks_tests_tools::gt::get_gt_chunk(2);
    gt_chunk_2.truncate(50);
    let block_546 = gt_chunk_2.get(46).cloned().expect("gt_chunk_2 is empty !");
    receive_valid_blocks(&bc_sender, &router_receiver, gt_chunk_2);

    // Generate 6 forks blocks from 547 to 553
    let signator = SignatorEnum::Ed25519(
        dup_crypto::keys::ed25519::Ed25519KeyPair::generate_random()
            .generate_signator()
            .expect("fail to generatye signator"),
    );
    let mut fork_blocks = Vec::new();
    let mut previous_hash = block_546.hash().expect("block_546 have None hash").0;
    let mut bc_time = block_546.common_time();
    for n in 547..=553 {
        bc_time += 301;
        let block = dubp_blocks_tests_tools::mocks::gen_empty_timed_issued_hashed_block_v10(
            BlockNumber(n),
            bc_time,
            signator.public_key(),
            previous_hash,
            &signator,
        );
        let block_hash = block.hash().clone().expect("block must have hash");
        fork_blocks.push(BlockDocument::V10(block));
        previous_hash = block_hash.0;
    }

    // Cause the revert of 3 blocks (send forks blocks from 547)
    receive_valid_blocks(&bc_sender, &router_receiver, fork_blocks);

    // TODO verify that we have switched to the new branch

    // let msg2 = router_receiver
    //     .recv()
    //     .expect("blockchain module disconnected.");
    // log::info!("Router receive: {:?}", msg2);

    // Stop and clean
    common::stop_and_clean(bc_sender, handle, tmp_profile_path);
}

fn receive_valid_blocks(
    bc_sender: &Sender<DursMsg>,
    router_receiver: &Receiver<RouterThreadMessage<DursMsg>>,
    blocks: Vec<BlockDocument>,
) {
    bc_sender
        .send(DursMsg::Event {
            event_from: ModuleStaticName("toto"),
            event_type: ModuleEvent::NewBlockFromNetwork,
            event_content: DursEvent::NetworkEvent(NetworkEvent::ReceiveBlocks(blocks.clone())),
        })
        .expect("Fail to send blocks to blockchain module.");
    for block in blocks {
        let msg = router_receiver
            .recv()
            .expect("blockchain module disconnected.");
        if let RouterThreadMessage::ModuleMessage(durs_msg) = msg {
            assert_eq!(
                DursMsg::Event {
                    event_from: ModuleStaticName("blockchain"),
                    event_type: ModuleEvent::NewValidBlock,
                    event_content: DursEvent::BlockchainEvent(Box::new(
                        BlockchainEvent::StackUpValidBlock(Box::new(block))
                    )),
                },
                durs_msg
            );
        //log::debug!("Router receive: {:?}", msg);
        } else {
            panic!("Expect ModuleMesage, found: {:?}", msg)
        }
    }
}
