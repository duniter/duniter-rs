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

//! Experimental implementation of the Web of Trust in a more "rusty" style.

use WebOfTrust;
use WotDistance;
use WotDistanceParameters;
use HasLinkResult;
use RemLinkResult;
use NewLinkResult;
use std::collections::HashSet;

use NodeId;

/// A node in the WoT graph.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Node {
    /// Is this node enabled ?
    enabled: bool,

    /// Set of links this node is the target.
    links_source: HashSet<NodeId>,

    /// Number of links the node issued.
    issued_count: usize,
}

/// A more idiomatic implementation of a Web of Trust.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustyWebOfTrust {
    /// List of nodes in the WoT.
    nodes: Vec<Node>,
    /// Maximum number of links a node can issue.
    max_links: usize,
}

impl RustyWebOfTrust {
    /// Create a new Web of Trust with the maximum of links a node can issue.
    pub fn new(max_links: usize) -> RustyWebOfTrust {
        RustyWebOfTrust {
            nodes: vec![],
            max_links,
        }
    }
}

impl WebOfTrust for RustyWebOfTrust {
    /// Get the maximum number of links per user.
    fn get_max_link(&self) -> usize {
        unimplemented!()
    }

    /// Set the maximum number of links per user.
    fn set_max_link(&mut self, max_link: usize) {
        unimplemented!()
    }

    /// Add a new node.
    fn add_node(&mut self) -> NodeId {
        unimplemented!()
    }

    /// Remove the last node.
    /// Returns `None` if the WoT was empty.
    fn rem_node(&mut self) -> Option<NodeId> {
        unimplemented!()
    }

    /// Get the size of the WoT.
    fn size(&self) -> usize {
        unimplemented!()
    }

    /// Check if given node is enabled.
    /// Returns `None` if this node doesn't exist.
    fn is_enabled(&self, id: NodeId) -> Option<bool> {
        unimplemented!()
    }

    /// Set if given node is enabled.
    /// Returns `Null` if this node doesn't exist, `enabled` otherwise.
    fn set_enabled(&mut self, id: NodeId, enabled: bool) -> Option<bool> {
        unimplemented!()
    }

    /// Get enabled node array.
    fn get_enabled(&self) -> Vec<NodeId> {
        unimplemented!()
    }

    /// Get disabled node array.
    fn get_disabled(&self) -> Vec<NodeId> {
        unimplemented!()
    }

    /// Try to add a link from the source to the target.
    fn add_link(&mut self, source: NodeId, target: NodeId) -> NewLinkResult {
        unimplemented!()
    }

    /// Try to remove a link from the source to the target.
    fn rem_link(&mut self, source: NodeId, target: NodeId) -> RemLinkResult {
        unimplemented!()
    }

    /// Test if there is a link from the source to the target.
    fn has_link(&self, source: NodeId, target: NodeId) -> HasLinkResult {
        unimplemented!()
    }

    /// Get the list of links source for this target.
    /// Returns `None` if this node doesn't exist.
    fn get_links_source(&self, target: NodeId) -> Option<Vec<NodeId>> {
        unimplemented!()
    }

    /// Get the number of issued links by a node.
    /// Returns `None` if this node doesn't exist.
    fn issued_count(&self, id: NodeId) -> Option<usize> {
        unimplemented!()
    }

    /// Get sentries array.
    fn get_sentries(&self, sentry_requirement: usize) -> Vec<NodeId> {
        unimplemented!()
    }

    /// Get non sentries array.
    fn get_non_sentries(&self, sentry_requirement: usize) -> Vec<NodeId> {
        unimplemented!()
    }

    /// Get paths from one node to the other.
    fn get_paths(&self, from: NodeId, to: NodeId, k_max: u32) -> Vec<Vec<NodeId>> {
        unimplemented!()
    }

    /// Compute distance between a node and the network.
    /// Returns `None` if this node doesn't exist.
    fn compute_distance(&self, params: WotDistanceParameters) -> Option<WotDistance> {
        unimplemented!()
    }

    /// Test if a node is outdistanced in the network.
    /// Returns `Node` if this node doesn't exist.
    fn is_outdistanced(&self, params: WotDistanceParameters) -> Option<bool> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tests::generic_wot_test;

    #[test]
    fn wot_tests() {
        let mut wot1 = RustyWebOfTrust::new(3);
        let mut wot2 = RustyWebOfTrust::new(3);
        generic_wot_test(&mut wot1, &mut wot2);
    }
}
