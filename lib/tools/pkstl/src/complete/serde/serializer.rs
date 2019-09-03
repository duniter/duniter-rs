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

//! Define PKSTL serializer.

use super::SerdeError;
use crate::format::MessageFormat;
use crate::{Error, Result, SecureLayer};
use serde::Serialize;
use std::io::{BufWriter, Write};

pub(crate) fn write_connect_msg<M, W>(
    sl: &mut SecureLayer,
    custom_datas: Option<&M>,
    writer: &mut BufWriter<W>,
) -> Result<()>
where
    M: Serialize,
    W: Write,
{
    // Serialize and compress custom datas
    let custom_datas = if let Some(custom_datas) = custom_datas {
        let bin_msg = serialize(custom_datas, sl.minimal_secure_layer.config.message_format)?;
        Some(sl.compress(&bin_msg[..])?)
    } else {
        None
    };

    // Write binary message on a writer
    crate::complete::writer::write_connect_msg::<W>(sl, custom_datas, writer)
}

pub(crate) fn write_ack_msg<M, W>(
    sl: &mut SecureLayer,
    custom_datas: Option<&M>,
    writer: &mut BufWriter<W>,
) -> Result<()>
where
    M: Serialize,
    W: Write,
{
    // Serialize and compress custom datas
    let custom_datas = if let Some(custom_datas) = custom_datas {
        let bin_msg = serialize(custom_datas, sl.minimal_secure_layer.config.message_format)?;
        Some(sl.compress(&bin_msg[..])?)
    } else {
        None
    };

    // Write binary message on a writer
    crate::complete::writer::write_ack_msg::<W>(sl, custom_datas, writer)
}

pub(crate) fn write_message<M, W>(
    sl: &mut SecureLayer,
    message: &M,
    writer: &mut BufWriter<W>,
) -> Result<()>
where
    M: Serialize,
    W: Write,
{
    // Serialize message
    let bin_msg = serialize(message, sl.minimal_secure_layer.config.message_format)?;

    // Compress message
    let bin_zip_msg = sl.compress(&bin_msg[..])?;

    // Write binary message on a writer
    crate::complete::writer::write_bin_message::<W>(sl, &bin_zip_msg, writer)
}

pub fn serialize<M>(message: &M, message_format: MessageFormat) -> Result<Vec<u8>>
where
    M: Serialize,
{
    let mut writer = BufWriter::new(Vec::with_capacity(1_024));
    writer
        .write(message_format.as_ref())
        .map_err(Error::WriteError)?;
    serialize_inner(message, message_format, &mut writer).map_err(Error::SerdeError)?;
    writer.into_inner().map_err(|_| Error::BufferFlushError)
}

pub fn serialize_inner<M, W>(
    message: &M,
    message_format: MessageFormat,
    writer: &mut W,
) -> std::result::Result<(), SerdeError>
where
    M: Serialize,
    W: Write,
{
    match message_format {
        MessageFormat::RawBinary => Err(SerdeError::UseSuffixedBinFunctions),
        #[cfg(feature = "bin")]
        MessageFormat::Bincode => Ok(bincode::serialize_into(writer, message)
            .map_err(|e| SerdeError::BincodeError(format!("{}", e)))?),
        #[cfg(feature = "cbor")]
        MessageFormat::Cbor => {
            Ok(serde_cbor::to_writer(writer, message).map_err(SerdeError::CborError)?)
        }
        #[cfg(feature = "json")]
        MessageFormat::Utf8Json => {
            Ok(serde_json::to_writer(writer, message).map_err(SerdeError::JsonError)?)
        }
        _ => unimplemented!(),
    }
}
