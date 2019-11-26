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

//! Durs-core cli : dbex subcommands.

use crate::commands::DursExecutableCoreCommand;
use crate::dbex;
use crate::errors::DursCoreError;
use crate::DursCore;
use durs_bc::dbex::{DbExBcQuery, DbExQuery, DbExTxQuery, DbExWotQuery};
use durs_conf::DuRsConf;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "dbex", setting(structopt::clap::AppSettings::ColoredHelp))]
/// durs databases explorer
pub struct DbExOpt {
    #[structopt(short = "c", long = "csv")]
    /// csv output
    pub csv: bool,
    #[structopt(subcommand)]
    /// DbExSubCommand
    pub subcommand: DbExSubCommand,
}

#[derive(StructOpt, Debug, Clone)]
/// dbex subcommands
pub enum DbExSubCommand {
    /// Pubkeys’ balances explorer
    #[structopt(name = "balance", setting(structopt::clap::AppSettings::ColoredHelp))]
    BalanceOpt(BalanceOpt),
    /// Display blocks current frame
    #[structopt(name = "blocks", setting(structopt::clap::AppSettings::ColoredHelp))]
    BlocksOpt(BlocksOpt),
    /// Web of Trust distances explorer
    #[structopt(name = "distance", setting(structopt::clap::AppSettings::ColoredHelp))]
    DistanceOpt(DistanceOpt),
    /// Forks tree explorer
    #[structopt(name = "forks", setting(structopt::clap::AppSettings::ColoredHelp))]
    ForksOpt(ForksOpt),
    /// Member explorer
    #[structopt(name = "member", setting(structopt::clap::AppSettings::ColoredHelp))]
    MemberOpt(MemberOpt),
    /// Members explorer
    #[structopt(name = "members")]
    MembersOpt(MembersOpt),
}

#[derive(StructOpt, Debug, Copy, Clone)]
/// DistanceOpt
pub struct DistanceOpt {
    #[structopt(short = "r", long = "reverse")]
    /// reverse order
    pub reverse: bool,
}

#[derive(StructOpt, Debug, Copy, Clone)]
/// ForksOpt
pub struct ForksOpt {}

#[derive(StructOpt, Debug, Copy, Clone)]
/// MembersOpt
pub struct MembersOpt {
    #[structopt(short = "r", long = "reverse")]
    /// reverse order
    pub reverse: bool,
    #[structopt(short = "e", long = "expire")]
    /// show members expire date
    pub expire: bool,
}

#[derive(StructOpt, Debug, Clone)]
/// MemberOpt
pub struct MemberOpt {
    /// choose member uid
    pub uid: String,
}

#[derive(StructOpt, Debug, Clone)]
/// BalanceOpt
pub struct BalanceOpt {
    /// public key or uid
    pub address: String,
}

#[derive(StructOpt, Debug, Copy, Clone)]
/// BlocksOpt
pub struct BlocksOpt {}

impl DursExecutableCoreCommand for DbExOpt {
    fn execute(self, durs_core: DursCore<DuRsConf>) -> Result<(), DursCoreError> {
        let profile_path = durs_core.soft_meta_datas.profile_path;

        match self.subcommand {
            DbExSubCommand::BalanceOpt(balance_opts) => dbex(
                profile_path,
                self.csv,
                &DbExQuery::TxQuery(DbExTxQuery::Balance(balance_opts.address)),
            ),
            DbExSubCommand::DistanceOpt(distance_opts) => dbex(
                profile_path,
                self.csv,
                &DbExQuery::WotQuery(DbExWotQuery::AllDistances(distance_opts.reverse)),
            ),
            DbExSubCommand::ForksOpt(_forks_opts) => {
                dbex(profile_path, self.csv, &DbExQuery::ForkTreeQuery)
            }
            DbExSubCommand::MemberOpt(member_opts) => dbex(
                profile_path,
                self.csv,
                &DbExQuery::WotQuery(DbExWotQuery::MemberDatas(member_opts.uid.into())),
            ),
            DbExSubCommand::MembersOpt(members_opts) => {
                if members_opts.expire {
                    dbex(
                        profile_path,
                        self.csv,
                        &DbExQuery::WotQuery(DbExWotQuery::ExpireMembers(members_opts.reverse)),
                    );
                } else {
                    dbex(
                        profile_path,
                        self.csv,
                        &DbExQuery::WotQuery(DbExWotQuery::ListMembers(members_opts.reverse)),
                    );
                }
            }
            DbExSubCommand::BlocksOpt(_blocks_opts) => dbex(
                profile_path,
                self.csv,
                &DbExQuery::BcQuery(DbExBcQuery::CountBlocksPerIssuer),
            ),
        }

        Ok(())
    }
}
