use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
};
use tracing::warn;

pub mod http;
pub mod session;
pub mod ws;

pub static RUNNING: AtomicBool = AtomicBool::new(false);

pub fn start(http_addr: String, ws_addr: String) {
    if RUNNING.swap(true, Ordering::SeqCst) {
        warn!("Computer server is already running");
        return;
    }

    thread::spawn(move || {
        http::listen(&http_addr);
    });

    thread::spawn(move || {
        ws::listen(&ws_addr);
    });
}

pub fn is_running() -> bool {
    RUNNING.load(Ordering::SeqCst)
}
