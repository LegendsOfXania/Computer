use std::{collections::HashMap, io::ErrorKind, net::TcpListener};

use computer_model::id::{Identifier, Token};
use parking_lot::Mutex;
use pumpkin_plugin_api::{Server, command::CommandSender, scheduler::SchedulerExt};

use crate::{
    data::library,
    net::{
        assets::{self, Asset},
        connection::Connection,
        limits::{
            FLUSH_INTERVAL, MAX_ACCEPT_PER_TICK, MAX_CONNECTIONS, SESSION_GRACE, SESSION_MAX_AGE,
            SESSION_PENDING_TTL,
        },
        ws,
    },
};

pub(crate) const REQUEST_BUFFER: usize = 8 * 1024;

pub(crate) struct Session {
    pub(crate) _sender: CommandSender,
    created: u32,
    last_active: u32,
    used: bool,
}

pub(crate) struct State {
    pub(crate) listener: TcpListener,
    pub(crate) connections: Vec<Connection>,
    pub(crate) assets: HashMap<String, Asset>,
    pub(crate) sessions: HashMap<Token, Session>,
    pub(crate) tick: u32,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

pub enum StartOutcome {
    Started,
    AlreadyRunning,
}

pub fn ensure_started(
    server: &Server,
    ip: &str,
    port: u16,
) -> Result<(StartOutcome, Token), String> {
    let token = Token::new();

    if STATE.lock().is_some() {
        return Ok((StartOutcome::AlreadyRunning, token));
    }

    let listener = TcpListener::bind((ip, port)).map_err(|error| error.to_string())?;

    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;

    let assets = assets::load().unwrap_or_default();

    library::init_dev()?;

    let mut guard = STATE.lock();

    if guard.is_some() {
        return Ok((StartOutcome::AlreadyRunning, token));
    }

    *guard = Some(State {
        listener,
        connections: Vec::new(),
        assets,
        sessions: HashMap::new(),
        tick: 0,
    });

    drop(guard);

    server.schedule_repeating_task(0, 1, |_| poll());

    tracing::info!("WebSocket server started: {ip}:{port}");

    Ok((StartOutcome::Started, token))
}

pub(crate) fn register_session(
    token: Token,
    sender: CommandSender,
) -> Result<(), (String, CommandSender)> {
    let mut guard = STATE.lock();

    let Some(state) = guard.as_mut() else {
        return Err(("The panel server is not running".to_string(), sender));
    };

    let tick = state.tick;

    state.sessions.insert(
        token,
        Session {
            _sender: sender,
            created: tick,
            last_active: tick,
            used: false,
        },
    );

    Ok(())
}

fn poll() {
    let mut guard = STATE.lock();

    let Some(state) = guard.as_mut() else {
        return;
    };

    accept_connections(state);

    let assets = &state.assets;
    let sessions = &state.sessions;
    let old_connections = std::mem::take(&mut state.connections);

    let mut connections = Vec::with_capacity(old_connections.len());
    let mut events = Vec::new();

    for connection in old_connections {
        let (connection, connection_events) = connection.poll(assets, sessions);

        if let Some(connection) = connection {
            connections.push(connection);
        }

        events.extend(connection_events);
    }

    state.connections = connections;

    refresh_sessions(state);

    for event in events {
        ws::broadcast_event(state, event);
    }

    ws::send_replies(state);

    state.tick = state.tick.wrapping_add(1);

    if state.tick % FLUSH_INTERVAL == 0 {
        library::flush_dirty();
    }
}

fn refresh_sessions(state: &mut State) {
    let tick = state.tick;

    for connection in &state.connections {
        let Connection::WebSocket(socket) = connection else {
            continue;
        };

        if !socket.ready {
            continue;
        }

        let Some(session) = socket
            .session
            .as_ref()
            .and_then(|token| state.sessions.get_mut(token))
        else {
            continue;
        };

        session.used = true;
        session.last_active = tick;
    }

    let before = state.sessions.len();

    state.sessions.retain(|_, session| {
        let idle = tick.wrapping_sub(session.last_active);
        let age = tick.wrapping_sub(session.created);

        let limit = if session.used {
            SESSION_GRACE
        } else {
            SESSION_PENDING_TTL
        };

        idle < limit && age < SESSION_MAX_AGE
    });

    let expired = before - state.sessions.len();

    if expired > 0 {
        tracing::debug!("Expired {expired} panel session(s)");
    }
}

fn accept_connections(state: &mut State) {
    for _ in 0..MAX_ACCEPT_PER_TICK {
        match state.listener.accept() {
            Ok((stream, address)) => {
                if state.connections.len() >= MAX_CONNECTIONS {
                    tracing::warn!("Too many connections, refused {address}");
                    continue;
                }

                let _ = stream.set_nodelay(true);

                if stream.set_nonblocking(true).is_ok() {
                    state
                        .connections
                        .push(Connection::pending(stream, REQUEST_BUFFER));
                }
            }

            Err(error) if error.kind() == ErrorKind::WouldBlock => break,

            Err(error) => {
                tracing::warn!("Failed to accept connection: {error}");
                break;
            }
        }
    }
}
