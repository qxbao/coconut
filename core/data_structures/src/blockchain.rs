use crate::{Block, Transaction};

pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub difficulty: usize,
    pub pending_transactions: Vec<Transaction>,
}

impl Blockchain {
    pub fn new(difficulty: usize) -> Self {
        let mut blockchain = Blockchain {
            blocks: Vec::new(),
            difficulty,
            pending_transactions: Vec::new(),
        };
        blockchain.create_genesis_block(constant::QXBAO_ADDRESS);
        blockchain
    }

    fn create_genesis_block(&mut self, miner_address: &str) {
        let mut genesis_block = Block::new(vec![], vec![]);
        genesis_block.transactions.push(Transaction::new_coinbase(
            miner_address.to_string(),
            self.reward(),
            true,
        ));
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
}
