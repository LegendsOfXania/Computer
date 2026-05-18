use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

const TOKEN_TTL: Duration = Duration::from_secs(600);

struct Session {
    expires_at: Instant,
}

static SESSIONS: OnceLock<Mutex<HashMap<String, Session>>> = OnceLock::new();

fn store() -> &'static Mutex<HashMap<String, Session>> {
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn create_token() -> String {
    let token = uuid::Uuid::new_v4().to_string();
    let session = Session {
        expires_at: Instant::now() + TOKEN_TTL,
    };

    let mut guard = store().lock().unwrap();
    purge_expired(&mut guard);
    guard.insert(token.clone(), session);
    token
}

pub fn consume_token(token: &str) -> bool {
    let mut guard = store().lock().unwrap();
    purge_expired(&mut guard);
    guard
        .remove(token)
        .map(|s| s.expires_at > Instant::now())
        .unwrap_or(false)
}

fn purge_expired(map: &mut HashMap<String, Session>) {
    let now = Instant::now();
    map.retain(|_, s| s.expires_at > now);
}
