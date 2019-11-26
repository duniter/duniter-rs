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

//! Transactions stored indexes: write requests.

use dubp_user_docs::documents::transaction::*;
use durs_bc_db_reader::constants::*;
use durs_bc_db_reader::{from_db_value, DbValue};
use durs_common_tools::fatal_error;

use crate::*;
use dubp_indexes::sindex::{SourceUniqueIdV10, UniqueIdUTXOv10};
use durs_bc_db_reader::indexes::sources::UTXOV10;

#[derive(Debug)]
/// Transaction error
pub enum TxError {
    /// UnkonwError
    UnkonwError(),
    /// DbError
    DbError(DbError),
}

impl From<DbError> for TxError {
    fn from(err: DbError) -> TxError {
        TxError::DbError(err)
    }
}

/// Apply transaction backwards
pub fn revert_tx<S: std::hash::BuildHasher>(
    db: &Db,
    w: &mut DbWriter,
    tx_doc: &TransactionDocument,
    block_consumed_sources: &mut HashMap<UniqueIdUTXOv10, TransactionOutput, S>,
) -> Result<(), DbError> {
    let tx_hash = tx_doc
        .get_hash_opt()
        .unwrap_or_else(|| tx_doc.compute_hash());

    // Index created utxos
    let created_utxos: Vec<UTXOV10> = tx_doc
        .get_outputs()
        .iter()
        .enumerate()
        .map(|(tx_index, output)| {
            UTXOV10(
                UniqueIdUTXOv10(tx_hash, OutputIndex(tx_index)),
                output.clone(),
            )
        })
        .collect();
    // Remove created UTXOs
    for utxo_v10 in created_utxos {
        let utxo_id_bytes: Vec<u8> = utxo_v10.0.into();
        db.get_store(UTXOS).delete(w.as_mut(), &utxo_id_bytes)?;
    }
    // Index consumed sources
    let consumed_sources_ids: HashSet<SourceUniqueIdV10> = tx_doc
        .get_inputs()
        .iter()
        .map(|input| match *input {
            TransactionInput::D(_tx_amout, _tx_amout_base, pubkey, block_id) => {
                SourceUniqueIdV10::UD(pubkey, block_id)
            }
            TransactionInput::T(_tx_amout, _tx_amout_base, hash, tx_index) => {
                SourceUniqueIdV10::UTXO(UniqueIdUTXOv10(hash, tx_index))
            }
        })
        .collect();
    // Recreate consumed sources
    for s_index in consumed_sources_ids {
        if let SourceUniqueIdV10::UTXO(utxo_id) = s_index {
            if let Some(utxo_content) = block_consumed_sources.remove(&utxo_id) {
                let utxo_id_bytes: Vec<u8> = utxo_id.into();
                let utxo_content_bytes = durs_dbs_tools::to_bytes(&utxo_content)?;
                db.get_store(UTXOS).put(
                    w.as_mut(),
                    &utxo_id_bytes,
                    &DbValue::Blob(&utxo_content_bytes[..]),
                )?;
            } else {
                fatal_error!(
                    "Revert invalid block: utxo {:?} not found in block.",
                    utxo_id
                );
            }
        } else if let SourceUniqueIdV10::UD(pubkey, block_id) = s_index {
            db.get_multi_store(DIVIDENDS).put(
                w.as_mut(),
                &pubkey.to_bytes_vector(),
                &DbValue::U64(u64::from(block_id.0)),
            )?;
        }
    }
    Ok(())
}

/// Apply and write transaction
pub fn apply_and_write_tx(
    db: &Db,
    w: &mut DbWriter,
    tx_doc: &TransactionDocument,
    in_fork_window: bool,
) -> Result<(), DbError> {
    let tx_hash = tx_doc
        .get_hash_opt()
        .unwrap_or_else(|| tx_doc.compute_hash());
    // Index consumed sources
    let consumed_sources_ids: HashSet<SourceUniqueIdV10> = tx_doc
        .get_inputs()
        .iter()
        .map(|input| match *input {
            TransactionInput::D(_tx_amout, _tx_amout_base, pubkey, block_id) => {
                SourceUniqueIdV10::UD(pubkey, block_id)
            }
            TransactionInput::T(_tx_amout, _tx_amout_base, hash, tx_index) => {
                SourceUniqueIdV10::UTXO(UniqueIdUTXOv10(hash, tx_index))
            }
        })
        .collect();
    if in_fork_window {
        // Persist consumed sources (for future revert)
        let consumed_sources = consumed_sources_ids
            .iter()
            .filter_map(|source_id| {
                if let SourceUniqueIdV10::UTXO(utxo_id) = source_id {
                    Some(utxo_id)
                } else {
                    None
                }
            })
            .map(|utxo_id| {
                let utxo_id_bytes: Vec<u8> = (*utxo_id).into();
                if let Some(value) = db.get_store(UTXOS).get(w.as_ref(), &utxo_id_bytes)? {
                    let utxo_content: TransactionOutput = from_db_value(value)?;
                    Ok((*utxo_id, utxo_content))
                } else {
                    fatal_error!("Try to persist unexist consumed source.");
                }
            })
            .collect::<Result<HashMap<UniqueIdUTXOv10, TransactionOutput>, DbError>>()?;
        let consumed_sources_bytes = durs_dbs_tools::to_bytes(&consumed_sources)?;
        let block_number =
            durs_bc_db_reader::current_metadata::get_current_blockstamp(&BcDbRwWithWriter {
                db,
                w,
            })?
            .unwrap_or_default()
            .id;
        db.get_int_store(CONSUMED_UTXOS).put(
            w.as_mut(),
            block_number.0,
            &DbValue::Blob(&consumed_sources_bytes[..]),
        )?;
    }
    // Remove consumed sources
    for source_id in consumed_sources_ids {
        if let SourceUniqueIdV10::UTXO(utxo_id) = source_id {
            let uxtx_id_bytes: Vec<u8> = utxo_id.into();
            db.get_store(UTXOS)
                .delete(w.as_mut(), uxtx_id_bytes)
                .map_err(|e| {
                    warn!("Fail to delete UTXO({:?}).", utxo_id);
                    e
                })?;
        } else if let SourceUniqueIdV10::UD(pubkey, block_id) = source_id {
            db.get_multi_store(DIVIDENDS)
                .delete(
                    w.as_mut(),
                    &pubkey.to_bytes_vector(),
                    &DbValue::U64(u64::from(block_id.0)),
                )
                .map_err(|e| {
                    warn!("Fail to delete UD({}-#{}).", pubkey, block_id);
                    e
                })?;
        }
    }
    let created_utxos: Vec<UTXOV10> = tx_doc
        .get_outputs()
        .iter()
        .enumerate()
        .map(|(tx_index, output)| {
            UTXOV10(
                UniqueIdUTXOv10(tx_hash, OutputIndex(tx_index)),
                output.clone(),
            )
        })
        .collect();
    // Insert created UTXOs
    for utxo_v10 in created_utxos {
        let utxo_id_bytes: Vec<u8> = utxo_v10.0.into();
        let utxo_value_bytes = durs_dbs_tools::to_bytes(&utxo_v10.1)?;
        db.get_store(UTXOS).put(
            w.as_mut(),
            utxo_id_bytes,
            &DbValue::Blob(&utxo_value_bytes[..]),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dubp_common_doc::traits::{Document, DocumentBuilder};
    use dubp_common_doc::BlockHash;
    use durs_bc_db_reader::current_metadata::CurrentMetaDataKey;
    use durs_bc_db_reader::indexes::sources::SourceAmount;
    use std::str::FromStr;

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
        )
        .unwrap();
        let builder = TransactionDocumentBuilder {
            currency: "g1",
            blockstamp: &block,
            locktime: &0,
            issuers: &vec![pubkey],
            inputs: &vec![TransactionInput::from_str(
                "1000:0:D:2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ:1",
            )
            .expect("fail to parse input !")],
            unlocks: &vec![
                TransactionInputUnlocks::from_str("0:SIG(0)").expect("fail to parse unlock !")
            ],
            outputs: &vec![
                TransactionOutput::from_str(
                    "1:0:SIG(Com8rJukCozHZyFao6AheSsfDQdPApxQRnz7QYFf64mm)",
                )
                .expect("fail to parse output !"),
                TransactionOutput::from_str(
                    "999:0:SIG(2ny7YAdmzReQxAayyJZsyVYwYhVyax2thKcGknmQy5nQ)",
                )
                .expect("fail to parse output !"),
            ],
            comment: "TEST",
            hash: None,
        };
        builder.build_with_signature(vec![sig])
    }

    #[test]
    fn apply_and_revert_one_tx() -> Result<(), DbError> {
        // Get document of first g1 transaction
        let tx_doc = build_first_tx_of_g1();
        assert_eq!(tx_doc.verify_signatures(), Ok(()));
        // Get pubkey of receiver
        let tortue_pubkey = PubKey::Ed25519(
            ed25519::PublicKey::from_base58("Com8rJukCozHZyFao6AheSsfDQdPApxQRnz7QYFf64mm")
                .unwrap(),
        );
        // Open blockchain DB
        let db = crate::tests::open_tmp_db()?;
        // Create first g1 UD for cgeek and tortue
        db.write(|mut w| {
            crate::indexes::dividends::create_du(
                &db,
                &mut w,
                &SourceAmount(TxAmount(1000), TxBase(0)),
                BlockNumber(1),
                &vec![tx_doc.issuers()[0], tortue_pubkey],
                false,
            )?;
            Ok(w)
        })?;

        db.write(|mut w| {
            // Update current blockstamp
            let new_current_blockstamp_bytes: Vec<u8> = Blockstamp {
                id: BlockNumber(52),
                hash: BlockHash(Hash::default()),
            }
            .into();
            db.get_int_store(CURRENT_METADATA).put(
                w.as_mut(),
                CurrentMetaDataKey::CurrentBlockstamp.to_u32(),
                &DbValue::Blob(&new_current_blockstamp_bytes),
            )?;
            // Apply first g1 transaction
            apply_and_write_tx(&db, &mut w, &tx_doc, true)?;
            Ok(w)
        })?;
        // Check new UTXOS
        // TODO
        //db.get_store(UTXOS).iter_start()?
        let count_utxos = db.read(|r| Ok(db.get_store(UTXOS).iter_start(&r)?.count()))?;
        assert_eq!(2, count_utxos);

        // Revert first g1 tx
        db.write(|mut w| {
            if let Some(mut block_consumed_sources_opt) =
                durs_bc_db_reader::indexes::sources::get_block_consumed_sources_(
                    &BcDbRwWithWriter { db: &db, w: &w },
                    BlockNumber(52),
                )?
            {
                revert_tx(&db, &mut w, &tx_doc, &mut block_consumed_sources_opt)?;
            } else {
                panic!(dbg!("No block consumed sources"));
            }
            Ok(w)
        })?;

        // UTXOS must be empty
        let count_utxos = db.read(|r| Ok(db.get_store(UTXOS).iter_start(&r)?.count()))?;
        assert_eq!(0, count_utxos);

        Ok(())
    }
}
