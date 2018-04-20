//  Copyright (C) 2017-2018  The Duniter Project Developers.
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

//! Provide data structures to manage web of trusts.
//! `LegacyWebOfTrust` is almost a translation of the legacy C++ coden while
//! `RustyWebOfTrust` is a brand new implementation with a more "rusty" style.

pub mod legacy;
pub mod rusty;

/// Wrapper for a node id.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub usize);

/// Results of a certification, with the current certification count
/// of the destination as parameter.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NewLinkResult {
    /// Certification worked.
    Ok(usize),
    /// This certification already exist.
    AlreadyCertified(usize),
    /// All available certifications has been used.
    AllCertificationsUsed(usize),
    /// Unknown source.
    UnknownSource(),
    /// Unknown target.
    UnknownTarget(),
    /// Self linking is forbidden.
    SelfLinkingForbidden(),
}

/// Results of a certification removal, with the current certification count
/// of the destination as parameter.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RemLinkResult {
    /// Certification has been removed.
    Removed(usize),
    /// Requested certification doesn't exist.
    UnknownCert(usize),
    /// Unknown source.
    UnknownSource(),
    /// Unknown target.
    UnknownTarget(),
}

/// Results of a certification test.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HasLinkResult {
    /// Both nodes are known, here is the result.
    Link(bool),
    /// Unknown source.
    UnknownSource(),
    /// Unknown target.
    UnknownTarget(),
}

/// Trait for a Web Of Trust.
/// Allow to provide other implementations of the `WoT` logic instead of the legacy C++
/// translated one.
pub trait WebOfTrust {
    /// Create a new Web of Trust with the maximum of links a node can issue.
    fn new(max_links: usize) -> Self;

    /// Get the maximum number of links per user.
    fn get_max_link(&self) -> usize;

    /// Set the maximum number of links per user.
    fn set_max_link(&mut self, max_link: usize);

    /// Add a new node.
    fn add_node(&mut self) -> NodeId;

    /// Remove the last node.
    /// Returns `None` if the WoT was empty, otherwise new top node id.
    fn rem_node(&mut self) -> Option<NodeId>;

    /// Get the size of the WoT.
    fn size(&self) -> usize;

    /// Check if given node is enabled.
    /// Returns `None` if this node doesn't exist.
    fn is_enabled(&self, id: NodeId) -> Option<bool>;

    /// Set the enabled state of given node.
    /// Returns `Null` if this node doesn't exist, `enabled` otherwise.
    fn set_enabled(&mut self, id: NodeId, enabled: bool) -> Option<bool>;

    /// Get enabled node array.
    fn get_enabled(&self) -> Vec<NodeId>;

    /// Get disabled node array.
    fn get_disabled(&self) -> Vec<NodeId>;

    /// Try to add a link from the source to the target.
    fn add_link(&mut self, source: NodeId, target: NodeId) -> NewLinkResult;

    /// Try to remove a link from the source to the target.
    fn rem_link(&mut self, source: NodeId, target: NodeId) -> RemLinkResult;

    /// Test if there is a link from the source to the target.
    fn has_link(&self, source: NodeId, target: NodeId) -> HasLinkResult;

    /// Get the list of links source for this target.
    /// Returns `None` if this node doesn't exist.
    fn get_links_source(&self, target: NodeId) -> Option<Vec<NodeId>>;

    /// Get the number of issued links by a node.
    /// Returns `None` if this node doesn't exist.
    fn issued_count(&self, id: NodeId) -> Option<usize>;

    /// Test if a node is a sentry.
    fn is_sentry(&self, node: NodeId, sentry_requirement: usize) -> Option<bool>;

    /// Get sentries array.
    fn get_sentries(&self, sentry_requirement: usize) -> Vec<NodeId>;

    /// Get non sentries array.
    fn get_non_sentries(&self, sentry_requirement: usize) -> Vec<NodeId>;
}
