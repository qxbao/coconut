use primitive_types::U256;

pub const COCONUT_VERSION: u32 = 1;

pub const COCO: u64 = 1;
pub const NUT: u64 = COCO * 1e8 as u64;
pub const INITIAL_DIFFICULTY: f64 = 1.0;
pub const MAX_TARGET: U256 = U256([
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x00000000ffff0000,
]);
pub const PRECISION: u128 = 100_000_000u128;

pub const BASE_REWARD: u64 = 100 * NUT;
pub const HALVING_INTERVAL: u64 = 21 * 1e4 as u64;
pub const QXBAO_ADDRESS: &str = "FILL LATER";

pub const COINBASE_MSG: &str = "Coconut Coinbase Transaction";
pub const SHA256_HEX_LEN: usize = 64;

// [Genesis Block Data]
pub const GENESIS_BLOCK_HASH: &str = "00000000513c25cdd070bd21db45dde4aac063983125a68e2360d77f2197c8f0";
pub const GENESIS_BLOCK_NONCE: u64 = 16909515401595071962;
pub const GENESIS_BLOCK_MSG: &str = "C u @ da nd of da world!";
pub const GENESIS_BLOCK_PREV_HASH: [u8; 32] = [0u8; 32];
pub const GENESIS_BLOCK_TIMESTAMP: u64 = 1766042962;

pub const DIFFICULTY_ADJUSTMENT_INTERVAL: u64 = 10;
pub const EXPECTED_BLOCK_TIME: u64 = 60;
pub const TARGET_TIMESPAN: u64 = DIFFICULTY_ADJUSTMENT_INTERVAL * EXPECTED_BLOCK_TIME;
