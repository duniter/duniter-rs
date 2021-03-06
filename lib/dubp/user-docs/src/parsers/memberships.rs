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

use crate::documents::membership::v10::*;
use dubp_common_doc::blockstamp::Blockstamp;
use dubp_common_doc::traits::DocumentBuilder;
use dup_crypto::keys::*;
use failure::Error;

#[derive(Debug, Fail, Copy, Clone)]
pub enum ParseMembershipError {
    #[fail(display = "Fail to parse membership : wrong format !")]
    WrongFormat,
}

/// Parse memberships documents from array of str
pub fn parse_compact_memberships(
    currency: &str,
    membership_type: MembershipType,
    array_memberships: &[&str],
) -> Result<Vec<MembershipDocumentV10>, Error> {
    //let memberships: Vec<MembershipDocumentV10> = Vec::new();
    array_memberships
        .iter()
        .map(|membership| {
            let membership_datas: Vec<&str> = membership.split(':').collect();
            if membership_datas.len() == 5 {
                let membership_doc_builder = MembershipDocumentV10Builder {
                    currency,
                    issuer: &PubKey::Ed25519(ed25519::PublicKey::from_base58(membership_datas[0])?),
                    blockstamp: &Blockstamp::from_string(membership_datas[2])?,
                    membership: membership_type,
                    identity_username: membership_datas[4],
                    identity_blockstamp: &Blockstamp::from_string(membership_datas[3])?,
                };
                let membership_sig =
                    Sig::Ed25519(ed25519::Signature::from_base64(membership_datas[1])?);
                Ok(membership_doc_builder.build_with_signature(vec![membership_sig]))
            } else {
                Err(ParseMembershipError::WrongFormat.into())
            }
        })
        .collect()
}
