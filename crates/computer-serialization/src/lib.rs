mod error;
mod kdl;
mod page;

pub use error::SerializationError;
pub use kdl::KdlFormat;
pub use model::{Number, Value};
pub use page::{
    RawEntry,
    RawPage,
    decode_page,
    encode_page,
};
