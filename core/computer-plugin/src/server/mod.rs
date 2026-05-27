mod auth;
mod http;
mod listener;
mod ws;

pub use auth::generate as generate_token;
pub use listener::{is_running, start};

pub fn poll() {
    listener::accept_pending();
    ws::poll_all();
}
