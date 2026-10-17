pub mod app;
pub mod mock;
pub mod ui;

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Connecting,
    Connected,
    Reconnecting(u32),
    Failed(String),
}