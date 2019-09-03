//  Copyright (C) 2019  Eloïs SANCHEZ.
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

//! Manage complete Public Key Secure Transport Layer.
//! Sub-module define write operations.

use super::SecureLayer;
use crate::{Error, Result};
use ring::signature::{Ed25519KeyPair, KeyPair};
use std::io::{BufWriter, Write};

#[inline]
pub fn write_connect_msg<W>(
    sl: &mut SecureLayer,
    custom_datas: Option<Vec<u8>>,
    writer: &mut BufWriter<W>,
) -> Result<()>
where
    W: Write,
{
    if let Some(ref sig_key_pair) = sl.sig_key_pair {
        // Create connect message
        let bin_connect_msg = sl.minimal_secure_layer.create_connect_message(
            sig_key_pair.public_key().as_ref(),
            match custom_datas {
                Some(ref d) => Some(&d[..]),
                None => None,
            },
        )?;

        // Write connect message
        writer
            .write(&bin_connect_msg)
            .map_err(|_| Error::BufferFlushError)?;

        // Sign message and write signature
        sign_bin_msg_and_write_sig(sig_key_pair, &bin_connect_msg, writer)
    } else {
        Err(Error::ConnectMsgAlreadyWritten)
    }
}

#[inline]
pub fn write_ack_msg<W>(
    sl: &mut SecureLayer,
    custom_datas: Option<Vec<u8>>,
    writer: &mut BufWriter<W>,
) -> Result<()>
where
    W: Write,
{
    if let Some(ref sig_key_pair) = sl.sig_key_pair {
        // Create ack message
        let bin_connect_msg = sl
            .minimal_secure_layer
            .create_ack_message(match custom_datas {
                Some(ref d) => Some(&d[..]),
                None => None,
            })?;

        // Write ack message
        writer
            .write(&bin_connect_msg)
            .map_err(|_| Error::BufferFlushError)?;

        // Sign message and write signature
        sign_bin_msg_and_write_sig(sig_key_pair, &bin_connect_msg, writer)
    } else {
        Err(Error::ConnectMsgAlreadyWritten)
    }
}

#[inline]
fn sign_bin_msg_and_write_sig<W>(
    sig_key_pair: &Ed25519KeyPair,
    bin_msg: &[u8],
    writer: &mut BufWriter<W>,
) -> Result<()>
where
    W: Write,
{
    writer
        .write(sig_key_pair.sign(bin_msg).as_ref())
        .map(|_| ())
        .map_err(|_| Error::BufferFlushError)
}

#[inline]
pub fn write_bin_message<W>(
    sl: &mut SecureLayer,
    message: &[u8],
    writer: &mut BufWriter<W>,
) -> Result<()>
where
    W: Write,
{
    sl.minimal_secure_layer.write_message(message, writer)
}
