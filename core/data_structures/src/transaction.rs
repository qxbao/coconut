use std::collections::HashMap;
use k256::ecdsa::{Signature, VerifyingKey, signature::Verifier};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxInput {
    pub txid: [u8; 32],
    pub out_idx: u32,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxOutput {
    pub value: u64,
    pub pubkey_hash: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub lock_time: u32,
}

impl Transaction {
    pub fn new(inputs: Vec<TxInput>, outputs: Vec<TxOutput>, lock_time: u32) -> Self {
        Self {
            inputs,
            outputs,
            lock_time,
        }
    }

    pub fn new_coinbase(receiver_pubkey_hash: String, amount: u64) -> Self {
        Self {
            inputs: vec![],
            outputs: vec![TxOutput {
                value: amount,
                pubkey_hash: receiver_pubkey_hash,
            }],
            lock_time: 0,
        }
    }

    pub fn hash(&self) -> [u8; 32] {
        let serialized = bincode2::serialize(&self).expect("Failed to serialize transaction");
        let hash = crypto::compute_sha256x2(&serialized);
        hash.to_big_endian()
    }

    pub fn hash_for_signature(&self) -> [u8; 32] {
        let mut tx_copy = self.clone();
        for input in &mut tx_copy.inputs {
            input.signature = vec![];
        }
        let serialized = bincode2::serialize(&tx_copy).expect("Serialization failed");
        let hash = crypto::compute_sha256x2(&serialized);
        hash.to_big_endian()
    }

    pub fn verify(&self, utxo_set: &UTXOSet) -> Result<(), String> {
        if self.is_coinbase() {
            return Ok(());
        }

        if self.inputs.is_empty() {
            return Err("Non-coinbase transaction must have inputs".into());
        }

        let message_hash = self.hash_for_signature();
        let mut input_sum = 0u64;

        for input in &self.inputs {
            let utxo = utxo_set.utxos
                .get(&input.txid)
                .and_then(|outputs| outputs.iter().find(|(idx, _)| *idx == input.out_idx))
                .ok_or(format!("UTXO not found for txid: {}", hex::encode(input.txid)))?;

            let calculated_hash = crypto::hash_public_key(&input.public_key);
            if hex::encode(&calculated_hash) != utxo.1.pubkey_hash {
                return Err("Signature belongs to a public key that doesn't own this UTXO".into());
            }

            self.verify_input_signature(input, &message_hash)?;

            input_sum += utxo.1.value;
        }

        let output_sum: u64 = self.outputs.iter().map(|o| o.value).sum();
        if input_sum < output_sum {
            return Err(format!("Insufficient funds: input {} < output {}", input_sum, output_sum));
        }

        Ok(())
    }

    fn verify_input_signature(&self, input: &TxInput, message_hash: &[u8; 32]) -> Result<(), String> {
        let verifying_key = VerifyingKey::from_sec1_bytes(&input.public_key)
            .map_err(|_| "Invalid public key format")?;
        
        let signature = Signature::from_der(&input.signature)
            .map_err(|_| "Invalid signature format")?;
        verifying_key
            .verify(message_hash, &signature)
            .map_err(|_| "Signature verification failed".to_string())
    }

    pub fn is_coinbase(&self) -> bool {
        self.inputs.is_empty()
    }
}

pub struct UTXOSet {
    pub utxos: HashMap<[u8; 32], Vec<(u32, TxOutput)>>,
}

impl UTXOSet {
    pub fn new() -> Self {
        Self {
            utxos: HashMap::new(),
        }
    }

    pub fn add_utxo(&mut self, txid: [u8; 32], out_idx: u32, output: TxOutput) {
        self.utxos
            .entry(txid)
            .or_insert_with(Vec::new)
            .push((out_idx, output));
    }

    pub fn spend_utxo(&mut self, txid: [u8; 32], index: u32) -> Option<TxOutput> {
        if let Some(outputs) = self.utxos.get_mut(&txid) {
            if let Some(pos) = outputs.iter().position(|(idx, _)| *idx == index) {
                return Some(outputs.remove(pos).1);
            }
        }
        None
    }

    pub fn get_balance(&self, pubkey_hash: &str) -> u64 {
        self.utxos
            .values()
            .flatten()
            .filter(|(_, output)| output.pubkey_hash == pubkey_hash)
            .map(|(_, output)| output.value)
            .sum()
    }
}

