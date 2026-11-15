pub mod app;
pub mod ui;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum ConnectionStatus {
    #[default]
    Connecting,
    Connected,
    Reconnecting(u32),
    Failed(String),
}
