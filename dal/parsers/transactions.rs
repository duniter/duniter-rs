extern crate serde;
extern crate serde_json;

use duniter_crypto::keys::{PublicKey, Signature};
use duniter_documents::blockchain::v10::documents::transaction::{
    TransactionDocument, TransactionDocumentBuilder, TransactionInput, TransactionInputUnlocks,
    TransactionOutput,
};
use duniter_documents::blockchain::DocumentBuilder;
use duniter_documents::Blockstamp;

pub fn parse_compact_transactions(
    currency: &str,
    json_datas: &str,
) -> Option<Vec<TransactionDocument>> {
    let raw_transactions: serde_json::Value =
        serde_json::from_str(json_datas).expect("Fatal error : fail to jsonifie tx from DB !");

    if raw_transactions.is_array() {
        let mut transactions = Vec::new();
        for transaction in raw_transactions.as_array().unwrap() {
            let transaction_lines: Vec<&str> = transaction
                .as_str()
                .expect("Fail to parse tx from DB !")
                .split('$')
                .collect();
            let tx_headers: Vec<&str> = transaction_lines[0].split(':').collect();
            let issuers_count = tx_headers[2]
                .parse()
                .expect("Fail to parse tx header NB_ISSUERS !");
            let inputs_count = tx_headers[3]
                .parse()
                .expect("Fail to parse tx header NB_INPUTS !");
            let unlocks_count = tx_headers[4]
                .parse()
                .expect("Fail to parse tx header NB_UNLOCKS !");
            let outputs_count = tx_headers[5]
                .parse()
                .expect("Fail to parse tx header NB_OUTPUTS !");
            let has_comment: usize = tx_headers[6]
                .parse()
                .expect("Fail to parse tx header HAS_COMMENT !");
            let locktime = tx_headers[7]
                .parse()
                .expect("Fail to parse tx header LOCKTIME !");
            let blockstamp = Blockstamp::from_string(transaction_lines[1])
                .expect("Fail to parse tx BLOCKSTAMP !");
            let mut line = 2;
            let mut issuers = Vec::new();
            for _ in 0..issuers_count {
                issuers.push(
                    PublicKey::from_base58(transaction_lines[line])
                        .expect("Fail to parse tx issuer !"),
                );
                line += 1;
            }
            let mut inputs = Vec::new();
            for _ in 0..inputs_count {
                inputs.push(
                    TransactionInput::parse_from_str(transaction_lines[line])
                        .expect("Fail to parse tx issuer !"),
                );
                line += 1;
            }
            let mut unlocks = Vec::new();
            for _ in 0..unlocks_count {
                unlocks.push(
                    TransactionInputUnlocks::parse_from_str(transaction_lines[line])
                        .expect("Fail to parse tx issuer !"),
                );
                line += 1;
            }
            let mut outputs = Vec::new();
            for _ in 0..outputs_count {
                outputs.push(
                    TransactionOutput::parse_from_str(transaction_lines[line])
                        .expect("Fail to parse tx issuer !"),
                );
                line += 1;
            }
            let mut comment = "";
            if has_comment == 1 {
                comment = transaction_lines[line];
                line += 1;
            }
            let mut signatures = Vec::new();
            for _ in 0..issuers_count {
                signatures.push(
                    Signature::from_base64(transaction_lines[line])
                        .expect("Fail to parse tx signature !"),
                );
                line += 1;
            }
            let tx_doc_builder = TransactionDocumentBuilder {
                currency,
                blockstamp: &blockstamp,
                locktime: &locktime,
                issuers: &issuers,
                inputs: &inputs,
                unlocks: &unlocks,
                outputs: &outputs,
                comment,
            };
            transactions.push(tx_doc_builder.build_with_signature(signatures));
        }
        Some(transactions)
    } else {
        None
    }
}

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
        match PublicKey::from_base58(issuer.as_str()?) {
            Ok(pubkey) => issuers.push(pubkey),
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
        match Signature::from_base64(signature.as_str()?) {
            Ok(signature) => signatures.push(signature),
            Err(_) => {
                return None;
            }
        }
    }
    let comment = source.get("comment")?.as_str()?;

    let tx_doc_builder = TransactionDocumentBuilder {
        currency,
        blockstamp: &blockstamp,
        locktime: &locktime,
        issuers: &issuers,
        inputs: &inputs,
        unlocks: &unlocks,
        outputs: &outputs,
        comment,
    };
    Some(tx_doc_builder.build_with_signature(signatures))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_compact_tx() {
        let compact_txs = "[\"TX:10:1:1:1:1:1:0$\
112533-000002150F2E805E604D9B31212D079570AAD8D3A4D8BB75F2C15A94A345B6B1$\
51EFVNZwpfmTXU7BSLpeh3PZFgfdmm5hq5MzCDopdH2$\
1000:0:D:51EFVNZwpfmTXU7BSLpeh3PZFgfdmm5hq5MzCDopdH2:46496$\
0:SIG(0)$\
1000:0:SIG(2yN8BRSkARcqE8NCxKMBiHfTpx1EvwULFn56Myf6qRmy)$\
Merci pour la calligraphie ;) de Liam$\
5olrjFylTCsVq8I5Yr7FpXeviynICyvIwe1yG5N0RJF+VZb+bCFBnLAMpmMCU2qzUvK7z41UXOrMRybXiLa2Dw==\"]";

        let tx_builder = TransactionDocumentBuilder {
            currency: "g1",
            blockstamp: &Blockstamp::from_string(
                "112533-000002150F2E805E604D9B31212D079570AAD8D3A4D8BB75F2C15A94A345B6B1",
            ).unwrap(),
            locktime: &0,
            issuers: &vec![
                PublicKey::from_base58("51EFVNZwpfmTXU7BSLpeh3PZFgfdmm5hq5MzCDopdH2").unwrap(),
            ],
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
        };

        assert_eq!(
            parse_compact_transactions("g1", compact_txs).expect("Fail to parse compact transactions !"),
            vec![tx_builder.build_with_signature(vec![Signature::from_base64("5olrjFylTCsVq8I5Yr7FpXeviynICyvIwe1yG5N0RJF+VZb+bCFBnLAMpmMCU2qzUvK7z41UXOrMRybXiLa2Dw==").unwrap()])]
        );
    }
}
