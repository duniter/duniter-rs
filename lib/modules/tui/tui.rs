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

//! Defined the few global types used by all modules,
//! as well as the DursModule trait that all modules must implement.

#![deny(
    clippy::option_unwrap_used,
    clippy::result_unwrap_used,
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

use dubp_currency_params::CurrencyName;
use durs_common_tools::fatal_error;
use durs_common_tools::traits::merge::Merge;
use durs_conf::DuRsConf;
use durs_message::events::*;
use durs_message::*;
use durs_module::*;
use durs_network::events::NetworkEvent;
use durs_network_documents::network_head::NetworkHead;
use durs_network_documents::NodeFullId;
use log::{debug, trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{stdout, Write};
use std::ops::Deref;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, SystemTime};
use structopt::StructOpt;
use termion::event::*;
use termion::input::{MouseTerminal, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, cursor, style};
use unwrap::unwrap;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Tui Module Configuration (For future use)
pub struct TuiConf {}

impl Default for TuiConf {
    fn default() -> Self {
        TuiConf {}
    }
}

impl Merge for TuiConf {
    fn merge(self, _: Self) -> Self {
        self
    }
}

#[derive(Debug, Clone)]
/// Format of messages received by the tui module
pub enum TuiMess {
    /// Message from another module
    DursMsg(Box<DursMsg>),
    /// Message from stdin (user event)
    TermionEvent(Event),
}

#[derive(Debug, Copy, Clone)]
/// Tui module
pub struct TuiModule {}

#[derive(StructOpt, Debug, Copy, Clone)]
#[structopt(name = "tui", setting(structopt::clap::AppSettings::ColoredHelp))]
/// Tui subcommand options
pub struct TuiOpt {}

#[derive(Debug, Clone)]
/// Network connexion (data to display)
pub struct Connection {
    /// Connexion status
    status: u32,
    /// Endpoint url
    url: String,
    /// Node uid at the other end of the connection (member nodes only)
    uid: Option<String>,
}

#[derive(Debug, Clone)]
/// Data that the Tui module needs to cache
pub struct TuiModuleDatas {
    /// Sender of all other modules
    pub router_sender: Sender<RouterThreadMessage<DursMsg>>,
    /// HEADs cache content
    pub heads_cache: HashMap<NodeFullId, NetworkHead>,
    /// Position of the 1st head displayed on the screen
    pub heads_index: usize,
    /// Connections cache content
    pub connections_status: HashMap<NodeFullId, Connection>,
    /// Number of connections in `Established` status
    pub established_conns_count: usize,
}

impl TuiModuleDatas {
    /// Draw terminal
    fn draw_term<W: Write>(
        &self,
        stdout: &mut RawTerminal<W>,
        start_time: SystemTime,
        heads_cache: &HashMap<NodeFullId, NetworkHead>,
        heads_index: usize,
        out_connections_status: &HashMap<NodeFullId, Connection>,
        _in_connections_status: &HashMap<NodeFullId, Connection>,
    ) -> Result<(), std::io::Error> {
        // Get Terminal size
        let (w, h) = termion::terminal_size().expect("Fail to get terminal size !");

        // Prepare connections screen
        let mut out_never_try_conns_count = 0;
        let mut out_unreachable_conns_count = 0;
        let mut out_trying_conns_count = 0;
        let mut out_denial_conns_count = 0;
        let mut out_disconnected_conns_count = 0;
        let mut out_established_conns = Vec::new();
        for (node_full_id, connection) in out_connections_status {
            match connection.status {
                0 => out_never_try_conns_count += 1,
                2 | 4 => out_unreachable_conns_count += 1,
                1 | 3 | 5 | 7 | 8 | 9 => out_trying_conns_count += 1,
                10 => out_denial_conns_count += 1,
                11 => out_disconnected_conns_count += 1,
                12 => out_established_conns.push((
                    node_full_id,
                    connection.uid.clone(),
                    connection.url.clone(),
                )),
                _ => {}
            }
        }

        // Prepare HEADs screen
        let mut heads = heads_cache.values().collect::<Vec<&NetworkHead>>();
        heads.sort_unstable_by(|a, b| b.cmp(a));
        let heads_window_size = h as isize - 8 - out_established_conns.len() as isize;
        let heads_index_max = if heads_window_size > 0 && heads.len() > heads_window_size as usize {
            heads.len() - heads_window_size as usize
        } else {
            0
        };

        // Clear term and reset background color
        write!(
            stdout,
            "{}{}{}",
            color::Bg(color::Black),
            clear::All,
            cursor::Goto(1, 1)
        )?;

        // Draw headers
        let mut line = 1;
        write!(
            stdout,
            "{}{}{} established connections : ",
            cursor::Goto(1, line),
            color::Fg(color::White),
            out_established_conns.len()
        )?;
        line += 1;
        write!(
            stdout,
            "{}{}{} NodeId-PubKey",
            cursor::Goto(1, line),
            color::Fg(color::White),
            style::Italic,
        )?;

        // Draw inter-nodes established connections
        if out_established_conns.is_empty() {
            line += 1;
            write!(
                stdout,
                "{}{}{}No established connections !",
                cursor::Goto(2, line),
                color::Fg(color::Red),
                style::Bold,
            )?;
        } else {
            for (ref node_full_id, ref uid, ref url) in out_established_conns {
                line += 1;
                let mut uid_string = uid
                    .clone()
                    .unwrap_or_else(|| String::from("----------------"));
                uid_string.truncate(16);
                write!(
                    stdout,
                    "{}{} {} {:16} {}",
                    cursor::Goto(2, line),
                    color::Fg(color::Green),
                    node_full_id.to_human_string(),
                    uid_string,
                    url,
                )?;
            }
        }

        // Draw number of conns per state
        line += 1;
        write!(
            stdout,
            "{}{}{} know endpoints : {} Never try, {} Unreach, {} on trial, {} Denial, {} Close.",
            cursor::Goto(2, line),
            color::Fg(color::Rgb(128, 128, 128)),
            out_connections_status.len(),
            out_never_try_conns_count,
            out_unreachable_conns_count,
            out_trying_conns_count,
            out_denial_conns_count,
            out_disconnected_conns_count,
        )?;

        // Draw separated line
        line += 1;
        let mut separated_line = String::with_capacity(w as usize);
        for _ in 0..w as usize {
            separated_line.push('-');
        }
        write!(
            stdout,
            "{}{}{}",
            cursor::Goto(1, line),
            color::Fg(color::White),
            separated_line,
        )?;

        // Draw HEADs
        line += 1;
        write!(
            stdout,
            "{}{}{} HEADs :",
            cursor::Goto(1, line),
            color::Fg(color::White),
            heads.len()
        )?;
        line += 1;
        if heads_index > 0 {
            write!(
                stdout,
                "{}{}/\\",
                cursor::Goto(35, line),
                color::Fg(color::Green),
            )?;
        } else {
            write!(
                stdout,
                "{}{}/\\",
                cursor::Goto(35, line),
                color::Fg(color::Black),
            )?;
        }
        line += 1;
        write!(
            stdout,
            "{}{}Step NodeId-Pubkey BlockId-BlockHash      Soft:Ver            Pre [ Api ] MeR:MiR uid",
            cursor::Goto(1, line),
            color::Fg(color::White)
        )?;
        for head in &heads[heads_index..] {
            if line < (h - 2) {
                line += 1;
                if head.step() == 0 {
                    write!(
                        stdout,
                        "{}{}{}",
                        cursor::Goto(1, line),
                        color::Fg(color::Blue),
                        head.to_human_string(w as usize),
                    )?;
                } else {
                    write!(
                        stdout,
                        "{}{}{}",
                        cursor::Goto(1, line),
                        color::Fg(color::Green),
                        head.to_human_string(w as usize),
                    )?;
                }
            } else {
                break;
            }
        }
        line += 1;
        if heads_index < heads_index_max {
            write!(
                stdout,
                "{}{}\\/",
                cursor::Goto(35, line),
                color::Fg(color::Green),
            )?;
        } else {
            write!(
                stdout,
                "{}{}\\/",
                cursor::Goto(35, line),
                color::Fg(color::Black),
            )?;
        }

        // Draw footer
        let mut runtime_in_secs = SystemTime::now()
            .duration_since(start_time)
            .expect("Fail to get runtime")
            .as_secs();
        let runtime_hours = runtime_in_secs / 3600;
        runtime_in_secs -= runtime_hours * 3600;
        let runtime_mins = runtime_in_secs / 60;
        let runtime_secs = runtime_in_secs % 60;
        let runtime_str = format!(
            "{:02}:{:02}:{:02}",
            runtime_hours, runtime_mins, runtime_secs
        );
        write!(
            stdout,
            "{}{}{}runtime : {}",
            cursor::Goto(1, h),
            color::Bg(color::Blue),
            color::Fg(color::White),
            runtime_str,
        )?;
        write!(
            stdout,
            "{}{}{}q : quit{}",
            cursor::Goto(w - 7, h),
            color::Bg(color::Blue),
            color::Fg(color::White),
            cursor::Hide,
        )?;

        // Flush stdout (i.e. make the output appear).
        stdout.flush()?;

        Ok(())
    }
    /// Restore Terminal
    fn restore_term<W: Write>(stdout: &RawTerminal<W>) {
        let _ = stdout.suspend_raw_mode();
    }
}

impl Default for TuiModule {
    fn default() -> TuiModule {
        TuiModule {}
    }
}

impl DursModule<DuRsConf, DursMsg> for TuiModule {
    type ModuleUserConf = TuiConf;
    type ModuleConf = TuiConf;
    type ModuleOpt = TuiOpt;

    fn name() -> ModuleStaticName {
        ModuleStaticName("tui")
    }
    fn priority() -> ModulePriority {
        ModulePriority::Recommended
    }
    fn ask_required_keys() -> RequiredKeys {
        RequiredKeys::None
    }
    fn generate_module_conf(
        _currency_name: Option<&CurrencyName>,
        _global_conf: &<DuRsConf as DursConfTrait>::GlobalConf,
        _module_user_conf: Option<Self::ModuleUserConf>,
    ) -> Result<(Self::ModuleConf, Option<Self::ModuleUserConf>), ModuleConfError> {
        Ok((TuiConf {}, None))
    }
    fn start(
        _soft_meta_datas: &SoftwareMetaDatas<DuRsConf>,
        _keys: RequiredKeysContent,
        _conf: Self::ModuleConf,
        router_sender: Sender<RouterThreadMessage<DursMsg>>,
    ) -> Result<(), failure::Error> {
        let start_time = SystemTime::now(); //: DateTime<Utc> = Utc::now();

        // Instanciate Tui module datas
        let mut tui = TuiModuleDatas {
            router_sender: router_sender.clone(),
            heads_cache: HashMap::new(),
            heads_index: 0,
            connections_status: HashMap::new(),
            established_conns_count: 0,
        };

        // Create tui main thread channel
        let (tui_sender, tui_receiver): (Sender<TuiMess>, Receiver<TuiMess>) = channel();

        // Create proxy channel
        let (proxy_sender, proxy_receiver): (Sender<DursMsg>, Receiver<DursMsg>) = channel();

        // Launch a proxy thread that transform DursMsg() to TuiMess::DursMsg(DursMsg())
        let tui_sender_clone = tui_sender.clone();
        thread::spawn(move || {
            // Send proxy sender to main
            router_sender
                .send(RouterThreadMessage::ModuleRegistration {
                    static_name: TuiModule::name(),
                    sender: proxy_sender,
                    roles: vec![ModuleRole::UserInterface],
                    events_subscription: vec![
                        ModuleEvent::NewValidBlock,
                        ModuleEvent::ConnectionsChangeNodeNetwork,
                        ModuleEvent::NewValidHeadFromNetwork,
                        ModuleEvent::NewValidPeerFromNodeNetwork,
                    ],
                    reserved_apis_parts: vec![],
                    endpoints: vec![],
                })
                .expect("Fatal error : tui module fail to send is sender channel !");
            debug!("Send tui sender to main thread.");
            loop {
                match proxy_receiver.recv() {
                    Ok(message) => {
                        let stop = if let DursMsg::Stop = message {
                            true
                        } else {
                            false
                        };
                        match tui_sender_clone.send(TuiMess::DursMsg(Box::new(message))) {
                            Ok(_) => {
                                if stop {
                                    break;
                                };
                            }
                            Err(_) => {
                                debug!("tui proxy : fail to relay DursMsg to tui main thread !");
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Tui: {}", e);
                        break;
                    }
                }
            }
        });

        // Enter raw mode.
        // Wait a short while before modifying stdout, in case errors should be reported regarding the launch of the other modules.
        thread::sleep(Duration::from_millis(500));
        let mut stdout = MouseTerminal::from(unwrap!(stdout().into_raw_mode()));

        // Initial draw
        let mut last_draw = SystemTime::now();
        unwrap!(tui.draw_term(
            &mut stdout,
            start_time,
            &tui.heads_cache,
            tui.heads_index,
            &tui.connections_status,
            &HashMap::with_capacity(0),
        ));

        // Launch stdin thread
        let _stdin_thread = thread::spawn(move || {
            // Get the standard input stream.
            let stdin = std::io::stdin();
            // Get stdin events
            for c in stdin.events() {
                tui_sender
                    .send(TuiMess::TermionEvent(
                        c.expect("error to read stdin event !"),
                    ))
                    .expect("Fatal error : tui stdin thread module fail to send message !");
                trace!("Send stdin event to tui main thread.");
            }
        });

        // ui main loop
        loop {
            let mut user_event = false;
            // Get messages
            match tui_receiver.recv_timeout(Duration::from_millis(250)) {
                Ok(ref message) => match *message {
                    TuiMess::DursMsg(ref durs_message) => match durs_message.deref() {
                        DursMsg::Stop => {
                            unwrap!(writeln!(
                                stdout,
                                "{}{}{}{}{}",
                                color::Fg(color::Reset),
                                cursor::Goto(1, 1),
                                color::Bg(color::Reset),
                                cursor::Show,
                                clear::All,
                            ));
                            break;
                        }
                        DursMsg::Event {
                            ref event_content, ..
                        } => match *event_content {
                            DursEvent::BlockchainEvent(ref dal_event) => match *dal_event.deref() {
                                BlockchainEvent::StackUpValidBlock(ref _block) => {}
                                BlockchainEvent::RevertBlocks(ref _blocks) => {}
                                _ => {}
                            },
                            DursEvent::NetworkEvent(ref network_event_box) => {
                                match *network_event_box.deref() {
                                    NetworkEvent::ConnectionStateChange(
                                        ref node_full_id,
                                        ref status,
                                        ref uid,
                                        ref url,
                                    ) => {
                                        if let Some(conn) =
                                            tui.connections_status.get(&node_full_id)
                                        {
                                            if *status == 12 && (*conn).status != 12 {
                                                tui.established_conns_count += 1;
                                            } else if *status != 12
                                                && (*conn).status == 12
                                                && tui.established_conns_count > 0
                                            {
                                                tui.established_conns_count -= 1;
                                            }
                                        };
                                        tui.connections_status.insert(
                                            *node_full_id,
                                            Connection {
                                                status: *status,
                                                url: url.clone(),
                                                uid: uid.clone(),
                                            },
                                        );
                                    }
                                    NetworkEvent::ReceiveHeads(ref heads) => {
                                        heads
                                            .iter()
                                            .map(|h| {
                                                tui.heads_cache.insert(h.node_full_id(), h.clone())
                                            })
                                            .for_each(drop);
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    },
                    TuiMess::TermionEvent(ref term_event) => match *term_event {
                        Event::Key(Key::Char('q')) => {
                            // Exit
                            unwrap!(writeln!(
                                stdout,
                                "{}{}{}{}{}",
                                color::Fg(color::Reset),
                                cursor::Goto(1, 1),
                                color::Bg(color::Reset),
                                cursor::Show,
                                clear::All,
                            ));
                            TuiModuleDatas::restore_term(&stdout);
                            if tui
                                .router_sender
                                .send(RouterThreadMessage::ModuleMessage(DursMsg::Stop))
                                .is_err()
                            {
                                warn!("Tui: Fail to send STOP message to router !");
                            }
                            break;
                        }
                        Event::Mouse(ref me) => match *me {
                            MouseEvent::Press(ref button, ref _a, ref _b) => match *button {
                                MouseButton::WheelDown => {
                                    // Get Terminal size
                                    let (_w, h) = termion::terminal_size()
                                        .expect("Fail to get terminal size !");
                                    // heads_index
                                    if h > 16 {
                                        let heads_index_max =
                                            if tui.heads_cache.len() > (h - 16) as usize {
                                                tui.heads_cache.len() - (h - 16) as usize
                                            } else {
                                                0
                                            };
                                        if tui.heads_index < heads_index_max {
                                            tui.heads_index += 1;
                                            user_event = true;
                                        } else {
                                            tui.heads_index = heads_index_max;
                                        }
                                    }
                                }
                                MouseButton::WheelUp => {
                                    // heads_index
                                    if tui.heads_index > 0 {
                                        tui.heads_index -= 1;
                                        user_event = true;
                                    }
                                }
                                _ => {}
                            },
                            MouseEvent::Release(ref _a, ref _b)
                            | MouseEvent::Hold(ref _a, ref _b) => {}
                        },
                        _ => {}
                    },
                },
                Err(e) => match e {
                    RecvTimeoutError::Disconnected => {
                        fatal_error!("Disconnected tui module !");
                    }
                    RecvTimeoutError::Timeout => {}
                },
            }
            let now = SystemTime::now();
            if user_event
                || now
                    .duration_since(last_draw)
                    .expect("Tui : Fatal error : fail to get duration since last draw !")
                    .subsec_nanos()
                    > 250_000_000
            {
                last_draw = now;
                unwrap!(tui.draw_term(
                    &mut stdout,
                    start_time,
                    &tui.heads_cache,
                    tui.heads_index,
                    &tui.connections_status,
                    &HashMap::with_capacity(0),
                ));
            }
        }
        Ok(())
    }
}
