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

//! Datas Access Layer

#![allow(clippy::large_enum_variant)]
#![deny(
    missing_docs,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

/// Define crate constants
pub mod constants;

/// Contains all entities stored in databases
pub mod entities;

/// Define all filters applicable to entities
pub mod filters;

/// Contains all read databases functions
pub mod readers;

//pub mod storage;

/// Tools
pub mod tools;

/// Contains all write databases functions
pub mod writers;

pub use durs_dbs_tools::kv_db::{
    KvFileDbHandler, KvFileDbRead as DbReadable, KvFileDbRoHandler, KvFileDbSchema,
    KvFileDbStoreType, KvFileDbValue,
};
pub use durs_dbs_tools::{
    open_free_struct_db, open_free_struct_file_db, open_free_struct_memory_db,
};
pub use durs_dbs_tools::{BinFreeStructDb, DALError};

use crate::constants::LOCAL_BC;
use crate::entities::block::DALBlock;
use crate::entities::identity::DALIdentity;
use crate::entities::sources::{SourceAmount, UTXOContentV10};
use crate::writers::transaction::DALTxV10;
use dubp_common_doc::{BlockNumber, Blockstamp, PreviousBlockstamp};
use dubp_indexes::sindex::UniqueIdUTXOv10;
use dubp_user_docs::documents::transaction::*;
use dup_crypto::hashs::Hash;
use dup_crypto::keys::*;
use durs_common_tools::fatal_error;
use durs_wot::data::{rusty::RustyWebOfTrust, WotId};
use fnv::FnvHashMap;
use maplit::hashmap;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Database handler
pub type Db = KvFileDbHandler;

/// Read-only database handler
pub type DbReader = KvFileDbRoHandler;

/// Forks tree meta datas (block number and hash only)
pub type ForksTreeV10Datas = entities::fork_tree::ForkTree;
/// Forks blocks referenced in tree indexed by their blockstamp
pub type ForksBlocksV10Datas = HashMap<Blockstamp, DALBlock>;
/// Blocks orphaned (no parent block) indexed by their previous blockstamp
pub type OrphanBlocksV10Datas = HashMap<PreviousBlockstamp, Vec<DALBlock>>;
/// Database containing the wot graph (each node of the graph in an u32)
pub type WotDB = RustyWebOfTrust;
/// V10 Identities indexed by public key
pub type IdentitiesV10Datas = HashMap<PubKey, DALIdentity>;
/// Memberships sorted by created block
pub type MsExpirV10Datas = FnvHashMap<BlockNumber, HashSet<WotId>>;
/// Certifications sorted by created block
pub type CertsExpirV10Datas = FnvHashMap<BlockNumber, HashSet<(WotId, WotId)>>;
/// V10 Transactions indexed by their hashs
pub type TxV10Datas = HashMap<Hash, DALTxV10>;
/// V10 Unused Transaction Output (=sources)
pub type UTXOsV10Datas = HashMap<UniqueIdUTXOv10, UTXOContentV10>;
/// V10 UDs sources
pub type UDsV10Datas = HashMap<PubKey, HashSet<BlockNumber>>;
/// V10 Balances accounts
pub type BalancesV10Datas = HashMap<UTXOConditionsGroup, (SourceAmount, HashSet<UniqueIdUTXOv10>)>;

/// Open database
pub fn open_db(path: &Path) -> Result<Db, DALError> {
    Db::open_db(
        path,
        &KvFileDbSchema {
            stores: hashmap![
                LOCAL_BC.to_owned() => KvFileDbStoreType::SingleIntKey,
            ],
        },
    )
}

#[derive(Debug)]
/// Set of databases storing forks informations
pub struct ForksDBs {
    /// Fork tree (store only blockstamp)
    pub fork_tree_db: BinFreeStructDb<ForksTreeV10Datas>,
    /// Blocks in fork tree
    pub fork_blocks_db: BinFreeStructDb<ForksBlocksV10Datas>,
    /// Orphan blocks
    pub orphan_blocks_db: BinFreeStructDb<OrphanBlocksV10Datas>,
}

impl ForksDBs {
    /// Open fork databases from their respective files
    pub fn open(db_path: Option<&PathBuf>) -> ForksDBs {
        ForksDBs {
            fork_tree_db: open_free_struct_db::<ForksTreeV10Datas>(db_path, "fork_tree.db")
                .expect("Fail to open ForksTreeV10Datas"),
            fork_blocks_db: open_free_struct_db::<ForksBlocksV10Datas>(db_path, "fork_blocks.db")
                .expect("Fail to open ForkForksBlocksV10DatassV10DB"),
            orphan_blocks_db: open_free_struct_db::<OrphanBlocksV10Datas>(
                db_path,
                "orphan_blocks.db",
            )
            .expect("Fail to open OrphanBlocksV10Datas"),
        }
    }
    /// Save fork databases in their respective files
    pub fn save_dbs(&self) {
        info!("BLOCKCHAIN-DAL: Save ForksDBs.");
        self.fork_tree_db
            .save()
            .expect("Fatal error : fail to save ForksTreeV10Datas !");
        self.fork_blocks_db
            .save()
            .expect("Fatal error : fail to save ForkForksBlocksV10DatassV10DB !");
        self.orphan_blocks_db
            .save()
            .expect("Fatal error : fail to save OrphanBlocksV10Datas !");
    }
}

#[derive(Debug)]
/// Set of databases storing web of trust information
pub struct WotsV10DBs {
    /// Store wot graph
    pub wot_db: BinFreeStructDb<WotDB>,
    /// Store idrntities
    pub identities_db: BinFreeStructDb<IdentitiesV10Datas>,
    /// Store memberships created_block_id (Use only to detect expirations)
    pub ms_db: BinFreeStructDb<MsExpirV10Datas>,
    /// Store certifications created_block_id (Use only to detect expirations)
    pub certs_db: BinFreeStructDb<CertsExpirV10Datas>,
}

impl WotsV10DBs {
    /// Open wot databases from their respective files
    pub fn open(db_path: Option<&PathBuf>) -> WotsV10DBs {
        WotsV10DBs {
            wot_db: open_free_struct_db::<RustyWebOfTrust>(db_path, "wot.db")
                .expect("Fail to open WotDB"),
            identities_db: open_free_struct_db::<IdentitiesV10Datas>(db_path, "identities.db")
                .expect("Fail to open IdentitiesV10DB"),
            ms_db: open_free_struct_db::<MsExpirV10Datas>(db_path, "ms.db")
                .expect("Fail to open MsExpirV10DB"),
            certs_db: open_free_struct_db::<CertsExpirV10Datas>(db_path, "certs.db")
                .expect("Fail to open CertsExpirV10DB"),
        }
    }
    /// Save wot databases from their respective files
    pub fn save_dbs(&self) {
        info!("BLOCKCHAIN-DAL: Save WotsV10DBs.");
        self.wot_db
            .save()
            .expect("Fatal error : fail to save WotDB !");
        self.save_dbs_except_graph();
    }
    /// Save wot databases from their respective files (except wot graph)
    pub fn save_dbs_except_graph(&self) {
        self.identities_db
            .save()
            .expect("Fatal error : fail to save IdentitiesV10DB !");
        self.ms_db
            .save()
            .expect("Fatal error : fail to save MsExpirV10DB !");
        self.certs_db
            .save()
            .expect("Fatal error : fail to save CertsExpirV10DB !");
    }
}

#[derive(Debug)]
/// Set of databases storing currency information
pub struct CurrencyV10DBs {
    /// Store all UD sources
    pub du_db: BinFreeStructDb<UDsV10Datas>,
    /// Store all Transactions
    pub tx_db: BinFreeStructDb<TxV10Datas>,
    /// Store all UTXOs
    pub utxos_db: BinFreeStructDb<UTXOsV10Datas>,
    /// Store balances of all address (and theirs UTXOs indexs)
    pub balances_db: BinFreeStructDb<BalancesV10Datas>,
}

impl CurrencyV10DBs {
    /// Open currency databases from their respective files
    pub fn open(db_path: Option<&PathBuf>) -> CurrencyV10DBs {
        CurrencyV10DBs {
            du_db: open_free_struct_db::<UDsV10Datas>(db_path, "du.db")
                .expect("Fail to open UDsV10DB"),
            tx_db: open_free_struct_db::<TxV10Datas>(db_path, "tx.db")
                .unwrap_or_else(|_| fatal_error!("Fail to open TxV10DB")),
            utxos_db: open_free_struct_db::<UTXOsV10Datas>(db_path, "sources.db")
                .expect("Fail to open UTXOsV10DB"),
            balances_db: open_free_struct_db::<BalancesV10Datas>(db_path, "balances.db")
                .expect("Fail to open BalancesV10DB"),
        }
    }
    /// Save currency databases in their respective files
    pub fn save_dbs(&self, tx: bool, du: bool) {
        if tx {
            info!("BLOCKCHAIN-DAL: Save CurrencyV10DBs.");
            self.tx_db
                .save()
                .expect("Fatal error : fail to save LocalBlockchainV10DB !");
            self.utxos_db
                .save()
                .expect("Fatal error : fail to save UTXOsV10DB !");
            self.balances_db
                .save()
                .expect("Fatal error : fail to save BalancesV10DB !");
        }
        if du {
            self.du_db
                .save()
                .expect("Fatal error : fail to save UDsV10DB !");
        }
    }
}

/*#[derive(Debug, Clone)]
pub struct WotStats {
    pub block_number: u32,
    pub block_hash: String,
    pub sentries_count: usize,
    pub average_density: usize,
    pub average_distance: usize,
    pub distances: Vec<usize>,
    pub average_connectivity: usize,
    pub connectivities: Vec<usize>,
    pub average_centrality: usize,
    pub centralities: Vec<u64>,
}*/

#[cfg(test)]
pub mod tests {

    use super::*;
    use tempfile::tempdir;

    #[inline]
    /// Open database in an arbitrary temporary directory given by OS
    /// and automatically cleaned when `Db` is dropped
    pub fn open_tmp_db() -> Result<Db, DALError> {
        open_db(tempdir().map_err(DALError::FileSystemError)?.path())
    }
}
