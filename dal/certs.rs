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

extern crate duniter_documents;
extern crate durs_wot;

use duniter_documents::BlockId;
use durs_wot::NodeId;
use std::collections::HashMap;
use {BinDB, CertsExpirV10Datas, DALError};

/// Find certifications that emitted in indicated blocks expiring
pub fn find_expire_certs(
    certs_db: &BinDB<CertsExpirV10Datas>,
    blocks_expiring: Vec<BlockId>,
) -> Result<HashMap<(NodeId, NodeId), BlockId>, DALError> {
    Ok(certs_db.read(|db| {
        let mut all_expire_certs = HashMap::new();
        for expire_block_id in blocks_expiring {
            if let Some(expire_certs) = db.get(&expire_block_id) {
                for (source, target) in expire_certs {
                    all_expire_certs.insert((*source, *target), expire_block_id);
                }
            }
        }
        all_expire_certs
    })?)
}
