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

// ! Module define graphql Node type and subtypes

use crate::context::QueryContext;
use juniper::Executor;
use juniper_from_schema::{QueryTrail, Walked};

pub struct Summary {
    pub software: &'static str,
    pub version: &'static str,
}

pub struct Node {
    pub summary: Summary,
}

impl super::super::NodeFields for Node {
    fn field_summary(
        &self,
        _executor: &Executor<'_, QueryContext>,
        _trail: &QueryTrail<'_, Summary, Walked>,
    ) -> &Summary {
        &self.summary
    }
}

impl super::super::SummaryFields for Summary {
    fn field_software(&self, _executor: &Executor<'_, QueryContext>) -> String {
        self.software.to_owned()
    }
    fn field_version(&self, _executor: &Executor<'_, QueryContext>) -> String {
        self.version.to_owned()
    }
}
