use crate::{Block, Transaction};
use primitive_types::U256;

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
        blockchain.create_genesis_block(constant::QXBAO_ADDRESS);
        blockchain
    }

    fn create_genesis_block(&mut self, miner_address: &str) {
        let mut genesis_block = Block::new([0u8; 32], vec![]);
        genesis_block.transactions.push(Transaction::new_coinbase(
            miner_address.to_string(),
            self.reward(),
            true,
        ));
        genesis_block.mine(self.target());
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

    pub fn target(&self) -> U256 {
        if self.difficulty < 1.0 {
            return constant::MAX_TARGET;
        }
        let diff_scaled = (self.difficulty * constant::PRECISION as f64) as u128;
        
        constant::MAX_TARGET
            .checked_mul(U256::from(constant::PRECISION))
            .expect("It should not overflow")
            .checked_div(U256::from(diff_scaled))
            .unwrap_or(U256::zero())
    }
}

#[cfg(test)]
mod blockchain_tests {
    use super::*;
    use constant;

    #[test]
    fn test_blockchain_creation() {
        let blockchain = Blockchain::new();
        assert_eq!(blockchain.blocks.len(), 1);
        assert_eq!(blockchain.difficulty, constant::INITIAL_DIFFICULTY);
    }

    #[test]
    fn test_reward_halving() {
        let blockchain = Blockchain::new();
        assert_eq!(blockchain.reward(), constant::BASE_REWARD);

        let mut blockchain = Blockchain::new();
        blockchain.blocks.resize(constant::HALVING_INTERVAL as usize, Block::new([0u8; 32], vec![]));
        assert_eq!(blockchain.reward(), constant::BASE_REWARD >> 1);

        blockchain.blocks.resize((constant::HALVING_INTERVAL * 2) as usize, Block::new([0u8; 32], vec![]));
        assert_eq!(blockchain.reward(), constant::BASE_REWARD >> 2);
    }
}
