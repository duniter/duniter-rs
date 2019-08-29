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

//! Manage PKSTL messages format.

use crate::errors::IncomingMsgErr;
use std::convert::TryFrom;

const RAW_BINARY: &[u8] = &[0, 0, 0, 0];
const UTF8_PLAIN_TEXT: &[u8] = &[0, 0, 0, 1];

#[cfg(feature = "bin")]
const BINCODE: &[u8] = &[0, 0, 0, 4];

#[cfg(feature = "cbor")]
const CBOR: &[u8] = &[0, 0, 0, 3];

#[cfg(feature = "json")]
const UTF8_JSON: &[u8] = &[0, 0, 0, 2];

/// Message format
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageFormat {
    /// raw binary
    RawBinary,
    /// UTF-8 plain text
    Utf8PlainText,
    #[cfg(feature = "json")]
    /// UTF-8 JSON
    Utf8Json,
    #[cfg(feature = "cbor")]
    /// CBOR (Binary JSON)
    Cbor,
    #[cfg(feature = "bin")]
    /// Bincode
    Bincode,
}

impl Default for MessageFormat {
    fn default() -> Self {
        Self::RawBinary
    }
}

impl TryFrom<&[u8]> for MessageFormat {
    type Error = IncomingMsgErr;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        match bytes {
            #[cfg(feature = "bin")]
            BINCODE => Ok(Self::Bincode),
            #[cfg(feature = "cbor")]
            CBOR => Ok(Self::Cbor),
            RAW_BINARY => Ok(Self::RawBinary),
            #[cfg(feature = "json")]
            UTF8_JSON => Ok(Self::Utf8Json),
            UTF8_PLAIN_TEXT => Ok(Self::Utf8PlainText),
            _ => Err(IncomingMsgErr::UnknownMessageFormat),
        }
    }
}

impl AsRef<[u8]> for MessageFormat {
    fn as_ref(&self) -> &[u8] {
        match self {
            #[cfg(feature = "bin")]
            Self::Bincode => BINCODE,
            #[cfg(feature = "cbor")]
            Self::Cbor => CBOR,
            Self::RawBinary => RAW_BINARY,
            #[cfg(feature = "json")]
            Self::Utf8Json => UTF8_JSON,
            Self::Utf8PlainText => UTF8_PLAIN_TEXT,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_default() {
        assert_eq!(MessageFormat::RawBinary, MessageFormat::default());
    }

    #[test]
    fn test_messafe_format_as_ref() {
        // RawBinary
        assert_eq!(RAW_BINARY, MessageFormat::RawBinary.as_ref());

        // Utf8PlainText
        assert_eq!(UTF8_PLAIN_TEXT, MessageFormat::Utf8PlainText.as_ref());

        // Utf8Json
        #[cfg(feature = "json")]
        assert_eq!(UTF8_JSON, MessageFormat::Utf8Json.as_ref());

        // Cbor
        #[cfg(feature = "cbor")]
        assert_eq!(CBOR, MessageFormat::Cbor.as_ref());

        // Bincode
        #[cfg(feature = "bin")]
        assert_eq!(BINCODE, MessageFormat::Bincode.as_ref());
    }

    #[test]
    fn test_message_format_try_from() -> Result<(), IncomingMsgErr> {
        // RawBinary
        assert_eq!(
            MessageFormat::RawBinary,
            MessageFormat::try_from(RAW_BINARY)?
        );

        // Utf8PlainText
        assert_eq!(
            MessageFormat::Utf8PlainText,
            MessageFormat::try_from(UTF8_PLAIN_TEXT)?
        );

        // Utf8Json
        #[cfg(feature = "json")]
        assert_eq!(MessageFormat::Utf8Json, MessageFormat::try_from(UTF8_JSON)?);

        // Cbor
        #[cfg(feature = "cbor")]
        assert_eq!(MessageFormat::Cbor, MessageFormat::try_from(CBOR)?);

        // Bincode
        #[cfg(feature = "bin")]
        assert_eq!(MessageFormat::Bincode, MessageFormat::try_from(BINCODE)?);

        // UnknownMessageFormat
        let bytes = vec![0, 0, 0, 5];
        assert_eq!(
            Err(IncomingMsgErr::UnknownMessageFormat),
            MessageFormat::try_from(&bytes[..]),
        );

        Ok(())
    }
}
