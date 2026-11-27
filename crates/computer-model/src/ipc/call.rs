use core::convert::Infallible;

use minicbor::{Decode, Encode, Encoder};

use super::codec::finish;

/// A function signature generated from WIT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signature {
    /// Stable function identifier.
    pub id: u32,

    /// Fully qualified WIT function name.
    pub name: &'static str,

    /// Parameter names and WIT types, in declaration order.
    pub parameters: &'static [(&'static str, &'static str)],

    /// WIT result type, or `None` when the function has no result.
    pub result: Option<&'static str>,

    /// Whether the WIT function is asynchronous.
    pub is_async: bool,

    /// Whether the interface is imported by the Cosmos world.
    ///
    /// `true`: extension calls engine.
    /// `false`: engine calls extension.
    pub imported: bool,
}

/// Encoder for the positional arguments of a WIT function call.
pub struct Arguments<'bytes> {
    encoder: Encoder<&'bytes mut Vec<u8>>,
    expected: usize,
    encoded: usize,
}

impl Arguments<'_> {
    /// Encodes the next positional argument.
    pub fn push<T>(&mut self, value: &T) -> Result<(), minicbor::encode::Error<Infallible>>
    where
        T: Encode<()>,
    {
        if self.encoded >= self.expected {
            return Err(minicbor::encode::Error::message(
                "too many arguments for WIT function",
            ));
        }

        value.encode(&mut self.encoder, &mut ())?;
        self.encoded += 1;

        Ok(())
    }
}

/// Builds a WIT function call.
///
/// The number of arguments is obtained from its generated WIT signature.
pub fn encode_call<F>(
    function: u32,
    encode_arguments: F,
) -> Result<Vec<u8>, minicbor::encode::Error<Infallible>>
where
    F: FnOnce(&mut Arguments<'_>) -> Result<(), minicbor::encode::Error<Infallible>>,
{
    let signature = super::function::signature(function).ok_or_else(|| {
        minicbor::encode::Error::message("unknown WIT function ID")
    })?;

    let expected = signature.parameters.len();

    let mut argument_bytes = Vec::new();

    {
        let encoder = Encoder::new(&mut argument_bytes);

        let mut arguments = Arguments {
            encoder,
            expected,
            encoded: 0,
        };

        encode_arguments(&mut arguments)?;

        if arguments.encoded != expected {
            return Err(minicbor::encode::Error::message(
                "function call argument count does not match its WIT signature",
            ));
        }
    }

    let argument_count = u64::try_from(expected).map_err(|_| {
        minicbor::encode::Error::message("too many WIT function arguments")
    })?;

    let mut bytes = Vec::with_capacity(argument_bytes.len() + 9);

    {
        let mut encoder = Encoder::new(&mut bytes);
        encoder
            .array(argument_count + 1)?
            .u32(function)?;
    }

    bytes.extend_from_slice(&argument_bytes);

    Ok(bytes)
}

/// A decoded WIT function call.
pub struct Call<'bytes> {
    signature: &'static Signature,
    decoder: minicbor::Decoder<'bytes>,
    decoded_arguments: usize,
}

impl Call<'_> {
    /// Returns the function ID.
    #[must_use]
    pub const fn function(&self) -> u32 {
        self.signature.id
    }

    /// Returns the generated WIT signature.
    #[must_use]
    pub const fn signature(&self) -> &'static Signature {
        self.signature
    }

    /// Returns the number of arguments not yet decoded.
    #[must_use]
    pub fn remaining_arguments(&self) -> usize {
        self.signature.parameters.len() - self.decoded_arguments
    }

    /// Returns the next parameter's name and WIT type.
    #[must_use]
    pub fn next_parameter(&self) -> Option<(&'static str, &'static str)> {
        self.signature.parameters.get(self.decoded_arguments).copied()
    }
}

impl<'bytes> Call<'bytes> {
    /// Decodes the next positional argument.
    pub fn decode_argument<T>(&mut self) -> Result<T, minicbor::decode::Error>
    where
        T: Decode<'bytes, ()>,
    {
        if self.decoded_arguments >= self.signature.parameters.len() {
            return Err(minicbor::decode::Error::message(
                "no remaining function arguments",
            ));
        }

        let value = self.decoder.decode()?;
        self.decoded_arguments += 1;

        Ok(value)
    }

    /// Ensures all arguments and the complete input have been consumed.
    pub fn finish(self) -> Result<(), minicbor::decode::Error> {
        if self.decoded_arguments != self.signature.parameters.len() {
            return Err(minicbor::decode::Error::message(
                "function call contains undecoded arguments",
            ));
        }

        finish(&self.decoder)
    }
}

/// Decodes a function call envelope.
pub fn decode_call(bytes: &[u8]) -> Result<Call<'_>, minicbor::decode::Error> {
    let mut decoder = minicbor::Decoder::new(bytes);

    let array_len = decoder.array()?.ok_or_else(|| {
        minicbor::decode::Error::message(
            "indefinite function call arrays are not supported",
        )
    })?;

    if array_len == 0 {
        return Err(minicbor::decode::Error::message(
            "function call is missing its function ID",
        ));
    }

    let function = decoder.u32()?;

    let signature = super::function::signature(function).ok_or_else(|| {
        minicbor::decode::Error::message("unknown WIT function ID")
    })?;

    let expected = u64::try_from(signature.parameters.len()).map_err(|_| {
        minicbor::decode::Error::message("too many WIT function arguments")
    })?;

    if array_len - 1 != expected {
        return Err(minicbor::decode::Error::message(
            "function call argument count does not match its WIT signature",
        ));
    }

    Ok(Call {
        signature,
        decoder,
        decoded_arguments: 0,
    })
}