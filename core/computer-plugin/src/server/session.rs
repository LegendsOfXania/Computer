use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

static SESSIONS: OnceLock<Mutex<HashMap<String, ()>>> = OnceLock::new();

fn store() -> &'static Mutex<HashMap<String, ()>> {
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn create_token() -> String {
    let token = generate_token();
    store().lock().unwrap().insert(token.clone(), ());
    token
}

pub fn consume_token(token: &str) -> bool {
    store().lock().unwrap().remove(token).is_some()
}

fn generate_token() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};

    let mut hasher = DefaultHasher::new();
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos()
        .hash(&mut hasher);
    std::thread::current().id().hash(&mut hasher);
    format!("{:x}{:x}", hasher.finish(), hasher.finish() ^ 0xdeadbeef)
}
