use duniter_documents::blockchain::v10::documents::BlockDocument;
use duniter_documents::blockchain::BlockchainProtocol;
use duniter_documents::Blockstamp;

#[derive(Debug, Clone)]
pub enum DALEvent {
    StackUpValidBlock(Box<BlockDocument>, Blockstamp),
    RevertBlocks(Vec<Box<BlockDocument>>),
    NewValidPendingDoc(BlockchainProtocol),
    RefusedPendingDoc(BlockchainProtocol),
}
