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

//! Provide a trait and implementations to compute distances.

use std::collections::HashSet;
use rayon::prelude::*;
use data::WebOfTrust;
use data::NodeId;

/// Paramters for `WoT` distance calculations
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
    /// Succes at border count
    pub success_at_border: u32,
    /// Reached count
    pub reached: u32,
    /// Reached at border count
    pub reached_at_border: u32,
    /// Is the node outdistanced ?
    pub outdistanced: bool,
}

/// Compute distance between nodes of a `WebOfTrust`.
pub trait DistanceCalculator<T: WebOfTrust> {
    /// Compute distance between a node and the network.
    /// Returns `None` if this node doesn't exist.
    fn compute_distance(&self, wot: &T, params: WotDistanceParameters) -> Option<WotDistance>;

    /// Test if a node is outdistanced in the network.
    /// Returns `Node` if this node doesn't exist.
    fn is_outdistanced(&self, wot: &T, params: WotDistanceParameters) -> Option<bool>;
}

/// Calculate distances between 2 members in a `WebOfTrust`.
#[derive(Debug, Clone, Copy)]
pub struct RustyDistanceCalculator;

impl<T: WebOfTrust + Sync> DistanceCalculator<T> for RustyDistanceCalculator {
    fn compute_distance(&self, wot: &T, params: WotDistanceParameters) -> Option<WotDistance> {
        let WotDistanceParameters {
            node,
            sentry_requirement,
            step_max,
            x_percent,
        } = params;

        if node.0 >= wot.size() {
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
                    wot.get_links_source(id)
                        .unwrap()
                        .iter()
                        .filter(|source| !area.contains(source))
                        .cloned()
                        .collect::<HashSet<_>>()
                })
                .reduce(HashSet::new, |mut acc, sources| {
                    for source in sources {
                        acc.insert(source);
                    }
                    acc
                });
            area.extend(border.iter());
        }

        let sentries: Vec<_> = wot.get_sentries(sentry_requirement as usize);
        let mut success = area.iter().filter(|n| sentries.contains(n)).count() as u32;
        let success_at_border = border.iter().filter(|n| sentries.contains(n)).count() as u32;
        let mut sentries = sentries.len() as u32;
        if wot.is_sentry(node, sentry_requirement as usize).unwrap() {
            sentries -= 1;
            success -= 1;
        }

        Some(WotDistance {
            sentries,
            reached: area.len() as u32,
            reached_at_border: border.len() as u32,
            success,
            success_at_border,
            outdistanced: f64::from(success) < x_percent * f64::from(sentries),
        })
    }

    fn is_outdistanced(&self, wot: &T, params: WotDistanceParameters) -> Option<bool> {
        Self::compute_distance(&self, wot, params).map(|result| result.outdistanced)
    }
}
