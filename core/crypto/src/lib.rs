use sha2::{Sha256, Digest};
use primitive_types::U256;

pub fn compute_sha256x2(data: &[u8]) -> U256 {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let first_hash = hasher.finalize();

    let mut hasher = Sha256::new();
    hasher.update(first_hash);
    U256::from_big_endian(&hasher.finalize())
}

pub fn to_hex(data: U256) -> String {
    let mut hex_string = String::new();
    let bytes = data.to_big_endian();

    for byte in &bytes {
        hex_string.push_str(&format!("{:02x}", byte));
    }

    hex_string
}