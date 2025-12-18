use primitive_types::U256;

use crate::{Block, Transaction};

pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub difficulty: f64,
    pub pending_transactions: Vec<Transaction>,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            blocks: Vec::new(),
            difficulty: constant::INITIAL_DIFFICULTY,
            pending_transactions: Vec::new(),
        };
        blockchain.create_genesis_block();
        blockchain
    }

    fn create_genesis_block(&mut self) {
        let genesis_block = Block::new_genesis();
        self.blocks.push(genesis_block);
    }
    
    pub fn add_transaction(&mut self, tx: Transaction) {
        self.pending_transactions.push(tx);
    }

    pub fn reward(&self) -> u64 {
        let chain_len = self.blocks.len() as u64;
        let halve_time = chain_len / constant::HALVING_INTERVAL;
      
        if halve_time >= 64 {
            0
        } else {
            constant::BASE_REWARD >> halve_time
        }
    }

    pub fn compute_next_bits(&self) -> u32 {
        let last_block = self.blocks.last().unwrap();
        
        if (self.blocks.len() as u64) % constant::DIFFICULTY_ADJUSTMENT_INTERVAL != 0 {
            return last_block.header.bits;
        }

        let first_block_index = self.blocks.len() - constant::DIFFICULTY_ADJUSTMENT_INTERVAL as usize;
        let first_block = &self.blocks[first_block_index];

        let actual_timespan = last_block.header.timestamp - first_block.header.timestamp;

        let adjusted_timespan = if actual_timespan < constant::TARGET_TIMESPAN / 4 {
            constant::TARGET_TIMESPAN / 4
        } else if actual_timespan > constant::TARGET_TIMESPAN * 4 {
            constant::TARGET_TIMESPAN * 4
        } else {
            actual_timespan
        };

        let mut target = last_block.header.target();
        target = target * U256::from(adjusted_timespan);
        target = target / U256::from(constant::TARGET_TIMESPAN);

        if target > constant::MAX_TARGET {
            target = constant::MAX_TARGET;
        }

        crypto::target_to_bits(target)
    }

    pub fn mine_pending_transactions(&mut self, miner_pubkey_hash: &str) {
        let bits = self.compute_next_bits();
        let coinbase = Transaction::new_coinbase(
            miner_pubkey_hash.to_string(),
            self.reward(),
        );
        
        let mut block_transactions = vec![coinbase];
        block_transactions.extend(self.pending_transactions.clone());
        
        let mut block = Block::new(
            bits,
            self.blocks.last().unwrap().hash().to_big_endian(),
            block_transactions,
        );

        block.mine(bits);

        self.blocks.push(block);
        self.pending_transactions.clear();
    }
}

#[cfg(test)]
mod blockchain_tests {
    use super::*;
    use constant;

    #[test]
    fn test_blockchain() {
        let blockchain = Blockchain::new();
        assert_eq!(blockchain.blocks.len(), 1);
        assert_eq!(blockchain.difficulty, constant::INITIAL_DIFFICULTY);
    }

    #[test]
    fn test_reward_halving() {
        let blockchain = Blockchain::new();
        assert_eq!(blockchain.reward(), constant::BASE_REWARD);
    }
}
