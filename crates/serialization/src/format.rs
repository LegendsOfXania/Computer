use crate::{
    SerializationError,
    Value,
};

pub trait Format {
    fn decode(input: &str) -> Result<Value, SerializationError>;

    fn encode(value: &Value) -> Result<String, SerializationError>;
}