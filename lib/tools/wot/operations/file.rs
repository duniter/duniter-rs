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

//! Provide a trait and implementation to read and write `WebOfTrust` to disk.

use crate::data::NodeId;
use durs_common_tools::fatal_error;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::data::WebOfTrust;

/// Results of `WebOfTrust` parsing from binary file.
#[derive(Debug)]
pub enum WotParseError {
    /// FailToOpenFile
    FailToOpenFile(io::Error),

    /// IOError
    IOError(io::Error),
}

impl From<io::Error> for WotParseError {
    fn from(e: io::Error) -> WotParseError {
        WotParseError::IOError(e)
    }
}

/// Results of `WebOfTrust` writing to binary file.
#[derive(Debug)]
pub enum WotWriteError {
    /// WrongWotSize
    WrongWotSize(),

    /// FailToCreateFile
    FailToCreateFile(io::Error),

    /// FailToWriteInFile
    FailToWriteInFile(io::Error),
}

impl From<io::Error> for WotWriteError {
    fn from(e: io::Error) -> WotWriteError {
        WotWriteError::FailToWriteInFile(e)
    }
}

/// Provide Read/Write functions for `WebOfTrust` objects.
pub trait FileFormater {
    /// Try to read a `WebOfTrust` from a file.
    fn from_file<T: WebOfTrust>(
        &self,
        path: &str,
        max_links: usize,
    ) -> Result<(T, Vec<u8>), WotParseError>;

    /// Tru to write a `WebOfTrust` in a file.
    fn to_file<T: WebOfTrust>(&self, wot: &T, data: &[u8], path: &str)
        -> Result<(), WotWriteError>;
}

/// Read and write WebOfTrust in a binary format.
#[derive(Debug, Clone, Copy)]
pub struct BinaryFileFormater;

impl FileFormater for BinaryFileFormater {
    /// Try to read a `WebOfTrust` from a file.
    fn from_file<T: WebOfTrust>(
        &self,
        path: &str,
        max_links: usize,
    ) -> Result<(T, Vec<u8>), WotParseError> {
        let mut wot = T::new(max_links);

        let file_size = fs::metadata(path).expect("fail to read wot file !").len();
        let mut file_pointing_to_blockstamp_size: Vec<u8> = vec![0; file_size as usize];
        match File::open(path) {
            Ok(mut file) => {
                file.read_exact(&mut file_pointing_to_blockstamp_size.as_mut_slice())?;
            }
            Err(e) => return Err(WotParseError::FailToOpenFile(e)),
        };
        // Read up to 4 bytes (blockstamp_size)
        let mut file_pointing_to_blockstamp = file_pointing_to_blockstamp_size.split_off(4);
        // Get blockstamp size
        let mut buf = &file_pointing_to_blockstamp_size[..];
        let blockstamp_size = buf.read_u32::<BigEndian>().unwrap();
        // Read up to blockstamp_size bytes (blockstamp)
        let mut file_pointing_to_nodes_count =
            file_pointing_to_blockstamp.split_off(blockstamp_size as usize);
        // Read up to 4 bytes (nodes_count)
        let mut file_pointing_to_nodes_states = file_pointing_to_nodes_count.split_off(4);
        // Read nodes_count
        let mut buf = &file_pointing_to_nodes_count[..];
        let nodes_count = buf.read_u32::<BigEndian>().unwrap();
        // Calcule nodes_state size
        let nodes_states_size = match nodes_count % 8 {
            0 => nodes_count / 8,
            _ => (nodes_count / 8) + 1,
        };
        // Read up to nodes_states_size bytes (nodes_states)
        let file_pointing_to_links =
            file_pointing_to_nodes_states.split_off(nodes_states_size as usize);
        let count_total_bytes_read = file_pointing_to_links.len()
            + nodes_states_size as usize
            + 4
            + blockstamp_size as usize
            + 4;
        if count_total_bytes_read != file_size as usize {
            fatal_error!("not read all wot file !");
        }
        // Apply nodes state
        let mut count_remaining_nodes = nodes_count;
        for byte in file_pointing_to_nodes_states {
            let mut byte_integer = u8::from_be(byte);
            let mut factor: u8 = 128;
            for _i in 0..8 {
                if count_remaining_nodes > 0 {
                    wot.add_node();
                    if byte_integer >= factor {
                        byte_integer -= factor;
                    } else {
                        let _test = wot.set_enabled(
                            NodeId((nodes_count - count_remaining_nodes) as usize),
                            false,
                        );
                    }
                    count_remaining_nodes -= 1;
                }
                factor /= 2;
            }
        }
        // Apply links
        let mut buffer_3b: Vec<u8> = Vec::with_capacity(3);
        let mut count_bytes = 0;
        let mut remaining_links: u8 = 0;
        let mut target: u32 = 0;
        for byte in file_pointing_to_links {
            if remaining_links == 0 {
                target += 1;
                remaining_links = u8::from_be(byte);
                count_bytes = 0;
            } else {
                buffer_3b.push(byte);
                if count_bytes % 3 == 2 {
                    let mut buf = &buffer_3b.clone()[..];
                    let source = buf.read_u24::<BigEndian>().expect("fail to parse source");
                    wot.add_link(NodeId(source as usize), NodeId((target - 1) as usize));
                    remaining_links -= 1;
                    buffer_3b.clear();
                }
                count_bytes += 1;
            }
        }
        if count_bytes % 3 != 0 {
            fatal_error!("not read all wot file !");
        }
        Ok((wot, file_pointing_to_blockstamp))
    }

    /// Try to write a `WebOfTrust` in a file.
    fn to_file<T: WebOfTrust>(
        &self,
        wot: &T,
        data: &[u8],
        path: &str,
    ) -> Result<(), WotWriteError> {
        let mut buffer: Vec<u8> = Vec::new();
        // Write blockstamp size
        let blockstamp_size = data.len() as u32;
        let mut bytes: Vec<u8> = Vec::with_capacity(4);
        bytes.write_u32::<BigEndian>(blockstamp_size).unwrap();
        buffer.append(&mut bytes);
        // Write blockstamp
        buffer.append(&mut data.to_vec());
        // Write nodes_count
        let nodes_count = wot.size() as u32;
        let mut bytes: Vec<u8> = Vec::with_capacity(4);
        bytes.write_u32::<BigEndian>(nodes_count).unwrap();
        buffer.append(&mut bytes);
        // Write enable state by groups of 8
        let mut enable_states: u8 = 0;
        let mut factor: u8 = 128;
        for n in 0..nodes_count {
            match wot.is_enabled(NodeId(n as usize)) {
                Some(enable) => {
                    if enable {
                        enable_states += factor;
                    }
                }
                None => {
                    return Err(WotWriteError::WrongWotSize());
                }
            }
            if n % 8 == 7 {
                factor = 128;
                let mut tmp_buf = Vec::with_capacity(1);
                tmp_buf.write_u8(enable_states).unwrap();
                buffer.append(&mut tmp_buf);
                enable_states = 0;
            } else {
                factor /= 2;
            }
        }
        // nodes_states padding
        if nodes_count % 8 != 7 {
            let mut tmp_buf = Vec::with_capacity(1);
            tmp_buf.write_u8(enable_states).unwrap();
            buffer.append(&mut tmp_buf);
        }
        // Write links
        for n in 0..nodes_count {
            if let Some(sources) = wot.get_links_source(NodeId(n as usize)) {
                // Write sources_counts
                let mut bytes = Vec::with_capacity(1);
                bytes.write_u8(sources.len() as u8).unwrap();
                buffer.append(&mut bytes);
                for source in &sources {
                    // Write source
                    let mut bytes: Vec<u8> = Vec::with_capacity(3);
                    bytes.write_u24::<BigEndian>(source.0 as u32).unwrap();
                    buffer.append(&mut bytes);
                }
            };
        }
        // Create or open file
        let mut file = match File::create(path) {
            Ok(file) => file,
            Err(e) => return Err(WotWriteError::FailToCreateFile(e)),
        };
        // Write buffer in file
        file.write_all(&buffer)?;

        Ok(())
    }
}
