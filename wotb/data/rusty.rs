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

use std::collections::HashSet;
use rayon::prelude::*;
use WebOfTrust;
use super::{HasLinkResult, NewLinkResult, RemLinkResult};
use NodeId;

/// A node in the `WoT` graph.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Node {
    /// Is this node enabled ?
    enabled: bool,
    /// Set of links this node is the target.
    links_source: HashSet<NodeId>,
    /// Number of links the node issued.
    issued_count: usize,
}

impl Node {
    /// Create a new node.
    pub fn new() -> Node {
        Node {
            enabled: true,
            links_source: HashSet::new(),
            issued_count: 0,
        }
    }
}

/// A more idiomatic implementation of a Web of Trust.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustyWebOfTrust {
    /// List of nodes in the WoT.
    nodes: Vec<Node>,
    /// Maximum number of links a node can issue.
    max_links: usize,
}

impl WebOfTrust for RustyWebOfTrust {
    fn new(max_links: usize) -> RustyWebOfTrust {
        RustyWebOfTrust {
            nodes: vec![],
            max_links,
        }
    }

    fn get_max_link(&self) -> usize {
        self.max_links
    }

    fn set_max_link(&mut self, max_links: usize) {
        self.max_links = max_links;
    }

    fn add_node(&mut self) -> NodeId {
        self.nodes.push(Node::new());
        NodeId(self.nodes.len() - 1)
    }

    fn rem_node(&mut self) -> Option<NodeId> {
        self.nodes.pop();

        if !self.nodes.is_empty() {
            Some(NodeId(self.nodes.len() - 1))
        } else {
            None
        }
    }

    fn size(&self) -> usize {
        self.nodes.len()
    }

    fn is_enabled(&self, id: NodeId) -> Option<bool> {
        self.nodes.get(id.0).map(|n| n.enabled)
    }

    fn set_enabled(&mut self, id: NodeId, enabled: bool) -> Option<bool> {
        self.nodes
            .get_mut(id.0)
            .map(|n| n.enabled = enabled)
            .map(|_| enabled)
    }

    fn get_enabled(&self) -> Vec<NodeId> {
        self.nodes
            .par_iter()
            .enumerate()
            .filter(|&(_, n)| n.enabled)
            .map(|(i, _)| NodeId(i))
            .collect()
    }

    fn get_disabled(&self) -> Vec<NodeId> {
        self.nodes
            .par_iter()
            .enumerate()
            .filter(|&(_, n)| !n.enabled)
            .map(|(i, _)| NodeId(i))
            .collect()
    }

    fn add_link(&mut self, source: NodeId, target: NodeId) -> NewLinkResult {
        if source == target {
            NewLinkResult::SelfLinkingForbidden()
        } else if source.0 >= self.size() {
            NewLinkResult::UnknownSource()
        } else if target.0 >= self.size() {
            NewLinkResult::UnknownTarget()
        } else if self.nodes[source.0].issued_count >= self.max_links {
            NewLinkResult::AllCertificationsUsed(self.nodes[target.0].links_source.len())
        } else if self.nodes[target.0].links_source.contains(&source) {
            NewLinkResult::AlreadyCertified(self.nodes[target.0].links_source.len())
        } else {
            self.nodes[source.0].issued_count += 1;
            self.nodes[target.0].links_source.insert(source);
            NewLinkResult::Ok(self.nodes[target.0].links_source.len())
        }
    }

    fn rem_link(&mut self, source: NodeId, target: NodeId) -> RemLinkResult {
        if source.0 >= self.size() {
            RemLinkResult::UnknownSource()
        } else if target.0 >= self.size() {
            RemLinkResult::UnknownTarget()
        } else if !self.nodes[target.0].links_source.contains(&source) {
            RemLinkResult::UnknownCert(self.nodes[target.0].links_source.len())
        } else {
            self.nodes[source.0].issued_count -= 1;
            self.nodes[target.0].links_source.remove(&source);
            RemLinkResult::Removed(self.nodes[target.0].links_source.len())
        }
    }

    fn has_link(&self, source: NodeId, target: NodeId) -> HasLinkResult {
        if source.0 >= self.size() {
            HasLinkResult::UnknownSource()
        } else if target.0 >= self.size() {
            HasLinkResult::UnknownTarget()
        } else {
            HasLinkResult::Link(self.nodes[target.0].links_source.contains(&source))
        }
    }

    fn get_links_source(&self, target: NodeId) -> Option<Vec<NodeId>> {
        self.nodes
            .get(target.0)
            .map(|n| n.links_source.iter().cloned().collect())
    }

    fn issued_count(&self, id: NodeId) -> Option<usize> {
        self.nodes.get(id.0).map(|n| n.issued_count)
    }

    fn is_sentry(&self, node: NodeId, sentry_requirement: usize) -> Option<bool> {
        if node.0 >= self.size() {
            return None;
        }

        let node = &self.nodes[node.0];

        Some(
            node.enabled && node.issued_count >= sentry_requirement
                && node.links_source.len() >= sentry_requirement,
        )
    }

    fn get_sentries(&self, sentry_requirement: usize) -> Vec<NodeId> {
        self.nodes
            .par_iter()
            .enumerate()
            .filter(|&(_, n)| {
                n.enabled && n.issued_count >= sentry_requirement
                    && n.links_source.len() >= sentry_requirement
            })
            .map(|(i, _)| NodeId(i))
            .collect()
    }

    fn get_non_sentries(&self, sentry_requirement: usize) -> Vec<NodeId> {
        self.nodes
            .par_iter()
            .enumerate()
            .filter(|&(_, n)| {
                n.enabled
                    && (n.issued_count < sentry_requirement
                        || n.links_source.len() < sentry_requirement)
            })
            .map(|(i, _)| NodeId(i))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tests::generic_wot_test;
    use path::RustyPathFinder;
    use distance::RustyDistanceCalculator;

    #[test]
    fn wot_tests() {
        generic_wot_test::<RustyWebOfTrust, RustyPathFinder, RustyDistanceCalculator>();
    }
}
