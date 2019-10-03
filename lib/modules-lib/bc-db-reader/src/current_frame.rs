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

//! Current frame : Interval of blocks taken for the calculation of the personalized difficulty.

//use crate::constants::*;
use crate::*;
use durs_dbs_tools::DbError;
use durs_wot::WotId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Describe a member in current frame
pub struct MemberInCurrentFrame {
    /// Number of blocks forged by the member in the current frame.
    pub forged_blocks: usize,
    /// Personal difficulty of the member.
    pub difficulty: PersonalDifficulty,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Personal difficulty of a member.
pub struct PersonalDifficulty {
    /// Exclusion factor
    pub exclusion_factor: usize,
    /// handicap
    pub handicap: usize,
}

/// Get current frame datas
pub fn get_current_frame<DB: DbReadable>(
    _db: &DB,
) -> Result<Vec<(WotId, MemberInCurrentFrame)>, DbError> {
    unimplemented!();
}

/// Get the personal difficulty of a member.
/// If the member is not in the current window, returns `pow_min`.
pub fn get_member_diffi<DB: DbReadable, R: DbReader>(
    _db: &DB,
    _r: &R,
    _wot_id: WotId,
) -> Result<PersonalDifficulty, DbError> {
    unimplemented!();
}
