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

//! Handle KPSTL status.

use crate::errors::IncomingMsgErr;
use crate::{Action, ActionSideEffects, Error, LocalNegoThread, MsgType, RemoteNegoThread, Result};

/// Secure layer status
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum SecureLayerStatus {
    /// An error has occurred or one peer message is wrong
    Fail,
    /// Negotiation in progress
    OngoingNegotiation {
        local: LocalNegoThread,
        remote: RemoteNegoThread,
    },
    /// Equivalent to "AckMsgWrittenAndPeerAckMsgOk"
    NegotiationSuccessful,
}

impl SecureLayerStatus {
    pub(crate) fn init() -> Self {
        SecureLayerStatus::OngoingNegotiation {
            local: LocalNegoThread::Created,
            remote: RemoteNegoThread::WaitConnectMsg,
        }
    }
    pub(crate) fn apply_action(&mut self, action: Action) -> Result<Option<ActionSideEffects>> {
        match self {
            Self::Fail => Err(Error::ConnectionHadFail),
            Self::OngoingNegotiation { local, remote } => match action {
                Action::Create(msg_type) => match msg_type {
                    MsgType::Connect => {
                        if *local == LocalNegoThread::Created {
                            *local = LocalNegoThread::ConnectMsgSent;
                            Ok(None)
                        } else {
                            Err(Error::ConnectMsgAlreadyWritten)
                        }
                    }
                    MsgType::Ack => {
                        if *remote == RemoteNegoThread::ValidConnectMsgReceived {
                            if *local == LocalNegoThread::ValidAckMsgReceived {
                                *self = Self::NegotiationSuccessful;
                            } else {
                                *remote = RemoteNegoThread::AckMsgSent;
                            }
                            Ok(None)
                        } else {
                            Err(Error::ForbidWriteAckMsgNow)
                        }
                    }
                    MsgType::UserMsg => {
                        *self = Self::Fail;
                        Err(Error::NegoMustHaveBeenSuccessful)
                    }
                },
                Action::Receive(msg_type) => match msg_type {
                    MsgType::Connect => {
                        if *remote == RemoteNegoThread::WaitConnectMsg {
                            *remote = RemoteNegoThread::ValidConnectMsgReceived;
                            Ok(None)
                        } else {
                            *self = Self::Fail;
                            Err(Error::RecvInvalidMsg(IncomingMsgErr::UnexpectedConnectMsg))
                        }
                    }
                    MsgType::Ack => {
                        if *local == LocalNegoThread::ConnectMsgSent {
                            if *remote == RemoteNegoThread::AckMsgSent {
                                *self = Self::NegotiationSuccessful;
                            } else {
                                *local = LocalNegoThread::ValidAckMsgReceived;
                            }
                            Ok(None)
                        } else {
                            *self = Self::Fail;
                            Err(Error::RecvInvalidMsg(IncomingMsgErr::UnexpectedAckMsg))
                        }
                    }
                    MsgType::UserMsg => {
                        if *remote == RemoteNegoThread::AckMsgSent
                            && *local == LocalNegoThread::ConnectMsgSent
                        {
                            Ok(Some(ActionSideEffects::PushUserMsgIntoTmpStack))
                        } else {
                            *self = Self::Fail;
                            Err(Error::RecvInvalidMsg(IncomingMsgErr::UnexpectedMessage))
                        }
                    }
                },
            },
            Self::NegotiationSuccessful => match action {
                Action::Create(msg_type) => match msg_type {
                    MsgType::Connect => Err(Error::ConnectMsgAlreadyWritten),
                    MsgType::Ack => Err(Error::ForbidWriteAckMsgNow),
                    MsgType::UserMsg => Ok(None),
                },
                Action::Receive(msg_type) => match msg_type {
                    MsgType::Connect => {
                        *self = Self::Fail;
                        Err(Error::RecvInvalidMsg(IncomingMsgErr::UnexpectedConnectMsg))
                    }
                    MsgType::Ack => {
                        *self = Self::Fail;
                        Err(Error::RecvInvalidMsg(IncomingMsgErr::UnexpectedAckMsg))
                    }
                    MsgType::UserMsg => Ok(None),
                },
            },
        }
    }
}
