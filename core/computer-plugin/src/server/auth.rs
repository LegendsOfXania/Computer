use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use tracing::debug;
use uuid::Uuid;

const TOKEN_TTL: Duration = Duration::from_secs(60);

struct Token {
    expires_at: Instant,
}

static TOKENS: Mutex<Option<HashMap<String, Token>>> = Mutex::new(None);

fn store() -> std::sync::MutexGuard<'static, Option<HashMap<String, Token>>> {
    TOKENS.lock().unwrap()
}

pub fn generate() -> String {
    let token = Uuid::new_v4().to_string();
    let entry = Token {
        expires_at: Instant::now() + TOKEN_TTL,
    };

    let mut guard = store();
    let map = guard.get_or_insert_with(HashMap::new);

    let now = Instant::now();
    map.retain(|_, e| e.expires_at > now);

    map.insert(token.clone(), entry);
    debug!("auth: token généré, expire dans {}s", TOKEN_TTL.as_secs());
    token
}

pub fn validate(token: &str) -> bool {
    let mut guard = store();
    let Some(map) = guard.as_mut() else {
        return false;
    };

    match map.remove(token) {
        Some(entry) if entry.expires_at > Instant::now() => {
            debug!("auth: token validé");
            true
        }
        Some(_) => {
            debug!("auth: token expiré");
            false
        }
        None => {
            debug!("auth: token inconnu");
            false
        }
    }
}
