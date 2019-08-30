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

/// Module name
pub static MODULE_NAME: &str = "blockchain";

/// Chunk size (in blocks)
pub static CHUNK_SIZE: &usize = &250;

/// Chunk file name begin
pub static CHUNK_FILE_NAME_BEGIN: &str = "chunk_";

/// Chunk file name end
pub static CHUNK_FILE_NAME_END: &str = "-250.json";

/// Low requency of request of main blocks
pub static REQUEST_MAIN_BLOCKS_LOW_FREQUENCY_IN_SEC: &u64 = &240;

/// High frequency of request of the main blocks
pub static REQUEST_MAIN_BLOCKS_HIGH_FREQUENCY_IN_SEC: &u64 = &30;

/// Frequency of request fork blocks (=request all blocks on fork window)
pub static REQUEST_FORK_BLOCKS_FREQUENCY_IN_SEC: &u64 = &180;

/// Blocks Delay threshold
pub static BLOCKS_DELAY_THRESHOLD: &u32 = &5;
