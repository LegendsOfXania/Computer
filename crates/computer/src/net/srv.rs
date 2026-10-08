use std::{collections::HashMap, io::ErrorKind, net::TcpListener};

use parking_lot::Mutex;
use pumpkin_plugin_api::{scheduler::SchedulerExt, Server};

use crate::{
    data::library,
    net::{
        assets::{self, Asset},
        connection::Connection,
        limits::{FLUSH_INTERVAL, MAX_ACCEPT_PER_TICK, MAX_CONNECTIONS},
        ws,
    },
};

pub(crate) const REQUEST_BUFFER: usize = 8 * 1024;

pub(crate) struct State {
    pub(crate) listener: TcpListener,
    pub(crate) connections: Vec<Connection>,
    pub(crate) assets: HashMap<String, Asset>,
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
) -> Result<StartOutcome, String> {
    if STATE.lock().is_some() {
        return Ok(StartOutcome::AlreadyRunning);
    }

    let listener = TcpListener::bind((ip, port))
        .map_err(|e| e.to_string())?;

    listener
        .set_nonblocking(true)
        .map_err(|e| e.to_string())?;

    let assets = assets::load().unwrap_or_default();

    library::init_dev()?;

    server.schedule_repeating_task(0, 1, |_| poll());

    *STATE.lock() = Some(State {
        listener,
        connections: Vec::new(),
        assets,
        tick: 0,
    });

    tracing::info!("Websocket server started: {}:{}", ip, port);

    Ok(StartOutcome::Started)
}

fn poll() {
    let mut guard = STATE.lock();

    let Some(state) = guard.as_mut() else {
        return;
    };

    accept_connections(state);

    let assets = &state.assets;
    let old_connections = std::mem::take(&mut state.connections);

    let mut connections = Vec::with_capacity(old_connections.len());
    let mut events = Vec::new();

    for connection in old_connections {
        let (connection, connection_events) =
            connection.poll(assets);

        if let Some(connection) = connection {
            connections.push(connection);
        }

        events.extend(connection_events);
    }

    state.connections = connections;

    for event in events {
        ws::broadcast_event(state, event);
    }

    ws::send_replies(state);

    state.tick = state.tick.wrapping_add(1);

    if state.tick % FLUSH_INTERVAL == 0 {
        library::flush_dirty();
    }
}

fn accept_connections(state: &mut State) {
    for _ in 0..MAX_ACCEPT_PER_TICK {
        match state.listener.accept() {
            Ok((stream, address)) => {
                if state.connections.len() >= MAX_CONNECTIONS {
                    tracing::warn!(
                        "Too many connections, refused {address}"
                    );

                    continue;
                }

                

                let _ = stream.set_nodelay(true);

                if stream.set_nonblocking(true).is_ok() {
                    state.connections.push(
                        Connection::pending(
                            stream,
                            REQUEST_BUFFER,
                        ),
                    );
                }
            }

            Err(error)
                if error.kind() == ErrorKind::WouldBlock =>
            {
                break;
            }

            Err(error) => {
                tracing::warn!(
                    "Failed to accept connection: {error}"
                );
                break;
            }
        }
    }
}
