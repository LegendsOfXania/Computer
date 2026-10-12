/// 2026-01-01T00:00:00Z Unix.
const EPOCH_MS: u64 = 1_767_225_600_000;
const ENTROPY_BITS: u32 = 22;

pub fn generate() -> u64 {
    let time = (js_sys::Date::now() as u64).saturating_sub(EPOCH_MS);
    let entropy = (js_sys::Math::random() * (1u64 << ENTROPY_BITS) as f64) as u64;

    (time << ENTROPY_BITS) | entropy
}