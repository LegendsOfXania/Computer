mod error;
mod format;
mod kdl;
mod page;

pub use error::SerializationError;
pub use format::Format;
pub use kdl::KdlFormat;
pub use model::{Number, Value};
pub use page::{
    RawEntry,
    RawPage,
    decode_page,
    encode_page,
};