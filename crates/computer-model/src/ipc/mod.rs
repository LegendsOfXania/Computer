pub mod call;
pub mod codec;

pub mod function {
    use super::call::Signature;

    include!(concat!(env!("OUT_DIR"), "/function_ids.rs"));
}