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
pub struct Block {
    pub timestamp: u64,
    pub prev_hash: Vec<u8>,
    pub transactions: Vec<Transaction>,
    pub nonce: u64,
}

impl Block {
    pub fn new(prev_hash: Vec<u8>, transactions: Vec<Transaction>) -> Self {
        let now = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        Self {
            timestamp: now,
            prev_hash,
            transactions,
            nonce: 0,
        }
    }

    pub fn hash(&self) -> U256 {
        let serialized: Vec<u8> = bincode2::serialize(self).expect("Failed to serialize block");
        crypto::compute_sha256x2(&serialized)
    }

    pub fn mine(&mut self, difficulty: usize) {
        let target = U256::from(difficulty);
        
        while &self.hash() <= &target {
            self.nonce += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_block_hash() {
        let tx = Transaction::new(
            "senderAddress".to_string(),
            "receiverAddress".to_string(),
            100,
            "sample message".to_string()
        );

        let block = Block::new("0".repeat(64).into(), vec![tx]);
        let hash = block.hash();
        let hash_hex = crypto::to_hex(hash);

        println!("Block Hash: {}", hash_hex);
        assert_eq!(hash_hex.len(), 64);
    }
}