//  Copyright (C) 2018  The Duniter Project Developers.
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

//! Defined all aspects of the inter-node network that concern all modules and are therefore independent of one implementation or another of this network layer.

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;
use std::mem;

/// Read u16 in big endian
pub fn read_u16_be(datas: &[u8]) -> Result<u16, ::std::io::Error> {
    let mut bytes = Cursor::new(datas.to_vec());
    Ok(bytes.read_u16::<BigEndian>()?)
}

/// Read u16 in little endian
pub fn read_u16_le(datas: &[u8]) -> Result<u16, ::std::io::Error> {
    let mut bytes = Cursor::new(datas.to_vec());
    Ok(bytes.read_u16::<LittleEndian>()?)
}

/// Write u16 in big endian
pub fn write_u16_be(buffer: &mut Vec<u8>, number: u16) -> Result<(), ::std::io::Error> {
    let mut buffer2 = [0u8; mem::size_of::<u16>()];
    buffer2.as_mut().write_u16::<BigEndian>(number)?;
    buffer.extend_from_slice(&buffer2);
    Ok(())
}

/// Write u16 in little endian
pub fn write_u16_le(buffer: &mut Vec<u8>, number: u16) -> Result<(), ::std::io::Error> {
    let mut buffer2 = [0u8; mem::size_of::<u16>()];
    buffer2.as_mut().write_u16::<LittleEndian>(number)?;
    buffer.extend_from_slice(&buffer2);
    Ok(())
}
