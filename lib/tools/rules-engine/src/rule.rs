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

//! Rules engine : rules

use crate::{EngineError, ProtocolVersion};
use failure::Fail;
use std::collections::BTreeMap;
use std::fmt::Debug;

#[derive(Copy, Clone, Debug, Ord, PartialEq, PartialOrd, Eq, Hash)]
/// Rule number
pub struct RuleNumber(pub usize);

impl std::fmt::Display for RuleNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Rule error
#[derive(Debug, Eq, Fail, PartialEq)]
#[fail(display = "An error occurred with rule n°{} : {}", rule_number, cause)]
pub struct RuleError<E: Eq + Fail + PartialEq> {
    /// Rule number
    pub rule_number: RuleNumber,
    /// Cause of the error
    pub cause: E,
}

/// Rule immutable execution function
pub type RuleFnRef<D, E> = fn(&D) -> Result<(), E>;

/// Rule mutable execution function
pub type RuleFnRefMut<D, E> = fn(&mut D) -> Result<(), E>;

/// Rule execution function
pub enum RuleFn<D, E> {
    Ref(RuleFnRef<D, E>),
    RefMut(RuleFnRefMut<D, E>),
}

#[derive(Debug, Copy, Clone, Eq, Fail, PartialEq)]
#[fail(
    display = "Fatal error: rules-engine: try to create rule n°{} without implementation !",
    rule_number
)]
pub struct RuleWithoutImpl {
    pub rule_number: RuleNumber,
}

/// Rule
pub struct Rule<D: Debug, E: Eq + Fail + PartialEq> {
    /// Dictionary of the different versions of the rule execution function
    rule_versions: BTreeMap<ProtocolVersion, RuleFn<D, E>>,
}

impl<D: Debug, E: Eq + Fail + PartialEq> Rule<D, E> {
    /// Create new rule
    pub fn new(
        rule_number: RuleNumber,
        rule_versions: BTreeMap<ProtocolVersion, RuleFn<D, E>>,
    ) -> Result<Self, RuleWithoutImpl> {
        if rule_versions.is_empty() {
            Err(RuleWithoutImpl { rule_number })
        } else {
            Ok(Rule { rule_versions })
        }
    }
    /// Executes the correct version of the rule
    pub fn execute(
        &self,
        protocol_version: ProtocolVersion,
        rule_number: RuleNumber,
        rule_datas: &D,
    ) -> Result<(), EngineError<E>> {
        let rule_opt: Option<(&ProtocolVersion, &RuleFn<D, E>)> =
            self.rule_versions.range(..=protocol_version).last();
        if let Some((_, rule_fn)) = rule_opt {
            match rule_fn {
                RuleFn::Ref(rule_fn_ref) => rule_fn_ref(rule_datas).map_err(|err| {
                    EngineError::RuleError(RuleError {
                        rule_number,
                        cause: err,
                    })
                }),
                RuleFn::RefMut(_) => Err(EngineError::MutRuleInPar {
                    rule_number,
                    protocol_version,
                }),
            }
        } else {
            Err(EngineError::RuleTooRecent {
                rule_number,
                protocol_version,
            })
        }
    }
    /// Executes the correct version of the rule
    pub fn execute_mut(
        &self,
        protocol_version: ProtocolVersion,
        rule_number: RuleNumber,
        rule_datas: &mut D,
    ) -> Result<(), EngineError<E>> {
        let rule_opt: Option<(&ProtocolVersion, &RuleFn<D, E>)> =
            self.rule_versions.range(..=protocol_version).last();
        if let Some((_, rule_fn)) = rule_opt {
            match rule_fn {
                RuleFn::Ref(rule_fn_ref) => rule_fn_ref(rule_datas),
                RuleFn::RefMut(rule_fn_ref_mut) => rule_fn_ref_mut(rule_datas),
            }
            .map_err(|err| {
                EngineError::RuleError(RuleError {
                    rule_number,
                    cause: err,
                })
            })
        } else {
            Err(EngineError::RuleTooRecent {
                rule_number,
                protocol_version,
            })
        }
    }
}
