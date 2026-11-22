use std::{fmt, str};

use serde::{Deserialize, Serialize, Serializer};

pub const ID_LEN: usize = 15;

const CHARSET: &[u8; 62] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

#[cfg(feature = "panel")]
fn random() -> Result<[u8; ID_LEN], String> {
    let mut bytes = [0; ID_LEN];

    getrandom::fill(&mut bytes).map_err(|error| error.to_string())?;

    Ok(bytes.map(|byte| {
        CHARSET
            .get(usize::from(byte) % CHARSET.len())
            .copied()
            .unwrap_or(b'0')
    }))
}

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize)]
        #[serde(from = "String")]
        pub struct $name([u8; ID_LEN]);

        impl $name {
            #[must_use]
            pub fn as_str(&self) -> &str {
                str::from_utf8(&self.0).unwrap_or_default()
            }

            #[cfg(feature = "panel")]
            pub fn generate() -> Result<Self, String> {
                random().map(Self)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                let mut bytes = [b'0'; ID_LEN];

                for (slot, character) in bytes.iter_mut().zip(value.chars()) {
                    if let Ok(byte) = u8::try_from(character)
                        && CHARSET.contains(&byte)
                    {
                        *slot = byte;
                    }
                }

                Self(bytes)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::from(value.as_str())
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_tuple(stringify!($name))
                    .field(&self.as_str())
                    .finish()
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(self.as_str())
            }
        }
    };
}

define_id!(PageId);
define_id!(EntryId);

#[cfg(feature = "extension")]
define_id!(TaskId);

#[cfg(feature = "extension")]
define_id!(InteractionId);

#[cfg(feature = "panel")]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Token(String);

#[cfg(feature = "panel")]
impl Token {
    pub fn generate() -> Result<Self, String> {
        Ok(Self(random()?.into_iter().map(char::from).collect()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(feature = "panel")]
impl From<String> for Token {
    fn from(value: String) -> Self {
        Self(value)
    }
}
