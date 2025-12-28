use primitive_types::U256;

use crate::block::Block;
use crate::error::BlockchainError;
use crate::mempool::Mempool;
use crate::transaction::{self, Transaction};

pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub difficulty: f64,
    pub mempool: Mempool,
    pub utxo_set: transaction::UTXOSet,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            blocks: Vec::new(),
            difficulty: constant::INITIAL_DIFFICULTY,
            mempool: Mempool::new(10000),
            utxo_set: transaction::UTXOSet::new(),
        };
        blockchain.create_genesis_block();
        blockchain
    }

    fn create_genesis_block(&mut self) {
        let genesis_block = Block::new_genesis();
        self.blocks.push(genesis_block);
        let genesis_block = self.blocks.last().expect("genesis block must exist").clone();
        self.update_utxo_set(&genesis_block);
    }

    pub fn add_transaction(&mut self, tx: Transaction) -> Result<(), String> {
        let fee = self.calculate_transaction_fee(&tx);
        self.mempool.add_transaction(tx, fee)
    }

    fn calculate_transaction_fee(&self, tx: &Transaction) -> u64 {
        if tx.is_coinbase() {
            return 0;
        }
        let input_sum: u64 = tx
            .inputs
            .iter()
            .filter_map(|input| {
                self.utxo_set
                    .utxos
                    .get(&input.txid)
                    .and_then(|outputs| outputs.iter().find(|(idx, _)| *idx == input.out_idx))
                    .map(|(_, output)| output.value)
            })
            .sum();
        let output_sum: u64 = tx.outputs.iter().map(|o| o.value).sum();
        input_sum.saturating_sub(output_sum)
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

    fn compute_next_bits(&self) -> u32 {
        let last_block = self.blocks.last().unwrap();

        if (self.blocks.len() as u64) % constant::DIFFICULTY_ADJUSTMENT_INTERVAL != 0 {
            return last_block.header.bits;
        }

        let first_block_index =
            self.blocks.len() - constant::DIFFICULTY_ADJUSTMENT_INTERVAL as usize;
        let first_block = &self.blocks[first_block_index];

        let actual_timespan = last_block.header.timestamp - first_block.header.timestamp;

        let adjusted_timespan = if actual_timespan < constant::TARGET_TIMESPAN / 4 {
            constant::TARGET_TIMESPAN / 4
        } else if actual_timespan > constant::TARGET_TIMESPAN * 4 {
            constant::TARGET_TIMESPAN * 4
        } else {
            actual_timespan
        };

        let mut target = crypto::bits_to_target(last_block.header.bits);
        target = target * U256::from(adjusted_timespan);
        target = target / U256::from(constant::TARGET_TIMESPAN);

        if target > constant::MAX_TARGET {
            target = constant::MAX_TARGET;
        }

        crypto::target_to_bits(target)
    }

    pub fn mine_pending_transactions(
        &mut self,
        miner_pubkey_hash: &[u8; 20],
        max_txs: usize,
        message: Option<String>,
    ) -> Result<(), BlockchainError> {
        // 1. Lấy đề xuất kèm phí
        let pending_data = self.mempool.get_proposals(max_txs);
        if pending_data.is_empty() {
            return Err(BlockchainError::InvalidTransaction("No pending txs".into()));
        }

        // 2. Verify trước khi đào (Fail-fast)
        for (tx, _) in &pending_data {
            tx.verify(&self.utxo_set)
                .map_err(|e| BlockchainError::InvalidTransaction(e))?;
        }

        let bits = self.compute_next_bits();
        let total_fees = Self::calculate_total_fees(&pending_data);
        let coinbase =
            Transaction::new_coinbase(*miner_pubkey_hash, self.reward() + total_fees, message);

        // 3. Chuẩn bị transactions cho Block
        let mut block_transactions = Vec::with_capacity(pending_data.len() + 1);
        block_transactions.push(coinbase);
        // Move hoặc Clone dữ liệu vào block
        block_transactions.extend(pending_data.iter().map(|(tx, _)| tx.clone()));

        let mut block = Block::new(
            bits,
            self.blocks.last().unwrap().hash().to_big_endian(),
            block_transactions,
        );

        // 4. Mining (CPU intensive)
        block
            .mine(bits)
            .map_err(|_| BlockchainError::MiningFailed)?;

        // 5. Chính xác: Xóa những txid đã được đóng vào block này
        let mined_txids: Vec<[u8; 32]> = pending_data.iter().map(|(tx, _)| tx.hash()).collect();
        self.mempool.remove_confirmed(&mined_txids);

        // 6. Cập nhật trạng thái
        self.update_utxo_set(&block);
        self.blocks.push(block);

        Ok(())
    }

    fn update_utxo_set(&mut self, block: &Block) {
        for tx in &block.transactions {
            // Remove spent UTXOs
            for input in &tx.inputs {
                self.utxo_set.spend_utxo(input.txid, input.out_idx);
            }

            // Add new UTXOs
            let txid = tx.hash();
            for (idx, output) in tx.outputs.iter().enumerate() {
                self.utxo_set.add_utxo(txid, idx as u32, output.clone());
            }
        }
    }

    fn calculate_total_fees(pending_data: &[(Transaction, u64)]) -> u64 {
        pending_data.iter().map(|(_, fee)| fee).sum()
    }

    pub fn verify(&self) -> bool {
        for i in 0..self.blocks.len() {
            if i == 0 {
                if crypto::to_hex(self.blocks[i].hash()) != constant::GENESIS_BLOCK_HASH
                    || self.blocks[i].verify() == false
                {
                    return false;
                }
                continue;
            }

            let current_block = &self.blocks[i];
            let previous_block = &self.blocks[i - 1];

            if current_block.header.prev_hash != previous_block.hash().to_big_endian() {
                return false;
            }

            let target = crypto::bits_to_target(current_block.header.bits);
            let block_hash = current_block.hash();
            if block_hash > target || !current_block.verify() {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod blockchain_tests {
    use super::*;
    use constant;

    #[test]
    fn test_blockchain() {
        let blockchain = Blockchain::new();
        assert!(blockchain.verify());
        assert_eq!(blockchain.blocks.len(), 1);
        assert_eq!(blockchain.difficulty, constant::INITIAL_DIFFICULTY);
    }

    #[test]
    fn test_reward_halving() {
        let blockchain = Blockchain::new();
        assert_eq!(blockchain.reward(), constant::BASE_REWARD);
    }
}
