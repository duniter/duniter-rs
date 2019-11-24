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

//! Sub-module checking if a block complies with all the rules of the (DUBP DUniter Blockchain Protocol).

pub mod global;
pub mod hashs;
pub mod local;
pub mod pow;

#[derive(Debug)]
pub enum InvalidBlockError {
    Global(global::GlobalVerifyBlockError),
    Hashs(dubp_block_doc::block::VerifyBlockHashError),
    Local(local::LocalVerifyBlockError),
    Pow(pow::BlockPoWError),
}
