pub mod error;
pub mod kdl;

pub use error::SerializationError;
pub use kdl::KdlFormat;
pub use model::{Number, Value};

pub trait Format {
    fn decode(input: &str) -> Result<Value, SerializationError>;

    fn encode(value: &Value) -> Result<String, SerializationError>;
}