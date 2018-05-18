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

//! Provide a legacy implementation of `WebOfTrust` storage and calculations.
//! Its mostly translated directly from the original C++ code.

use std::collections::hash_set::Iter;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;

use bincode::{deserialize, serialize, Infinite};

use super::{HasLinkResult, NewLinkResult, RemLinkResult};
use NodeId;
use WebOfTrust;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Node {
    id: NodeId,
    /// Is the node enabled ?
    pub enabled: bool,
    certs: HashSet<NodeId>,
    issued_count: usize,
}

impl Node {
    /// Create a new node.
    pub fn new(id: usize) -> Node {
        Node {
            id: NodeId(id),
            enabled: true,
            certs: HashSet::new(),
            issued_count: 0,
        }
    }

    /// Getter of node id.
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// Add a certification from this node to the given node.
    ///
    /// Certification will fail if this node already used all its certs.
    pub fn link_to(&mut self, to: &mut Node, max_certs: usize) -> NewLinkResult {
        if self.issued_count >= max_certs {
            NewLinkResult::AllCertificationsUsed(to.certs.len())
        } else if to.certs.contains(&self.id()) {
            NewLinkResult::AlreadyCertified(to.certs.len())
        } else {
            to.certs.insert(self.id());
            self.issued_count += 1;
            NewLinkResult::Ok(to.certs.len())
        }
    }

    /// Remove a certification (if it exist) from this node to the given node.
    pub fn unlink_to(&mut self, to: &mut Node) -> RemLinkResult {
        if to.certs.contains(&self.id()) {
            to.certs.remove(&self.id());
            self.issued_count -= 1;
            RemLinkResult::Removed(to.certs.len())
        } else {
            RemLinkResult::UnknownCert(to.certs.len())
        }
    }

    /// Tells if this node has a link from the given node.
    pub fn has_link_from(&self, from: &Node) -> bool {
        self.certs.contains(&from.id())
    }

    /// Tells if this node has a link to the given node.
    pub fn has_link_to(&self, to: &Node) -> bool {
        to.has_link_from(self)
    }

    /// Give an iterator of node certs.
    pub fn links_iter(&self) -> Iter<NodeId> {
        self.certs.iter()
    }

    /// Getter of the issued count.
    pub fn issued_count(&self) -> usize {
        self.issued_count
    }
}

/// Store a Web of Trust.
///
/// Allow to create/remove nodes and links between them.
///
/// It provides methods to find sentries nodes, find all paths
/// between 2 nodes and to compute distances in the web.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyWebOfTrust {
    nodes: Vec<Node>,
    /// Maxiumum amout of certifications a node can provide.
    ///
    /// It can be changed afterward, and will be accounted for every future calculations.
    ///
    /// In practice it should not change after initialization.
    pub max_cert: usize,
}

impl LegacyWebOfTrust {
    /// Read `WoT` from file.
    pub fn legacy_from_file(path: &str) -> Option<LegacyWebOfTrust> {
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return None,
        };

        let mut content: Vec<u8> = vec![];
        if file.read_to_end(&mut content).is_err() {
            return None;
        }

        match deserialize::<LegacyWebOfTrust>(&content[..]) {
            Ok(wot) => Some(wot),
            Err(_) => None,
        }
    }

    /// Write `WoT` to file.
    pub fn legacy_to_file(&self, path: &str) -> bool {
        let encoded: Vec<u8> = serialize(self, Infinite).unwrap();

        match File::create(path) {
            Ok(mut buffer) => buffer.write_all(&encoded).is_ok(),
            Err(_) => false,
        }
    }
}

impl WebOfTrust for LegacyWebOfTrust {
    fn new(max_cert: usize) -> LegacyWebOfTrust {
        LegacyWebOfTrust {
            nodes: vec![],
            max_cert,
        }
    }

    fn get_max_link(&self) -> usize {
        self.max_cert
    }

    fn set_max_link(&mut self, max_link: usize) {
        self.max_cert = max_link;
    }

    fn add_node(&mut self) -> NodeId {
        let node_id = self.nodes.len();
        self.nodes.push(Node::new(node_id));

        NodeId(node_id)
    }

    fn rem_node(&mut self) -> Option<NodeId> {
        self.nodes.pop();

        if !self.nodes.is_empty() {
            Some(NodeId(self.nodes.iter().len() - 1))
        } else {
            None
        }
    }

    fn size(&self) -> usize {
        self.nodes.iter().count()
    }

    fn is_enabled(&self, node: NodeId) -> Option<bool> {
        if node.0 >= self.size() {
            None
        } else {
            Some(self.nodes[node.0].enabled)
        }
    }

    fn set_enabled(&mut self, node: NodeId, state: bool) -> Option<bool> {
        if node.0 >= self.size() {
            None
        } else {
            self.nodes[node.0].enabled = state;
            Some(state)
        }
    }

    fn get_enabled(&self) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|x| x.enabled)
            .map(|x| x.id())
            .collect()
    }

    fn get_disabled(&self) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|x| !x.enabled)
            .map(|x| x.id())
            .collect()
    }

    fn add_link(&mut self, from: NodeId, to: NodeId) -> NewLinkResult {
        if from.0 == to.0 {
            NewLinkResult::SelfLinkingForbidden()
        } else if from.0 >= self.size() {
            NewLinkResult::UnknownSource()
        } else if to.0 >= self.size() {
            NewLinkResult::UnknownTarget()
        } else if from.0 < to.0 {
            // split `nodes` in two part to allow borrowing 2 nodes at the same time
            let (start, end) = self.nodes.split_at_mut(to.0);
            start[from.0].link_to(&mut end[0], self.max_cert)
        } else {
            // split `nodes` in two part to allow borrowing 2 nodes at the same time
            let (start, end) = self.nodes.split_at_mut(from.0);
            end[0].link_to(&mut start[to.0], self.max_cert)
        }
    }

    fn rem_link(&mut self, from: NodeId, to: NodeId) -> RemLinkResult {
        if from.0 >= self.size() {
            RemLinkResult::UnknownSource()
        } else if to.0 >= self.size() {
            RemLinkResult::UnknownTarget()
        } else if from.0 < to.0 {
            // split `nodes` in two part to allow borrowing 2 nodes at the same time
            let (start, end) = self.nodes.split_at_mut(to.0);
            start[from.0].unlink_to(&mut end[0])
        } else {
            // split `nodes` in two part to allow borrowing 2 nodes at the same time
            let (start, end) = self.nodes.split_at_mut(from.0);
            end[0].unlink_to(&mut start[to.0])
        }
    }

    fn has_link(&self, from: NodeId, to: NodeId) -> HasLinkResult {
        if from.0 >= self.size() {
            HasLinkResult::UnknownSource()
        } else if to.0 >= self.size() {
            HasLinkResult::UnknownTarget()
        } else {
            HasLinkResult::Link(self.nodes[from.0].has_link_to(&self.nodes[to.0]))
        }
    }

    fn is_sentry(&self, node: NodeId, sentry_requirement: usize) -> Option<bool> {
        if node.0 >= self.size() {
            return None;
        }

        let node = &self.nodes[node.0];

        Some(
            node.enabled
                && node.issued_count() >= sentry_requirement
                && node.links_iter().count() >= sentry_requirement,
        )
    }

    fn get_sentries(&self, sentry_requirement: usize) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|x| {
                x.enabled
                    && x.issued_count() >= sentry_requirement
                    && x.links_iter().count() >= sentry_requirement
            })
            .map(|x| x.id())
            .collect()
    }

    fn get_non_sentries(&self, sentry_requirement: usize) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|x| {
                x.enabled
                    && (x.issued_count < sentry_requirement
                        || x.links_iter().count() < sentry_requirement)
            })
            .map(|x| x.id())
            .collect()
    }

    fn get_links_source(&self, target: NodeId) -> Option<Vec<NodeId>> {
        if target.0 >= self.size() {
            None
        } else {
            Some(self.nodes[target.0].certs.iter().cloned().collect())
        }
    }

    fn issued_count(&self, id: NodeId) -> Option<usize> {
        if id.0 >= self.size() {
            None
        } else {
            Some(self.nodes[id.0].issued_count)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tests::generic_wot_test;

    #[test]
    fn node_tests() {
        // Correct node id
        assert_eq!(Node::new(1).id().0, 1);

        // Create 2 nodes
        let mut node1 = Node::new(1);
        let mut node2 = Node::new(2);

        // Default value
        assert_eq!(node1.issued_count(), 0);
        assert_eq!(node2.links_iter().count(), 0);
        assert!(!node1.has_link_to(&node2));
        assert!(!node2.has_link_to(&node2));
        assert!(!node1.has_link_from(&node1));
        assert!(!node2.has_link_from(&node1));

        // New link 1 -> 2
        match node1.link_to(&mut node2, 10) {
            NewLinkResult::Ok(1) => (),
            _ => panic!(),
        };

        assert_eq!(node1.issued_count(), 1);
        assert_eq!(node2.links_iter().count(), 1);
        assert!(node1.has_link_to(&node2));
        assert!(!node2.has_link_to(&node2));
        assert!(!node1.has_link_from(&node1));
        assert!(node2.has_link_from(&node1));

        // Existing link 1 -> 2
        match node1.link_to(&mut node2, 10) {
            NewLinkResult::AlreadyCertified(1) => (),
            _ => panic!(),
        };

        assert_eq!(node1.issued_count(), 1);
        assert_eq!(node2.links_iter().count(), 1);
        assert!(node1.has_link_to(&node2));
        assert!(!node2.has_link_to(&node2));
        assert!(!node1.has_link_from(&node1));
        assert!(node2.has_link_from(&node1));

        // Max certification count
        let mut node3 = Node::new(3);
        match node1.link_to(&mut node3, 1) {
            NewLinkResult::AllCertificationsUsed(0) => (),
            _ => panic!(),
        };

        assert_eq!(node1.issued_count(), 1);
        assert_eq!(node2.links_iter().count(), 1);
        assert_eq!(node3.links_iter().count(), 0);
        assert!(node1.has_link_to(&node2));
        assert!(!node2.has_link_to(&node2));
        assert!(!node1.has_link_from(&node1));
        assert!(node2.has_link_from(&node1));
    }

    #[test]
    fn wot_tests() {
        generic_wot_test::<LegacyWebOfTrust>();
    }
}
