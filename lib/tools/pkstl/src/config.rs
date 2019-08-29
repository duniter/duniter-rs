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

//! Manage PKSTL configuration.

use crate::encryption::EncryptAlgo;

#[cfg(feature = "zip-sign")]
const DEFAULT_COMPRESSION_MIN_SIZE: usize = 8_192;

#[cfg(feature = "ser")]
use crate::format::MessageFormat;

#[cfg(feature = "zip-sign")]
#[derive(Clone, Copy, Debug, PartialEq)]
/// PKSTL Configuration
pub struct SdtlConfig {
    /// Compression level
    pub compression: flate2::Compression,
    /// Compression minimal size in bytes
    pub compression_min_size: usize,
    #[cfg(feature = "ser")]
    /// Message format
    pub message_format: MessageFormat,
    /// PKSTL minimum Configuration
    pub minimal: SdtlMinimalConfig,
}

impl Default for SdtlConfig {
    fn default() -> Self {
        SdtlConfig {
            compression: flate2::Compression::fast(),
            compression_min_size: DEFAULT_COMPRESSION_MIN_SIZE,
            #[cfg(feature = "ser")]
            message_format: MessageFormat::default(),
            minimal: SdtlMinimalConfig::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
/// PKSTL minimum Configuration
pub struct SdtlMinimalConfig {
    /// Encryption algorithm
    pub encrypt_algo: EncryptAlgo,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_default_config() {
        assert_eq!(
            SdtlConfig {
                compression: flate2::Compression::fast(),
                compression_min_size: DEFAULT_COMPRESSION_MIN_SIZE,
                #[cfg(feature = "ser")]
                message_format: MessageFormat::default(),
                minimal: SdtlMinimalConfig::default(),
            },
            SdtlConfig::default()
        )
    }
}
