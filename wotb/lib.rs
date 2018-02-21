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

//! `wotb` is a crate making "Web of Trust" computations for
//! the [Duniter] project.
//!
//! It defines a structure storing a Web of Trust.
//!
//! Web of Trust tests are translated from [duniter/wotb Javascript test][js-tests].
//!
//! [Duniter]: https://duniter.org/
//! [js-tests]: https://github.com/duniter/wotb/blob/master/wotcpp/webOfTrust.cpp

#![deny(missing_docs, missing_debug_implementations, missing_copy_implementations, trivial_casts,
        trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces,
        unused_qualifications)]

extern crate bincode;
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod legacy;
pub use legacy::LegacyWebOfTrust;

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

/// Paramters for WoT distance calculations
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct WotDistanceParameters {
    /// Node from where distances are calculated.
    pub node: NodeId,
    /// Links count received AND issued to be a sentry.
    pub sentry_requirement: u32,
    /// Currency parameter.
    pub step_max: u32,
    /// Currency parameter.
    pub x_percent: f64,
}

/// Results of `WebOfTrust::compute_distance`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct WotDistance {
    /// Sentries count
    pub sentries: u32,
    /// Success count
    pub success: u32,
    /// Reached count
    pub reached: u32,
    /// Is the node outdistanced ?
    pub outdistanced: bool,
}

/// Trait for a WebOfTrust.
/// Allow to provide other implementations of the WebOfTrust logic instead of the legacy C++
/// translated one.
pub trait WebOfTrust {  
    /// Get the maximum number of links per user.
    fn get_max_link(&self) -> usize;

    /// Set the maximum number of links per user.
    fn set_max_link(&mut self, max_link: usize);

    /// Add a new node.
    fn add_node(&mut self) -> NodeId;

    /// Remove the last node.
    /// Returns `None` if the WoT was empty.
    fn rem_node(&mut self) -> Option<NodeId>;

    /// Get the size of the WoT.
    fn size(&self) -> usize;

    /// Check if given node is enabled.
    /// Returns `None` if this node doesn't exist. 
    fn is_enabled(&self, id: NodeId) -> Option<bool>;    

    /// Set if given node is enabled.
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
    fn issued_count(&mut self, id: NodeId) -> Option<usize>;

     /// Get sentries array.
    fn get_sentries(&self, sentry_requirement: usize) -> Vec<NodeId>;

    /// Get non sentries array.
    fn get_non_sentries(&self, sentry_requirement: usize) -> Vec<NodeId>;
      
    /// Get paths from one node to the other.
    fn get_paths(&self, from: NodeId, to: NodeId, k_max: u32) -> Vec<Vec<NodeId>>;

    /// Compute distance between a node and the network.
    /// Returns `None` if this node doesn't exist.
    fn compute_distance(&self, params: WotDistanceParameters) -> Option<WotDistance>;

    /// Test if a node is outdistanced in the network.
    /// Returns `Node` if this node doesn't exist.
    fn is_outdistanced(&self, params: WotDistanceParameters) -> Option<bool>;
}