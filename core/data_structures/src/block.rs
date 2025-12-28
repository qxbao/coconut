use crate::error;
use crate::transaction::Transaction;
use primitive_types::U256;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockHeader {
    pub version: u32,
    pub timestamp: u64,
    pub prev_hash: [u8; 32],
    pub merkle_root: [u8; 32],
    pub bits: u32,
    pub nonce: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(bits: u32, prev_hash: [u8; 32], transactions: Vec<Transaction>) -> Self {
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
                bits,
                nonce: 0,
            },
            transactions,
        }
    }

    pub fn new_genesis() -> Self {
        let genesis_coinbase = Transaction::new_coinbase(
            crypto::address_to_pubkey_hash(&constant::QXBAO_ADDRESS)
                .expect("Invalid Genesis address"),
            constant::BASE_REWARD,
            Some(constant::GENESIS_BLOCK_MSG.to_string()),
        );

        let mut gb = Self {
            header: BlockHeader {
                version: constant::COCONUT_VERSION,
                timestamp: constant::GENESIS_BLOCK_TIMESTAMP,
                prev_hash: constant::GENESIS_BLOCK_PREV_HASH,
                merkle_root: [0u8; 32],
                bits: crypto::target_to_bits(constant::MAX_TARGET),
                nonce: constant::GENESIS_BLOCK_NONCE,
            },
            transactions: vec![genesis_coinbase],
        };
        gb.set_merkle_root();
        gb
    }

    fn set_merkle_root(&mut self) {
        let serialized =
            bincode2::serialize(&self.transactions).expect("Failed to serialize transactions");
        let root = crypto::compute_merkel_root(&serialized);
        self.header.merkle_root = root
    }

    fn header_bin(&self) -> Vec<u8> {
        bincode2::serialize(&self.header).expect("Failed to serialize block")
    }

    pub fn hash(&self) -> U256 {
        let serialized: Vec<u8> = self.header_bin();
        crypto::compute_sha256x2(&serialized)
    }

    pub fn mine_nonce(&mut self, bits: u32) -> Result<(), error::BlockchainError> {
        self.set_merkle_root();
        let header_buffer = bincode2::serialize(&self.header).expect("Failed to serialize transactions");
        let nonce_pos = header_buffer.len() - 8;
        let is_found = Arc::new(AtomicBool::new(false));
        let result_nonce = Arc::new(AtomicU64::new(0));
        let num_threads = rayon::current_num_threads() as u64;
        let chunk_size: u64 = u64::MAX / num_threads;
        let target = crypto::bits_to_target(bits);

        println!("Mining with {} threads...", num_threads);

        (0..num_threads).into_par_iter().for_each(|thread_id| {
            let mut local_header = header_buffer.clone();
            let start_nonce = thread_id as u64 * chunk_size;
            let end_nonce = if thread_id == num_threads - 1 {
                u64::MAX
            } else {
                (thread_id as u64 + 1) * chunk_size
            };
            for nonce in start_nonce..end_nonce {
                if nonce & constant::MINING_THREAD_BREAK_INTERVAL == 0 && is_found.load(Ordering::Relaxed) {
                    break;
                }
                local_header[nonce_pos..nonce_pos + 8].copy_from_slice(&nonce.to_le_bytes());
                let hash_result = crypto::compute_sha256x2(&local_header);
                if hash_result <= target {
                    result_nonce.store(nonce, Ordering::Release);
                    is_found.store(true, Ordering::Release);
                    println!(
                        "Thread {} found solution! Nonce: {}, Hash: {}",
                        thread_id,
                        nonce,
                        crypto::to_hex(hash_result)
                    );
                    break;
                }
                // TODO: indicatif -> Progress Bar
                if nonce & 0x7FFFFF == 0 && thread_id == 0 {
                    println!("Mining... Thread {}: nonce {}", thread_id, nonce);
                }
            }
        });
        if !is_found.load(Ordering::Acquire) {
            return Err(error::BlockchainError::MiningFailed);
        }
        self.header.nonce = result_nonce.load(Ordering::Acquire);
        Ok(())
    }

    pub fn mine(&mut self, bits: u32) -> Result<(), error::BlockchainError> {
        loop {
            match self.mine_nonce(bits) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    println!("Mining attempt failed: {:?}, incrementing timestamp and retrying...", e);
                    self.header.timestamp += 1;
                    continue;
                }
            }
        }
    }

    pub fn verify(&self) -> bool {
        let hash = self.hash();
        let target = crypto::bits_to_target(self.header.bits);

        if target.is_zero() {
            return false;
        }

        if hash > target {
            println!(
                "Block hash does not meet the target requirement. Hash: {}, Target: {}",
                crypto::to_hex(hash),
                crypto::to_hex(target)
            );
            return false;
        }

        let serialized_txs = bincode2::serialize(&self.transactions).expect("Failed to serialize");
        let calculated_root = crypto::compute_merkel_root(&serialized_txs);
        if calculated_root != self.header.merkle_root {
            return false;
        }

        if self.transactions.is_empty() || !self.transactions[0].is_coinbase() {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod block_tests {
    use super::*;
    #[test]
    #[ignore = "Heavy mining test - run with `cargo test -- --ignored`"]
    fn test_genesis_block_mining() {
        let mut genesis_block = Block::new_genesis();
        let bits = crypto::target_to_bits(constant::MAX_TARGET);
        genesis_block
            .mine(bits)
            .expect("Failed to mine genesis block");
        let hash = genesis_block.hash();
        let target = crypto::bits_to_target(bits);
        println!("Block information: {:?}", genesis_block);
        println!("Mined Genesis Block Hash: {}", crypto::to_hex(hash));
        println!("Mined Genesis Block Nonce: {}", genesis_block.header.nonce);
        println!("Mined Genesis Block Bits: {}", genesis_block.header.bits);
        assert!(hash <= target);
    }

    #[test]
    fn test_genesis_block_validation() {
        let genesis_block = Block::new_genesis();
        assert!(genesis_block.verify());
    }

    #[test]
    fn test_genesis_block_hash_matches_constant() {
        let genesis_block = Block::new_genesis();
        let hash = genesis_block.hash();
        let hash_hex = crypto::to_hex(hash);

        println!("Genesis Block Hash: {}", hash_hex);
        println!("Expected Hash: {}", constant::GENESIS_BLOCK_HASH);

        assert_eq!(
            hash_hex,
            constant::GENESIS_BLOCK_HASH,
            "Genesis block hash must match the hardcoded constant"
        );
    }

    #[test]
    fn test_modified_genesis_block_should_fail() {
        let mut genesis_block = Block::new_genesis();
        genesis_block.header.nonce += 1;

        assert!(
            !genesis_block.verify(),
            "Modified genesis block should be invalid"
        );
    }
}
