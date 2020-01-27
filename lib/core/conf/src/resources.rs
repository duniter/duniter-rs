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

//! Dunitrust resources usage configuration

#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Serialize)]
/// Ressource usage
#[serde(rename_all = "lowercase")]
pub enum ResourceUsage {
    /// Minimal use of the resource, to the detriment of performance
    Minimal,
    /// Trade-off between resource use and performance
    Medium,
    /// A performance-oriented trade-off, the use of the resource is slightly limited
    Large,
    /// No restrictions on the use of the resource, maximizes performance
    Infinite,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        ResourceUsage::Large
    }
}

#[derive(Debug, Default, Copy, Clone, Deserialize, PartialEq, Serialize)]
/// Ressources usage
pub struct ResourcesUsage {
    /// Cpu usage
    #[serde(default)]
    pub cpu_usage: ResourceUsage,
    /// Network usage
    #[serde(default)]
    pub network_usage: ResourceUsage,
    /// Memory usage
    #[serde(default)]
    pub memory_usage: ResourceUsage,
    /// Disk space usage
    #[serde(default)]
    pub disk_space_usage: ResourceUsage,
}
