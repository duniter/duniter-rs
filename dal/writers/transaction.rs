//  Copyright (C) 2018  The Duniter Project Developers.
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

use duniter_documents::blockchain::v10::documents::transaction::*;
use rustbreak::backend::Backend;
use sources::{SourceAmount, SourceIndexV10, UTXOIndexV10, UTXOV10};
use std::fmt::Debug;
use *;

#[derive(Debug, Copy, Clone)]
pub enum TxError {
    UnkonwError(),
    DALError(DALError),
}

impl From<DALError> for TxError {
    fn from(err: DALError) -> TxError {
        TxError::DALError(err)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DALTxV10 {
    tx_doc: TransactionDocument,
    sources_destroyed: HashSet<UTXOIndexV10>,
}

pub fn apply_and_write_tx<B: Backend + Debug>(
    tx_db: &BinDB<TxV10Datas, B>,
    utxos_db: &BinDB<UTXOsV10Datas, B>,
    dus_db: &BinDB<DUsV10Datas, B>,
    balances_db: &BinDB<BalancesV10Datas, B>,
    tx_doc: &TransactionDocument,
) -> Result<(), DALError> {
    let mut tx_doc = tx_doc.clone();
    let tx_hash = tx_doc.get_hash();
    let mut sources_destroyed = HashSet::new();
    // Index consumed sources
    let consumed_sources: HashMap<SourceIndexV10, SourceAmount> = tx_doc
        .get_inputs()
        .iter()
        .map(|input| match *input {
            TransactionInput::D(tx_amout, tx_amout_base, pubkey, block_id) => (
                SourceIndexV10::DU(pubkey, block_id),
                SourceAmount(tx_amout, tx_amout_base),
            ),
            TransactionInput::T(tx_amout, tx_amout_base, hash, tx_index) => (
                SourceIndexV10::UTXO(UTXOIndexV10(hash, tx_index)),
                SourceAmount(tx_amout, tx_amout_base),
            ),
        })
        .collect();
    // Find adress of consumed sources
    let consumed_adress: HashMap<
        TransactionOutputConditionGroup,
        (SourceAmount, HashSet<UTXOIndexV10>),
    > = utxos_db.read(|db| {
        let mut consumed_adress: HashMap<
            TransactionOutputConditionGroup,
            (SourceAmount, HashSet<UTXOIndexV10>),
        > = HashMap::new();
        for (source_index, source_amount) in &consumed_sources {
            if let SourceIndexV10::UTXO(utxo_index) = source_index {
                // Get utxo
                let utxo = db
                    .get(&utxo_index)
                    .expect("ApplyBLockError : unknow UTXO in inputs !");
                // Get utxo conditions(=address)
                let conditions = &utxo.conditions;
                // Calculate new balances datas for "conditions" address
                let (mut balance, mut utxos_index) =
                    consumed_adress.get(conditions).cloned().unwrap_or_default();
                balance = balance + *source_amount;
                utxos_index.insert(*utxo_index);
                // Write new balances datas for "conditions" address
                consumed_adress.insert(conditions.clone(), (balance, utxos_index));
            } else if let SourceIndexV10::DU(pubkey, _block_id) = source_index {
                let address = TransactionOutputConditionGroup::Single(
                    TransactionOutputCondition::Sig(*pubkey),
                );
                let (mut balance, utxos_index) =
                    consumed_adress.get(&address).cloned().unwrap_or_default();
                balance = balance + *source_amount;
                consumed_adress.insert(address, (balance, utxos_index));
            }
        }
        consumed_adress
    })?;
    // Recalculate balance of consumed adress
    let new_balances_consumed_adress = balances_db.read(|db| {
        let mut new_balances_consumed_adress = Vec::new();
        for (conditions, (amount_consumed, adress_consumed_sources)) in consumed_adress {
            if let Some((balance, sources)) = db.get(&conditions) {
                let mut new_balance = *balance - amount_consumed;
                if new_balance.0 < TxAmount(100) {
                    sources_destroyed = sources.union(&sources_destroyed).cloned().collect();
                    new_balance = SourceAmount(TxAmount(0), new_balance.1);
                }
                let mut new_sources_index = sources.clone();
                for source in adress_consumed_sources {
                    new_sources_index.remove(&source);
                }
                new_balances_consumed_adress
                    .push((conditions.clone(), (new_balance, new_sources_index)));
            } else {
                panic!("Apply Tx : try to consume a source, but the owner address is not found in balances db : {:?}", conditions)
            }
        }
        new_balances_consumed_adress
    })?;
    // Write new balance of consumed adress
    balances_db.write(|db| {
        for (conditions, (balance, sources_index)) in new_balances_consumed_adress {
            db.insert(conditions, (balance, sources_index));
        }
    })?;
    // Remove consumed sources
    for source_index in consumed_sources.keys() {
        if let SourceIndexV10::UTXO(utxo_index) = source_index {
            utxos_db.write(|db| {
                db.remove(utxo_index);
            })?;
        } else if let SourceIndexV10::DU(pubkey, block_id) = source_index {
            let mut pubkey_dus: HashSet<BlockId> =
                dus_db.read(|db| db.get(&pubkey).cloned().unwrap_or_default())?;
            pubkey_dus.remove(block_id);
            dus_db.write(|db| {
                db.insert(*pubkey, pubkey_dus);
            })?;
        }
    }
    // Index created sources
    /*let mut created_utxos: Vec<UTXOV10> = Vec::new();
    let mut output_index = 0;
    for output in tx_doc.get_outputs() {
        created_utxos.push(UTXOV10(
            UTXOIndexV10(tx_hash, TxIndex(output_index)),
            output.clone(),
        ));
        output_index += 1;
    }*/
    let created_utxos: Vec<UTXOV10> = tx_doc
        .get_outputs()
        .iter()
        .enumerate()
        .map(|(tx_index, output)| UTXOV10(UTXOIndexV10(tx_hash, TxIndex(tx_index)), output.clone()))
        .collect();
    // Recalculate balance of supplied adress
    let new_balances_supplied_adress = balances_db.read(|db| {
        let mut new_balances_supplied_adress: HashMap<
            TransactionOutputConditionGroup,
            (SourceAmount, HashSet<UTXOIndexV10>),
        > = HashMap::new();
        for source in &created_utxos {
            let source_amount = source.get_amount();
            let conditions = source.get_conditions();
            let (balance, new_sources_index) = if let Some((balance, sources_index)) =
                new_balances_supplied_adress.get(&conditions)
            {
                let mut new_sources_index = sources_index.clone();
                new_sources_index.insert(source.0);
                (*balance, new_sources_index)
            } else if let Some((balance, sources_index)) = db.get(&conditions) {
                let mut new_sources_index = sources_index.clone();
                new_sources_index.insert(source.0);
                (*balance, new_sources_index)
            } else {
                let mut new_sources_index = HashSet::new();
                new_sources_index.insert(source.0);
                (SourceAmount::default(), new_sources_index)
            };
            new_balances_supplied_adress
                .insert(conditions, (balance + source_amount, new_sources_index));
        }
        new_balances_supplied_adress
    })?;
    // Insert created UTXOs
    utxos_db.write(|db| {
        for utxo_v10 in created_utxos {
            db.insert(utxo_v10.0, utxo_v10.1);
        }
    })?;
    // Write new balance of supplied adress
    balances_db.write(|db| {
        for (conditions, (balance, sources_index)) in new_balances_supplied_adress {
            db.insert(conditions, (balance, sources_index));
        }
    })?;
    // Write tx
    tx_doc.reduce();
    tx_db.write(|db| {
        db.insert(
            tx_hash,
            DALTxV10 {
                tx_doc,
                sources_destroyed,
            },
        );
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use duniter_documents::blockchain::{Document, DocumentBuilder, VerificationResult};

    fn build_first_tx_of_g1() -> TransactionDocument {
        let pubkey = PubKey::Ed25519(
            ed25519::PublicKey::from_base58("2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ")
                .unwrap(),
        );
        let sig = Sig::Ed25519(ed25519::Signature::from_base64(
            "fAH5Gor+8MtFzQZ++JaJO6U8JJ6+rkqKtPrRr/iufh3MYkoDGxmjzj6jCADQL+hkWBt8y8QzlgRkz0ixBcKHBw==",
        ).unwrap());
        let block = Blockstamp::from_string(
            "50-00001DAA4559FEDB8320D1040B0F22B631459F36F237A0D9BC1EB923C12A12E7",
        ).unwrap();
        let builder = TransactionDocumentBuilder {
            currency: "g1",
            blockstamp: &block,
            locktime: &0,
            issuers: &vec![pubkey],
            inputs: &vec![
                TransactionInput::parse_from_str(
                    "1000:0:D:2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ:1",
                ).expect("fail to parse input !"),
            ],
            unlocks: &vec![
                TransactionInputUnlocks::parse_from_str("0:SIG(0)")
                    .expect("fail to parse unlock !"),
            ],
            outputs: &vec![
                TransactionOutput::parse_from_str(
                    "1:0:SIG(Com8rJukCozHZyFao6AheSsfDQdPApxQRnz7QYFf64mm)",
                ).expect("fail to parse output !"),
                TransactionOutput::parse_from_str(
                    "999:0:SIG(2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ)",
                ).expect("fail to parse output !"),
            ],
            comment: "TEST",
        };
        builder.build_with_signature(vec![sig])
    }

    #[test]
    fn apply_one_tx() {
        // Get document of first g1 transaction
        let tx_doc = build_first_tx_of_g1();
        assert_eq!(tx_doc.verify_signatures(), VerificationResult::Valid());
        // Get pubkey of receiver
        let tortue_pubkey = PubKey::Ed25519(
            ed25519::PublicKey::from_base58("Com8rJukCozHZyFao6AheSsfDQdPApxQRnz7QYFf64mm")
                .unwrap(),
        );
        // Open currencys_db in memory mode
        let currency_dbs = CurrencyV10DBs::open_memory_mode();
        // Create first g1 DU for cgeek and tortue
        writers::dividend::create_du(
            &currency_dbs.du_db,
            &currency_dbs.balances_db,
            &SourceAmount(TxAmount(1000), TxBase(0)),
            &BlockId(1),
            &vec![tx_doc.issuers()[0], tortue_pubkey],
        ).expect("Fail to create first g1 DU !");
        // Check members balance
        let cgeek_new_balance = currency_dbs
            .balances_db
            .read(|db| {
                db.get(&TransactionOutputConditionGroup::Single(
                    TransactionOutputCondition::Sig(tx_doc.issuers()[0]),
                )).cloned()
            })
            .expect("Fail to read cgeek new balance")
            .expect("Error : cgeek is not referenced in balances_db !");
        assert_eq!(cgeek_new_balance.0, SourceAmount(TxAmount(1000), TxBase(0)));
        let tortue_new_balance = currency_dbs
            .balances_db
            .read(|db| {
                db.get(&TransactionOutputConditionGroup::Single(
                    TransactionOutputCondition::Sig(tortue_pubkey),
                )).cloned()
            })
            .expect("Fail to read receiver new balance")
            .expect("Error : receiver is not referenced in balances_db !");
        assert_eq!(
            tortue_new_balance.0,
            SourceAmount(TxAmount(1000), TxBase(0))
        );
        // Apply first g1 transaction
        apply_and_write_tx(
            &currency_dbs.tx_db,
            &currency_dbs.utxos_db,
            &currency_dbs.du_db,
            &currency_dbs.balances_db,
            &tx_doc,
        ).expect("Fail to apply first g1 tx");
        // Check issuer new balance
        let cgeek_new_balance = currency_dbs
            .balances_db
            .read(|db| {
                db.get(&TransactionOutputConditionGroup::Single(
                    TransactionOutputCondition::Sig(tx_doc.issuers()[0]),
                )).cloned()
            })
            .expect("Fail to read cgeek new balance")
            .expect("Error : cgeek is not referenced in balances_db !");
        assert_eq!(cgeek_new_balance.0, SourceAmount(TxAmount(999), TxBase(0)));

        // Check receiver new balance
        let receiver_new_balance = currency_dbs
            .balances_db
            .read(|db| {
                db.get(&TransactionOutputConditionGroup::Single(
                    TransactionOutputCondition::Sig(tortue_pubkey),
                )).cloned()
            })
            .expect("Fail to read receiver new balance")
            .expect("Error : receiver is not referenced in balances_db !");
        assert_eq!(
            receiver_new_balance.0,
            SourceAmount(TxAmount(1001), TxBase(0))
        );
    }
}
