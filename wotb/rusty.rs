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
use WotDistance;
use WotDistanceParameters;
use HasLinkResult;
use RemLinkResult;
use NewLinkResult;

use NodeId;

/// A node in the *WoT* graph.
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
        self.max_links
    }

    /// Set the maximum number of links per user.
    fn set_max_link(&mut self, max_links: usize) {
        self.max_links = max_links;
    }

    /// Add a new node.
    fn add_node(&mut self) -> NodeId {
        self.nodes.push(Node::new());
        NodeId(self.nodes.len() - 1)
    }

    /// Remove the last node.
    /// Returns `None` if the WoT was empty, otherwise new top node id.
    fn rem_node(&mut self) -> Option<NodeId> {
        self.nodes.pop();

        if !self.nodes.is_empty() {
            Some(NodeId(self.nodes.len() - 1))
        } else {
            None
        }
    }

    /// Get the size of the WoT.
    fn size(&self) -> usize {
        self.nodes.len()
    }

    /// Check if given node is enabled.
    /// Returns `None` if this node doesn't exist.
    fn is_enabled(&self, id: NodeId) -> Option<bool> {
        self.nodes.get(id.0).map(|n| n.enabled)
    }

    /// Set if given node is enabled.
    /// Returns `Null` if this node doesn't exist, `enabled` otherwise.
    fn set_enabled(&mut self, id: NodeId, enabled: bool) -> Option<bool> {
        self.nodes
            .get_mut(id.0)
            .map(|n| n.enabled = enabled)
            .map(|_| enabled)
    }

    /// Get enabled nodes array.
    fn get_enabled(&self) -> Vec<NodeId> {
        self.nodes
            .par_iter()
            .enumerate()
            .filter(|&(_, n)| n.enabled)
            .map(|(i, _)| NodeId(i))
            .collect()
    }

    /// Get disabled nodes array.
    fn get_disabled(&self) -> Vec<NodeId> {
        self.nodes
            .par_iter()
            .enumerate()
            .filter(|&(_, n)| !n.enabled)
            .map(|(i, _)| NodeId(i))
            .collect()
    }

    /// Try to add a link from the source to the target.
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

    /// Try to remove a link from the source to the target.
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

    /// Test if there is a link from the source to the target.
    fn has_link(&self, source: NodeId, target: NodeId) -> HasLinkResult {
        if source.0 >= self.size() {
            HasLinkResult::UnknownSource()
        } else if target.0 >= self.size() {
            HasLinkResult::UnknownTarget()
        } else {
            HasLinkResult::Link(self.nodes[target.0].links_source.contains(&source))
        }
    }

    /// Get the list of links source for this target.
    /// Returns `None` if this node doesn't exist.
    fn get_links_source(&self, target: NodeId) -> Option<Vec<NodeId>> {
        self.nodes
            .get(target.0)
            .map(|n| n.links_source.iter().cloned().collect())
    }

    /// Get the number of issued links by a node.
    /// Returns `None` if this node doesn't exist.
    fn issued_count(&self, id: NodeId) -> Option<usize> {
        self.nodes.get(id.0).map(|n| n.issued_count)
    }

    /// Get sentries array.
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

    /// Get non sentries array.
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

    /// Get paths from one node to the other.
    fn get_paths(&self, from: NodeId, to: NodeId, k_max: u32) -> Vec<Vec<NodeId>> {
        if from == to {
            vec![vec![to]]
        } else if k_max > 0 {
            self.nodes[to.0]
                .links_source
                .par_iter()
                .map(|&source| self.get_paths(from, source, k_max - 1))
                .map(|paths| {
                    paths
                        .iter()
                        .map(|path| {
                            let mut path = path.clone();
                            path.push(to);
                            path
                        })
                        .collect::<Vec<Vec<NodeId>>>()
                })
                .reduce(
                    || vec![],
                    |mut acc, mut paths| {
                        acc.append(&mut paths);
                        acc
                    },
                )
        } else {
            vec![]
        }
    }

    /// Compute distance between a node and the network.
    /// Returns `None` if this node doesn't exist.
    fn compute_distance(&self, params: WotDistanceParameters) -> Option<WotDistance> {
        let WotDistanceParameters {
            node,
            sentry_requirement,
            step_max,
            x_percent,
        } = params;

        if node.0 >= self.size() {
            return None;
        }

        let mut area = HashSet::new();
        area.insert(node);
        let mut border = HashSet::new();
        border.insert(node);

        for _ in 0..step_max {
            border = border
                .par_iter()
                .map(|&id| {
                    self.nodes[id.0]
                        .links_source
                        .iter()
                        .filter(|source| !area.contains(&source))
                        .cloned()
                        .collect::<HashSet<_>>()
                })
                .reduce(
                    || HashSet::new(),
                    |mut acc, sources| {
                        for source in sources {
                            acc.insert(source);
                        }
                        acc
                    },
                );
            area.extend(border.iter());
        }

        let sentries: Vec<_> = self.get_sentries(sentry_requirement as usize)
            .into_iter()
            .filter(|sentry| sentry.0 != node.0)
            .collect();

        let success = area.iter().filter(|n| sentries.contains(&n)).count() as u32;
        let sentries = sentries.len() as u32;

        let distance = WotDistance {
            sentries,
            reached: (area.len() - 1) as u32,
            success: success,
            outdistanced: f64::from(success) < x_percent * f64::from(sentries),
        };

        Some(distance)
    }

    /// Test if a node is outdistanced in the network.
    /// Returns `Node` if this node doesn't exist.
    fn is_outdistanced(&self, params: WotDistanceParameters) -> Option<bool> {
        self.compute_distance(params)
            .map(|result| result.outdistanced)
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
