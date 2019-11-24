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

//! Trait AsResult.

/// Transform any type to Result
pub trait BoolExt {
    /// Transform any type to result<(), E>
    fn or_err<E>(self, err: E) -> Result<(), E>;
    /// Transform any type to result<T, E>
    fn as_result<T, E>(self, ok: T, err: E) -> Result<T, E>;
    /// Reverse bool
    fn not(self) -> bool;
}

impl BoolExt for bool {
    #[inline]
    fn or_err<E>(self, err: E) -> Result<(), E> {
        if self {
            Ok(())
        } else {
            Err(err)
        }
    }
    #[inline]
    fn as_result<T, E>(self, ok: T, err: E) -> Result<T, E> {
        if self {
            Ok(ok)
        } else {
            Err(err)
        }
    }
    #[inline]
    fn not(self) -> bool {
        !self
    }
}
