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
    missing_debug_implementations,
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

/// Tools
pub mod tools;

/// Contains all write databases functions
pub mod writers;

use dubp_common_doc::{BlockNumber, Blockstamp, PreviousBlockstamp};
use dubp_user_docs::documents::transaction::*;
use dup_crypto::hashs::Hash;
use dup_crypto::keys::*;
use durs_common_tools::fatal_error;
use durs_wot::data::{rusty::RustyWebOfTrust, NodeId};
use fnv::FnvHashMap;
use rustbreak::backend::{FileBackend, MemoryBackend};
use rustbreak::error::{RustbreakError, RustbreakErrorKind};
use rustbreak::{deser::Bincode, Database, FileDatabase, MemoryDatabase};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::default::Default;
use std::fmt::Debug;
use std::fs;
use std::panic::UnwindSafe;
use std::path::PathBuf;

use crate::entities::block::DALBlock;
use crate::entities::identity::DALIdentity;
use crate::entities::sources::{SourceAmount, UTXOContentV10, UTXOIndexV10};
use crate::writers::transaction::DALTxV10;

/// All blocks of local blockchain indexed by block number
pub type LocalBlockchainV10Datas = FnvHashMap<BlockNumber, DALBlock>;
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
pub type MsExpirV10Datas = FnvHashMap<BlockNumber, HashSet<NodeId>>;
/// Certifications sorted by created block
pub type CertsExpirV10Datas = FnvHashMap<BlockNumber, HashSet<(NodeId, NodeId)>>;
/// V10 Transactions indexed by their hashs
pub type TxV10Datas = HashMap<Hash, DALTxV10>;
/// V10 Unused Transaction Output (=sources)
pub type UTXOsV10Datas = HashMap<UTXOIndexV10, UTXOContentV10>;
/// V10 UDs sources
pub type UDsV10Datas = HashMap<PubKey, HashSet<BlockNumber>>;
/// V10 Balances accounts
pub type BalancesV10Datas = HashMap<UTXOConditionsGroup, (SourceAmount, HashSet<UTXOIndexV10>)>;

#[derive(Debug)]
/// Database
pub enum BinDB<D: Serialize + DeserializeOwned + Debug + Default + Clone + Send> {
    /// File database
    File(Database<D, FileBackend, Bincode>),
    /// Memory database
    Mem(Database<D, MemoryBackend, Bincode>),
}

impl<D: Serialize + DeserializeOwned + Debug + Default + Clone + Send> BinDB<D> {
    /// Flush the data structure to the backend
    pub fn save(&self) -> Result<(), RustbreakError> {
        match *self {
            BinDB::File(ref file_db) => file_db.save(),
            BinDB::Mem(ref mem_db) => mem_db.save(),
        }
    }
    /// Read lock the database and get write access to the Data container
    /// This gives you a read-only lock on the database. You can have as many readers in parallel as you wish.
    pub fn read<T, R>(&self, task: T) -> Result<R, RustbreakError>
    where
        T: FnOnce(&D) -> R,
    {
        match *self {
            BinDB::File(ref file_db) => file_db.read(task),
            BinDB::Mem(ref mem_db) => mem_db.read(task),
        }
    }
    /// Write lock the database and get write access to the Data container
    /// This gives you an exclusive lock on the memory object. Trying to open the database in writing will block if it is currently being written to.
    pub fn write<T>(&self, task: T) -> Result<(), RustbreakError>
    where
        T: FnOnce(&mut D),
    {
        match *self {
            BinDB::File(ref file_db) => file_db.write(task),
            BinDB::Mem(ref mem_db) => mem_db.write(task),
        }
    }
    /// Write lock the database and get write access to the Data container in a safe way (clone of the internal data is made).
    pub fn write_safe<T>(&self, task: T) -> Result<(), RustbreakError>
    where
        T: FnOnce(&mut D) + UnwindSafe,
    {
        match *self {
            BinDB::File(ref file_db) => file_db.write_safe(task),
            BinDB::Mem(ref mem_db) => mem_db.write_safe(task),
        }
    }
    /// Load the Data from the backend
    pub fn load(&self) -> Result<(), RustbreakError> {
        match *self {
            BinDB::File(ref file_db) => file_db.load(),
            BinDB::Mem(ref mem_db) => mem_db.load(),
        }
    }
}

#[derive(Debug)]
/// Set of databases storing block information
pub struct BlocksV10DBs {
    /// Local blockchain database
    pub blockchain_db: BinDB<LocalBlockchainV10Datas>,
}

impl BlocksV10DBs {
    /// Open blocks databases from their respective files
    pub fn open(db_path: Option<&PathBuf>) -> BlocksV10DBs {
        BlocksV10DBs {
            blockchain_db: open_db::<LocalBlockchainV10Datas>(db_path, "blockchain.db")
                .expect("Fail to open LocalBlockchainV10DB"),
        }
    }
    /// Save blocks databases in their respective files
    pub fn save_dbs(&self) {
        info!("BLOCKCHAIN-DAL: Save LocalBlockchainV10DB.");
        self.blockchain_db
            .save()
            .expect("Fatal error : fail to save LocalBlockchainV10DB !");
    }
}

#[derive(Debug)]
/// Set of databases storing forks informations
pub struct ForksDBs {
    /// Fork tree (store only blockstamp)
    pub fork_tree_db: BinDB<ForksTreeV10Datas>,
    /// Blocks in fork tree
    pub fork_blocks_db: BinDB<ForksBlocksV10Datas>,
    /// Orphan blocks
    pub orphan_blocks_db: BinDB<OrphanBlocksV10Datas>,
}

impl ForksDBs {
    /// Open fork databases from their respective files
    pub fn open(db_path: Option<&PathBuf>) -> ForksDBs {
        ForksDBs {
            fork_tree_db: open_db::<ForksTreeV10Datas>(db_path, "fork_tree.db")
                .expect("Fail to open ForksTreeV10Datas"),
            fork_blocks_db: open_db::<ForksBlocksV10Datas>(db_path, "fork_blocks.db")
                .expect("Fail to open ForkForksBlocksV10DatassV10DB"),
            orphan_blocks_db: open_db::<OrphanBlocksV10Datas>(db_path, "orphan_blocks.db")
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
    pub wot_db: BinDB<WotDB>,
    /// Store idrntities
    pub identities_db: BinDB<IdentitiesV10Datas>,
    /// Store memberships created_block_id (Use only to detect expirations)
    pub ms_db: BinDB<MsExpirV10Datas>,
    /// Store certifications created_block_id (Use only to detect expirations)
    pub certs_db: BinDB<CertsExpirV10Datas>,
}

impl WotsV10DBs {
    /// Open wot databases from their respective files
    pub fn open(db_path: Option<&PathBuf>) -> WotsV10DBs {
        WotsV10DBs {
            wot_db: open_db::<RustyWebOfTrust>(db_path, "wot.db").expect("Fail to open WotDB"),
            identities_db: open_db::<IdentitiesV10Datas>(db_path, "identities.db")
                .expect("Fail to open IdentitiesV10DB"),
            ms_db: open_db::<MsExpirV10Datas>(db_path, "ms.db").expect("Fail to open MsExpirV10DB"),
            certs_db: open_db::<CertsExpirV10Datas>(db_path, "certs.db")
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
    pub du_db: BinDB<UDsV10Datas>,
    /// Store all Transactions
    pub tx_db: BinDB<TxV10Datas>,
    /// Store all UTXOs
    pub utxos_db: BinDB<UTXOsV10Datas>,
    /// Store balances of all address (and theirs UTXOs indexs)
    pub balances_db: BinDB<BalancesV10Datas>,
}

impl CurrencyV10DBs {
    /// Open currency databases from their respective files
    pub fn open(db_path: Option<&PathBuf>) -> CurrencyV10DBs {
        CurrencyV10DBs {
            du_db: open_db::<UDsV10Datas>(db_path, "du.db").expect("Fail to open UDsV10DB"),
            tx_db: open_db::<TxV10Datas>(db_path, "tx.db")
                .unwrap_or_else(|_| fatal_error!("Fail to open TxV10DB")),
            utxos_db: open_db::<UTXOsV10Datas>(db_path, "sources.db")
                .expect("Fail to open UTXOsV10DB"),
            balances_db: open_db::<BalancesV10Datas>(db_path, "balances.db")
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

#[derive(Debug, Deserialize, Copy, Clone, PartialEq, Eq, Hash, Serialize)]
/// Data Access Layer Error
pub enum DALError {
    /// Error in write operation
    WriteError,
    /// Error in read operation
    ReadError,
    /// A database is corrupted, you have to reset the data completely
    DBCorrupted,
    /// Error with the file system
    FileSystemError,
    /// Capturing a panic signal during a write operation
    WritePanic,
    /// Unknown error
    UnknowError,
}

impl From<RustbreakError> for DALError {
    fn from(rust_break_error: RustbreakError) -> DALError {
        match rust_break_error.kind() {
            RustbreakErrorKind::Serialization => DALError::WriteError,
            RustbreakErrorKind::Deserialization => DALError::ReadError,
            RustbreakErrorKind::Poison => DALError::DBCorrupted,
            RustbreakErrorKind::Backend => DALError::FileSystemError,
            RustbreakErrorKind::WritePanic => DALError::WritePanic,
            _ => DALError::UnknowError,
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

/// Open Rustbreak database
pub fn open_db<D: Serialize + DeserializeOwned + Debug + Default + Clone + Send>(
    dbs_folder_path: Option<&PathBuf>,
    db_file_name: &str,
) -> Result<BinDB<D>, DALError> {
    if let Some(dbs_folder_path) = dbs_folder_path {
        Ok(BinDB::File(open_file_db::<D>(
            dbs_folder_path,
            db_file_name,
        )?))
    } else {
        Ok(BinDB::Mem(open_memory_db::<D>()?))
    }
}

/// Open Rustbreak memory database
pub fn open_memory_db<D: Serialize + DeserializeOwned + Debug + Default + Clone + Send>(
) -> Result<MemoryDatabase<D, Bincode>, DALError> {
    let backend = MemoryBackend::new();
    let db = MemoryDatabase::<D, Bincode>::from_parts(D::default(), backend, Bincode);
    Ok(db)
}

/// Open Rustbreak file database
pub fn open_file_db<D: Serialize + DeserializeOwned + Debug + Default + Clone + Send>(
    dbs_folder_path: &PathBuf,
    db_file_name: &str,
) -> Result<FileDatabase<D, Bincode>, DALError> {
    let mut db_path = dbs_folder_path.clone();
    db_path.push(db_file_name);
    let file_path = db_path.as_path();
    if file_path.exists()
        && fs::metadata(file_path)
            .expect("fail to get file size")
            .len()
            > 0
    {
        let backend = FileBackend::open(db_path.as_path())?;
        let db = FileDatabase::<D, Bincode>::from_parts(D::default(), backend, Bincode);
        db.load()?;
        Ok(db)
    } else {
        Ok(FileDatabase::<D, Bincode>::from_path(
            db_path.as_path(),
            D::default(),
        )?)
    }
}
