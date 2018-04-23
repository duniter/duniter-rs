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

#![cfg_attr(feature = "strict", deny(warnings))]
#![deny(
    missing_docs, missing_debug_implementations, missing_copy_implementations, trivial_casts,
    trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces,
    unused_qualifications
)]

extern crate bincode;
extern crate byteorder;
extern crate rayon;
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod data;
pub mod operations;

pub use data::{NodeId, WebOfTrust};

#[cfg(test)]
mod tests {
    use super::*;
    use data::*;
    use operations::centrality::*;
    use operations::distance::*;
    use operations::file::*;
    use operations::path::*;

    /// Test translated from https://github.com/duniter/wotb/blob/master/tests/test.js
    ///
    /// Clone and file tests are not included in this generic test and should be done in
    /// the implementation test.
    pub fn generic_wot_test<W>()
    where
        W: WebOfTrust + Sync,
    {
        let centralities_calculator = UlrikBrandesCentralityCalculator {};
        let distance_calculator = RustyDistanceCalculator {};
        let path_finder = RustyPathFinder {};
        let mut wot = W::new(3);

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
        assert_eq!(
            wot.has_link(NodeId(0), NodeId(1)),
            HasLinkResult::Link(true)
        );
        assert_eq!(
            wot.has_link(NodeId(0), NodeId(2)),
            HasLinkResult::Link(true)
        );
        assert_eq!(
            wot.has_link(NodeId(0), NodeId(3)),
            HasLinkResult::Link(true)
        );
        assert_eq!(
            wot.has_link(NodeId(0), NodeId(4)),
            HasLinkResult::Link(false)
        );

        wot.set_max_link(4);
        assert_eq!(wot.get_max_link(), 4);
        assert_eq!(
            wot.has_link(NodeId(0), NodeId(4)),
            HasLinkResult::Link(false)
        );
        wot.add_link(NodeId(0), NodeId(4));
        assert_eq!(
            wot.has_link(NodeId(0), NodeId(4)),
            HasLinkResult::Link(true)
        );
        wot.rem_link(NodeId(0), NodeId(1));
        wot.rem_link(NodeId(0), NodeId(2));
        wot.rem_link(NodeId(0), NodeId(3));
        wot.rem_link(NodeId(0), NodeId(4));

        // false when not linked + test out of bounds
        assert_eq!(
            wot.has_link(NodeId(0), NodeId(6)),
            HasLinkResult::Link(false)
        );
        assert_eq!(
            wot.has_link(NodeId(23), NodeId(0)),
            HasLinkResult::UnknownSource()
        );
        assert_eq!(
            wot.has_link(NodeId(2), NodeId(53)),
            HasLinkResult::UnknownTarget()
        );

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
        assert_eq!(
            wot.has_link(NodeId(2), NodeId(0)),
            HasLinkResult::Link(false)
        );

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

        assert_eq!(
            wot.has_link(NodeId(2), NodeId(0)),
            HasLinkResult::Link(true)
        );
        assert_eq!(
            wot.has_link(NodeId(4), NodeId(0)),
            HasLinkResult::Link(true)
        );
        assert_eq!(
            wot.has_link(NodeId(5), NodeId(0)),
            HasLinkResult::Link(true)
        );
        assert_eq!(
            wot.has_link(NodeId(2), NodeId(1)),
            HasLinkResult::Link(false)
        );

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
        assert_eq!(
            wot.has_link(NodeId(2), NodeId(0)),
            HasLinkResult::Link(true)
        );
        assert_eq!(
            wot.has_link(NodeId(4), NodeId(0)),
            HasLinkResult::Link(false)
        );
        assert_eq!(
            wot.has_link(NodeId(5), NodeId(0)),
            HasLinkResult::Link(true)
        );
        assert_eq!(
            wot.has_link(NodeId(2), NodeId(1)),
            HasLinkResult::Link(false)
        );

        // should successfully use distance rule
        assert_eq!(
            distance_calculator.is_outdistanced(
                &wot,
                WotDistanceParameters {
                    node: NodeId(0),
                    sentry_requirement: 1,
                    step_max: 1,
                    x_percent: 1.0,
                },
            ),
            Some(false)
        );
        // => no because 2,4,5 have certified him
        assert_eq!(
            distance_calculator.is_outdistanced(
                &wot,
                WotDistanceParameters {
                    node: NodeId(0),
                    sentry_requirement: 2,
                    step_max: 1,
                    x_percent: 1.0,
                },
            ),
            Some(false)
        );
        // => no because only member 2 has 2 certs, and has certified him
        assert_eq!(
            distance_calculator.is_outdistanced(
                &wot,
                WotDistanceParameters {
                    node: NodeId(0),
                    sentry_requirement: 3,
                    step_max: 1,
                    x_percent: 1.0,
                },
            ),
            Some(false)
        );
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
        assert_eq!(
            path_finder.find_paths(&wot, NodeId(3), NodeId(0), 1).len(),
            0
        ); // KO
        assert_eq!(
            path_finder.find_paths(&wot, NodeId(3), NodeId(0), 2).len(),
            1
        ); // It exists 3 -> 2 -> 0
        assert!(
            path_finder
                .find_paths(&wot, NodeId(3), NodeId(0), 2)
                .contains(&vec![NodeId(3), NodeId(2), NodeId(0)])
        );

        assert_eq!(
            distance_calculator.is_outdistanced(
                &wot,
                WotDistanceParameters {
                    node: NodeId(0),
                    sentry_requirement: 1,
                    step_max: 1,
                    x_percent: 1.0,
                },
            ),
            Some(false)
        ); // OK : 2 -> 0
        assert_eq!(
            distance_calculator.is_outdistanced(
                &wot,
                WotDistanceParameters {
                    node: NodeId(0),
                    sentry_requirement: 2,
                    step_max: 1,
                    x_percent: 1.0,
                },
            ),
            Some(false)
        ); // OK : 2 -> 0
        assert_eq!(
            distance_calculator.is_outdistanced(
                &wot,
                WotDistanceParameters {
                    node: NodeId(0),
                    sentry_requirement: 3,
                    step_max: 1,
                    x_percent: 1.0,
                },
            ),
            Some(false)
        ); // OK : no stry \w 3 lnk
        assert_eq!(
            distance_calculator.is_outdistanced(
                &wot,
                WotDistanceParameters {
                    node: NodeId(0),
                    sentry_requirement: 2,
                    step_max: 2,
                    x_percent: 1.0,
                },
            ),
            Some(false)
        ); // OK : 2 -> 0

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
        assert_eq!(
            path_finder.find_paths(&wot, NodeId(3), NodeId(0), 1).len(),
            0
        ); // KO
        assert_eq!(
            path_finder.find_paths(&wot, NodeId(3), NodeId(0), 2).len(),
            1
        ); // It exists 3 -> 2 -> 0
        assert!(
            path_finder
                .find_paths(&wot, NodeId(3), NodeId(0), 2)
                .contains(&vec![NodeId(3), NodeId(2), NodeId(0)])
        );

        assert_eq!(
            distance_calculator.is_outdistanced(
                &wot,
                WotDistanceParameters {
                    node: NodeId(0),
                    sentry_requirement: 1,
                    step_max: 1,
                    x_percent: 1.0,
                },
            ),
            Some(true)
        ); // KO : No path 3 -> 0
        assert_eq!(
            distance_calculator.is_outdistanced(
                &wot,
                WotDistanceParameters {
                    node: NodeId(0),
                    sentry_requirement: 2,
                    step_max: 1,
                    x_percent: 1.0,
                },
            ),
            Some(true)
        ); // KO : No path 3 -> 0
        assert_eq!(
            distance_calculator.is_outdistanced(
                &wot,
                WotDistanceParameters {
                    node: NodeId(0),
                    sentry_requirement: 3,
                    step_max: 1,
                    x_percent: 1.0,
                },
            ),
            Some(false)
        ); // OK : no stry \w 3 lnk
        assert_eq!(
            distance_calculator.is_outdistanced(
                &wot,
                WotDistanceParameters {
                    node: NodeId(0),
                    sentry_requirement: 2,
                    step_max: 2,
                    x_percent: 1.0,
                },
            ),
            Some(false)
        ); // OK : 3 -> 2 -> 0

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
        assert_eq!(
            distance_calculator.is_outdistanced(
                &wot,
                WotDistanceParameters {
                    node: NodeId(0),
                    sentry_requirement: 2,
                    step_max: 1,
                    x_percent: 1.0,
                },
            ),
            Some(false)
        ); // OK : Disabled

        let file_formater = BinaryFileFormater {};

        // Write wot in file
        assert_eq!(
            file_formater
                .to_file(
                    &wot,
                    &[0b0000_0000, 0b0000_0001, 0b0000_0001, 0b0000_0000],
                    "test.wot"
                )
                .unwrap(),
            ()
        );

        let (wot2, blockstamp2) = file_formater.from_file::<W>("test.wot", 3).unwrap();

        // Read wot from file
        {
            assert_eq!(
                blockstamp2,
                vec![0b0000_0000, 0b0000_0001, 0b0000_0001, 0b0000_0000]
            );
            assert_eq!(wot.size(), wot2.size());
            assert_eq!(
                wot.get_non_sentries(1).len(),
                wot2.get_non_sentries(1).len()
            );
            assert_eq!(wot.get_disabled().len(), wot2.get_disabled().len());
            assert_eq!(wot2.get_disabled().len(), 1);
            assert_eq!(wot2.is_enabled(NodeId(3)), Some(false));
            assert_eq!(
                distance_calculator.is_outdistanced(
                    &wot2,
                    WotDistanceParameters {
                        node: NodeId(0),
                        sentry_requirement: 2,
                        step_max: 1,
                        x_percent: 1.0,
                    },
                ),
                Some(false)
            );
        }

        // Read g1_genesis wot
        let (wot3, blockstamp3) = file_formater
            .from_file::<W>("tests/g1_genesis.bin", 100)
            .unwrap();
        assert_eq!(
            blockstamp3,
            vec![
                57, 57, 45, 48, 48, 48, 48, 49, 50, 65, 68, 52, 57, 54, 69, 67, 65, 53, 54, 68, 69,
                48, 66, 56, 69, 53, 68, 54, 70, 55, 52, 57, 66, 55, 67, 66, 69, 55, 56, 53, 53, 51,
                69, 54, 51, 56, 53, 51, 51, 51, 65, 52, 52, 69, 48, 52, 51, 55, 55, 69, 70, 70, 67,
                67, 65, 53, 51,
            ]
        );

        // Check g1_genesis wot members_count
        let members_count = wot3.get_enabled().len() as u64;
        assert_eq!(members_count, 59);

        // Test compute_distance in g1_genesis wot
        assert_eq!(
            distance_calculator.compute_distance(
                &wot3,
                WotDistanceParameters {
                    node: NodeId(37),
                    sentry_requirement: 3,
                    step_max: 5,
                    x_percent: 0.8,
                },
            ),
            Some(WotDistance {
                sentries: 48,
                success: 48,
                success_at_border: 3,
                reached: 51,
                reached_at_border: 3,
                outdistanced: false,
            },)
        );

        // Test betweenness centralities computation in g1_genesis wot
        let centralities = centralities_calculator.betweenness_centralities(&wot3);
        assert_eq!(centralities.len(), 59);
        assert_eq!(
            centralities,
            vec![
                148, 30, 184, 11, 60, 51, 40, 115, 24, 140, 47, 69, 16, 34, 94, 126, 151, 0, 34,
                133, 20, 103, 38, 144, 73, 523, 124, 23, 47, 17, 9, 64, 77, 281, 6, 105, 54, 0,
                111, 21, 6, 2, 0, 1, 47, 59, 28, 236, 0, 0, 0, 0, 60, 6, 0, 1, 8, 33, 169,
            ]
        );

        // Test stress centralities computation in g1_genesis wot
        let stress_centralities = centralities_calculator.stress_centralities(&wot3);
        assert_eq!(stress_centralities.len(), 59);
        assert_eq!(
            stress_centralities,
            vec![
                848, 240, 955, 80, 416, 203, 290, 645, 166, 908, 313, 231, 101, 202, 487, 769, 984,
                0, 154, 534, 105, 697, 260, 700, 496, 1726, 711, 160, 217, 192, 89, 430, 636, 1276,
                41, 420, 310, 0, 357, 125, 50, 15, 0, 12, 275, 170, 215, 1199, 0, 0, 0, 0, 201, 31,
                0, 9, 55, 216, 865,
            ]
        );
        /*let wot_size = wot3.size();
        let members_count = wot3.get_enabled().len() as u64;
        assert_eq!(members_count, 59);
        let oriented_couples_count: u64 = members_count * (members_count - 1);
        let mut centralities = vec![0; wot_size];
        for i in 0..wot_size {
            for j in 0..wot_size {
                let paths = path_finder.find_paths(&wot3, NodeId(i), NodeId(j), 5);
                let mut intermediate_members: Vec<NodeId> = Vec::new();
                for path in paths {
                    if path.len() > 2 {
                        for node_id in &path[1..path.len() - 1] {
                            if !intermediate_members.contains(node_id) {
                                intermediate_members.push(*node_id);
                            }
                        }
                    }
                }
                let centralities_copy = centralities.clone();
                for node_id in intermediate_members {
                    let centrality = &centralities_copy[node_id.0];
                    if let Some(tmp) = centralities.get_mut(node_id.0) {
                        *tmp = *centrality + 1;
                    }
                }
            }
        }
        let mut relative_centralities = Vec::with_capacity(wot_size);
        for centrality in centralities {
            relative_centralities.push((centrality * 100_000 / oriented_couples_count) as usize);
        }
        assert_eq!(relative_centralities.len(), 59);*/
    }
}
