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

//! Durs-core cli : dbex subcommands.

extern crate structopt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "dbex",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp")
)]
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
    #[structopt(
        name = "distance",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    /// durs databases explorer (distances datas)
    DistanceOpt(DistanceOpt),
    #[structopt(
        name = "members",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    /// durs databases explorer (members datas)
    MembersOpt(MembersOpt),
    #[structopt(
        name = "member",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    /// durs databases explorer (member datas)
    MemberOpt(MemberOpt),
    #[structopt(
        name = "balance",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    /// durs databases explorer (balances datas)
    BalanceOpt(BalanceOpt),
}

#[derive(StructOpt, Debug, Copy, Clone)]
/// DistanceOpt
pub struct DistanceOpt {
    #[structopt(short = "r", long = "reverse")]
    /// reverse order
    pub reverse: bool,
}

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
