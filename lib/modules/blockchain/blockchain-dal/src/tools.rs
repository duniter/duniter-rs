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

use crate::entities::block::DALBlock;
use dubp_block_doc::block::BlockDocumentTrait;
use dup_crypto::keys::PubKey;
use durs_common_tools::fatal_error;
use durs_wot::operations::centrality::{CentralitiesCalculator, UlrikBrandesCentralityCalculator};
use durs_wot::operations::distance::{
    DistanceCalculator, RustyDistanceCalculator, WotDistance, WotDistanceParameters,
};
use durs_wot::{WebOfTrust, WotId};
use std::collections::HashMap;

/// CENTRALITY_CALCULATOR
pub static CENTRALITY_CALCULATOR: UlrikBrandesCentralityCalculator =
    UlrikBrandesCentralityCalculator {};

/// DISTANCE_CALCULATOR
pub static DISTANCE_CALCULATOR: RustyDistanceCalculator = RustyDistanceCalculator {};

/// Get sentry requirement
pub fn get_sentry_requirement(members_count: usize, step_max: u32) -> u32 {
    match step_max {
        5 => {
            if members_count < 33 {
                2
            } else if members_count < 244 {
                3
            } else if members_count < 1_025 {
                4
            } else if members_count < 3_126 {
                5
            } else if members_count < 7_777 {
                6
            } else if members_count < 16_808 {
                7
            } else if members_count < 32_769 {
                8
            } else if members_count < 59_050 {
                9
            } else if members_count < 100_001 {
                10
            } else if members_count < 161_052 {
                11
            } else if members_count < 248_833 {
                12
            } else if members_count < 371_294 {
                13
            } else if members_count < 537_825 {
                14
            } else if members_count < 759_376 {
                15
            } else if members_count < 1_048_577 {
                16
            } else if members_count < 1_419_858 {
                17
            } else if members_count < 1_889_569 {
                18
            } else {
                fatal_error!(
                    "get_sentry_requirement not define for members_count greater than 1_889_569 !"
                );
            }
        }
        _ => fatal_error!("get_sentry_requirement not define for step_max != 5 !"),
    }
}

/// Compute average density
pub fn calculate_average_density<T: WebOfTrust>(wot: &T) -> usize {
    let enabled_members = wot.get_enabled();
    let enabled_members_count = enabled_members.len();
    let mut count_actives_links: usize = 0;
    for member in &enabled_members {
        count_actives_links += wot
            .issued_count(*member)
            .unwrap_or_else(|| fatal_error!("Fail to get issued_count of wot_id {}", (*member).0));
    }
    ((count_actives_links as f32 / enabled_members_count as f32) * 1_000.0) as usize
}

/// Compute distances
pub fn compute_distances<T: WebOfTrust + Sync>(
    wot: &T,
    sentry_requirement: u32,
    step_max: u32,
    x_percent: f64,
) -> (usize, Vec<usize>, usize, Vec<usize>) {
    let members_count = wot.get_enabled().len();
    let mut distances = Vec::new();
    let mut average_distance: usize = 0;
    let mut connectivities = Vec::new();
    let mut average_connectivity: usize = 0;
    for i in 0..wot.size() {
        let distance_datas: WotDistance = DISTANCE_CALCULATOR
            .compute_distance(
                wot,
                WotDistanceParameters {
                    node: WotId(i),
                    sentry_requirement,
                    step_max,
                    x_percent,
                },
            )
            .expect("Fatal Error: compute_distance return None !");
        let distance = ((f64::from(distance_datas.success)
            / (x_percent * f64::from(distance_datas.sentries)))
            * 100.0) as usize;
        distances.push(distance);
        average_distance += distance;
        let connectivity = ((f64::from(distance_datas.success - distance_datas.success_at_border)
            / (x_percent * f64::from(distance_datas.sentries)))
            * 100.0) as usize;
        connectivities.push(connectivity);
        average_connectivity += connectivity;
    }
    average_distance /= members_count;
    average_connectivity /= members_count;
    (
        average_distance,
        distances,
        average_connectivity,
        connectivities,
    )
}

/// Compute distance stress centralities
pub fn calculate_distance_stress_centralities<T: WebOfTrust>(wot: &T, step_max: u32) -> Vec<u64> {
    CENTRALITY_CALCULATOR.distance_stress_centralities(wot, step_max as usize)
}

/// Compute median issuers frame
pub fn compute_median_issuers_frame<S: std::hash::BuildHasher>(
    current_block: &DALBlock,
    current_frame: &HashMap<PubKey, usize, S>,
) -> usize {
    if !current_frame.is_empty() {
        let mut current_frame_vec: Vec<_> = current_frame.values().cloned().collect();
        current_frame_vec.sort_unstable();

        // Calculate median
        let mut median_index = match current_block.block.issuers_count() % 2 {
            1 => (current_block.block.issuers_count() / 2) + 1,
            _ => current_block.block.issuers_count() / 2,
        };
        if median_index >= current_block.block.issuers_count() {
            median_index = current_block.block.issuers_count() - 1;
        }
        current_frame_vec[median_index]

    /*// Calculate second tiercile index
    let mut second_tiercile_index = match self.block.issuers_count % 3 {
        1 | 2 => (self.block.issuers_count as f64 * (2.0 / 3.0)) as usize + 1,
        _ => (self.block.issuers_count as f64 * (2.0 / 3.0)) as usize,
    };
    if second_tiercile_index >= self.block.issuers_count {
        second_tiercile_index = self.block.issuers_count - 1;
    }
    self.second_tiercile_frame = current_frame_vec[second_tiercile_index];*/
    } else {
        0
    }
}
