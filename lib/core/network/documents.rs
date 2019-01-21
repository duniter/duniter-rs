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

#[derive(Debug, Clone)]
/// Network Document
pub enum BlockchainDocument {
    /// Network Block
    Block(Box<BlockDocument>),
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
