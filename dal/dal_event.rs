extern crate duniter_documents;
extern crate serde;

use self::duniter_documents::blockchain::v10::documents::BlockDocument;
use self::duniter_documents::blockchain::BlockchainProtocol;

#[derive(Debug, Clone)]
pub enum DALEvent {
    StackUpValidBlock(Box<BlockDocument>),
    RevertBlocks(Vec<Box<BlockDocument>>),
    NewValidPendingDoc(BlockchainProtocol),
    RefusedPendingDoc(BlockchainProtocol),
}
