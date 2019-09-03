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

//! Manage Secure and decentralized transport layer errors.

/// PKSTL Error
#[derive(Debug)]
pub enum Error {
    /// Error when flush writer buffer
    BufferFlushError,
    /// The connection had already failed earlier
    ConnectionHadFail,
    /// Connect msg already written
    ConnectMsgAlreadyWritten,
    /// Fail to compute agreement
    FailToComputeAgreement,
    /// Fail to decrypt datas
    FailToDecryptDatas(chacha20_poly1305_aead::DecryptError),
    /// Fail to encrypt datas
    FailToEncryptDatas(std::io::Error),
    /// Fail to generate ephemeral key pair
    FailToGenEphemerKeyPair,
    /// Fail to generate ephemeral public key
    FailToGenEphemerPubKey,
    /// Fail to generate signature key pair
    FailtoGenSigKeyPair,
    /// Forbidden to change the configuration after the security layer has been cloned
    ForbidChangeConfAfterClone,
    /// Forbidden to write the ACK message now
    ForbidWriteAckMsgNow,
    /// Message must be signed
    MessageMustBeSigned,
    /// The negotiation must have been successful
    NegoMustHaveBeenSuccessful,
    #[cfg(feature = "ser")]
    /// Error in serialization/deserialization
    SerdeError(crate::complete::serde::SerdeError),
    /// Serialization error
    SerializationError(std::io::Error),
    /// Tru to generate connect message too late
    TryToGenConnectMsgTooLate,
    /// Trying to write a message when the negotiation is not successful
    TryToWriteMsgWhenNegoNotSuccessful,
    /// Receive invalid message
    RecvInvalidMsg(IncomingMsgErr),
    /// Unexpected remote signature public key
    UnexpectedRemoteSigPubKey,
    /// Error on writer
    WriteError(std::io::Error),
    /// Written length error
    WrittenLenError {
        /// Expected
        expected: usize,
        /// Found
        found: usize,
    },
    #[cfg(feature = "zip-sign")]
    /// Compression or decompression error
    ZipError(std::io::Error),
}

impl From<IncomingMsgErr> for Error {
    fn from(e: IncomingMsgErr) -> Self {
        Self::RecvInvalidMsg(e)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Incoming message error
pub enum IncomingMsgErr {
    /// Invalid challenge
    InvalidChallenge,
    /// Invalid hash or signature
    InvalidHashOrSig,
    /// Invalid magic value
    InvalidMagicValue,
    /// Message too short
    MessageTooShort,
    /// Unexpected a cck message
    UnexpectedAckMsg,
    /// Unexpected connect message
    UnexpectedConnectMsg,
    /// Unexpected user message
    UnexpectedMessage,
    /// Unexpected encryption state
    /// It may be that the message is in clear when we expect it to be encrypted.
    UnexpectedEncryptionState,
    /// Unknown message format
    UnknownMessageFormat,
    /// Unknown message type
    UnknownMessageType,
    /// Unsupported signature algorithm
    UnsupportedSigAlgo,
    /// Unsupported version
    UnsupportedVersion,
}
