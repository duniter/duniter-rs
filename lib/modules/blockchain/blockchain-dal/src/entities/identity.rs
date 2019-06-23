//  Copyright (C) 2018  The Duniter Project Developers.
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

use dubp_documents::documents::identity::IdentityDocumentV10;
use dubp_documents::{BlockNumber, Blockstamp};
use durs_wot::NodeId;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
/// Identity state
pub enum DALIdentityState {
    /// Member
    Member(Vec<usize>),
    /// Expire Member
    ExpireMember(Vec<usize>),
    /// Explicit Revoked
    ExplicitRevoked(Vec<usize>),
    /// Explicit Revoked after expire
    ExplicitExpireRevoked(Vec<usize>),
    /// Implicit revoked
    ImplicitRevoked(Vec<usize>),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
/// Identity in database
pub struct DALIdentity {
    /// Identity hash
    pub hash: String,
    /// Identity state
    pub state: DALIdentityState,
    /// Blockstamp the identity was written
    pub joined_on: Blockstamp,
    /// Blockstamp the identity was expired
    pub expired_on: Option<Blockstamp>,
    /// Blockstamp the identity was revoked
    pub revoked_on: Option<Blockstamp>,
    /// Identity document
    pub idty_doc: IdentityDocumentV10,
    /// Identity wot id
    pub wot_id: NodeId,
    /// Membership created block number
    pub ms_created_block_id: BlockNumber,
    /// Timestamp from which membership can be renewed
    pub ms_chainable_on: Vec<u64>,
    /// Timestamp from which the identity can write a new certification
    pub cert_chainable_on: Vec<u64>,
}
