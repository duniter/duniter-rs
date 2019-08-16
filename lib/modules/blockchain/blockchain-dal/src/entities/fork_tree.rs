//  Copyright (C) 2017-2019  The AXIOM TEAM Association.
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

//! Describe fork tree

use dubp_common_doc::{BlockHash, BlockNumber, Blockstamp};
use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::{Serialize, Serializer};
use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// unique identifier of tree node
pub struct TreeNodeId(pub usize);

impl Serialize for TreeNodeId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.0 as u32)
    }
}

struct TreeNodeIdVisitor;

impl<'de> Visitor<'de> for TreeNodeIdVisitor {
    type Value = TreeNodeId;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer between -2^31 and 2^31")
    }

    fn visit_u8<E>(self, value: u8) -> Result<TreeNodeId, E>
    where
        E: de::Error,
    {
        Ok(TreeNodeId(value as usize))
    }

    fn visit_u32<E>(self, value: u32) -> Result<TreeNodeId, E>
    where
        E: de::Error,
    {
        Ok(TreeNodeId(value as usize))
    }

    fn visit_u64<E>(self, value: u64) -> Result<TreeNodeId, E>
    where
        E: de::Error,
    {
        use std::usize;
        if value >= usize::MIN as u64 && value <= usize::MAX as u64 {
            Ok(TreeNodeId(value as usize))
        } else {
            Err(E::custom(format!("u32 out of range: {}", value)))
        }
    }
}

impl<'de> Deserialize<'de> for TreeNodeId {
    fn deserialize<D>(deserializer: D) -> Result<TreeNodeId, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u32(TreeNodeIdVisitor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Tree node
pub struct TreeNode {
    /// Parent node
    pub parent: Option<TreeNodeId>,
    /// Children nodes
    pub children: Vec<TreeNodeId>,
    /// Node data
    pub data: Blockstamp,
}

impl TreeNode {
    /// Instantiate new TreeNode
    pub fn new(parent: Option<TreeNodeId>, data: Blockstamp) -> Self {
        TreeNode {
            parent,
            children: Vec::new(),
            data,
        }
    }
    /// Add child to node
    pub fn add_child(&mut self, id: TreeNodeId) {
        self.children.push(id);
    }
    /// Get node children
    pub fn children(&self) -> &[TreeNodeId] {
        &self.children
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Tree store all forks branchs
pub struct ForkTree {
    main_branch: HashMap<BlockNumber, TreeNodeId>,
    max_depth: usize,
    nodes: Vec<Option<TreeNode>>,
    removed_blockstamps: Vec<Blockstamp>,
    root: Option<TreeNodeId>,
    sheets: HashSet<TreeNodeId>,
}

impl Default for ForkTree {
    #[inline]
    fn default() -> Self {
        ForkTree::new(*dubp_currency_params::constants::DEFAULT_FORK_WINDOW_SIZE)
    }
}

impl ForkTree {
    /// Instanciate new fork tree
    #[inline]
    pub fn new(max_depth: usize) -> Self {
        ForkTree {
            main_branch: HashMap::with_capacity(max_depth + 1),
            max_depth,
            nodes: Vec::with_capacity(max_depth * 2),
            removed_blockstamps: Vec::with_capacity(max_depth),
            root: None,
            sheets: HashSet::new(),
        }
    }
    /// Set max depth
    #[inline]
    pub fn set_max_depth(&mut self, max_depth: usize) {
        self.max_depth = max_depth;
    }
    /// Get tree size
    #[inline]
    pub fn size(&self) -> usize {
        self.nodes
            .iter()
            .map(|n| if n.is_some() { 1 } else { 0 })
            .sum()
    }
    /// Get root node id
    #[inline]
    pub fn get_root_id(&self) -> Option<TreeNodeId> {
        self.root
    }
    /// Get blockstamp for each sheet of tree
    pub fn get_sheets(&self) -> Vec<(TreeNodeId, Blockstamp)> {
        self.sheets
            .iter()
            .map(|s| (*s, self.get_ref_node(*s).data))
            .collect()
    }
    ///
    pub fn get_removed_blockstamps(&self) -> Vec<Blockstamp> {
        self.removed_blockstamps.clone()
    }
    /// Get specific tree node
    #[inline]
    fn get_node(&self, id: TreeNodeId) -> TreeNode {
        self.nodes
            .get(id.0)
            .cloned()
            .expect("Dev error: fork tree : get unexist node !")
            .expect("Dev error: fork tree : get removed node !")
    }
    /// Get reference to a specific tree node
    #[inline]
    fn get_ref_node(&self, id: TreeNodeId) -> &TreeNode {
        if let Some(Some(ref node)) = self.nodes.get(id.0) {
            node
        } else {
            durs_common_tools::fatal_error!("Dev error: fork tree : get unexist or removed node !");
        }
    }
    /// Get mutable reference to a specific tree node
    #[inline]
    fn get_mut_node(&mut self, id: TreeNodeId) -> &mut TreeNode {
        if let Some(Some(ref mut node)) = self.nodes.get_mut(id.0) {
            node
        } else {
            durs_common_tools::fatal_error!("Dev error: fork tree : get unexist or removed node !");
        }
    }
    /// Get free identifier
    fn get_free_node_id(&self) -> Option<TreeNodeId> {
        let mut new_node_id = None;
        for i in 0..self.nodes.len() {
            if self.nodes.get(i).is_none() {
                new_node_id = Some(TreeNodeId(i));
            }
        }

        new_node_id
    }
    /// Get main branch node
    #[inline]
    pub fn get_main_branch_node_id(&self, block_id: BlockNumber) -> Option<TreeNodeId> {
        if let Some(node_id) = self.main_branch.get(&block_id) {
            Some(*node_id)
        } else {
            None
        }
    }
    /// Get main branch block hash
    #[inline]
    pub fn get_main_branch_block_hash(&self, block_id: BlockNumber) -> Option<BlockHash> {
        if let Some(node_id) = self.main_branch.get(&block_id) {
            Some(self.get_ref_node(*node_id).data.hash)
        } else {
            None
        }
    }
    /// Get fork branch nodes ids
    pub fn get_fork_branch_nodes_ids(&self, node_id: TreeNodeId) -> Vec<TreeNodeId> {
        let mut branch = Vec::with_capacity(self.max_depth);

        let node = self.get_ref_node(node_id);
        if !self.main_branch.contains_key(&node.data.id)
            || self
                .get_main_branch_block_hash(node.data.id)
                .expect("safe unwrap")
                != node.data.hash
        {
            branch.push(node_id);
        }

        if let Some(first_parent_id) = node.parent {
            let mut parent = self.get_ref_node(first_parent_id);
            let mut parent_id = first_parent_id;
            while !self.main_branch.contains_key(&parent.data.id)
                || self
                    .get_main_branch_block_hash(parent.data.id)
                    .expect("safe unwrap")
                    != parent.data.hash
            {
                branch.push(parent_id);

                if let Some(next_parent_id) = parent.parent {
                    parent = self.get_ref_node(next_parent_id);
                    parent_id = next_parent_id;
                } else {
                    break;
                }
            }
        }

        branch.reverse();
        branch
    }
    /// Get fork branch
    pub fn get_fork_branch(&self, node_id: TreeNodeId) -> Vec<Blockstamp> {
        let mut branch = Vec::with_capacity(self.max_depth);
        let node = self.get_ref_node(node_id);
        branch.push(node.data);

        if let Some(parent_id) = node.parent {
            let mut parent = self.get_ref_node(parent_id);
            while !self.main_branch.contains_key(&parent.data.id)
                || self
                    .get_main_branch_block_hash(parent.data.id)
                    .expect("safe unwrap")
                    != parent.data.hash
            {
                branch.push(parent.data);

                if let Some(parent_id) = parent.parent {
                    parent = self.get_ref_node(parent_id);
                } else {
                    break;
                }
            }
        }

        branch.reverse();
        branch
    }
    /// Modify the main branch (function to call after a successful roolback)
    pub fn change_main_branch(
        &mut self,
        old_current_blockstamp: Blockstamp,
        new_current_blockstamp: Blockstamp,
    ) {
        let target_node = self
            .find_node_with_blockstamp(&new_current_blockstamp)
            .expect("Dev error: change main branch: target branch don't exist !");
        let selected_fork_branch = self.get_fork_branch_nodes_ids(target_node);

        // // Delete the part of the old branch at same level to the new branch
        if !selected_fork_branch.is_empty() {
            let selected_fork_branch_first_node_id =
                selected_fork_branch.get(0).copied().expect("safe unwrap");
            let first_fork_block_id = self.nodes[selected_fork_branch_first_node_id.0]
                .clone()
                .expect("Dev error: node must exist !")
                .data
                .id;
            for block_id in first_fork_block_id.0..=new_current_blockstamp.id.0 {
                self.main_branch.remove(&BlockNumber(block_id));
            }
        }
        // Delete the part of the old branch that exceeds the new branch
        if old_current_blockstamp.id > new_current_blockstamp.id {
            for block_id in (new_current_blockstamp.id.0 + 1)..=old_current_blockstamp.id.0 {
                self.main_branch.remove(&BlockNumber(block_id));
            }
        }

        for node_id in selected_fork_branch {
            let node = self.nodes[node_id.0]
                .clone()
                .expect("Dev error: node must exist !");
            self.main_branch.insert(node.data.id, node_id);
            if node.data.id > old_current_blockstamp.id {
                self.pruning();
            }
        }
    }
    /// Find node with specific blockstamp
    pub fn find_node_with_blockstamp(&self, blockstamp: &Blockstamp) -> Option<TreeNodeId> {
        for (node_id, node_opt) in self.nodes.iter().enumerate() {
            if let Some(node) = node_opt {
                if node.data == *blockstamp {
                    return Some(TreeNodeId(node_id));
                }
            }
        }

        None
    }
    /// Insert new node with specified identifier,
    /// return blockstamps of deleted blocks
    pub fn insert_new_node(
        &mut self,
        data: Blockstamp,
        parent: Option<TreeNodeId>,
        main_branch: bool,
    ) {
        let new_node = TreeNode::new(parent, data);
        let mut new_node_id = self.get_free_node_id();

        if new_node_id.is_none() {
            new_node_id = Some(TreeNodeId(self.nodes.len()));
            self.nodes.push(Some(new_node));
        } else {
            self.nodes[new_node_id.expect("safe unwrap").0] = Some(new_node);
        }
        let new_node_id = new_node_id.expect("safe unwrap");

        if let Some(parent) = parent {
            // Remove previous sheet
            if !self.sheets.is_empty() {
                self.sheets.remove(&parent);
            }
            // Add new node in parent
            self.get_mut_node(parent).add_child(new_node_id);
        } else if self.root.is_none() {
            self.root = Some(new_node_id);
        } else {
            durs_common_tools::fatal_error!("Dev error: Insert root node in not empty tree !")
        }

        self.removed_blockstamps.clear();
        if main_branch {
            self.main_branch.insert(data.id, new_node_id);
            if self.main_branch.len() > self.max_depth {
                self.pruning();
            }
        }

        // Add new sheet
        self.sheets.insert(new_node_id);
    }

    fn pruning(&mut self) {
        // get root node infos
        let root_node_id = self.root.expect("safe unwrap");
        let root_node = self.get_node(root_node_id);
        let root_node_block_id: BlockNumber = root_node.data.id;

        // Shift the tree one step : remove root node and all his children except main child
        self.main_branch.remove(&root_node_block_id);
        self.removed_blockstamps.push(root_node.data);
        let root_node_main_child_id = self
            .main_branch
            .get(&BlockNumber(root_node_block_id.0 + 1))
            .copied()
            .expect("safe unwrap");
        let mut children_to_remove = Vec::new();
        for child_id in root_node.children() {
            if *child_id != root_node_main_child_id {
                children_to_remove.push(*child_id);
            }
        }

        for child_id in children_to_remove {
            self.remove_node_children(child_id);
            self.removed_blockstamps
                .push(self.nodes[child_id.0].clone().expect("safe unwrap").data);
            self.nodes[child_id.0] = None;
            self.sheets.remove(&child_id);
        }

        // Remove root node
        self.nodes[root_node_id.0] = None;
        self.root = Some(root_node_main_child_id);
    }

    /// Return removed blockstamps
    fn remove_node_children(&mut self, id: TreeNodeId) {
        let mut ids_to_rm: Vec<TreeNodeId> = Vec::new();

        let mut node = self.get_ref_node(id);
        while node.children.len() <= 1 {
            if let Some(child_id) = node.children.get(0) {
                ids_to_rm.push(*child_id);
                node = self.get_ref_node(*child_id);
            } else {
                break;
            }
        }

        for child_id in node.children.clone() {
            self.remove_node_children(child_id);
        }

        for node_id in ids_to_rm {
            self.removed_blockstamps
                .push(self.nodes[node_id.0].clone().expect("safe unwrap").data);
            self.nodes[node_id.0] = None;
            self.sheets.remove(&node_id);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use dubp_currency_params::constants::DEFAULT_FORK_WINDOW_SIZE;

    #[test]
    fn insert_root_nodes() {
        let mut tree = ForkTree::default();
        assert_eq!(0, tree.size());

        let root_blockstamp = Blockstamp {
            id: BlockNumber(0),
            hash: BlockHash(dup_crypto_tests_tools::mocks::hash('A')),
        };
        tree.insert_new_node(root_blockstamp, None, true);
        assert_eq!(1, tree.size());
        assert_eq!(
            TreeNodeId(0),
            tree.get_root_id().expect("tree without root")
        );
        assert_eq!(vec![(TreeNodeId(0), root_blockstamp)], tree.get_sheets());
        assert_eq!(None, tree.get_free_node_id());
        assert_eq!(
            Some(TreeNodeId(0)),
            tree.get_main_branch_node_id(BlockNumber(0))
        );
        assert_eq!(
            Some(root_blockstamp.hash),
            tree.get_main_branch_block_hash(BlockNumber(0))
        );
        assert_eq!(
            Some(TreeNodeId(0)),
            tree.find_node_with_blockstamp(&root_blockstamp)
        );
    }

    #[test]
    fn insert_some_nodes() {
        let mut tree = ForkTree::default();
        let blockstamps = vec![
            Blockstamp {
                id: BlockNumber(0),
                hash: BlockHash(dup_crypto_tests_tools::mocks::hash('A')),
            },
            Blockstamp {
                id: BlockNumber(1),
                hash: BlockHash(dup_crypto_tests_tools::mocks::hash('B')),
            },
            Blockstamp {
                id: BlockNumber(2),
                hash: BlockHash(dup_crypto_tests_tools::mocks::hash('C')),
            },
        ];

        tree.insert_new_node(blockstamps[0], None, true);
        tree.insert_new_node(blockstamps[1], Some(TreeNodeId(0)), true);
        tree.insert_new_node(blockstamps[2], Some(TreeNodeId(1)), true);
        assert_eq!(3, tree.size());
        assert_eq!(
            TreeNodeId(0),
            tree.get_root_id().expect("tree without root")
        );
        assert_eq!(vec![(TreeNodeId(2), blockstamps[2])], tree.get_sheets());
        assert_eq!(None, tree.get_free_node_id());
        for i in 0..=2 {
            assert_eq!(
                Some(TreeNodeId(i)),
                tree.get_main_branch_node_id(BlockNumber(i as u32))
            );
            assert_eq!(
                Some(blockstamps[i].hash),
                tree.get_main_branch_block_hash(BlockNumber(i as u32))
            );
            assert_eq!(
                Some(TreeNodeId(i)),
                tree.find_node_with_blockstamp(&blockstamps[i])
            );
        }
    }

    #[test]
    fn insert_fork_blocks() {
        // Fill tree with 10 nodes
        let mut tree = ForkTree::default();
        let blockstamps: Vec<Blockstamp> =
            dubp_user_docs_tests_tools::mocks::generate_blockstamps(10);
        tree.insert_new_node(blockstamps[0], None, true);
        for i in 1..10 {
            tree.insert_new_node(blockstamps[i], Some(TreeNodeId(i - 1)), true);
        }
        assert_eq!(10, tree.size());

        // Insert fork block after block 5
        let fork_blockstamp = Blockstamp {
            id: BlockNumber(6),
            hash: BlockHash(dup_crypto_tests_tools::mocks::hash('B')),
        };
        tree.insert_new_node(
            fork_blockstamp,
            tree.get_main_branch_node_id(BlockNumber(5)),
            false,
        );

        // Check that the tree is indeed 2 sheets
        let sheets = tree.get_sheets();
        assert_eq!(2, sheets.len());

        // Check sheets content
        let expected_sheets = vec![
            (
                TreeNodeId(9),
                Blockstamp {
                    id: BlockNumber(9),
                    hash: BlockHash(dup_crypto_tests_tools::mocks::hash_from_byte(9u8)),
                },
            ),
            (TreeNodeId(10), fork_blockstamp),
        ];
        assert!(
            (sheets[0] == expected_sheets[0] && sheets[1] == expected_sheets[1])
                || (sheets[0] == expected_sheets[1] && sheets[1] == expected_sheets[0])
        );

        // Get fork branch
        assert_eq!(vec![fork_blockstamp], tree.get_fork_branch(TreeNodeId(10)));
        assert_eq!(
            vec![TreeNodeId(10)],
            tree.get_fork_branch_nodes_ids(TreeNodeId(10))
        );

        // Insert child to fork block
        let child_fork_blockstamp = Blockstamp {
            id: BlockNumber(7),
            hash: BlockHash(dup_crypto_tests_tools::mocks::hash('C')),
        };
        tree.insert_new_node(child_fork_blockstamp, Some(TreeNodeId(10)), false);

        // Check that the tree still has 2 leaves
        let sheets = tree.get_sheets();
        assert_eq!(2, sheets.len());

        // Check sheets content
        let expected_sheets = vec![
            (
                TreeNodeId(9),
                Blockstamp {
                    id: BlockNumber(9),
                    hash: BlockHash(dup_crypto_tests_tools::mocks::hash_from_byte(9u8)),
                },
            ),
            (TreeNodeId(11), child_fork_blockstamp),
        ];
        assert!(durs_common_tests_tools::collections::slice_same_elems(
            &expected_sheets,
            &sheets
        ));

        // Get fork branch
        assert_eq!(
            vec![fork_blockstamp, child_fork_blockstamp],
            tree.get_fork_branch(TreeNodeId(11))
        );
        assert_eq!(
            vec![TreeNodeId(10), TreeNodeId(11)],
            tree.get_fork_branch_nodes_ids(TreeNodeId(11))
        );
    }

    #[test]
    fn insert_more_fork_window_size_nodes() {
        let mut tree = ForkTree::default();
        let blockstamps: Vec<Blockstamp> =
            dubp_user_docs_tests_tools::mocks::generate_blockstamps(*DEFAULT_FORK_WINDOW_SIZE + 2);

        // Fill tree with MAX_DEPTH nodes
        tree.insert_new_node(blockstamps[0], None, true);
        for i in 1..*DEFAULT_FORK_WINDOW_SIZE {
            tree.insert_new_node(blockstamps[i], Some(TreeNodeId(i - 1)), true);
        }

        // The tree-root must not have been shifted yet
        assert_eq!(*DEFAULT_FORK_WINDOW_SIZE, tree.size());
        assert_eq!(Some(TreeNodeId(0)), tree.get_root_id());

        // Inserting a node that exceeds FORK_WIN_SIZE,
        // the tree size must not be increased and the root must shift
        tree.insert_new_node(
            blockstamps[*DEFAULT_FORK_WINDOW_SIZE],
            Some(TreeNodeId(*DEFAULT_FORK_WINDOW_SIZE - 1)),
            true,
        );
        assert_eq!(*DEFAULT_FORK_WINDOW_SIZE, tree.size());
        assert_eq!(Some(TreeNodeId(1)), tree.get_root_id());

        // Repeating the insertion of a node that exceeds FORK_WIN_SIZE,
        // the tree size must still not be increased and the root must still shift
        tree.insert_new_node(
            blockstamps[*DEFAULT_FORK_WINDOW_SIZE + 1],
            Some(TreeNodeId(*DEFAULT_FORK_WINDOW_SIZE)),
            true,
        );
        assert_eq!(*DEFAULT_FORK_WINDOW_SIZE, tree.size());
        assert_eq!(Some(TreeNodeId(2)), tree.get_root_id());
    }

    #[test]
    fn test_change_main_branch() {
        let mut tree = ForkTree::default();
        let blockstamps: Vec<Blockstamp> =
            dubp_user_docs_tests_tools::mocks::generate_blockstamps(*DEFAULT_FORK_WINDOW_SIZE + 2);

        // Fill tree with MAX_DEPTH nodes
        tree.insert_new_node(blockstamps[0], None, true);
        for i in 1..*DEFAULT_FORK_WINDOW_SIZE {
            tree.insert_new_node(blockstamps[i], Some(TreeNodeId(i - 1)), true);
        }

        // Insert 2 forks blocks after block (MAX_DEPTH - 2)
        let fork_blockstamp = Blockstamp {
            id: BlockNumber(*DEFAULT_FORK_WINDOW_SIZE as u32 - 1),
            hash: BlockHash(dup_crypto_tests_tools::mocks::hash('A')),
        };
        tree.insert_new_node(
            fork_blockstamp,
            tree.get_main_branch_node_id(BlockNumber(*DEFAULT_FORK_WINDOW_SIZE as u32 - 2)),
            false,
        );
        let fork_blockstamp_2 = Blockstamp {
            id: BlockNumber(*DEFAULT_FORK_WINDOW_SIZE as u32),
            hash: BlockHash(dup_crypto_tests_tools::mocks::hash('B')),
        };
        tree.insert_new_node(
            fork_blockstamp_2,
            Some(TreeNodeId(*DEFAULT_FORK_WINDOW_SIZE)),
            false,
        );

        // Check tree size
        assert_eq!(*DEFAULT_FORK_WINDOW_SIZE + 2, tree.size());

        // Check that the root of the shaft has not shifted
        assert_eq!(Some(TreeNodeId(0)), tree.get_root_id());

        // Check that the tree is indeed 2 sheets
        let sheets = tree.get_sheets();
        assert_eq!(2, sheets.len());

        // Check sheets content
        let expected_sheets = vec![
            (
                TreeNodeId(*DEFAULT_FORK_WINDOW_SIZE - 1),
                blockstamps[*DEFAULT_FORK_WINDOW_SIZE - 1],
            ),
            (TreeNodeId(*DEFAULT_FORK_WINDOW_SIZE + 1), fork_blockstamp_2),
        ];
        println!("{:?}", sheets);
        assert!(durs_common_tests_tools::collections::slice_same_elems(
            &expected_sheets,
            &sheets
        ));

        // Switch to fork branch
        tree.change_main_branch(
            blockstamps[*DEFAULT_FORK_WINDOW_SIZE - 1],
            fork_blockstamp_2,
        );

        // Check that the shaft still has 2 same sheets
        assert!(durs_common_tests_tools::collections::slice_same_elems(
            &expected_sheets,
            &sheets
        ));

        // Check that tree size decrease
        assert_eq!(*DEFAULT_FORK_WINDOW_SIZE + 1, tree.size());

        // Check that the root of the tree has shifted
        assert_eq!(Some(TreeNodeId(1)), tree.get_root_id());
    }

}
