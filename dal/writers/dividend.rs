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

use duniter_crypto::keys::PubKey;
use duniter_documents::blockchain::v10::documents::transaction::*;
use duniter_documents::BlockId;
use sources::SourceAmount;
use std::collections::{HashMap, HashSet};
use *;

pub fn create_du(
    du_db: &BinFileDB<DUsV10Datas>,
    balances_db: &BinFileDB<BalancesV10Datas>,
    du_amount: &SourceAmount,
    du_block_id: &BlockId,
    members: &[PubKey],
) -> Result<(), DALError> {
    // Insert DU sources in DUsV10DB
    du_db.write(|db| {
        for pubkey in members {
            let mut pubkey_dus = db.get(&pubkey).cloned().unwrap_or_default();
            pubkey_dus.insert(*du_block_id);
            db.insert(*pubkey, pubkey_dus);
        }
    })?;
    // Get members balances
    let members_balances: HashMap<PubKey, (SourceAmount, HashSet<UTXOIndexV10>)> = balances_db
        .read(|db| {
            let mut members_balances = HashMap::new();
            for pubkey in members {
                members_balances.insert(
                    *pubkey,
                    db.get(&TransactionOutputConditionGroup::Single(
                        TransactionOutputCondition::Sig(*pubkey),
                    )).cloned()
                        .unwrap_or_default(),
                );
            }
            members_balances
        })?;
    // Increment members balance
    let members_balances: Vec<(PubKey, (SourceAmount, HashSet<UTXOIndexV10>))> = members_balances
        .iter()
        .map(|(pubkey, (balance, utxos_indexs))| {
            let new_balance = *balance + *du_amount;
            (*pubkey, (new_balance, utxos_indexs.clone()))
        })
        .collect();
    // Write new members balance
    balances_db.write(|db| {
        for (pubkey, (balance, utxos_indexs)) in members_balances {
            db.insert(
                TransactionOutputConditionGroup::Single(TransactionOutputCondition::Sig(pubkey)),
                (balance, utxos_indexs),
            );
        }
    })?;
    Ok(())
}
