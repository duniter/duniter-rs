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
//! [Duniter]: https://duniter.org/
//!
//! It defines a trait representing a Web of Trust and allow to do calculations on it.
//! 
//! It also contains an "legacy" implementation translated from the original C++ code.
//!
//! Web of Trust tests are translated from [duniter/wotb Javascript test][js-tests].
//!
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Test translated from https://github.com/duniter/wotb/blob/master/tests/test.js
    /// 
    /// Clone and file tests are not included in this generic test and should be done in
    /// the implementation test.
    pub fn generic_wot_test<T: WebOfTrust>(wot: &mut T) {
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

        assert_eq!(wot.get_max_link(), 3);
        assert_eq!(wot.has_link(NodeId(0), NodeId(1)), HasLinkResult::Link(true));
        assert_eq!(wot.has_link(NodeId(0), NodeId(2)), HasLinkResult::Link(true));
        assert_eq!(wot.has_link(NodeId(0), NodeId(3)), HasLinkResult::Link(true));
        assert_eq!(wot.has_link(NodeId(0), NodeId(4)), HasLinkResult::Link(false));

        wot.set_max_link(4);
        assert_eq!(wot.get_max_link(), 4);
        assert_eq!(wot.has_link(NodeId(0), NodeId(4)), HasLinkResult::Link(false));
        wot.add_link(NodeId(0), NodeId(4));
        assert_eq!(wot.has_link(NodeId(0), NodeId(4)), HasLinkResult::Link(true));
        wot.rem_link(NodeId(0), NodeId(1));
        wot.rem_link(NodeId(0), NodeId(2));
        wot.rem_link(NodeId(0), NodeId(3));
        wot.rem_link(NodeId(0), NodeId(4));

        // false when not linked + test out of bounds
        assert_eq!(wot.has_link(NodeId(0), NodeId(6)), HasLinkResult::Link(false));
        assert_eq!(wot.has_link(NodeId(23), NodeId(0)), HasLinkResult::UnknownSource());
        assert_eq!(wot.has_link(NodeId(2), NodeId(53)), HasLinkResult::UnknownTarget());

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
        assert_eq!(wot.has_link(NodeId(2), NodeId(0)), HasLinkResult::Link(false));

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

        assert_eq!(wot.has_link(NodeId(2), NodeId(0)), HasLinkResult::Link(true));
        assert_eq!(wot.has_link(NodeId(4), NodeId(0)), HasLinkResult::Link(true));
        assert_eq!(wot.has_link(NodeId(5), NodeId(0)), HasLinkResult::Link(true));
        assert_eq!(wot.has_link(NodeId(2), NodeId(1)), HasLinkResult::Link(false));

        // should be able to remove some links
        assert_eq!(
            wot.rem_link(NodeId(4), NodeId(0)),
            RemLinkResult::Removed(2)
        );
        /*
         * WoT is now:
         *
         * 2 --> 0
         * 5 --> 0
         */

        // should exist less links
        assert_eq!(wot.has_link(NodeId(2), NodeId(0)), HasLinkResult::Link(true));
        assert_eq!(wot.has_link(NodeId(4), NodeId(0)), HasLinkResult::Link(false));
        assert_eq!(wot.has_link(NodeId(5), NodeId(0)), HasLinkResult::Link(true));
        assert_eq!(wot.has_link(NodeId(2), NodeId(1)), HasLinkResult::Link(false));

        // should successfully use distance rule
        assert_eq!(wot.is_outdistanced(WotDistanceParameters {
            node: NodeId(0),
            sentry_requirement: 1,
            step_max: 1,
            x_percent: 1.0
        }), Some(false));
        // => no because 2,4,5 have certified him
        assert_eq!(wot.is_outdistanced(WotDistanceParameters {
            node: NodeId(0),
            sentry_requirement: 2,
            step_max: 1,
            x_percent: 1.0
        }), Some(false));
        // => no because only member 2 has 2 certs, and has certified him
        assert_eq!(wot.is_outdistanced(WotDistanceParameters {
            node: NodeId(0),
            sentry_requirement: 3,
            step_max: 1,
            x_percent: 1.0
        }), Some(false));
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
        assert_eq!(wot.is_outdistanced(WotDistanceParameters {
            node: NodeId(0),
            sentry_requirement: 1,
            step_max: 1,
            x_percent: 1.0
        }), Some(false)); // OK : 2 -> 0
        assert_eq!(wot.is_outdistanced(WotDistanceParameters {
            node: NodeId(0),
            sentry_requirement: 2,
            step_max: 1, 
            x_percent: 1.0
        }), Some(false)); // OK : 2 -> 0
        assert_eq!(wot.is_outdistanced(WotDistanceParameters {
            node: NodeId(0),
            sentry_requirement: 3,
            step_max: 1, 
            x_percent: 1.0
        }), Some(false)); // OK : no stry \w 3 lnk
        assert_eq!(wot.is_outdistanced(WotDistanceParameters {
            node: NodeId(0),
            sentry_requirement: 2, 
            step_max: 2, 
            x_percent: 1.0
        }), Some(false)); // OK : 2 -> 0

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
        assert_eq!(wot.is_outdistanced(WotDistanceParameters {
            node: NodeId(0),
            sentry_requirement: 1,
            step_max: 1, 
            x_percent: 1.0
        }), Some(true)); // KO : No path 3 -> 0
        assert_eq!(wot.is_outdistanced(WotDistanceParameters {
            node: NodeId(0),
            sentry_requirement: 2, 
            step_max: 1, 
            x_percent: 1.0
        }), Some(true)); // KO : No path 3 -> 0
        assert_eq!(wot.is_outdistanced(WotDistanceParameters {
            node: NodeId(0), 
            sentry_requirement: 3, 
            step_max: 1, 
            x_percent: 1.0
        }), Some(false)); // OK : no stry \w 3 lnk
        assert_eq!(wot.is_outdistanced(WotDistanceParameters {
            node: NodeId(0),
            sentry_requirement: 2, 
            step_max: 2, 
            x_percent: 1.0
        }), Some(false)); // OK : 3 -> 2 -> 0

        // should have 12 nodes
        assert_eq!(wot.size(), 12);

        // delete top node (return new top node id)
        assert_eq!(wot.rem_node(), Some(NodeId(10)));

        // should have 11 nodes
        assert_eq!(wot.size(), 11);

        // should work with member 3 disabled
        // - with member 3 disabled (non-member)
        assert_eq!(wot.set_enabled(NodeId(3), false), Some(false));
        assert_eq!(wot.get_disabled().len(), 1);
        assert_eq!(wot.is_outdistanced(WotDistanceParameters {
            node: NodeId(0), 
            sentry_requirement: 2, 
            step_max: 1, 
            x_percent: 1.0
        }), Some(false)); // OK : Disabled
    }
}