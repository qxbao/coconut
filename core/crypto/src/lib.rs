use ripemd::Ripemd160;
use sha2::{Sha256, Digest};
use primitive_types::U256;

pub fn compute_merkel_root(txs: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(txs);
    hasher.finalize().into()
}

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

    format!("{:0>width$}", hex_string, width = constant::SHA256_HEX_LEN)
}

pub fn target_to_bits(target: U256) -> u32 {
    let mut size = (target.bits() + 7) / 8;
    let mut compact: u32;
    
    if size <= 3 {
        compact = (target.low_u64() << (8 * (3 - size))) as u32;
    } else {
        let shift = 8 * (size - 3);
        let target_shifted = target >> shift;
        compact = target_shifted.low_u32();
    }
    if (compact & 0x00800000) != 0 {
        compact >>= 8;
        size += 1;
    }

    (size << 24) as u32 | (compact & 0x00ffffff)
}

pub fn difficulty_to_target(difficulty: f64) -> U256 {
    if difficulty < 1.0 {
        return constant::MAX_TARGET;
    }
    let diff_scaled = (difficulty * constant::PRECISION as f64) as u128;
    
    constant::MAX_TARGET
        .checked_mul(U256::from(constant::PRECISION))
        .expect("It should not overflow :)")
        .checked_div(U256::from(diff_scaled))
        .unwrap_or(U256::zero())
}

pub fn bits_to_target(bits: u32) -> U256 {
    let exponent = (bits >> 24) as u32;
    let coefficient = (bits & 0x00ffffff) as u64;
    if exponent <= 3 {
        U256::from(coefficient) >> (8 * (3 - exponent))
    } else {
        U256::from(coefficient) << (8 * (exponent - 3))
    }
}

pub fn hash_public_key(pubkey_bytes: &[u8]) -> [u8; 20] {
    let mut sha256_hasher = Sha256::new();
    sha256_hasher.update(pubkey_bytes);
    let sha256_result = sha256_hasher.finalize();

    let mut ripemd_hasher = Ripemd160::new();
    ripemd_hasher.update(sha256_result);
    
    let mut address_hash = [0u8; 20];
    address_hash.copy_from_slice(&ripemd_hasher.finalize());
    
    address_hash
}

pub fn pubkey_hash_to_address(pubkey_hash: &[u8; 20], network: &constant::BitcoinNetwork) -> String {
    let version_byte: u8 = network.p2pkh_prefix();
    
    let mut payload = Vec::with_capacity(25);
    payload.push(version_byte);
    payload.extend_from_slice(pubkey_hash);

    let first_sha = Sha256::digest(&payload);
    let second_sha = Sha256::digest(&first_sha);
    let checksum = &second_sha[0..4];

    payload.extend_from_slice(checksum);
    bs58::encode(payload).into_string()
}

pub fn address_to_pubkey_hash(address: &str) -> Result<[u8; 20], String> {
    let decoded = bs58::decode(address)
        .into_vec()
        .map_err(|_| "Invalid Base58")?;
    
    if decoded.len() != 25 {
        return Err("Invalid address length".into());
    }
    
    let mut hash = [0u8; 20];
    hash.copy_from_slice(&decoded[1..21]);
    Ok(hash)
}
