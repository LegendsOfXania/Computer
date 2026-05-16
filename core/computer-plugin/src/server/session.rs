use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use uuid::Uuid;

pub struct Session {
    pub id: Uuid,
}

static SESSIONS: OnceLock<Mutex<HashMap<String, Session>>> = OnceLock::new();

fn store() -> &'static Mutex<HashMap<String, Session>> {
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn create_token(uuid: Uuid) -> String {
    let token = Uuid::new_v4().to_string();
    store()
        .lock()
        .unwrap()
        .insert(token.clone(), Session { id: uuid });
    token
}

pub fn consume_token(token: &str) -> Option<Session> {
    store().lock().unwrap().remove(token)
}
