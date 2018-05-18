extern crate duniter_wotb;

use duniter_wotb::operations::centrality::{
    CentralitiesCalculator, UlrikBrandesCentralityCalculator,
};
use duniter_wotb::operations::distance::{
    DistanceCalculator, RustyDistanceCalculator, WotDistance, WotDistanceParameters,
};
use duniter_wotb::operations::path::{PathFinder, RustyPathFinder};
use duniter_wotb::{NodeId, WebOfTrust};

pub static CENTRALITY_CALCULATOR: UlrikBrandesCentralityCalculator =
    UlrikBrandesCentralityCalculator {};
pub static DISTANCE_CALCULATOR: RustyDistanceCalculator = RustyDistanceCalculator {};
pub static PATH_FINDER: RustyPathFinder = RustyPathFinder {};

pub fn get_sentry_requirement(members_count: usize, step_max: u32) -> u32 {
    match step_max {
        5 => {
            if members_count < 33 {
                2
            } else if members_count < 244 {
                3
            } else if members_count < 1025 {
                4
            } else if members_count < 3126 {
                5
            } else if members_count < 7777 {
                6
            } else {
                panic!("get_sentry_requirement not define for members_count greater than 7777 !");
            }
        }
        _ => panic!("get_sentry_requirement not define for step_max != 5 !"),
    }
}

pub fn calculate_average_density<T: WebOfTrust>(wot: &T) -> usize {
    let enabled_members = wot.get_enabled();
    let enabled_members_count = enabled_members.len();
    let mut count_actives_links: usize = 0;
    for member in &enabled_members {
        count_actives_links += wot.issued_count(*member).unwrap();
    }
    ((count_actives_links as f32 / enabled_members_count as f32) * 1_000.0) as usize
}

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
                    node: NodeId(i),
                    sentry_requirement,
                    step_max,
                    x_percent,
                },
            )
            .expect("Fatal Error: compute_distance return None !");
        let mut distance = ((f64::from(distance_datas.success)
            / (x_percent * f64::from(distance_datas.sentries))) * 100.0)
            as usize;
        distances.push(distance);
        average_distance += distance;
        let mut connectivity =
            ((f64::from(distance_datas.success - distance_datas.success_at_border)
                / (x_percent * f64::from(distance_datas.sentries))) * 100.0) as usize;
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

pub fn calculate_distance_stress_centralities<T: WebOfTrust>(wot: &T, step_max: u32) -> Vec<u64> {
    CENTRALITY_CALCULATOR.distance_stress_centralities(wot, step_max as usize)
}

pub fn calculate_centralities_degree<T: WebOfTrust>(wot: &T, step_max: u32) -> Vec<usize> {
    let wot_size = wot.size();
    let members_count = wot.get_enabled().len() as u64;
    let oriented_couples_count: u64 = members_count * (members_count - 1);
    let mut centralities: Vec<u64> = vec![0; wot_size];
    for i in 0..wot_size {
        for j in 0..wot_size {
            let mut paths = PATH_FINDER.find_paths(wot, NodeId(i), NodeId(j), step_max);
            if paths.is_empty() {
                break;
            }
            //paths.sort_unstable_by(|a, b| a.len().cmp(&b.len()));
            let shortest_path_len = paths[0].len();
            let mut intermediate_members: Vec<NodeId> = Vec::new();
            if shortest_path_len > 2 {
                for path in paths {
                    //if path.len() == shortest_path_len {
                    for node_id in &path {
                        if !intermediate_members.contains(node_id) {
                            intermediate_members.push(*node_id);
                        }
                    }
                    /*} else {
                        break;
                    }*/
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
    relative_centralities
}
