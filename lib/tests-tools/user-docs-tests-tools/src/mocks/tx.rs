//  Copyright (C) 2019  Éloïs SANCHEZ
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

//! Mocks for projects use dubp-user-docs

use dubp_common_doc::parser::TextDocumentParser;
use dubp_common_doc::traits::DocumentBuilder;
use dubp_common_doc::Blockstamp;
use dubp_user_docs::documents::transaction::v10::TransactionInputUnlocksV10;
use dubp_user_docs::documents::transaction::*;
use dup_crypto::keys::*;
use std::str::FromStr;

/// Generate first G1 transaction !
pub fn first_g1_tx_doc() -> TransactionDocument {
    let expected_tx_builder = TransactionDocumentV10Builder {
        currency: &"g1",
        blockstamp: &Blockstamp::from_string(
            "50-00001DAA4559FEDB8320D1040B0F22B631459F36F237A0D9BC1EB923C12A12E7",
        )
        .expect("Fail to parse blockstamp"),
        locktime: &0,
        issuers: &[PubKey::Ed25519(
            ed25519::PublicKey::from_base58("2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ")
                .expect("Fail to parse issuer !"),
        )],
        inputs: &[TransactionInputV10::from_str(
            "1000:0:D:2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ:1",
        )
        .expect("Fail to parse inputs")],
        unlocks: &[TransactionInputUnlocksV10::from_str("0:SIG(0)").expect("Fail to parse unlocks")],
        outputs: &[
            TransactionOutputV10::from_str("1:0:SIG(Com8rJukCozHZyFao6AheSsfDQdPApxQRnz7QYFf64mm)")
                .expect("Fail to parse outputs"),
            TransactionOutputV10::from_str(
                "999:0:SIG(2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ)",
            )
            .expect("Fail to parse outputs"),
        ],
        comment: "TEST",
        hash: None,
    };

    TransactionDocumentBuilder::V10(expected_tx_builder).build_with_signature(vec![Sig::Ed25519(
                ed25519::Signature::from_base64("fAH5Gor+8MtFzQZ++JaJO6U8JJ6+rkqKtPrRr/iufh3MYkoDGxmjzj6jCADQL+hkWBt8y8QzlgRkz0ixBcKHBw==").expect("Fail to parse sig !")
            )])
}

/// Generate mock transaction document
pub fn gen_mock_tx_doc() -> TransactionDocument {
    TransactionDocumentParser::parse("Version: 10
Type: Transaction
Currency: g1
Blockstamp: 107982-000001242F6DA51C06A915A96C58BAA37AB3D1EB51F6E1C630C707845ACF764B
Locktime: 0
Issuers:
8dkCwvAqSczUjKsoVMDPVbQ3i6bBQeBQYawL87kqTSQ3
Inputs:
1002:0:D:8dkCwvAqSczUjKsoVMDPVbQ3i6bBQeBQYawL87kqTSQ3:106345
Unlocks:
0:SIG(0)
Outputs:
1002:0:SIG(CitdnuQgZ45tNFCagay7Wh12gwwHM8VLej1sWmfHWnQX)
Comment: DU symbolique pour demander le codage de nouvelles fonctionnalites cf. https://forum.monnaie-libre.fr/t/creer-de-nouvelles-fonctionnalites-dans-cesium-les-autres-applications/2025  Merci
T0LlCcbIn7xDFws48H8LboN6NxxwNXXTovG4PROLf7tkUAueHFWjfwZFKQXeZEHxfaL1eYs3QspGtLWUHPRVCQ==").expect("Fail to parse tx1")
}

/// Generate mock transaction document with wrong version
pub fn gen_mock_tx_doc_wrong_version() -> TransactionDocument {
    TransactionDocumentParser::parse("Version: 12
Type: Transaction
Currency: g1
Blockstamp: 107982-000001242F6DA51C06A915A96C58BAA37AB3D1EB51F6E1C630C707845ACF764B
Locktime: 0
Issuers:
8dkCwvAqSczUjKsoVMDPVbQ3i6bBQeBQYawL87kqTSQ3
Inputs:
1002:0:D:8dkCwvAqSczUjKsoVMDPVbQ3i6bBQeBQYawL87kqTSQ3:106345
Unlocks:
0:SIG(0)
Outputs:
1002:0:SIG(CitdnuQgZ45tNFCagay7Wh12gwwHM8VLej1sWmfHWnQX)
Comment: DU symbolique pour demander le codage de nouvelles fonctionnalites cf. https://forum.monnaie-libre.fr/t/creer-de-nouvelles-fonctionnalites-dans-cesium-les-autres-applications/2025  Merci
T0LlCcbIn7xDFws48H8LboN6NxxwNXXTovG4PROLf7tkUAueHFWjfwZFKQXeZEHxfaL1eYs3QspGtLWUHPRVCQ==").expect("Fail to parse tx1")
}
