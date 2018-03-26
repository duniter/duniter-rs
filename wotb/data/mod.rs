//  Copyright (C) 2017-2018  The Duniter Project Developers.
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

//! Provide data structures to manage web of trusts.
//! `LegacyWebOfTrust` is almost a translation of the legacy C++ coden while
//! `RustyWebOfTrust` is a brand new implementation with a more "rusty" style.

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use std::io::prelude::*;
use std::io;
use std::fs;
use std::fs::File;

pub mod legacy;
pub mod rusty;

/// Wrapper for a node id.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub usize);

/// Results of a certification, with the current certification count
/// of the destination as parameter.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NewLinkResult {
    /// Certification worked.
    Ok(usize),
    /// This certification already exist.
    AlreadyCertified(usize),
    /// All available certifications has been used.
    AllCertificationsUsed(usize),
    /// Unknown source.
    UnknownSource(),
    /// Unknown target.
    UnknownTarget(),
    /// Self linking is forbidden.
    SelfLinkingForbidden(),
}

/// Results of a certification removal, with the current certification count
/// of the destination as parameter.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RemLinkResult {
    /// Certification has been removed.
    Removed(usize),
    /// Requested certification doesn't exist.
    UnknownCert(usize),
    /// Unknown source.
    UnknownSource(),
    /// Unknown target.
    UnknownTarget(),
}

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

/// Results of a certification test.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HasLinkResult {
    /// Both nodes are known, here is the result.
    Link(bool),
    /// Unknown source.
    UnknownSource(),
    /// Unknown target.
    UnknownTarget(),
}

/// Trait for a Web Of Trust.
/// Allow to provide other implementations of the `WoT` logic instead of the legacy C++
/// translated one.
pub trait WebOfTrust {
    /// Create a new Web of Trust with the maximum of links a node can issue.
    fn new(max_links: usize) -> Self;

    /// Get the maximum number of links per user.
    fn get_max_link(&self) -> usize;

    /// Set the maximum number of links per user.
    fn set_max_link(&mut self, max_link: usize);

    /// Add a new node.
    fn add_node(&mut self) -> NodeId;

    /// Remove the last node.
    /// Returns `None` if the WoT was empty, otherwise new top node id.
    fn rem_node(&mut self) -> Option<NodeId>;

    /// Get the size of the WoT.
    fn size(&self) -> usize;

    /// Check if given node is enabled.
    /// Returns `None` if this node doesn't exist.
    fn is_enabled(&self, id: NodeId) -> Option<bool>;

    /// Set the enabled state of given node.
    /// Returns `Null` if this node doesn't exist, `enabled` otherwise.
    fn set_enabled(&mut self, id: NodeId, enabled: bool) -> Option<bool>;

    /// Get enabled node array.
    fn get_enabled(&self) -> Vec<NodeId>;

    /// Get disabled node array.
    fn get_disabled(&self) -> Vec<NodeId>;

    /// Try to add a link from the source to the target.
    fn add_link(&mut self, source: NodeId, target: NodeId) -> NewLinkResult;

    /// Try to remove a link from the source to the target.
    fn rem_link(&mut self, source: NodeId, target: NodeId) -> RemLinkResult;

    /// Test if there is a link from the source to the target.
    fn has_link(&self, source: NodeId, target: NodeId) -> HasLinkResult;

    /// Get the list of links source for this target.
    /// Returns `None` if this node doesn't exist.
    fn get_links_source(&self, target: NodeId) -> Option<Vec<NodeId>>;

    /// Get the number of issued links by a node.
    /// Returns `None` if this node doesn't exist.
    fn issued_count(&self, id: NodeId) -> Option<usize>;

    /// Test if a node is a sentry.
    fn is_sentry(&self, node: NodeId, sentry_requirement: usize) -> Option<bool>;

    /// Get sentries array.
    fn get_sentries(&self, sentry_requirement: usize) -> Vec<NodeId>;

    /// Get non sentries array.
    fn get_non_sentries(&self, sentry_requirement: usize) -> Vec<NodeId>;

    /// Load WebOfTrust from binary file
    fn from_file(&mut self, path: &str) -> Result<Vec<u8>, WotParseError> {
        let file_size = fs::metadata(path).expect("fail to read wotb file !").len();
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
        // Apply nodes state
        let mut count_remaining_nodes = nodes_count;
        for byte in file_pointing_to_nodes_states {
            let mut byte_integer = u8::from_be(byte);
            let mut factor: u8 = 128;
            for _i in 0..8 {
                if count_remaining_nodes > 0 {
                    self.add_node();
                    if byte_integer >= factor {
                        byte_integer -= factor;
                    } else {
                        let _test = self.set_enabled(
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
                    self.add_link(NodeId(source as usize), NodeId((target - 1) as usize));
                    remaining_links -= 1;
                    buffer_3b.clear();
                }
                count_bytes += 1;
            }
        }
        Ok(file_pointing_to_blockstamp)
    }

    /// Write WebOfTrust to binary file
    fn to_file(&self, path: &str, blockstamp: &[u8]) -> Result<(), WotWriteError> {
        let mut buffer: Vec<u8> = Vec::new();
        // Write blockstamp size
        let blockstamp_size = blockstamp.len() as u32;
        let mut bytes: Vec<u8> = Vec::with_capacity(4);
        bytes.write_u32::<BigEndian>(blockstamp_size).unwrap();
        buffer.append(&mut bytes);
        // Write blockstamp
        buffer.append(&mut blockstamp.to_vec());
        // Write nodes_count
        let nodes_count = self.size() as u32;
        let mut bytes: Vec<u8> = Vec::with_capacity(4);
        bytes.write_u32::<BigEndian>(nodes_count).unwrap();
        buffer.append(&mut bytes);
        // Write enable state by groups of 8 (count links at the same time)
        let mut enable_states: u8 = 0;
        let mut factor: u8 = 128;
        for n in 0..nodes_count {
            match self.is_enabled(NodeId(n as usize)) {
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
            if let Some(sources) = self.get_links_source(NodeId(n as usize)) {
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
