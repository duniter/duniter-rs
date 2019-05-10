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

//! Common rust functions for read/write binary files.

use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;

/// Read bin file
pub fn read_bin_file(file_path: &Path) -> Result<Vec<u8>, std::io::Error> {
    let mut file = File::open(file_path)?;
    if file.metadata()?.len() == 0 {
        Ok(vec![])
    } else {
        let mut bin_datas = Vec::new();
        file.read_to_end(&mut bin_datas)?;

        Ok(bin_datas)
    }
}

/// Write bin file
pub fn write_bin_file(file_path: &Path, datas: &[u8]) -> Result<(), std::io::Error> {
    let mut file = File::create(file_path)?;
    file.write_all(&datas[..])?;

    Ok(())
}
