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

use dubp_currency_params::genesis_block_params::v10::BlockV10Parameters;
use dubp_currency_params::{CurrencyName, CurrencyParameters};
use durs_bc::BlockchainModule;
use durs_bc_db_writer::WotsV10DBs;
use durs_message::requests::DursReqContent;
use durs_message::DursMsg;
use durs_module::{ModuleReqFullId, ModuleReqId, ModuleRole, RouterThreadMessage};
use durs_network::requests::OldNetworkRequest;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;
use tempfile::TempDir;

pub static TEST_CURRENCY: &str = "test_currency";

/// Init logger and user datas directory
pub fn init() -> PathBuf {
    durs_common_tests_tools::logger::init_logger_stdout(vec![]);
    TempDir::new().expect("Fail to create tmp dir.").into_path()
}

/// Stop and clear test
pub fn stop_and_clean(
    _bc_sender: Sender<DursMsg>,
    _handle: JoinHandle<()>,
    tmp_profile_path: PathBuf,
) {
    // Send STOP signal to blockchain module
    /*bc_sender
        .send(DursMsg::Stop)
        .expect("Fail to send stop signal to blockchain module.");
    handle
        .join()
        .expect("Blockchain module fail to stop correctly.");*/

    // Clear user datas
    std::fs::remove_dir_all(tmp_profile_path).expect("Fail to remove tmp dir.");
}

/// Initialize a BlockchainModule with empty blockchain
pub fn init_bc_module(
    fake_router_sender: Sender<RouterThreadMessage<DursMsg>>,
    genesis_block_parameters: BlockV10Parameters,
    tmp_path: &Path,
    cautious_mode: bool,
) -> BlockchainModule {
    let currency_name = CurrencyName(TEST_CURRENCY.to_owned());
    //let profile_path = tmp_profile_path.to_owned();

    //let dbs_path = durs_conf::get_blockchain_db_path(profile_path.clone());
    let db = durs_bc_db_writer::open_db(tmp_path).expect("Fail to open blockchain DB.");

    BlockchainModule::new(
        cautious_mode,
        fake_router_sender,
        tmp_path.to_owned(),
        Some(currency_name.clone()),
        Some(CurrencyParameters::from((
            &currency_name,
            genesis_block_parameters,
        ))),
        db,
        WotsV10DBs::open(None),
    )
    .expect("Fail to init BlockchainModule with empty blockchain.")
}

pub fn recv_n_queries_get_blocks(
    n: usize,
    router_receiver: &Receiver<RouterThreadMessage<DursMsg>>,
) {
    for i in 0..n {
        let msg = router_receiver
            .recv()
            .expect("blockchain module disconnected.");
        if let RouterThreadMessage::ModuleMessage(durs_msg) = msg {
            assert_eq!(
                DursMsg::Request {
                    req_from: BlockchainModule::name(),
                    req_to: ModuleRole::InterNodesNetwork,
                    req_id: ModuleReqId(i as u32),
                    req_content: DursReqContent::OldNetworkRequest(OldNetworkRequest::GetBlocks(
                        ModuleReqFullId(BlockchainModule::name(), ModuleReqId(i as u32)),
                        50,
                        (i * 50) as u32
                    )),
                },
                durs_msg
            );
            log::info!("Router receive: {:?}", durs_msg);
        } else {
            panic!("Expect ModuleMesage, found: {:?}", msg)
        }
    }
}
