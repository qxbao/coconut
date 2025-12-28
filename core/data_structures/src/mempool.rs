use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
};

use crate::transaction::Transaction;

#[derive(Clone)]
struct PrioritizedTx {
    tx: Transaction,
    fee_per_byte: u64,
    total_fee: u64,
}

impl Ord for PrioritizedTx {
    fn cmp(&self, other: &Self) -> Ordering {
        self.fee_per_byte.cmp(&other.fee_per_byte)
    }
}

impl PartialOrd for PrioritizedTx {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for PrioritizedTx {}

impl PartialEq for PrioritizedTx {
    fn eq(&self, other: &Self) -> bool {
        self.fee_per_byte == other.fee_per_byte
    }
}

pub struct Mempool {
    pub transactions: HashMap<[u8; 32], Transaction>,
    priority_queue: BinaryHeap<PrioritizedTx>,
    max_size: usize,
}

impl Mempool {
    pub fn new(max_size: usize) -> Self {
        Self {
            transactions: HashMap::new(),
            priority_queue: BinaryHeap::new(),
            max_size,
        }
    }

    pub fn get_proposals(&self, max_count: usize) -> Vec<(Transaction, u64)> {
        self.priority_queue
            .iter()
            .take(max_count)
            .map(|ptx| (ptx.tx.clone(), ptx.total_fee))
            .collect()
    }

    pub fn pop_best(&mut self) -> Option<Transaction> {
        while let Some(ptx) = self.priority_queue.pop() {
            let txid = ptx.tx.hash();
            if self.transactions.contains_key(&txid) {
                self.transactions.remove(&txid);
                return Some(ptx.tx);
            }
        }
        None
    }

    pub fn remove_confirmed(&mut self, txids: &[[u8; 32]]) {
        for txid in txids {
            self.transactions.remove(txid);
        }
    }

    pub fn add_transaction(&mut self, tx: Transaction, fee: u64) -> Result<(), String> {
        if self.transactions.len() >= self.max_size {
            return Err("Mempool is full".into());
        }

        let txid = tx.hash();
        if self.transactions.contains_key(&txid) {
            return Err("Transaction already in mempool".into());
        }

        let tx_size = bincode2::serialize(&tx).unwrap().len() as u64;
        let fee_per_byte = fee / tx_size;

        if self.transactions.len() >= self.max_size {
            if let Some(lowest) = self.priority_queue.peek() {
                if fee_per_byte <= lowest.fee_per_byte {
                    return Err(format!(
                        "Transaction rejected: fee of {} per byte is too low. Mempool is full and minimum required fee is {} per byte",
                        fee_per_byte,
                        lowest.fee_per_byte
                    ));
                }
                let to_remove = self.priority_queue.pop().unwrap();
                self.transactions.remove(&to_remove.tx.hash());
            }
        }

        self.transactions.insert(txid, tx.clone());
        self.priority_queue.push(PrioritizedTx {
            tx,
            fee_per_byte,
            total_fee: fee,
        });

        Ok(())
    }
}
