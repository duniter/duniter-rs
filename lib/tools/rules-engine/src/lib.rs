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

//! Rules engine

#![deny(
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces
)]

pub mod rule;

use failure::Fail;
use rayon::prelude::*;
use rule::{Rule, RuleError, RuleNumber};
use std::collections::BTreeMap;

#[derive(Copy, Clone, Debug, Ord, PartialEq, PartialOrd, Eq, Hash)]
pub struct ProtocolVersion(pub usize);

impl std::fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProtocolRules(pub Vec<RulesGroup>);

impl From<Vec<usize>> for ProtocolRules {
    fn from(rules_numbers: Vec<usize>) -> Self {
        ProtocolRules(vec![RulesGroup::Ser(
            rules_numbers.into_iter().map(RuleNumber).collect(),
        )])
    }
}

impl From<Vec<RulesGroup>> for ProtocolRules {
    fn from(rules_groups: Vec<RulesGroup>) -> Self {
        ProtocolRules(rules_groups)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Protocol
pub struct Protocol(BTreeMap<ProtocolVersion, ProtocolRules>);

impl Protocol {
    /// Create new protocol
    /// protocol_versions: Dictionary of rules to be applied for each version of the protocol (rules will be applied in the order provided)
    pub fn new(protocol_versions: BTreeMap<ProtocolVersion, ProtocolRules>) -> Self {
        Protocol(protocol_versions)
    }

    /// Get specific protocol version
    pub fn get(&self, protocol_version: ProtocolVersion) -> Option<&ProtocolRules> {
        self.0.get(&protocol_version)
    }
}

/// Rules groups
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RulesGroup {
    /// In serial
    Ser(Vec<RuleNumber>),
    /// In parallel
    Par(Vec<RulesGroup>),
}

impl RulesGroup {
    #[inline]
    /// Create singleton rules group
    pub fn s1(rule_number: usize) -> Self {
        RulesGroup::Ser(vec![RuleNumber(rule_number)])
    }
    #[inline]
    /// Create serial set of rules
    pub fn ser(rules_numbers: Vec<usize>) -> Self {
        RulesGroup::Ser(rules_numbers.into_iter().map(RuleNumber).collect())
    }
    #[inline]
    /// Create parallel set of rules
    pub fn pr(rules_numbers: Vec<usize>) -> Self {
        RulesGroup::Par(rules_numbers.into_iter().map(RulesGroup::s1).collect())
    }
}

/// Rules engine
pub struct RulesEngine<D: Sync, DNotSync, E: Eq + Fail + PartialEq> {
    /// All rules
    all_rules: BTreeMap<RuleNumber, Rule<D, DNotSync, E>>,
}

impl<D: Sync, DNotSync, E: Eq + Fail + PartialEq> RulesEngine<D, DNotSync, E> {
    /// Create new rules engine
    pub fn new(all_rules: BTreeMap<RuleNumber, Rule<D, DNotSync, E>>) -> Self {
        RulesEngine { all_rules }
    }

    fn apply_rules_group_ref(
        &self,
        protocol_version: ProtocolVersion,
        rules_group: RulesGroup,
        rule_datas: &D,
    ) -> Result<(), EngineError<E>> {
        match rules_group {
            RulesGroup::Ser(rules_numbers) => rules_numbers
                .into_iter()
                .map(|rule_number| self.apply_rule_ref(protocol_version, rule_number, rule_datas))
                .collect(),
            RulesGroup::Par(rules_group) => rules_group
                .into_par_iter()
                .map(|rg| self.apply_rules_group_ref(protocol_version, rg, rule_datas))
                .collect(),
        }
    }

    fn apply_rule_ref(
        &self,
        protocol_version: ProtocolVersion,
        rule_number: RuleNumber,
        rule_datas: &D,
    ) -> Result<(), EngineError<E>> {
        if let Some(rule) = self.all_rules.get(&rule_number) {
            rule.execute(protocol_version, rule_number, rule_datas)
        } else {
            Err(EngineError::RuleNotExist {
                rule_number,
                protocol_version,
            })
        }
    }

    fn apply_rule_mut(
        &self,
        protocol_version: ProtocolVersion,
        rule_number: RuleNumber,
        rule_datas: &mut D,
        rule_datas_not_sync: &mut DNotSync,
    ) -> Result<(), EngineError<E>> {
        if let Some(rule) = self.all_rules.get(&rule_number) {
            rule.execute_mut(
                protocol_version,
                rule_number,
                rule_datas,
                rule_datas_not_sync,
            )
        } else {
            Err(EngineError::RuleNotExist {
                rule_number,
                protocol_version,
            })
        }
    }

    /// Apply a specific version of the protocol
    pub fn apply_protocol(
        &self,
        protocol: Protocol,
        protocol_version: ProtocolVersion,
        rule_datas: &mut D,
        rule_datas_not_sync: &mut DNotSync,
    ) -> Result<(), EngineError<E>> {
        if let Some(protocol_rules) = protocol.get(protocol_version) {
            for rules_group in &protocol_rules.0 {
                let result: Result<(), EngineError<E>> = match rules_group {
                    RulesGroup::Ser(rules_numbers) => rules_numbers
                        .iter()
                        .map(|rule_number| {
                            self.apply_rule_mut(
                                protocol_version,
                                *rule_number,
                                rule_datas,
                                rule_datas_not_sync,
                            )
                        })
                        .collect(),
                    RulesGroup::Par(rules_group) => rules_group
                        .par_iter()
                        .map(|rg| {
                            self.apply_rules_group_ref(protocol_version, rg.clone(), rule_datas)
                        })
                        .collect(),
                };
                if let Err(err) = result {
                    return Err(err);
                }
            }

            Ok(())
        } else {
            Err(EngineError::ProtocolVersionNotExist { protocol_version })
        }
    }
}

/// Protocol error
#[derive(Debug, Eq, Fail, PartialEq)]
pub enum EngineError<E: Eq + Fail + PartialEq> {
    #[fail(display = "{}", _0)]
    /// Rule Error
    RuleError(RuleError<E>),
    #[fail(display = "protocol V{} not exist", protocol_version)]
    /// The protocol version does not exist
    ProtocolVersionNotExist {
        /// Protocole version
        protocol_version: ProtocolVersion,
    },
    #[fail(
        display = "Rule n°{} not exist (require by protocol V{})",
        rule_number, protocol_version
    )]
    /// A rule required by the protocol version does not exist
    RuleNotExist {
        /// Rule number
        rule_number: RuleNumber,
        /// Protocole version
        protocol_version: ProtocolVersion,
    },
    #[fail(
        display = "Rule n°{} is mutable and called in parallel in the V{} protocol, this is prohibited.
    A rule can be mutable or called in parallel but not both at the same time.",
        rule_number, protocol_version
    )]
    /// Calling a mutable rule in a part executed in parallel
    MutRuleInPar {
        /// Rule number
        rule_number: RuleNumber,
        /// Protocole version
        protocol_version: ProtocolVersion,
    },
    #[fail(
        display = "Rule n°{} does not exist in a version less than or equal to the protocol version (V{})",
        rule_number, protocol_version
    )]
    /// Calling a rule too recent
    RuleTooRecent {
        /// Rule number
        rule_number: RuleNumber,
        /// Protocole version
        protocol_version: ProtocolVersion,
    },
}

#[cfg(test)]
mod tests {

    use super::rule::*;
    use super::*;
    use maplit::btreemap;

    #[derive(Debug)]
    struct Datas {
        i: usize,
    }

    #[derive(Debug)]
    struct DatasNotSync {
        j: usize,
    }

    #[derive(Debug, Eq, Fail, PartialEq)]
    #[fail(display = "")]
    struct Error {}

    fn r2_v1(datas: &mut Datas, datas_not_sync: &mut DatasNotSync) -> Result<(), Error> {
        if datas.i == 0 && datas_not_sync.j < 2 {
            datas.i += 1;
            Ok(())
        } else {
            Err(Error {})
        }
    }

    fn r3_v2(datas: &Datas) -> Result<(), Error> {
        if datas.i == 1 {
            Ok(())
        } else {
            Err(Error {})
        }
    }

    fn get_test_engine() -> RulesEngine<Datas, DatasNotSync, Error> {
        let all_rules: BTreeMap<RuleNumber, Rule<Datas, DatasNotSync, Error>> = btreemap![
            RuleNumber(2) => Rule::new(RuleNumber(2), btreemap![
                ProtocolVersion(1) => RuleFn::RefMut(r2_v1),
            ]).expect("Fail to create rule n°2"),
            RuleNumber(3) => Rule::new(RuleNumber(3), btreemap![
                ProtocolVersion(2) => RuleFn::Ref(r3_v2),
            ]).expect("Fail to create rule n°2"),
        ];

        RulesEngine::new(all_rules)
    }

    #[test]
    fn rule_without_impl() {
        if let Err(err) = Rule::<Datas, DatasNotSync, Error>::new(RuleNumber(1), btreemap![]) {
            assert_eq!(
                RuleWithoutImpl {
                    rule_number: RuleNumber(1),
                },
                err,
            )
        } else {
            panic!("Rule creation must be fail")
        }

        println!("{}", ProtocolVersion(1));
        println!("{}", RuleNumber(1));
    }

    #[test]
    fn protocol_empty() -> Result<(), EngineError<Error>> {
        let engine = get_test_engine();

        let mut datas = Datas { i: 0 };
        let mut datas_not_sync = DatasNotSync { j: 1 };

        let protocol_empty: Protocol = Protocol::new(btreemap![
            ProtocolVersion(1) => Vec::<usize>::with_capacity(0).into()
        ]);

        engine.apply_protocol(
            protocol_empty,
            ProtocolVersion(1),
            &mut datas,
            &mut datas_not_sync,
        )
    }

    #[test]
    fn protocol_version_not_exist() {
        let engine = get_test_engine();

        let mut datas = Datas { i: 0 };
        let mut datas_not_sync = DatasNotSync { j: 1 };

        let protocol_empty: Protocol = Protocol::new(btreemap![
            ProtocolVersion(1) => Vec::<usize>::with_capacity(0).into()
        ]);

        assert_eq!(
            Err(EngineError::ProtocolVersionNotExist {
                protocol_version: ProtocolVersion(2),
            }),
            engine.apply_protocol(
                protocol_empty,
                ProtocolVersion(2),
                &mut datas,
                &mut datas_not_sync
            )
        )
    }

    #[test]
    fn rule_not_exist() {
        let engine = get_test_engine();

        let mut datas = Datas { i: 0 };
        let mut datas_not_sync = DatasNotSync { j: 1 };

        let protocol: Protocol = Protocol::new(btreemap![
            ProtocolVersion(1) => vec![1usize].into()
        ]);

        assert_eq!(
            Err(EngineError::RuleNotExist {
                rule_number: RuleNumber(1),
                protocol_version: ProtocolVersion(1)
            }),
            engine.apply_protocol(
                protocol,
                ProtocolVersion(1),
                &mut datas,
                &mut datas_not_sync
            )
        );

        let mut datas = Datas { i: 0 };
        let mut datas_not_sync = DatasNotSync { j: 1 };

        let protocol_par: Protocol = Protocol::new(btreemap![
            ProtocolVersion(1) => vec![RulesGroup::pr(vec![1usize])].into()
        ]);

        assert_eq!(
            Err(EngineError::RuleNotExist {
                rule_number: RuleNumber(1),
                protocol_version: ProtocolVersion(1)
            }),
            engine.apply_protocol(
                protocol_par,
                ProtocolVersion(1),
                &mut datas,
                &mut datas_not_sync
            )
        );
    }

    #[test]
    fn rule_fail() {
        let engine = get_test_engine();

        let mut datas = Datas { i: 1 };
        let mut datas_not_sync = DatasNotSync { j: 1 };

        let protocol: Protocol = Protocol::new(btreemap![
            ProtocolVersion(1) => vec![2usize].into()
        ]);

        assert_eq!(
            Err(EngineError::RuleError(RuleError {
                rule_number: RuleNumber(2),
                cause: Error {},
            })),
            engine.apply_protocol(
                protocol,
                ProtocolVersion(1),
                &mut datas,
                &mut datas_not_sync
            )
        )
    }

    #[test]
    fn par_rule_fail() {
        let engine = get_test_engine();

        let mut datas = Datas { i: 0 };
        let mut datas_not_sync = DatasNotSync { j: 1 };

        let protocol: Protocol = Protocol::new(btreemap![
            ProtocolVersion(2) => vec![RulesGroup::pr(vec![3usize])].into()
        ]);

        assert_eq!(
            Err(EngineError::RuleError(RuleError {
                rule_number: RuleNumber(3),
                cause: Error {},
            })),
            engine.apply_protocol(
                protocol,
                ProtocolVersion(2),
                &mut datas,
                &mut datas_not_sync
            )
        )
    }

    #[test]
    fn rule_too_recent() {
        let engine = get_test_engine();

        let mut datas = Datas { i: 0 };
        let mut datas_not_sync = DatasNotSync { j: 1 };

        let protocol: Protocol = Protocol::new(btreemap![
            ProtocolVersion(1) => vec![2usize, 3].into()
        ]);

        assert_eq!(
            Err(EngineError::RuleTooRecent {
                protocol_version: ProtocolVersion(1),
                rule_number: RuleNumber(3),
            }),
            engine.apply_protocol(
                protocol,
                ProtocolVersion(1),
                &mut datas,
                &mut datas_not_sync
            )
        )
    }

    #[test]
    fn par_rule_too_recent() {
        let engine = get_test_engine();

        let mut datas = Datas { i: 0 };
        let mut datas_not_sync = DatasNotSync { j: 1 };

        let protocol: Protocol = Protocol::new(btreemap![
            ProtocolVersion(1) => vec![RulesGroup::pr(vec![3])].into()
        ]);

        assert_eq!(
            Err(EngineError::RuleTooRecent {
                protocol_version: ProtocolVersion(1),
                rule_number: RuleNumber(3),
            }),
            engine.apply_protocol(
                protocol,
                ProtocolVersion(1),
                &mut datas,
                &mut datas_not_sync
            )
        )
    }

    #[test]
    fn mut_rule_in_par_protocol() {
        let engine = get_test_engine();

        let mut datas = Datas { i: 1 };
        let mut datas_not_sync = DatasNotSync { j: 1 };

        let protocol: Protocol = Protocol::new(btreemap![
            ProtocolVersion(2) => vec![RulesGroup::pr(vec![2usize, 3])].into()
        ]);

        assert_eq!(
            Err(EngineError::MutRuleInPar {
                protocol_version: ProtocolVersion(2),
                rule_number: RuleNumber(2),
            }),
            engine.apply_protocol(
                protocol,
                ProtocolVersion(2),
                &mut datas,
                &mut datas_not_sync
            )
        )
    }

    #[test]
    fn protocol_success() -> Result<(), EngineError<Error>> {
        let engine = get_test_engine();

        let mut datas = Datas { i: 0 };
        let mut datas_not_sync = DatasNotSync { j: 1 };

        let protocol: Protocol = Protocol::new(btreemap![
            ProtocolVersion(2) => vec![2usize, 3].into()
        ]);

        engine.apply_protocol(
            protocol,
            ProtocolVersion(2),
            &mut datas,
            &mut datas_not_sync,
        )
    }
}
