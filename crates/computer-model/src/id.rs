use rand::RngExt;
use serde::{Deserialize, Serialize};

const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

pub trait Identifier {
    fn new() -> Self;
    fn from_string(value: String) -> Self;
    fn into_string(self) -> String;
}

macro_rules! define_identifier {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl Identifier for $name {
            fn new() -> Self {
                let mut rng = rand::rng();
                let value = (0..15)
                    .map(|_| CHARSET[rng.random_range(0..CHARSET.len())] as char)
                    .collect();

                Self(value)
            }

            fn from_string(value: String) -> Self {
                Self(value)
            }

            fn into_string(self) -> String {
                self.0
            }
        }
    };
}

define_identifier!(PageId);
define_identifier!(EntryId);
define_identifier!(TaskId);
define_identifier!(InteractionId);
