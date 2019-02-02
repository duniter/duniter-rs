//  Copyright (C) 2018  The Durs Project Developers.
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

/// Parsers for block
pub mod blocks;

/// Parsers for certifications
pub mod certifications;

/// Parsers for identities
pub mod identities;

/// Parsers for memberships
pub mod memberships;

/// Parsers for revocations
pub mod revoked;

/// Parsers for transactions
pub mod transactions;

type DefaultHasher = std::hash::BuildHasherDefault<std::collections::hash_map::DefaultHasher>;

#[cfg(test)]
mod tests {
    use crate::blockstamp::Blockstamp;
    use crate::documents::transaction::*;
    use crate::*;
    use std::str::FromStr;

    pub fn first_g1_tx_doc() -> TransactionDocument {
        let expected_tx_builder = TransactionDocumentBuilder {
            currency: &"g1",
            blockstamp: &Blockstamp::from_string(
                "50-00001DAA4559FEDB8320D1040B0F22B631459F36F237A0D9BC1EB923C12A12E7",
            )
            .expect("Fail to parse blockstamp"),
            locktime: &0,
            issuers: &vec![PubKey::Ed25519(
                ed25519::PublicKey::from_base58("2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ")
                    .expect("Fail to parse issuer !"),
            )],
            inputs: &vec![TransactionInput::from_str(
                "1000:0:D:2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ:1",
            )
            .expect("Fail to parse inputs")],
            unlocks: &vec![
                TransactionInputUnlocks::from_str("0:SIG(0)").expect("Fail to parse unlocks")
            ],
            outputs: &vec![
                TransactionOutput::from_str(
                    "1:0:SIG(Com8rJukCozHZyFao6AheSsfDQdPApxQRnz7QYFf64mm)",
                )
                .expect("Fail to parse outputs"),
                TransactionOutput::from_str(
                    "999:0:SIG(2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ)",
                )
                .expect("Fail to parse outputs"),
            ],
            comment: "TEST",
            hash: None,
        };

        expected_tx_builder.build_with_signature(vec![Sig::Ed25519(
                ed25519::Signature::from_base64("fAH5Gor+8MtFzQZ++JaJO6U8JJ6+rkqKtPrRr/iufh3MYkoDGxmjzj6jCADQL+hkWBt8y8QzlgRkz0ixBcKHBw==").expect("Fail to parse sig !")
            )])
    }
}
