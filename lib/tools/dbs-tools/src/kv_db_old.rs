///////////////////////
/// OLD A SUPPRIMER ///
///////////////////////
mod file;

#[cfg(feature = "mock")]
pub use file::MockKvFileDbReader;
pub use file::{
    from_db_value, KvFileDbHandler, KvFileDbRead, KvFileDbReader, KvFileDbRoHandler,
    KvFileDbSchema, KvFileDbStoreType, KvFileDbWriter, WriteResp,
};
pub use rkv::{
    store::multi::Iter, IntegerStore, MultiIntegerStore, MultiStore,
    OwnedValue as KvFileDbOwnedValue, Readable, SingleStore, Value as KvFileDbValue,
};
