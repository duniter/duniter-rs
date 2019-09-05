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

//! Define BlockChain database constants needed for read operations.

/// Default page size for requests responses
pub static DEFAULT_PAGE_SIZE: &usize = &50;

////////////////////////////////
// BLOCKCHAIN DATABASE STORES //
////////////////////////////////

/// Current meta datas (CurrentMetaDataKey, ?)
pub static CURRENT_METAS_DATAS: &str = "cmd";

/// Fork blocks referenced in tree or in orphan blockstamps (Blockstamp, DbBlock)
pub static FORK_BLOCKS: &str = "fb";

/// Blocks in main branch (BlockNumber, DbBlock)
pub static MAIN_BLOCKS: &str = "bc";

/// Blockstamp orphaned (no parent block) indexed by their previous blockstamp (PreviousBlockstamp, Vec<Blockstamp>)
pub static ORPHAN_BLOCKSTAMP: &str = "ob";

/// Identities (PubKey, DbIdentity)
pub static IDENTITIES: &str = "idty";
