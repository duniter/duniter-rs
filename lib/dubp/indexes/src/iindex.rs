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

//! Provides the definition of the identity index (IINDEX) described in the DUBP RFC.

pub mod v11;

use durs_common_tools::fns::arrays::copy_into_array;
use std::convert::AsMut;
use std::fmt::{Debug, Error, Formatter};
use std::iter::Iterator;
use std::str::FromStr;

const USERNAME_MAX_LEN: usize = 100;

#[derive(Copy, Clone)]
/// Identity username
pub struct Username {
    chars: UsernameChars,
    real_len: usize,
}

impl Debug for Username {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{:?}", &self.chars.0[..self.real_len])
    }
}

#[derive(Copy, Clone)]
struct UsernameChars([char; USERNAME_MAX_LEN]);

impl AsMut<[char]> for UsernameChars {
    fn as_mut(&mut self) -> &mut [char] {
        &mut self.0[..]
    }
}

impl Default for UsernameChars {
    fn default() -> Self {
        UsernameChars([
            ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
            ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
            ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
            ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
            ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
            ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
        ])
    }
}

impl ToString for Username {
    fn to_string(&self) -> String {
        self.chars.0[..self.real_len].iter().collect()
    }
}

/// Error when parsing username
#[derive(Clone, Copy, Debug)]
pub enum ParseUsernameErr {
    /// username too long
    UsernameTooLong,
}

impl FromStr for Username {
    type Err = ParseUsernameErr;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        if source.len() > USERNAME_MAX_LEN {
            return Err(ParseUsernameErr::UsernameTooLong);
        }

        let mut chars: Vec<char> = source.chars().collect();
        let real_len = chars.len();
        if real_len < USERNAME_MAX_LEN {
            let mut whitespaces: Vec<char> =
                (0..USERNAME_MAX_LEN - real_len).map(|_| ' ').collect();
            chars.append(&mut whitespaces);
        }

        Ok(Username {
            chars: copy_into_array(&chars[..]),
            real_len,
        })
    }
}
