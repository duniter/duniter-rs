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

//! Provides the definition of the indexes described in the DUBP RFC.

#![deny(
    clippy::option_unwrap_used,
    clippy::result_unwrap_used,
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces
)]

pub mod cindex;
pub mod iindex;
pub mod mindex;
pub mod sindex;

use shrinkwraprs::Shrinkwrap;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

/// Index line op column (CREATE or UPDATE)
///
/// Stored in a boolean :
/// CREATE encoded as true
/// UPDATE encoded as false
#[derive(Clone, Copy, Debug, PartialEq, Shrinkwrap)]
pub struct IndexLineOp(bool);

/// Generic INDEX
#[derive(Clone, Debug)]
pub struct Index<ID, IndexLine>
where
    ID: Clone + Debug + Eq + Hash,
    IndexLine: Debug + MergeIndexLine,
{
    datas: HashMap<ID, Vec<IndexLine>>,
}

impl<ID, IndexLine> Index<ID, IndexLine>
where
    ID: Clone + Debug + Eq + Hash,
    IndexLine: Copy + Debug + MergeIndexLine,
{
    /// Get entity state
    pub fn get_state(&self, entity_id: &ID) -> Option<IndexLine> {
        self.get_events(entity_id).map(Self::reduce)
    }
}

impl<ID, IndexLine> Index<ID, IndexLine>
where
    ID: Clone + Debug + Eq + Hash,
    IndexLine: Clone + Debug + MergeIndexLine,
{
    /// Get entity state
    pub fn get_state_by_cloning(&self, entity_id: &ID) -> Option<IndexLine> {
        self.get_events(entity_id).map(Self::reduce_by_cloning)
    }
}

impl<ID, IndexLine> Index<ID, IndexLine>
where
    ID: Clone + Debug + Eq + Hash,
    IndexLine: Clone + Debug + MergeIndexLine,
{
    /// Get entity events
    pub fn get_events(&self, entity_id: &ID) -> Option<&[IndexLine]> {
        self.datas
            .get(entity_id)
            .map(|index_lines| &index_lines[..])
    }
}

impl<ID, IndexLine> ReduceIndexLines for Index<ID, IndexLine>
where
    ID: Clone + Debug + Eq + Hash,
    IndexLine: Copy + Debug + MergeIndexLine,
{
    type IndexLine = IndexLine;
}

impl<ID, IndexLine> ReduceNotCopyableIndexLines for Index<ID, IndexLine>
where
    ID: Clone + Debug + Eq + Hash,
    IndexLine: Clone + Debug + MergeIndexLine,
{
    type IndexLine = IndexLine;
}

/// Merge one index line with another
pub trait MergeIndexLine {
    /// Merge one index line with another
    ///
    /// `self` is the current state of the entity.
    /// `index_line` is the event to "apply" to the entity
    fn merge_index_line(&mut self, index_line: Self);
}

/// Reduce all index lines into one to obtain the current state of the entity.
pub trait ReduceIndexLines {
    /// Index line (represent one event on an entity)
    type IndexLine: Copy + MergeIndexLine;

    /// Each index line represents an event on an entity.
    /// This function reduce all index lines into one to obtain the current state of the entity.
    fn reduce(index_lines: &[Self::IndexLine]) -> Self::IndexLine {
        let mut entity_state = index_lines[0];

        for index_line in &index_lines[1..] {
            entity_state.merge_index_line(*index_line);
        }

        entity_state
    }
}

/// Reduce all index lines into one to obtain the current state of the entity.
pub trait ReduceNotCopyableIndexLines {
    /// Index line (represent one event on an entity)
    type IndexLine: Clone + MergeIndexLine;

    /// Each index line represents an event on an entity.
    /// This function reduce all index lines into one to obtain the current state of the entity.
    fn reduce_by_cloning(index_lines: &[Self::IndexLine]) -> Self::IndexLine {
        let mut entity_state = index_lines[0].clone();

        for index_line in &index_lines[1..] {
            entity_state.merge_index_line(index_line.clone());
        }

        entity_state
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    type TestIndex = Index<usize, TestIndexLine>;
    type TestIndexNotCopyable = Index<usize, TestIndexLineNotCopyable>;

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct TestIndexLine {
        op: IndexLineOp,
        id: usize,
        field: Option<bool>,
    }

    impl MergeIndexLine for TestIndexLine {
        fn merge_index_line(&mut self, index_line: Self) {
            self.op = index_line.op;
            index_line.field.map(|v| self.field.replace(v));
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    struct TestIndexLineNotCopyable {
        op: IndexLineOp,
        id: usize,
        field: Option<String>,
    }

    impl MergeIndexLine for TestIndexLineNotCopyable {
        fn merge_index_line(&mut self, index_line: Self) {
            self.op = index_line.op;
            index_line.field.map(|v| self.field.replace(v));
        }
    }

    fn new_test_index() -> TestIndex {
        let index_lines = vec![
            TestIndexLine {
                op: IndexLineOp(true),
                id: 0,
                field: Some(true),
            },
            TestIndexLine {
                op: IndexLineOp(false),
                id: 0,
                field: None,
            },
        ];
        let mut datas = HashMap::new();
        datas.insert(0, index_lines);
        TestIndex { datas }
    }

    fn new_test_index_not_copyable() -> TestIndexNotCopyable {
        let index_lines = vec![
            TestIndexLineNotCopyable {
                op: IndexLineOp(true),
                id: 0,
                field: Some("toto".to_owned()),
            },
            TestIndexLineNotCopyable {
                op: IndexLineOp(false),
                id: 0,
                field: None,
            },
        ];
        let mut datas = HashMap::new();
        datas.insert(0, index_lines);
        TestIndexNotCopyable { datas }
    }

    #[test]
    fn test_reduce_copyable_index() {
        let index = new_test_index();
        assert_eq!(
            Some(TestIndexLine {
                op: IndexLineOp(false),
                id: 0,
                field: Some(true),
            }),
            index.get_state(&0)
        )
    }

    #[test]
    fn test_reduce_not_copyable_index() {
        let index = new_test_index_not_copyable();
        assert_eq!(
            Some(TestIndexLineNotCopyable {
                op: IndexLineOp(false),
                id: 0,
                field: Some("toto".to_owned()),
            }),
            index.get_state_by_cloning(&0)
        )
    }
}
