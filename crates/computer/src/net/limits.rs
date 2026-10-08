pub const TICKS_PER_SECOND: u32 = 20;
pub const MAX_CONNECTIONS: usize = 32;

pub const MAX_ACCEPT_PER_TICK: usize = 16;

pub const PENDING_TIMEOUT: u32 = 10 * TICKS_PER_SECOND;

pub const WRITE_STALL_TIMEOUT: u32 = 10 * TICKS_PER_SECOND;

pub const PING_INTERVAL: u32 = 15 * TICKS_PER_SECOND;

pub const IDLE_TIMEOUT: u32 = 45 * TICKS_PER_SECOND;

pub const MAX_MESSAGES_PER_TICK: usize = 64;

pub const MAX_INVALID_FRAMES: u32 = 5;

pub const MAX_MESSAGE_SIZE: usize = 4 << 20;

pub const MAX_WRITE_BUFFER: usize = 16 << 20;

pub const FLUSH_INTERVAL: u32 = TICKS_PER_SECOND;
