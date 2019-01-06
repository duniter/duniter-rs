//  Copyright (C) 2018  The Durs Project Developers.
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

//! Defined all network documents

use dubp_documents::documents::block::BlockDocument;
use dubp_documents::documents::certification::CertificationDocument;
use dubp_documents::documents::identity::IdentityDocument;
use dubp_documents::documents::membership::MembershipDocument;
use dubp_documents::documents::revocation::RevocationDocument;
use dubp_documents::documents::transaction::TransactionDocument;
use dubp_documents::Document;
use dubp_documents::{BlockHash, BlockId, Blockstamp};
use serde_json;
use std::ops::Deref;

#[derive(Debug, Clone)]
/// Block v10 in network format (Some events require a blockchain access to reconstitute the corresponding document)
pub struct NetworkBlockV10 {
    /// Uncompleted block document
    pub uncompleted_block_doc: BlockDocument,
    /// revoked
    pub revoked: Vec<serde_json::Value>,
    /// certifications
    pub certifications: Vec<serde_json::Value>,
}

#[derive(Debug, Clone)]
/// Block in network format (Some events require a blockchain access to reconstitute the corresponding document)
pub enum NetworkBlock {
    /// Block V1
    V10(Box<NetworkBlockV10>),
    /// Block V11
    V11(),
}

impl NetworkBlock {
    /// Return uncompleted block document
    pub fn uncompleted_block_doc(&self) -> BlockDocument {
        match *self {
            NetworkBlock::V10(ref network_block_v10) => {
                network_block_v10.deref().uncompleted_block_doc.clone()
            }
            _ => panic!("Block version not supported !"),
        }
    }
    /// Return blockstamp
    pub fn blockstamp(&self) -> Blockstamp {
        match *self {
            NetworkBlock::V10(ref network_block_v10) => {
                network_block_v10.deref().uncompleted_block_doc.blockstamp()
            }
            _ => panic!("Block version not supported !"),
        }
    }
    /// Return previous blockstamp
    pub fn previous_blockstamp(&self) -> Blockstamp {
        match *self {
            NetworkBlock::V10(ref network_block_v10) => Blockstamp {
                id: BlockId(network_block_v10.deref().uncompleted_block_doc.number.0 - 1),
                hash: BlockHash(
                    network_block_v10
                        .deref()
                        .uncompleted_block_doc
                        .previous_hash,
                ),
            },
            _ => panic!("Block version not supported !"),
        }
    }
}

#[derive(Debug, Clone)]
/// Network Document
pub enum BlockchainDocument {
    /// Network Block
    Block(NetworkBlock),
    /// Identity Document
    Identity(Box<IdentityDocument>),
    /// Membership Document
    Membership(Box<MembershipDocument>),
    /// Certification Document
    Certification(Box<CertificationDocument>),
    /// Revocation Document
    Revocation(Box<RevocationDocument>),
    /// Transaction Document
    Transaction(Box<TransactionDocument>),
}
