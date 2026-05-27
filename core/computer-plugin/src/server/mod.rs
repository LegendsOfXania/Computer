mod http;
mod listener;
mod ws;

pub use listener::{is_running, start};

pub fn poll() {
    listener::accept_pending();
    ws::poll_all();
}
