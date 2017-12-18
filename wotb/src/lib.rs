//! `wotb` is a crate making "Web of Trust" computations for
//! the [Duniter] project.
//!
//! It defines a structure storing a Web of Trust.
//!
//! Web of Trust tests are translated from [duniter/wotb Javascript test][js-tests].
//!
//! [js-tests]: https://github.com/duniter/wotb/blob/master/wotcpp/webOfTrust.cpp

#![deny(missing_docs,
        missing_debug_implementations, missing_copy_implementations,
        trivial_casts, trivial_numeric_casts,
        unsafe_code,
        unstable_features,
        unused_import_braces, unused_qualifications)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;

use std::collections::HashSet;
use std::collections::hash_set::Iter;
use std::rc::Rc;
use std::fs::File;
use std::io::prelude::*;

use bincode::{serialize, deserialize, Infinite};

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
pub enum RemovedLinkResult {
    /// Certification has been removed.
    Removed(usize),
    /// Requested certification doesn't exist.
    UnknownCert(usize),
    /// Unknown source.
    UnknownSource(),
    /// Unknown target.
    UnknownTarget(),
}

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
    pub fn unlink_to(&mut self, to: &mut Node) -> RemovedLinkResult {
        if to.certs.contains(&self.id()) {
            to.certs.remove(&self.id());
            self.issued_count -= 1;
            RemovedLinkResult::Removed(to.certs.len())
        } else {
            RemovedLinkResult::UnknownCert(to.certs.len())
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

#[derive(Debug)]
struct WotStep {
    pub previous: Option<Rc<Box<WotStep>>>,
    pub node: NodeId,
    pub distance: u32,
}

struct LookupStep {
    paths: Vec<Rc<Box<WotStep>>>,
    matching_paths: Vec<Rc<Box<WotStep>>>,
    distances: Vec<u32>,
}

/// Store a Web of Trust.
///
/// Allow to create/remove nodes and links between them.
///
/// It provides methods to find sentries nodes, find all paths
/// between 2 nodes and to compute distances in the web.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebOfTrust {
    nodes: Vec<Node>,
    /// Maxiumum amout of certifications a node can provide.
    ///
    /// It can be changed afterward, and will be accounted for every future calculations.
    ///
    /// In practice it should not change after initialization.
    pub max_cert: usize,
}

impl WebOfTrust {
    /// Create a new Web of Trust with the maxium certificications count.
    pub fn new(max_cert: usize) -> WebOfTrust {
        WebOfTrust {
            nodes: vec![],
            max_cert,
        }
    }

    /// Read WoT from file.
    pub fn from_file(path: &str) -> Option<WebOfTrust> {
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return None,
        };

        let mut content: Vec<u8> = vec![];
        if file.read_to_end(&mut content).is_err() {
            return None;
        }

        match deserialize::<WebOfTrust>(&content[..]) {
            Ok(wot) => Some(wot),
            Err(_) => None,
        }
    }

    /// Write WoT to file.
    pub fn to_file(&self, path: &str) -> bool {
        let encoded: Vec<u8> = serialize(self, Infinite).unwrap();

        match File::create(path) {
            Ok(mut buffer) => buffer.write_all(&encoded).is_ok(),
            Err(_) => false,
        }
    }

    /// Add a new node.
    pub fn add_node(&mut self) -> NodeId {
        let node_id = self.nodes.len();
        self.nodes.push(Node::new(node_id));

        NodeId(node_id)
    }

    /// Remove given node if it exits.
    pub fn remove_node(&mut self) -> Option<NodeId> {
        self.nodes.pop();

        if !self.nodes.is_empty() {
            Some(NodeId(self.nodes.iter().len() - 1))
        } else {
            None
        }
    }

    fn check_matches(&self, node: NodeId, d: u32, d_max: u32, mut checked: Vec<bool>) -> Vec<bool> {
        let mut linked_nodes = Vec::new();

        for linked_node in self.nodes[node.0].links_iter() {
            checked[linked_node.0] = true;
            linked_nodes.push(*linked_node);
        }

        if d < d_max {
            for linked_node in &linked_nodes {
                checked = self.check_matches(*linked_node, d + 1, d_max, checked);
            }
        }

        checked
    }

    /// Compute distance between a node and the network.
    pub fn compute_distance(
        &self,
        member: NodeId,
        d_min: u32,
        k_max: u32,
        x_percent: f64,
    ) -> WotDistance {
        let d_min = d_min as usize;

        let mut result = WotDistance {
            sentries: 0,
            success: 0,
            reached: 0,
            outdistanced: false,
        };

        let mut sentries: Vec<bool> = self.nodes
            .iter()
            .map(|x| {
                x.enabled && x.issued_count() >= d_min && x.links_iter().count() >= d_min
            })
            .collect();
        sentries[member.0] = false;

        let mut checked: Vec<bool> = self.nodes.iter().map(|_| false).collect();

        if k_max >= 1 {
            checked = self.check_matches(member, 1, k_max, checked);
        }

        for (&sentry, &check) in sentries.iter().zip(checked.iter()) {
            if sentry {
                result.sentries += 1;
                if check {
                    result.success += 1;
                    result.reached += 1;
                }
            } else if check {
                result.reached += 1;
            }
        }

        result.outdistanced = f64::from(result.success) < x_percent * f64::from(result.sentries);
        result
    }

    /// Get sentries array.
    pub fn get_sentries(&self, d_min: usize) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|x| {
                x.enabled && x.issued_count() >= d_min && x.links_iter().count() >= d_min
            })
            .map(|x| x.id())
            .collect()
    }

    /// Get non sentries array.
    pub fn get_non_sentries(&self, d_min: usize) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|x| {
                x.enabled && (x.issued_count < d_min || x.links_iter().count() < d_min)
            })
            .map(|x| x.id())
            .collect()
    }

    /// Get disabled array.
    pub fn get_disabled(&self) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter(|x| !x.enabled)
            .map(|x| x.id())
            .collect()
    }

    fn lookup(
        &self,
        source: NodeId,
        target: NodeId,
        distance: u32,
        distance_max: u32,
        previous: &Rc<Box<WotStep>>,
        mut lookup_step: LookupStep,
    ) -> LookupStep {
        if source != target && distance <= distance_max {
            let mut local_paths: Vec<Rc<Box<WotStep>>> = vec![];

            for &by in self.nodes[target.0].links_iter() {
                if distance < lookup_step.distances[by.0] {
                    lookup_step.distances[by.0] = distance;
                    let step = Rc::new(Box::new(WotStep {
                        previous: Some(Rc::clone(previous)),
                        node: by,
                        distance,
                    }));

                    lookup_step.paths.push(Rc::clone(&step));
                    local_paths.push(Rc::clone(&step));
                    if by == source {
                        lookup_step.matching_paths.push(Rc::clone(&step));
                    }
                }
            }

            if distance <= distance_max {
                for path in &local_paths {
                    lookup_step = self.lookup(
                        source,
                        path.node,
                        distance + 1,
                        distance_max,
                        &Rc::clone(path),
                        lookup_step,
                    );
                }
            }
        }

        lookup_step
    }

    /// Get paths from one node to the other.
    pub fn get_paths(&self, from: NodeId, to: NodeId, k_max: u32) -> Vec<Vec<NodeId>> {
        let mut lookup_step = LookupStep {
            paths: vec![],
            matching_paths: vec![],
            distances: self.nodes.iter().map(|_| k_max + 1).collect(),
        };

        lookup_step.distances[to.0] = 0;

        let root = Rc::new(Box::new(WotStep {
            previous: None,
            node: to,
            distance: 0,
        }));

        lookup_step.paths.push(Rc::clone(&root));

        lookup_step = self.lookup(from, to, 1, k_max, &root, lookup_step);

        let mut result: Vec<Vec<NodeId>> = Vec::with_capacity(lookup_step.matching_paths.len());

        for step in &lookup_step.matching_paths {
            let mut vecpath = vec![];
            let mut step = Rc::clone(step);

            loop {
                vecpath.push(step.node);
                if step.previous.is_none() {
                    break;
                }
                step = step.previous.clone().unwrap();
            }

            result.push(vecpath);
        }

        result
    }

    /// Number of nodes in the WoT.
    pub fn size(&self) -> usize {
        self.nodes.iter().count()
    }

    /// Tells if requested node is enabled (None if doesn't exist).
    pub fn is_enabled(&self, node: NodeId) -> Option<bool> {
        if node.0 >= self.size() {
            None
        } else {
            Some(self.nodes[node.0].enabled)
        }
    }

    /// Set if a node is enabled.
    pub fn set_enabled(&mut self, node: NodeId, state: bool) -> Option<bool> {
        if node.0 >= self.size() {
            None
        } else {
            self.nodes[node.0].enabled = state;
            Some(state)
        }
    }

    /// Add link from a node to another.
    pub fn add_link(&mut self, from: NodeId, to: NodeId) -> NewLinkResult {
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

    /// Remove a link from a node to another.
    pub fn remove_link(&mut self, from: NodeId, to: NodeId) -> RemovedLinkResult {
        if from.0 >= self.size() {
            RemovedLinkResult::UnknownSource()
        } else if to.0 >= self.size() {
            RemovedLinkResult::UnknownTarget()
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

    /// Test if a link exist from a node to another.
    pub fn exists_link(&self, from: NodeId, to: NodeId) -> bool {
        if from.0 >= self.size() || to.0 >= self.size() {
            false
        } else {
            self.nodes[from.0].has_link_to(&self.nodes[to.0])
        }
    }

    /// Test if a node is outdistanced in the network.
    pub fn is_outdistanced(
        &self,
        node: NodeId,
        d_min: u32,
        d_max: u32,
        x_percent: f64,
    ) -> Option<bool> {
        if node.0 >= self.size() {
            None
        } else {
            Some(
                self.compute_distance(node, d_min, d_max, x_percent)
                    .outdistanced,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    /// This test is a translation of https://github.com/duniter/wotb/blob/master/tests/test.js
    #[test]
    fn wot_tests() {
        let mut wot = WebOfTrust::new(3);

        // should have an initial size of 0
        assert_eq!(wot.size(), 0);

        // should return `None()` if testing `is_enabled()` with out-of-bounds node
        assert_eq!(wot.is_enabled(NodeId(0)), None);
        assert_eq!(wot.is_enabled(NodeId(23)), None);

        // should give nomber 0 if we add a node
        // - add a node
        assert_eq!(wot.add_node(), NodeId(0));
        assert_eq!(wot.size(), 1);
        assert_eq!(wot.get_disabled().len(), 0);

        // - add another
        assert_eq!(wot.add_node(), NodeId(1));
        assert_eq!(wot.size(), 2);
        assert_eq!(wot.get_disabled().len(), 0);

        // - add 10 nodes
        for i in 0..10 {
            assert_eq!(wot.add_node(), NodeId(i + 2));
        }

        assert_eq!(wot.size(), 12);

        // shouldn't be able to self cert
        assert_eq!(
            wot.add_link(NodeId(0), NodeId(0)),
            NewLinkResult::SelfLinkingForbidden()
        );

        // should add certs only in the boundaries of max_cert
        assert_eq!(wot.add_link(NodeId(0), NodeId(1)), NewLinkResult::Ok(1));
        assert_eq!(wot.add_link(NodeId(0), NodeId(2)), NewLinkResult::Ok(1));
        assert_eq!(wot.add_link(NodeId(0), NodeId(3)), NewLinkResult::Ok(1));
        assert_eq!(
            wot.add_link(NodeId(0), NodeId(4)),
            NewLinkResult::AllCertificationsUsed(0)
        );

        assert_eq!(wot.max_cert, 3);
        assert_eq!(wot.exists_link(NodeId(0), NodeId(1)), true);
        assert_eq!(wot.exists_link(NodeId(0), NodeId(2)), true);
        assert_eq!(wot.exists_link(NodeId(0), NodeId(3)), true);
        assert_eq!(wot.exists_link(NodeId(0), NodeId(4)), false);

        wot.max_cert = 4;
        assert_eq!(wot.max_cert, 4);
        assert_eq!(wot.exists_link(NodeId(0), NodeId(4)), false);
        wot.add_link(NodeId(0), NodeId(4));
        assert_eq!(wot.exists_link(NodeId(0), NodeId(4)), true);
        wot.remove_link(NodeId(0), NodeId(1));
        wot.remove_link(NodeId(0), NodeId(2));
        wot.remove_link(NodeId(0), NodeId(3));
        wot.remove_link(NodeId(0), NodeId(4));

        // false when not linked or out of bounds
        assert_eq!(wot.exists_link(NodeId(0), NodeId(6)), false);
        assert_eq!(wot.exists_link(NodeId(23), NodeId(0)), false);
        assert_eq!(wot.exists_link(NodeId(2), NodeId(53)), false);

        // created nodes should be enabled
        assert_eq!(wot.is_enabled(NodeId(0)), Some(true));
        assert_eq!(wot.is_enabled(NodeId(1)), Some(true));
        assert_eq!(wot.is_enabled(NodeId(2)), Some(true));
        assert_eq!(wot.is_enabled(NodeId(3)), Some(true));
        assert_eq!(wot.is_enabled(NodeId(11)), Some(true));

        // should be able to disable some nodes
        assert_eq!(wot.set_enabled(NodeId(0), false), Some(false));
        assert_eq!(wot.set_enabled(NodeId(1), false), Some(false));
        assert_eq!(wot.set_enabled(NodeId(2), false), Some(false));
        assert_eq!(wot.get_disabled().len(), 3);
        assert_eq!(wot.set_enabled(NodeId(1), true), Some(true));

        // node 0 and 2 should be disabled
        assert_eq!(wot.is_enabled(NodeId(0)), Some(false));
        assert_eq!(wot.is_enabled(NodeId(1)), Some(true));
        assert_eq!(wot.is_enabled(NodeId(2)), Some(false));
        assert_eq!(wot.is_enabled(NodeId(3)), Some(true));
        // - set enabled again
        assert_eq!(wot.set_enabled(NodeId(0), true), Some(true));
        assert_eq!(wot.set_enabled(NodeId(1), true), Some(true));
        assert_eq!(wot.set_enabled(NodeId(2), true), Some(true));
        assert_eq!(wot.set_enabled(NodeId(1), true), Some(true));
        assert_eq!(wot.get_disabled().len(), 0);

        // should not exist a link from 2 to 0
        assert_eq!(wot.exists_link(NodeId(2), NodeId(0)), false);

        // should be able to add some links, cert count is returned
        assert_eq!(wot.add_link(NodeId(2), NodeId(0)), NewLinkResult::Ok(1));
        assert_eq!(wot.add_link(NodeId(4), NodeId(0)), NewLinkResult::Ok(2));
        assert_eq!(
            wot.add_link(NodeId(4), NodeId(0)),
            NewLinkResult::AlreadyCertified(2)
        );
        assert_eq!(
            wot.add_link(NodeId(4), NodeId(0)),
            NewLinkResult::AlreadyCertified(2)
        );
        assert_eq!(wot.add_link(NodeId(5), NodeId(0)), NewLinkResult::Ok(3));

        // should exist new links
        /* WoT is:
         *
         * 2 --> 0
         * 4 --> 0
         * 5 --> 0
         */

        assert_eq!(wot.exists_link(NodeId(2), NodeId(0)), true);
        assert_eq!(wot.exists_link(NodeId(4), NodeId(0)), true);
        assert_eq!(wot.exists_link(NodeId(5), NodeId(0)), true);
        assert_eq!(wot.exists_link(NodeId(2), NodeId(1)), false);

        // should be able to remove some links
        assert_eq!(
            wot.remove_link(NodeId(4), NodeId(0)),
            RemovedLinkResult::Removed(2)
        );
        /*
         * WoT is now:
         *
         * 2 --> 0
         * 5 --> 0
         */

        // should exist less links
        assert_eq!(wot.exists_link(NodeId(2), NodeId(0)), true);
        assert_eq!(wot.exists_link(NodeId(4), NodeId(0)), false);
        assert_eq!(wot.exists_link(NodeId(5), NodeId(0)), true);
        assert_eq!(wot.exists_link(NodeId(2), NodeId(1)), false);

        // should successfully use distance rule
        assert_eq!(wot.is_outdistanced(NodeId(0), 1, 1, 1.0), Some(false));
        // => no because 2,4,5 have certified him
        assert_eq!(wot.is_outdistanced(NodeId(0), 2, 1, 1.0), Some(false));
        // => no because only member 2 has 2 certs, and has certified him
        assert_eq!(wot.is_outdistanced(NodeId(0), 3, 1, 1.0), Some(false));
        // => no because no member has issued 3 certifications

        // - we add links from member 3
        assert_eq!(wot.add_link(NodeId(3), NodeId(1)), NewLinkResult::Ok(1));
        assert_eq!(wot.add_link(NodeId(3), NodeId(2)), NewLinkResult::Ok(1));
        /*
         * WoT is now:
         *
         * 2 --> 0
         * 5 --> 0
         * 3 --> 1
         * 3 --> 2
         */
        assert_eq!(wot.size(), 12);
        assert_eq!(wot.get_sentries(1).len(), 1);
        assert_eq!(wot.get_sentries(1)[0], NodeId(2));
        assert_eq!(wot.get_sentries(2).len(), 0);
        assert_eq!(wot.get_sentries(3).len(), 0);
        assert_eq!(wot.get_non_sentries(1).len(), 11); // 12 - 1
        assert_eq!(wot.get_non_sentries(2).len(), 12); // 12 - 0
        assert_eq!(wot.get_non_sentries(3).len(), 12); // 12 - 0
        assert_eq!(wot.get_paths(NodeId(3), NodeId(0), 1).len(), 0); // KO
        assert_eq!(wot.get_paths(NodeId(3), NodeId(0), 2).len(), 1); // It exists 3 -> 2 -> 0
        assert_eq!(wot.get_paths(NodeId(3), NodeId(0), 2)[0].len(), 3); // It exists 3 -> 2 -> 0
        assert_eq!(wot.is_outdistanced(NodeId(0), 1, 1, 1.0), Some(false)); // OK : 2 -> 0
        assert_eq!(wot.is_outdistanced(NodeId(0), 2, 1, 1.0), Some(false)); // OK : 2 -> 0
        assert_eq!(wot.is_outdistanced(NodeId(0), 3, 1, 1.0), Some(false)); // OK : no stry \w 3 lnk
        assert_eq!(wot.is_outdistanced(NodeId(0), 2, 2, 1.0), Some(false)); // OK : 2 -> 0

        wot.add_link(NodeId(1), NodeId(3));
        wot.add_link(NodeId(2), NodeId(3));

        assert_eq!(wot.size(), 12);
        assert_eq!(wot.get_sentries(1).len(), 3);
        assert_eq!(wot.get_sentries(1)[0], NodeId(1));
        assert_eq!(wot.get_sentries(1)[1], NodeId(2));
        assert_eq!(wot.get_sentries(1)[2], NodeId(3));

        assert_eq!(wot.get_sentries(2).len(), 1);
        assert_eq!(wot.get_sentries(2)[0], NodeId(3));
        assert_eq!(wot.get_sentries(3).len(), 0);
        assert_eq!(wot.get_non_sentries(1).len(), 9); // 12 - 3
        assert_eq!(wot.get_non_sentries(2).len(), 11); // 12 - 1
        assert_eq!(wot.get_non_sentries(3).len(), 12); // 12 - 0
        assert_eq!(wot.get_paths(NodeId(3), NodeId(0), 1).len(), 0); // KO
        assert_eq!(wot.get_paths(NodeId(3), NodeId(0), 2).len(), 1); // It exists 3 -> 2 -> 0
        assert_eq!(wot.get_paths(NodeId(3), NodeId(0), 2)[0].len(), 3); // It exists 3 -> 2 -> 0
        assert_eq!(wot.is_outdistanced(NodeId(0), 1, 1, 1.0), Some(true)); // KO : No path 3 -> 0
        assert_eq!(wot.is_outdistanced(NodeId(0), 2, 1, 1.0), Some(true)); // KO : No path 3 -> 0
        assert_eq!(wot.is_outdistanced(NodeId(0), 3, 1, 1.0), Some(false)); // OK : no stry \w 3 lnk
        assert_eq!(wot.is_outdistanced(NodeId(0), 2, 2, 1.0), Some(false)); // OK : 3 -> 2 -> 0

        // should have 12 nodes
        assert_eq!(wot.size(), 12);

        // delete top node (return new top node id)
        assert_eq!(wot.remove_node(), Some(NodeId(10)));

        // should have 11 nodes
        assert_eq!(wot.size(), 11);

        // should work with member 3 disabled
        // - with member 3 disabled (non-member)
        assert_eq!(wot.set_enabled(NodeId(3), false), Some(false));
        assert_eq!(wot.get_disabled().len(), 1);
        assert_eq!(wot.is_outdistanced(NodeId(0), 2, 1, 1.0), Some(false)); // OK : Disabled

        // should be able to make a mem copy
        {
            let wot2 = wot.clone();

            assert_eq!(wot.size(), wot2.size());
            assert_eq!(
                wot.get_non_sentries(1).len(),
                wot2.get_non_sentries(1).len()
            );
        }

        // serialization
        assert_eq!(wot.to_file("test.wot"), true);

        // deserialization
        {
            let wot2 = WebOfTrust::from_file("test.wot").unwrap();

            assert_eq!(wot.size(), wot2.size());
            assert_eq!(
                wot.get_non_sentries(1).len(),
                wot2.get_non_sentries(1).len()
            );
        }

    }
}
