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
pub struct SecureLayerConfig {
    #[cfg(feature = "zip-sign")]
    /// Compression level
    pub compression: flate2::Compression,
    #[cfg(feature = "zip-sign")]
    /// Compression minimal size in bytes
    pub compression_min_size: usize,
    #[cfg(feature = "ser")]
    /// Message format
    pub message_format: MessageFormat,
    /// Encryption algorithm
    pub encrypt_algo: EncryptAlgo,
}

impl Default for SecureLayerConfig {
    fn default() -> Self {
        SecureLayerConfig {
            #[cfg(feature = "zip-sign")]
            compression: flate2::Compression::fast(),
            #[cfg(feature = "zip-sign")]
            compression_min_size: DEFAULT_COMPRESSION_MIN_SIZE,
            #[cfg(feature = "ser")]
            message_format: MessageFormat::default(),
            encrypt_algo: EncryptAlgo::default(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_default_config() {
        assert_eq!(
            SecureLayerConfig {
                compression: flate2::Compression::fast(),
                compression_min_size: DEFAULT_COMPRESSION_MIN_SIZE,
                #[cfg(feature = "ser")]
                message_format: MessageFormat::default(),
                encrypt_algo: EncryptAlgo::default(),
            },
            SecureLayerConfig::default()
        )
    }
}
