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

//! Rule BR_G100 - issuerIsMember

use super::{InvalidRuleError, RuleDatas, RuleNotSyncDatas};
use dubp_common_doc::traits::Document;
use durs_bc_db_reader::indexes::identities::DbIdentityState;
use durs_bc_db_reader::BcDbInReadTx;
use rules_engine::rule::{Rule, RuleFn, RuleNumber};
use rules_engine::ProtocolVersion;
use unwrap::unwrap;

#[inline]
pub fn rule<'d, 'db, DB: BcDbInReadTx>(
) -> Rule<RuleDatas<'d>, RuleNotSyncDatas<'db, DB>, InvalidRuleError> {
    unwrap!(Rule::new(
        RuleNumber(100),
        maplit::btreemap![
            ProtocolVersion(10) => RuleFn::RefMut(v10),
        ]
    ))
}

fn v10<DB: BcDbInReadTx>(
    datas: &mut RuleDatas,
    not_sync_datas: &mut RuleNotSyncDatas<DB>,
) -> Result<(), InvalidRuleError> {
    let RuleDatas { ref block, .. } = datas;
    let RuleNotSyncDatas { ref db } = not_sync_datas;

    if let Some(idty_state) = db.get_idty_state_by_pubkey(&block.issuers()[0])? {
        if let DbIdentityState::Member(_) = idty_state {
            Ok(())
        } else {
            Err(InvalidRuleError::NotMemberIssuer(idty_state))
        }
    } else {
        Err(InvalidRuleError::IssuerNotExist)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use dubp_block_doc::BlockDocument;
    use durs_bc_db_reader::MockBcDbInReadTx;
    use mockall::predicate::eq;

    #[test]
    fn test_br_g100_issuer_not_exist() {
        let pubkey = dup_crypto_tests_tools::mocks::pubkey('A');
        let block = BlockDocument::V10(dubp_blocks_tests_tools::mocks::gen_empty_issued_block_v10(
            pubkey,
        ));

        let mut mock_db = MockBcDbInReadTx::new();
        mock_db
            .expect_get_idty_state_by_pubkey()
            .times(1)
            .with(eq(pubkey))
            .returning(|_| Ok(None));

        let mut datas = RuleDatas {
            block: &block,
            previous_block: &block,
        };
        let mut not_sync_datas = RuleNotSyncDatas { db: &mock_db };

        assert_eq!(
            Err(InvalidRuleError::IssuerNotExist),
            v10(&mut datas, &mut not_sync_datas)
        )
    }

    #[test]
    fn test_br_g100_issuer_is_member() {
        let pubkey = dup_crypto_tests_tools::mocks::pubkey('A');
        let block = BlockDocument::V10(dubp_blocks_tests_tools::mocks::gen_empty_issued_block_v10(
            pubkey,
        ));

        let mut mock_db = MockBcDbInReadTx::new();
        mock_db
            .expect_get_idty_state_by_pubkey()
            .times(1)
            .with(eq(pubkey))
            .returning(|_| Ok(Some(DbIdentityState::Member(vec![1]))));

        let mut datas = RuleDatas {
            block: &block,
            previous_block: &block,
        };
        let mut not_sync_datas = RuleNotSyncDatas { db: &mock_db };

        assert_eq!(Ok(()), v10(&mut datas, &mut not_sync_datas))
    }
}
