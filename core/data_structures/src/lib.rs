pub mod blockchain;

use primitive_types::U256;
use serde::{Deserialize, Serialize};
use std::time;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub sender: Option<String>,
    pub receiver: String,
    pub amount: u64,
    pub message: String,
}

impl Transaction {
    pub fn new(sender: String, receiver: String, amount: u64, message: String) -> Self {
        Self {
            sender: Some(sender),
            receiver,
            amount,
            message,
        }
    }

    pub fn new_coinbase(receiver: String, amount: u64, is_genesis: bool) -> Self {
        match is_genesis {
            true => Self {
                sender: None,
                receiver,
                amount,
                message: constant::GENESIS_BLOCK_MSG.to_string(),
            },
            false => Self {
                sender: None,
                receiver,
                amount,
                message: constant::COINBASE_MSG.to_string(),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockHeader {
    pub version: u32,
    pub timestamp: u64,
    pub prev_hash: [u8; 32],
    pub merkle_root: [u8; 32],
    pub bits: u32,
    pub nonce: u64,
}

impl BlockHeader {
    pub fn target(&self) -> U256 {
        let exponent = (self.bits >> 24) as u32;
        let coefficient = (self.bits & 0x00ffffff) as u64;
        U256::from(coefficient) << (8 * (exponent - 3))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(prev_hash: [u8; 32], transactions: Vec<Transaction>) -> Self {
        let now = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        Self {
            header: BlockHeader {
                version: constant::COCONUT_VERSION,
                timestamp: now,
                prev_hash,
                merkle_root: [0u8; 32],
                bits: 0,
                nonce: 0,
            },
            transactions,
        }
    }

    pub fn set_merkle_root(&mut self) {
        let serialized = bincode2::serialize(&self.transactions).expect("Failed to serialize transactions");
        let root = crypto::compute_merkel_root(&serialized);
        self.header.merkle_root = root 
    }

    pub fn header_bin(&self) -> Vec<u8> {
        bincode2::serialize(&self.header).expect("Failed to serialize block")
    }

    pub fn hash(&self) -> U256 {
        let serialized: Vec<u8> = self.header_bin();
        crypto::compute_sha256x2(&serialized)
    }

    pub fn mine(&mut self, target: U256) {
        self.set_merkle_root();
        let mut header_buffer = bincode2::serialize(&self.header).expect("Failed to serialize transactions");
        let nonce_pos = header_buffer.len() - 8;

        loop {
            let hash_result = crypto::compute_sha256x2(&header_buffer);
            
            if hash_result <= target {
                break;
            }

            self.header.nonce += 1;
            header_buffer[nonce_pos..nonce_pos+8].copy_from_slice(&self.header.nonce.to_le_bytes());
            
            if self.header.nonce % 10_000_000 == 0 {
                println!("Mining... Current nonce: {}, Hash: {:x}", self.header.nonce, hash_result);
            }
        }

        println!("Block Mined! Hash: {:x}", self.hash());
    }
}

#[cfg(test)]
mod block_tests {
    use super::*;
    #[test]
    fn test_block_hash() {
        let tx = Transaction::new(
            "senderAddress".to_string(),
            "receiverAddress".to_string(),
            100,
            "sample message".to_string()
        );

        let block = Block::new([0u8; 32], vec![tx]);
        let hash = block.hash();
        let hash_hex = crypto::to_hex(hash);

        println!("Block Hash: {}", hash_hex);
        assert_eq!(hash_hex.len(), 64);
    }
}
