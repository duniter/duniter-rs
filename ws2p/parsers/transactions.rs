extern crate serde;
extern crate serde_json;

use duniter_crypto::hashs::Hash;
use duniter_crypto::keys::*;
use duniter_documents::blockchain::v10::documents::transaction::{
    TransactionDocument, TransactionDocumentBuilder, TransactionInput, TransactionInputUnlocks,
    TransactionOutput,
};
use duniter_documents::blockchain::DocumentBuilder;
use duniter_documents::Blockstamp;

pub fn parse_transaction(
    currency: &str,
    source: &serde_json::Value,
) -> Option<TransactionDocument> {
    //debug!("transaction={:#?}", source);
    let blockstamp = match Blockstamp::from_string(source.get("blockstamp")?.as_str()?) {
        Ok(blockstamp) => blockstamp,
        Err(_) => {
            return None;
        }
    };
    let locktime = source.get("locktime")?.as_i64()? as u64;
    let issuers_array = source.get("issuers")?.as_array()?;
    let mut issuers = Vec::with_capacity(issuers_array.len());
    for issuer in issuers_array {
        match ed25519::PublicKey::from_base58(issuer.as_str()?) {
            Ok(pubkey) => issuers.push(PubKey::Ed25519(pubkey)),
            Err(_) => {
                return None;
            }
        }
    }
    let inputs_array = source.get("inputs")?.as_array()?;
    let mut inputs = Vec::with_capacity(inputs_array.len());
    for input in inputs_array {
        let input_str = input.as_str()?;
        match TransactionInput::parse_from_str(input_str) {
            Ok(input) => inputs.push(input),
            Err(_) => {
                return None;
            }
        }
    }
    let unlocks_array = source.get("unlocks")?.as_array()?;
    let mut unlocks = Vec::with_capacity(unlocks_array.len());
    for unlock in unlocks_array {
        match TransactionInputUnlocks::parse_from_str(unlock.as_str()?) {
            Ok(unlock) => unlocks.push(unlock),
            Err(_) => {
                return None;
            }
        }
    }
    let outputs_array = source.get("outputs")?.as_array()?;
    let mut outputs = Vec::with_capacity(outputs_array.len());
    for output in outputs_array {
        match TransactionOutput::parse_from_str(output.as_str()?) {
            Ok(output) => outputs.push(output),
            Err(_) => {
                return None;
            }
        }
    }
    let signatures_array = source.get("signatures")?.as_array()?;
    let mut signatures = Vec::with_capacity(signatures_array.len());
    for signature in signatures_array {
        match ed25519::Signature::from_base64(signature.as_str()?) {
            Ok(signature) => signatures.push(Sig::Ed25519(signature)),
            Err(_) => {
                return None;
            }
        }
    }
    let comment = source.get("comment")?.as_str()?;
    let hash = match Hash::from_hex(source.get("hash")?.as_str()?) {
        Ok(hash) => hash,
        Err(_) => {
            return None;
        }
    };

    let tx_doc_builder = TransactionDocumentBuilder {
        currency,
        blockstamp: &blockstamp,
        locktime: &locktime,
        issuers: &issuers,
        inputs: &inputs,
        unlocks: &unlocks,
        outputs: &outputs,
        comment,
        hash: Some(hash),
    };
    Some(tx_doc_builder.build_with_signature(signatures))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_compact_tx() {
        let _compact_txs = "[\"TX:10:1:1:1:1:1:0$\
112533-000002150F2E805E604D9B31212D079570AAD8D3A4D8BB75F2C15A94A345B6B1$\
51EFVNZwpfmTXU7BSLpeh3PZFgfdmm5hq5MzCDopdH2$\
1000:0:D:51EFVNZwpfmTXU7BSLpeh3PZFgfdmm5hq5MzCDopdH2:46496$\
0:SIG(0)$\
1000:0:SIG(2yN8BRSkARcqE8NCxKMBiHfTpx1EvwULFn56Myf6qRmy)$\
Merci pour la calligraphie ;) de Liam$\
5olrjFylTCsVq8I5Yr7FpXeviynICyvIwe1yG5N0RJF+VZb+bCFBnLAMpmMCU2qzUvK7z41UXOrMRybXiLa2Dw==\"]";

        let _tx_builder = TransactionDocumentBuilder {
            currency: "g1",
            blockstamp: &Blockstamp::from_string(
                "112533-000002150F2E805E604D9B31212D079570AAD8D3A4D8BB75F2C15A94A345B6B1",
            ).unwrap(),
            locktime: &0,
            issuers: &vec![PubKey::Ed25519(
                ed25519::PublicKey::from_base58("51EFVNZwpfmTXU7BSLpeh3PZFgfdmm5hq5MzCDopdH2")
                    .unwrap(),
            )],
            inputs: &vec![
                TransactionInput::parse_from_str(
                    "1000:0:D:51EFVNZwpfmTXU7BSLpeh3PZFgfdmm5hq5MzCDopdH2:46496",
                ).unwrap(),
            ],
            outputs: &vec![
                TransactionOutput::parse_from_str(
                    "1000:0:SIG(2yN8BRSkARcqE8NCxKMBiHfTpx1EvwULFn56Myf6qRmy)",
                ).unwrap(),
            ],
            unlocks: &vec![TransactionInputUnlocks::parse_from_str("0:SIG(0)").unwrap()],
            comment: "Merci pour la calligraphie ;) de Liam",
            hash: None,
        };
    }
}
