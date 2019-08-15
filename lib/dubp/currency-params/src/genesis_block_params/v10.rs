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

//! Duniter protocol currency parameters in genesis block v10

use crate::genesis_block_params::ParseParamsError;
use serde::{Deserialize, Serialize};

/// Currency parameters
#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq)]
pub struct BlockV10Parameters {
    /// UD target growth rate (see Relative Theorie of Money)
    pub c: f64,
    /// Duration between the creation of two UD (in seconds)
    pub dt: u64,
    /// Amount of the initial UD
    pub ud0: usize,
    /// Minimum duration between the writing of 2 certifications from the same issuer (in seconds)
    pub sig_period: u64,
    /// Maximum number of active certifications at the same time (for the same issuer)
    pub sig_stock: usize,
    /// Maximum retention period of a pending certification
    pub sig_window: u64,
    /// Time to expiry of written certification
    pub sig_validity: u64,
    /// Minimum number of certifications required to become a member
    pub sig_qty: usize,
    /// Maximum retention period of a pending identity
    pub idty_window: u64,
    /// Maximum retention period of a pending membership
    pub ms_window: u64,
    /// Percentage of referring members who must be within step_max steps of each member
    pub x_percent: f64,
    /// Time to expiry of written membership
    pub ms_validity: u64,
    /// For a member to respect the distance rule,
    /// there must exist for more than x_percent % of the referring members
    /// a path of less than step_max steps from the referring member to the evaluated member.
    pub step_max: usize,
    /// Number of blocks used for calculating median time.
    pub median_time_blocks: usize,
    /// The average time for writing 1 block (wished time)
    pub avg_gen_time: u64,
    /// The number of blocks required to evaluate again PoWMin value
    pub dt_diff_eval: usize,
    /// The percent of previous issuers to reach for personalized difficulty
    pub percent_rot: f64,
    /// Time of first UD.
    pub ud_time0: u64,
    /// Time of first reevaluation of the UD.
    pub ud_reeval_time0: u64,
    /// Time period between two re-evaluation of the UD.
    pub dt_reeval: u64,
}

impl Default for BlockV10Parameters {
    fn default() -> BlockV10Parameters {
        BlockV10Parameters {
            c: 0.0488,
            dt: 86_400,
            ud0: 1_000,
            sig_period: 432_000,
            sig_stock: 100,
            sig_window: 5_259_600,
            sig_validity: 63_115_200,
            sig_qty: 5,
            idty_window: 5_259_600,
            ms_window: 5_259_600,
            x_percent: 0.8,
            ms_validity: 31_557_600,
            step_max: 5,
            median_time_blocks: 24,
            avg_gen_time: 300,
            dt_diff_eval: 12,
            percent_rot: 0.67,
            ud_time0: 1_488_970_800,
            ud_reeval_time0: 1_490_094_000,
            dt_reeval: 15_778_800,
        }
    }
}

impl ::std::str::FromStr for BlockV10Parameters {
    type Err = ParseParamsError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let params: Vec<&str> = source.split(':').collect();
        Ok(BlockV10Parameters {
            c: params[0].parse()?,
            dt: params[1].parse()?,
            ud0: params[2].parse()?,
            sig_period: params[3].parse()?,
            sig_stock: params[4].parse()?,
            sig_window: params[5].parse()?,
            sig_validity: params[6].parse()?,
            sig_qty: params[7].parse()?,
            idty_window: params[8].parse()?,
            ms_window: params[9].parse()?,
            x_percent: params[10].parse()?,
            ms_validity: params[11].parse()?,
            step_max: params[12].parse()?,
            median_time_blocks: params[13].parse()?,
            avg_gen_time: params[14].parse()?,
            dt_diff_eval: params[15].parse()?,
            percent_rot: params[16].parse()?,
            ud_time0: params[17].parse()?,
            ud_reeval_time0: params[18].parse()?,
            dt_reeval: params[19].parse()?,
        })
    }
}

impl ToString for BlockV10Parameters {
    fn to_string(&self) -> String {
        format!(
            "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
            self.c,
            self.dt,
            self.ud0,
            self.sig_period,
            self.sig_stock,
            self.sig_window,
            self.sig_validity,
            self.sig_qty,
            self.idty_window,
            self.ms_window,
            self.x_percent,
            self.ms_validity,
            self.step_max,
            self.median_time_blocks,
            self.avg_gen_time,
            self.dt_diff_eval,
            self.percent_rot,
            self.ud_time0,
            self.ud_reeval_time0,
            self.dt_reeval,
        )
    }
}

impl Eq for BlockV10Parameters {}
