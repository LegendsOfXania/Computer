use core::convert::Infallible;

use minicbor::{CborLen, Decode, Encode, Encoder};

pub fn encode<T>(value: &T) -> Result<Vec<u8>, minicbor::encode::Error<Infallible>>
where
    T: Encode<()> + CborLen<()>,
{
    let mut bytes = Vec::with_capacity(value.cbor_len(&mut ()));
    encode_into(value, &mut bytes)?;
    Ok(bytes)
}

pub fn encode_into<T>(
    value: &T,
    bytes: &mut Vec<u8>,
) -> Result<(), minicbor::encode::Error<Infallible>>
where
    T: Encode<()>,
{
    bytes.clear();

    let mut encoder = Encoder::new(bytes);
    value.encode(&mut encoder, &mut ())
}

pub fn decode<'bytes, T>(bytes: &'bytes [u8]) -> Result<T, minicbor::decode::Error>
where
    T: Decode<'bytes, ()>,
{
    let mut decoder = minicbor::Decoder::new(bytes);

    let value = decode_from(&mut decoder)?;
    finish(&decoder)?;

    Ok(value)
}

pub fn decode_from<'bytes, T>(
    decoder: &mut minicbor::Decoder<'bytes>,
) -> Result<T, minicbor::decode::Error>
where
    T: Decode<'bytes, ()>,
{
    decoder.decode()
}

pub fn finish(
    decoder: &minicbor::Decoder<'_>,
) -> Result<(), minicbor::decode::Error> {
    if decoder.position() == decoder.input().len() {
        Ok(())
    } else {
        Err(
            minicbor::decode::Error::message("trailing CBOR data")
                .at(decoder.position()),
        )
    }
}