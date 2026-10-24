mod entry;
mod field;
mod ipc;

pub use computer_edk_macro::{Field, entry};

pub use computer_model as model;

pub use entry::Entry;
pub use field::Field;
pub use ipc::{handle_ipc, register};

pub mod prelude {
    pub use crate::{Entry, Field, entry, handle_ipc, register};
}
