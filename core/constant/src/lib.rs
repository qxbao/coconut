use primitive_types::U256;
use serde::{Deserialize, Serialize};

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
pub const QXBAO_ADDRESS: &str = "1HwKr3zhCNMhE8WaUNsiREbcykYqCq6yeV";

pub const COINBASE_MSG: &str = "Coconut Coinbase Transaction";
pub const SHA256_HEX_LEN: usize = 64;

// [Genesis Block Data]
pub const GENESIS_BLOCK_HASH: &str = "0000000016b1c7d4793408fbcaaa15bd80b173e9e43830a3756453be452bcc8a";
pub const GENESIS_BLOCK_NONCE: u64 = 1729382257010256396;
pub const GENESIS_BLOCK_MSG: &str = "C u @ da nd of da world!";
pub const GENESIS_BLOCK_PREV_HASH: [u8; 32] = [0u8; 32];
pub const GENESIS_BLOCK_TIMESTAMP: u64 = 1766042962;

pub const DIFFICULTY_ADJUSTMENT_INTERVAL: u64 = 10;
pub const EXPECTED_BLOCK_TIME: u64 = 60;
pub const TARGET_TIMESPAN: u64 = DIFFICULTY_ADJUSTMENT_INTERVAL * EXPECTED_BLOCK_TIME;

pub mod address {
    pub const MAINNET_PREFIX: u8 = 0x4C;
    pub const TESTNET_PREFIX: u8 = 0x8C;
} 

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum BitcoinNetwork {
    Mainnet,
    Testnet,
}

impl BitcoinNetwork {
    pub fn p2pkh_prefix(&self) -> u8 {
        match self {
            BitcoinNetwork::Mainnet => 0x00,
            BitcoinNetwork::Testnet => 0x6F,
        }
    }
}

pub const NETWORK_TYPE: BitcoinNetwork = BitcoinNetwork::Mainnet;
