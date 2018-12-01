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

use dubp_documents::v10::transaction::*;
use dubp_documents::BlockId;
use dup_crypto::keys::PubKey;
use sources::SourceAmount;
use std::collections::{HashMap, HashSet};
use *;

/// Apply UD creation in databases
pub fn create_du(
    du_db: &BinDB<UDsV10Datas>,
    balances_db: &BinDB<BalancesV10Datas>,
    du_amount: &SourceAmount,
    du_block_id: BlockId,
    members: &[PubKey],
    revert: bool,
) -> Result<(), DALError> {
    // Insert/Remove UD sources in UDsV10DB
    du_db.write(|db| {
        for pubkey in members {
            let mut pubkey_dus = db.get(&pubkey).cloned().unwrap_or_default();
            if revert {
                pubkey_dus.remove(&du_block_id);
            } else {
                pubkey_dus.insert(du_block_id);
            }
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
                    db.get(&UTXOConditionsGroup::Single(
                        TransactionOutputCondition::Sig(*pubkey),
                    ))
                    .cloned()
                    .unwrap_or_default(),
                );
            }
            members_balances
        })?;
    // Increase/Decrease members balance
    let members_balances: Vec<(PubKey, (SourceAmount, HashSet<UTXOIndexV10>))> = members_balances
        .iter()
        .map(|(pubkey, (balance, utxos_indexs))| {
            let new_balance = if revert {
                *balance - *du_amount
            } else {
                *balance + *du_amount
            };
            (*pubkey, (new_balance, utxos_indexs.clone()))
        })
        .collect();
    // Write new members balance
    balances_db.write(|db| {
        for (pubkey, (balance, utxos_indexs)) in members_balances {
            db.insert(
                UTXOConditionsGroup::Single(TransactionOutputCondition::Sig(pubkey)),
                (balance, utxos_indexs),
            );
        }
    })?;
    Ok(())
}
