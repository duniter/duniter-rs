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

//! Fatal error macro for DURS project.

/// Interrupts the program and log error message
/// WARNING: this macro must not be called before the logger is initialized !
#[macro_export]
macro_rules! fatal_error {
    ($msg:expr) => ({
        error!("{}", &dbg!($msg));
        panic!($msg);
    });
    ($msg:expr,) => ({
        error!("{}", &dbg!($msg));
        panic!($msg);
    });
    ($fmt:expr, $($arg:tt)+) => ({
        error!("{}", dbg!(format!($fmt, $($arg)+)));
        panic!($fmt, $($arg)+);
    });
}
