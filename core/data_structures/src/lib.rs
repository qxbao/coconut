use serde::{Deserialize, Serialize};
use std::time;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub message: String,
}

impl Transaction {
    pub fn new(sender: String, receiver: String, amount: u64, message: String) -> Self {
        Self {
            sender,
            receiver,
            amount,
            message,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub timestamp: u64,
    pub prev_hash: String,
    pub transaction: Vec<Transaction>,
    pub nonce: u64,
}

impl Block {
    pub fn new(prev_hash: String, transaction: Vec<Transaction>) -> Self {
        let now = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        Self {
            timestamp: now,
            prev_hash,
            transaction,
            nonce: 0,
        }
    }
}
