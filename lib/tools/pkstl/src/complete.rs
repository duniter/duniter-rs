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

//! Manage complete secure and decentralized transport layer.

pub mod message;
#[cfg(feature = "ser")]
pub mod serde;
pub mod writer;

#[cfg(feature = "ser")]
pub use self::serde::IncomingMessage;

use crate::{Error, Message, MinimalSecureLayer, Result, SdtlConfig, Seed32};
use flate2::write::{DeflateDecoder, DeflateEncoder};
use message::IncomingBinaryMessage;
use ring::signature::Ed25519KeyPair;
use std::io::{BufWriter, Write};

#[cfg(feature = "ser")]
use ::serde::de::DeserializeOwned;
#[cfg(feature = "ser")]
use ::serde::Serialize;
#[cfg(feature = "ser")]
use std::fmt::Debug;

/// Secure layer
pub struct SecureLayer {
    config: SdtlConfig,
    minimal_secure_layer: MinimalSecureLayer,
    sig_key_pair: Ed25519KeyPair,
}

impl SecureLayer {
    /// Change configuration
    #[inline]
    pub fn change_config(&mut self, new_config: SdtlConfig) {
        self.config = new_config;
        self.minimal_secure_layer.change_config(new_config.minimal);
    }
    fn compress(&self, bin_message: &[u8]) -> Result<Vec<u8>> {
        // Create buffer
        let buffer = BufWriter::new(Vec::with_capacity(bin_message.len()));

        // Determine compression level
        let compression_level = if bin_message.len() < self.config.compression_min_size {
            flate2::Compression::none()
        } else {
            self.config.compression
        };

        // Create compressor
        let mut deflate_encoder = DeflateEncoder::new(buffer, compression_level);

        // Write message in compressor buffer
        deflate_encoder
            .write_all(&bin_message[..])
            .map_err(Error::ZipError)?;

        // Finalize compression
        let bin_msg_compressed: BufWriter<Vec<u8>> =
            deflate_encoder.finish().map_err(Error::ZipError)?;

        // Flush buffer
        let bin_msg_compressed = bin_msg_compressed
            .into_inner()
            .map_err(|_| Error::BufferFlushError)?;

        Ok(bin_msg_compressed)
    }
    /// Create secure layer
    #[inline]
    pub fn create(
        config: SdtlConfig,
        sig_key_pair_seed: Option<Seed32>,
        expected_remote_sig_pubkey: Option<Vec<u8>>,
    ) -> Result<Self> {
        let seed = sig_key_pair_seed.unwrap_or_else(Seed32::random);

        let secure_layer = SecureLayer {
            config,
            minimal_secure_layer: MinimalSecureLayer::create(
                config.minimal,
                expected_remote_sig_pubkey,
            )?,
            sig_key_pair: Ed25519KeyPair::from_seed_unchecked(seed.as_ref())
                .map_err(|_| Error::FailtoGenSigKeyPair)?,
        };

        Ok(secure_layer)
    }
    /// Read binary incoming datas
    pub fn read_bin(&mut self, incoming_datas: &[u8]) -> Result<Option<IncomingBinaryMessage>> {
        let message_opt = self.minimal_secure_layer.read(incoming_datas)?;

        if let Some(message) = message_opt {
            let user_message = match message {
                Message::Connect {
                    custom_datas,
                    sig_pubkey,
                    ..
                } => IncomingBinaryMessage::Connect {
                    custom_datas: if let Some(custom_datas) = custom_datas {
                        Some(Self::uncompress(&custom_datas)?)
                    } else {
                        None
                    },
                    peer_sig_public_key: sig_pubkey,
                },
                Message::Ack { custom_datas } => IncomingBinaryMessage::Ack {
                    custom_datas: if let Some(custom_datas) = custom_datas {
                        Some(Self::uncompress(&custom_datas)?)
                    } else {
                        None
                    },
                },
                Message::Message { custom_datas } => IncomingBinaryMessage::Message {
                    datas: if let Some(custom_datas) = custom_datas {
                        Some(Self::uncompress(&custom_datas)?)
                    } else {
                        None
                    },
                },
            };
            Ok(Some(user_message))
        } else {
            Ok(None)
        }
    }
    /// Read incoming datas
    #[cfg(feature = "ser")]
    #[inline]
    pub fn read<M>(&mut self, incoming_datas: &[u8]) -> Result<Option<IncomingMessage<M>>>
    where
        M: Debug + DeserializeOwned,
    {
        self::serde::deserializer::read::<M>(self, incoming_datas)
    }
    fn uncompress(bin_zip_msg: &[u8]) -> Result<Vec<u8>> {
        let mut deflate_decoder = DeflateDecoder::new(Vec::with_capacity(bin_zip_msg.len() * 5));
        deflate_decoder
            .write_all(&bin_zip_msg)
            .map_err(Error::ZipError)?;
        deflate_decoder.finish().map_err(Error::ZipError)
    }
    /// Write ack message with optional binary custom datas
    pub fn write_ack_msg_bin<W>(
        &mut self,
        custom_datas: Option<&[u8]>,
        writer: &mut BufWriter<W>,
    ) -> Result<()>
    where
        W: Write,
    {
        // Serialize and compress custom datas
        let custom_datas = if let Some(custom_datas) = custom_datas {
            Some(self.compress(custom_datas)?)
        } else {
            None
        };

        writer::write_ack_msg::<W>(self, custom_datas, writer)
    }
    /// Write ack message with optional custom datas
    #[cfg(feature = "ser")]
    #[inline]
    pub fn write_ack_msg<M, W>(
        &mut self,
        custom_datas: Option<&M>,
        writer: &mut BufWriter<W>,
    ) -> Result<()>
    where
        M: Serialize,
        W: Write,
    {
        self::serde::serializer::write_ack_msg::<M, W>(self, custom_datas, writer)
    }
    /// Write connect message with optional binary custom datas
    pub fn write_connect_msg_bin<W>(
        &mut self,
        custom_datas: Option<&[u8]>,
        writer: &mut BufWriter<W>,
    ) -> Result<()>
    where
        W: Write,
    {
        // Compress custom datas
        let custom_datas = if let Some(custom_datas) = custom_datas {
            Some(self.compress(custom_datas)?)
        } else {
            None
        };

        writer::write_connect_msg(self, custom_datas, writer)
    }
    /// Write connect message with optional custom datas
    #[cfg(feature = "ser")]
    #[inline]
    pub fn write_connect_msg<M, W>(
        &mut self,
        custom_datas: Option<&M>,
        writer: &mut BufWriter<W>,
    ) -> Result<()>
    where
        M: Serialize,
        W: Write,
    {
        self::serde::serializer::write_connect_msg::<M, W>(self, custom_datas, writer)
    }
    /*/// Split secure layer in writer and reader
    pub fn split(self) -> Result<(SecureWriter, SecureReader)> {
        unimplemented!()
    }*/
    /// Write message on a writer
    #[cfg(feature = "ser")]
    #[inline]
    pub fn write<M, W>(&mut self, message: &M, writer: &mut BufWriter<W>) -> Result<()>
    where
        M: Serialize,
        W: Write,
    {
        self::serde::serializer::write_message::<M, W>(self, message, writer)
    }
    /// Write binary message on a writer
    pub fn write_bin<W>(&mut self, binary_message: &[u8], writer: &mut BufWriter<W>) -> Result<()>
    where
        W: Write,
    {
        // Compress message
        let bin_zip_msg = self.compress(&binary_message[..])?;

        writer::write_bin_message::<W>(self, &bin_zip_msg, writer)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[cfg(feature = "ser")]
    use crate::MessageFormat;
    use crate::SdtlMinimalConfig;

    #[test]
    fn test_change_config() -> Result<()> {
        let mut msl = SecureLayer::create(SdtlConfig::default(), None, None)?;
        msl.change_config(SdtlConfig {
            compression: flate2::Compression::fast(),
            compression_min_size: 8_192,
            #[cfg(feature = "ser")]
            message_format: MessageFormat::RawBinary,
            minimal: SdtlMinimalConfig::default(),
        });
        Ok(())
    }
}
