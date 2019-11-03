//  Copyright (C) 2019  Éloïs SANCHEZ
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

//! G1-test blocks

use dubp_block_doc::BlockDocument;

/// Give g1-test chunks (packages of 250 blocks)
pub fn get_gt_chunk(chunk_number: usize) -> Vec<BlockDocument> {
    let mut gt_json_chunks_path = std::env::current_exe().expect("Fail to get current exe path.");
    gt_json_chunks_path.pop();
    gt_json_chunks_path.pop();
    gt_json_chunks_path.pop();
    gt_json_chunks_path.pop();
    gt_json_chunks_path.push("lib");
    gt_json_chunks_path.push("tests-tools");
    gt_json_chunks_path.push("blocks-tests-tools");
    gt_json_chunks_path.push("rsc");
    crate::json_chunk_parser::open_and_parse_one_json_chunk(&gt_json_chunks_path, chunk_number).1
}
