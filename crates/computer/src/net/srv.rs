use std::{
    collections::HashMap,
    io::ErrorKind,
    net::TcpListener,
    sync::Mutex,
};

use pumpkin_plugin_api::{scheduler::SchedulerExt, Server};

use crate::{
    data::library,
    net::{
        assets::{self, Asset},
        connection::Connection,
        ws,
    },
};

pub(crate) const REQUEST_BUFFER: usize = 8 * 1024;

pub(crate) struct State {
    pub(crate) listener: TcpListener,
    pub(crate) connections: Vec<Connection>,
    pub(crate) assets: HashMap<String, Asset>,
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
    if STATE.lock().map_err(|e| e.to_string())?.is_some() {
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

    *STATE.lock().map_err(|e| e.to_string())? = Some(State {
        listener,
        connections: Vec::new(),
        assets,
    });

    Ok(StartOutcome::Started)
}

fn poll() {
    let Ok(mut guard) = STATE.lock() else {
        return;
    };

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
}

fn accept_connections(state: &mut State) {
    loop {
        match state.listener.accept() {
            Ok((stream, _)) => {
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