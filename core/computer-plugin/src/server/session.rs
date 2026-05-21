use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use uuid::Uuid;

static TOKEN: OnceLock<Mutex<Option<(String, Instant)>>> = OnceLock::new();

const TOKEN_TTL: Duration = Duration::from_secs(600);

fn store() -> &'static Mutex<Option<(String, Instant)>> {
    TOKEN.get_or_init(|| Mutex::new(None))
}

pub fn create_token() -> String {
    let token = Uuid::new_v4().to_string();
    *store().lock().unwrap() = Some((token.clone(), Instant::now() + TOKEN_TTL));
    token
}

pub fn consume_token(token: &str) -> bool {
    let mut guard = store().lock().unwrap();
    match guard.as_ref() {
        Some((t, expires_at)) if t == token && *expires_at > Instant::now() => {
            *guard = None;
            true
        }
        _ => false,
    }
}
