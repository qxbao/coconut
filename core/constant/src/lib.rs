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

pub const GENESIS_BLOCK_MSG: &str = "See you at the end of the world!";
pub const COINBASE_MSG: &str = "Coconut Coinbase Transaction";
