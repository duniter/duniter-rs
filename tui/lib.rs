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

//! Defined the few global types used by all modules,
//! as well as the DuniterModule trait that all modules must implement.

#![cfg_attr(feature = "strict", deny(warnings))]
#![deny(
    missing_docs, missing_debug_implementations, missing_copy_implementations, trivial_casts,
    trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces,
    unused_qualifications
)]

#[macro_use]
extern crate log;

extern crate chrono;
extern crate duniter_conf;
extern crate duniter_crypto;
extern crate duniter_dal;
extern crate duniter_documents;
extern crate duniter_message;
extern crate duniter_module;
extern crate duniter_network;
extern crate serde_json;
extern crate termion;

use chrono::prelude::*;
use duniter_crypto::keys::ed25519;
use duniter_dal::dal_event::DALEvent;
use duniter_message::DuniterMessage;
use duniter_module::*;
use duniter_network::network_head::NetworkHead;
use duniter_network::{NetworkEvent, NodeFullId};
use std::collections::HashMap;
use std::io::{stdout, Write};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime};
use termion::event::*;
use termion::input::{MouseTerminal, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, cursor, style};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// Tui Module Configuration (For future use)
pub struct TuiConf {}

#[derive(Debug, Clone)]
/// Format of messages received by the tui module
pub enum TuiMess {
    /// Message from another module
    DuniterMessage(DuniterMessage),
    /// Message from stdin (user event)
    TermionEvent(Event),
}

#[derive(Debug, Copy, Clone)]
/// Tui module
pub struct TuiModule {}

#[derive(Debug, Clone)]
/// Network connexion (data to display)
pub struct Connection {
    /// connexion status
    status: u32,
    /// Node uid at the other end of the connection (member nodes only)
    uid: Option<String>,
}

#[derive(Debug, Clone)]
/// Data that the Tui module needs to cache
pub struct TuiModuleDatas {
    /// Sender of all other modules
    pub followers: Vec<mpsc::Sender<DuniterMessage>>,
    /// HEADs cache content
    pub heads_cache: HashMap<NodeFullId, NetworkHead>,
    /// Position of the 1st head displayed on the screen
    pub heads_index: usize,
    /// Connections cache content
    pub connections_status: HashMap<NodeFullId, Connection>,
    /// Number of connections in `Established` status
    pub established_conns_count: usize,
    /// Position of the 1st connection displayed on the screen
    pub conns_index: usize,
}

impl TuiModuleDatas {
    /// Parse tui configuration
    fn parse_tui_conf(_json_conf: &serde_json::Value) -> TuiConf {
        TuiConf {}
    }
    /// Draw terminal
    fn draw_term<W: Write>(
        &self,
        stdout: &mut RawTerminal<W>,
        start_time: &DateTime<Utc>,
        heads_cache: &HashMap<NodeFullId, NetworkHead>,
        heads_index: usize,
        out_connections_status: &HashMap<NodeFullId, Connection>,
        _in_connections_status: &HashMap<NodeFullId, Connection>,
        conns_index: usize,
    ) {
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
                12 => out_established_conns.push((node_full_id, connection.uid.clone())),
                _ => {}
            }
        }

        // Prepare HEADs screen
        let mut heads = heads_cache.values().collect::<Vec<&NetworkHead>>();
        heads.sort_unstable_by(|a, b| b.cmp(a));
        let heads_index_max = if heads.len() > (h - 14) as usize {
            heads.len() - (h - 14) as usize
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
        ).unwrap();

        // Draw headers
        let mut line = 1;
        write!(
            stdout,
            "{}{}{} established connections : ",
            cursor::Goto(1, line),
            color::Fg(color::White),
            out_established_conns.len()
        ).unwrap();
        line += 1;
        write!(
            stdout,
            "{}{}{} NodeId-PubKey",
            cursor::Goto(1, line),
            color::Fg(color::White),
            style::Italic,
        ).unwrap();
        line += 1;
        write!(
            stdout,
            "{}{}/\\",
            cursor::Goto(29, line),
            color::Fg(color::Black),
        ).unwrap();

        // Draw inter-nodes established connections
        if out_established_conns.is_empty() {
            line += 1;
            write!(
                stdout,
                "{}{}{}No established connections !",
                cursor::Goto(2, line),
                color::Fg(color::Red),
                style::Bold,
            ).unwrap();
        } else {
            let mut count_conns = 0;
            let conns_index_use = if conns_index > (out_established_conns.len() - 5) {
                out_established_conns.len() - 5
            } else {
                conns_index
            };
            for &(node_full_id, ref uid) in &out_established_conns[conns_index_use..] {
                line += 1;
                count_conns += 1;
                write!(
                    stdout,
                    "{}{} {} {}",
                    cursor::Goto(2, line),
                    color::Fg(color::Green),
                    node_full_id,
                    uid.clone().unwrap_or_else(String::new),
                ).unwrap();
                if count_conns == 5 {
                    line += 1;
                    write!(
                        stdout,
                        "{}{}\\/",
                        cursor::Goto(29, line),
                        color::Fg(color::Black)
                    ).unwrap();
                    break;
                }
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
        ).unwrap();

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
        ).unwrap();

        // Draw HEADs
        line += 1;
        write!(
            stdout,
            "{}{}{} HEADs :",
            cursor::Goto(1, line),
            color::Fg(color::White),
            heads.len()
        ).unwrap();
        line += 1;
        if heads_index > 0 {
            write!(
                stdout,
                "{}{}/\\",
                cursor::Goto(35, line),
                color::Fg(color::Green),
            ).unwrap();
        } else {
            write!(
                stdout,
                "{}{}/\\",
                cursor::Goto(35, line),
                color::Fg(color::Black),
            ).unwrap();
        }
        line += 1;
        write!(
            stdout,
            "{}{}Step NodeId-Pubkey BlockId-BlockHash    Soft:Ver     Pre [ Api ] MeR:MiR uid",
            cursor::Goto(1, line),
            color::Fg(color::White)
        ).unwrap();
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
                    ).unwrap();
                } else {
                    write!(
                        stdout,
                        "{}{}{}",
                        cursor::Goto(1, line),
                        color::Fg(color::Green),
                        head.to_human_string(w as usize),
                    ).unwrap();
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
            ).unwrap();
        } else {
            write!(
                stdout,
                "{}{}\\/",
                cursor::Goto(35, line),
                color::Fg(color::Black),
            ).unwrap();
        }

        // Draw footer
        let runtime_in_secs = Utc::now().timestamp() - (*start_time).timestamp();
        let runtime_str =
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(runtime_in_secs, 0), Utc)
                .format("%H:%M:%S")
                .to_string();
        write!(
            stdout,
            "{}{}{}runtime : {}",
            cursor::Goto(1, h),
            color::Bg(color::Blue),
            color::Fg(color::White),
            runtime_str,
        ).unwrap();
        write!(
            stdout,
            "{}{}{}q : quit{}",
            cursor::Goto(w - 7, h),
            color::Bg(color::Blue),
            color::Fg(color::White),
            cursor::Hide,
        ).unwrap();

        // Flush stdout (i.e. make the output appear).
        stdout.flush().unwrap();
    }
}

impl Default for TuiModule {
    fn default() -> TuiModule {
        TuiModule {}
    }
}

impl DuniterModule<ed25519::PublicKey, ed25519::KeyPair, DuniterMessage> for TuiModule {
    fn id() -> ModuleId {
        ModuleId::Str("tui")
    }
    fn priority() -> ModulePriority {
        ModulePriority::Recommended()
    }
    fn ask_required_keys() -> RequiredKeys {
        RequiredKeys::None()
    }
    fn default_conf() -> serde_json::Value {
        serde_json::Value::default()
    }
    fn start(
        _soft_name: &str,
        _soft_version: &str,
        _keys: RequiredKeysContent<ed25519::PublicKey, ed25519::KeyPair>,
        _conf: &DuniterConf,
        module_conf: &serde_json::Value,
        main_sender: mpsc::Sender<RooterThreadMessage<DuniterMessage>>,
        load_conf_only: bool,
    ) -> Result<(), ModuleInitError> {
        let start_time: DateTime<Utc> = Utc::now();

        // load conf
        let _conf = TuiModuleDatas::parse_tui_conf(module_conf);
        if load_conf_only {
            return Ok(());
        }

        // Instanciate Tui module datas
        let mut tui = TuiModuleDatas {
            followers: Vec::new(),
            heads_cache: HashMap::new(),
            heads_index: 0,
            connections_status: HashMap::new(),
            established_conns_count: 0,
            conns_index: 0,
        };

        // Create tui main thread channel
        let (tui_sender, tui_receiver): (mpsc::Sender<TuiMess>, mpsc::Receiver<TuiMess>) =
            mpsc::channel();

        // Create proxy channel
        let (proxy_sender, proxy_receiver): (
            mpsc::Sender<DuniterMessage>,
            mpsc::Receiver<DuniterMessage>,
        ) = mpsc::channel();

        // Launch a proxy thread that transform DuniterMessage() to TuiMess::DuniterMessage(DuniterMessage())
        let tui_sender_clone = tui_sender.clone();
        thread::spawn(move || {
            // Send proxy sender to main
            match main_sender.send(RooterThreadMessage::ModuleSender(proxy_sender)) {
                Ok(_) => {
                    debug!("Send tui sender to main thread.");
                }
                Err(_) => panic!("Fatal error : tui module fail to send is sender channel !"),
            }
            loop {
                match proxy_receiver.recv() {
                    Ok(message) => {
                        match tui_sender_clone.send(TuiMess::DuniterMessage(message.clone())) {
                            Ok(_) => {
                                if let DuniterMessage::Stop() = message {
                                    break;
                                };
                            }
                            Err(_) => debug!(
                                "tui proxy : fail to relay DuniterMessage to tui main thread !"
                            ),
                        }
                    }
                    Err(e) => {
                        warn!("{}", e);
                        break;
                    }
                }
            }
        });

        // Enter raw mode.
        //let mut stdout = stdout().into_raw_mode().unwrap();
        let mut stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());

        // Initial draw
        let mut last_draw = SystemTime::now();
        tui.draw_term(
            &mut stdout,
            &start_time,
            &tui.heads_cache,
            tui.heads_index,
            &tui.connections_status,
            &HashMap::with_capacity(0),
            tui.conns_index,
        );

        // Launch stdin thread
        let _stdin_thread = thread::spawn(move || {
            // Get the standard input stream.
            let stdin = std::io::stdin();
            // Get stdin events
            for c in stdin.events() {
                match tui_sender.send(TuiMess::TermionEvent(
                    c.expect("error to read stdin event !"),
                )) {
                    Ok(_) => {
                        trace!("Send stdin event to tui main thread.");
                    }
                    Err(_) => {
                        panic!("Fatal error : tui stdin thread module fail to send message !")
                    }
                }
            }
        });

        // ui main loop
        loop {
            let mut user_event = false;
            // Get messages
            match tui_receiver.recv_timeout(Duration::from_millis(250)) {
                Ok(ref message) => match message {
                    &TuiMess::DuniterMessage(ref duniter_message) => match duniter_message {
                        &DuniterMessage::Stop() => {
                            writeln!(
                                stdout,
                                "{}{}{}{}{}",
                                color::Fg(color::Reset),
                                cursor::Goto(1, 1),
                                color::Bg(color::Reset),
                                cursor::Show,
                                clear::All,
                            ).unwrap();
                            let _result_stop_propagation: Result<
                                (),
                                mpsc::SendError<DuniterMessage>,
                            > = tui
                                .followers
                                .iter()
                                .map(|f| f.send(DuniterMessage::Stop()))
                                .collect();
                            break;
                        }
                        &DuniterMessage::Followers(ref new_followers) => {
                            info!("Tui module receive followers !");
                            for new_follower in new_followers {
                                debug!("TuiModule : push one follower.");
                                tui.followers.push(new_follower.clone());
                            }
                        }
                        &DuniterMessage::DALEvent(ref dal_event) => match dal_event {
                            &DALEvent::StackUpValidBlock(ref _block) => {}
                            &DALEvent::RevertBlocks(ref _blocks) => {}
                            _ => {}
                        },
                        &DuniterMessage::NetworkEvent(ref network_event) => match network_event {
                            &NetworkEvent::ConnectionStateChange(
                                ref node_full_id,
                                ref status,
                                ref uid,
                            ) => {
                                if let Some(conn) = tui.connections_status.get(node_full_id) {
                                    if *status == 12 && (*conn).status != 12 {
                                        tui.established_conns_count += 1;
                                    } else if *status != 12 && (*conn).status == 12 {
                                        tui.established_conns_count -= 1;
                                    }
                                };
                                tui.connections_status.insert(
                                    *node_full_id,
                                    Connection {
                                        status: *status,
                                        uid: uid.clone(),
                                    },
                                );
                            }
                            &NetworkEvent::ReceiveHeads(ref heads) => {
                                heads
                                    .iter()
                                    .map(|h| tui.heads_cache.insert(h.node_full_id(), h.clone()))
                                    .collect::<Vec<Option<NetworkHead>>>();
                            }
                            _ => {}
                        },
                        _ => {}
                    },
                    &TuiMess::TermionEvent(ref term_event) => match term_event {
                        &Event::Key(Key::Char('q')) => {
                            // Exit
                            writeln!(
                                stdout,
                                "{}{}{}{}{}",
                                color::Fg(color::Reset),
                                cursor::Goto(1, 1),
                                color::Bg(color::Reset),
                                cursor::Show,
                                clear::All,
                            ).unwrap();
                            let _result_stop_propagation: Result<
                                (),
                                mpsc::SendError<DuniterMessage>,
                            > = tui
                                .followers
                                .iter()
                                .map(|f| f.send(DuniterMessage::Stop()))
                                .collect();
                            break;
                        }
                        &Event::Mouse(ref me) => match me {
                            &MouseEvent::Press(ref button, ref _a, ref b) => match button {
                                &MouseButton::WheelDown => {
                                    // Get Terminal size
                                    let (_w, h) = termion::terminal_size()
                                        .expect("Fail to get terminal size !");
                                    if *b < 11 {
                                        // conns_index
                                        let conns_index_max = if tui.established_conns_count > 5 {
                                            tui.established_conns_count - 5
                                        } else {
                                            0
                                        };
                                        if tui.heads_index < conns_index_max {
                                            tui.conns_index += 1;
                                            user_event = true;
                                        } else {
                                            tui.conns_index = conns_index_max;
                                        }
                                    } else {
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
                                }
                                &MouseButton::WheelUp => {
                                    if *b < 11 {
                                        // conns_index
                                        if tui.conns_index > 0 {
                                            tui.conns_index -= 1;
                                            user_event = true;
                                        }
                                    } else {
                                        // heads_index
                                        if tui.heads_index > 0 {
                                            tui.heads_index -= 1;
                                            user_event = true;
                                        }
                                    }
                                }
                                _ => {}
                            },
                            &MouseEvent::Release(ref _a, ref _b)
                            | &MouseEvent::Hold(ref _a, ref _b) => {}
                        },
                        _ => {}
                    },
                },
                Err(e) => match e {
                    mpsc::RecvTimeoutError::Disconnected => {
                        panic!("Disconnected tui module !");
                    }
                    mpsc::RecvTimeoutError::Timeout => {}
                },
            }
            let now = SystemTime::now();
            if user_event
                || now
                    .duration_since(last_draw)
                    .expect("Tui : Fatal error : fail to get duration since last draw !")
                    .subsec_nanos() > 250_000_000
            {
                last_draw = now;
                tui.draw_term(
                    &mut stdout,
                    &start_time,
                    &tui.heads_cache,
                    tui.heads_index,
                    &tui.connections_status,
                    &HashMap::with_capacity(0),
                    tui.conns_index,
                );
            }
        }
        Ok(())
    }
}
