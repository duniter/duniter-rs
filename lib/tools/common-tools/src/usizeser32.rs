//  Copyright (C) 2019  Éloïs SANCHEZ
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

//! Common rust tools for DURS project.

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::{Serialize, Serializer};
use shrinkwraprs::Shrinkwrap;
use std::fmt::{self, Debug, Display, Error, Formatter};

/// Wrapper for a usize value serialized in u32.
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Shrinkwrap)]
#[shrinkwrap]
pub struct UsizeSer32(pub usize);

impl Display for UsizeSer32 {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

impl From<UsizeSer32> for i32 {
    fn from(value: UsizeSer32) -> Self {
        value.0 as Self
    }
}

impl From<UsizeSer32> for u32 {
    fn from(value: UsizeSer32) -> Self {
        value.0 as Self
    }
}

impl From<UsizeSer32> for u64 {
    fn from(value: UsizeSer32) -> Self {
        value.0 as Self
    }
}

impl From<UsizeSer32> for usize {
    fn from(value: UsizeSer32) -> Self {
        value.0
    }
}

impl Serialize for UsizeSer32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.0 as u32)
    }
}

struct UsizeSer32Visitor;

impl<'de> Visitor<'de> for UsizeSer32Visitor {
    type Value = UsizeSer32;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an unsigned integer between 0 and 2^32-1")
    }

    fn visit_u8<E>(self, value: u8) -> Result<UsizeSer32, E>
    where
        E: de::Error,
    {
        Ok(UsizeSer32(value as usize))
    }

    fn visit_u32<E>(self, value: u32) -> Result<UsizeSer32, E>
    where
        E: de::Error,
    {
        Ok(UsizeSer32(value as usize))
    }

    fn visit_u64<E>(self, value: u64) -> Result<UsizeSer32, E>
    where
        E: de::Error,
    {
        use std::usize;
        if value >= usize::MIN as u64 && value <= usize::MAX as u64 {
            Ok(UsizeSer32(value as usize))
        } else {
            Err(E::custom(format!("u32 out of range: {}", value)))
        }
    }
}

impl<'de> Deserialize<'de> for UsizeSer32 {
    fn deserialize<D>(deserializer: D) -> Result<UsizeSer32, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u32(UsizeSer32Visitor)
    }
}
