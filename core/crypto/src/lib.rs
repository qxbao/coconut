use primitive_types::U256;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

pub fn compute_merkle_root(txs: &[u8]) -> [u8; 32] {
    if txs.is_empty() {
        return [0u8; 32];
    }

    let mut hashes: Vec<[u8; 32]> = txs
        .chunks_exact(32)
        .map(|chunk| {
            let mut hash = [0u8; 32];
            hash.copy_from_slice(chunk);
            hash
        })
        .collect();

    if !txs.chunks_exact(32).remainder().is_empty() {
        let remainder = txs.chunks_exact(32).remainder();
        let mut hasher = Sha256::new();
        hasher.update(remainder);
        hashes.push(hasher.finalize().into());
    }

    while hashes.len() > 1 {
        let mut next_level = Vec::new();

        for pair in hashes.chunks(2) {
            let hash = if pair.len() == 2 {
                let mut hasher = Sha256::new();
                hasher.update(pair[0]);
                hasher.update(pair[1]);
                let first_hash = hasher.finalize();

                let mut hasher = Sha256::new();
                hasher.update(first_hash);
                hasher.finalize().into()
            } else {
                let mut hasher = Sha256::new();
                hasher.update(pair[0]);
                hasher.update(pair[0]);
                let first_hash = hasher.finalize();

                let mut hasher = Sha256::new();
                hasher.update(first_hash);
                hasher.finalize().into()
            };
            next_level.push(hash);
        }

        hashes = next_level;
    }

    hashes[0]
}

#[derive(Debug, Clone)]
pub struct MerkleProof {
    pub tx_index: usize,
    pub path: Vec<([u8; 32], bool)>,
}

fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    let first_hash = hasher.finalize();
    
    let mut hasher = Sha256::new();
    hasher.update(first_hash);
    hasher.finalize().into()
}

pub fn generate_merkle_proof(txs: &[u8], tx_index: usize) -> Option<MerkleProof> {
    if txs.is_empty() {
        return None;
    }

    let mut hashes: Vec<[u8; 32]> = txs
        .chunks_exact(32)
        .map(|chunk| {
            let mut hash = [0u8; 32];
            hash.copy_from_slice(chunk);
            hash
        })
        .collect();

    if !txs.chunks_exact(32).remainder().is_empty() {
        let remainder = txs.chunks_exact(32).remainder();
        let mut hasher = Sha256::new();
        hasher.update(remainder);
        hashes.push(hasher.finalize().into());
    }

    if tx_index >= hashes.len() {
        return None;
    }

    let mut path = Vec::new();
    let mut current_index = tx_index;

    while hashes.len() > 1 {
        let mut next_level = Vec::new();
        let next_index = current_index / 2;

        for (i, pair) in hashes.chunks(2).enumerate() {
            if i == current_index / 2 {
                if current_index % 2 == 0 {
                    if pair.len() == 2 {
                        path.push((pair[1], true));
                    } else {
                        path.push((pair[0], true));
                    }
                } else {
                    path.push((pair[0], false));
                }
            }

            let hash = if pair.len() == 2 {
                hash_pair(&pair[0], &pair[1])
            } else {
                hash_pair(&pair[0], &pair[0])
            };
            next_level.push(hash);
        }

        hashes = next_level;
        current_index = next_index;
    }

    Some(MerkleProof { tx_index, path })
}

pub fn verify_merkle_proof(
    tx_hash: &[u8; 32],
    proof: &MerkleProof,
    merkle_root: &[u8; 32],
) -> bool {
    let mut current_hash = *tx_hash;
    for (sibling, is_right) in &proof.path {
        current_hash = if *is_right {
            hash_pair(&current_hash, sibling)
        } else {
            hash_pair(sibling, &current_hash)
        };
    }

    &current_hash == merkle_root
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

pub fn pubkey_hash_to_address(
    pubkey_hash: &[u8; 20],
    network: &constant::BitcoinNetwork,
) -> String {
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
