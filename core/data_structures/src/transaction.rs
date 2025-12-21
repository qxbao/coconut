use k256::ecdsa::{Signature, VerifyingKey, signature::Verifier};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxInput {
    pub txid: [u8; 32],
    pub out_idx: u32,
    #[serde(with = "BigArray")]
    pub signature: [u8; 72],
    #[serde(with = "BigArray")]
    pub public_key: [u8; 33],
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxOutput {
    pub value: u64,
    pub address: [u8; 20],
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

    pub fn new_coinbase(receiver_address: [u8; 20], amount: u64) -> Self {
        Self {
            inputs: vec![],
            outputs: vec![TxOutput {
                value: amount,
                address: receiver_address,
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
            input.signature = [0u8; 72];
        }
        let serialized = bincode2::serialize(&tx_copy).expect("Serialization failed");
        let hash = crypto::compute_sha256x2(&serialized);
        hash.to_big_endian()
    }

    pub fn verify(&self, utxo_set: &UTXOSet) -> Result<(), String> {
        if self.is_coinbase() {
            return Ok(());
        }

        if self.inputs.is_empty() || self.outputs.is_empty() {
            return Err("Transaction must have at least one input and one output".into());
        }
        let mut output_sum = 0u64;
        for output in &self.outputs {
            if output.value == 0 {
                return Err("Transaction output value must be greater than zero".into());
            }
            output_sum = output_sum
                .checked_add(output.value)
                .ok_or("Output sum overflowed u64")?;
        }

        let message_hash = self.hash_for_signature();

        let input_sum = self.inputs.iter().try_fold(0u64, |acc, input| {
            let utxo = utxo_set
                .utxos
                .get(&input.txid)
                .and_then(|outputs| outputs.iter().find(|(idx, _)| *idx == input.out_idx))
                .ok_or_else(|| format!("UTXO not found for txid: {}", hex::encode(input.txid)))?;

            let calculated_hash = crypto::hash_public_key(&input.public_key);
            if calculated_hash != utxo.1.address {
                return Err("Signature belongs to a public key that doesn't own this UTXO".into());
            }

            self.verify_input_signature(input, &message_hash)?;

            acc.checked_add(utxo.1.value)
                .ok_or_else(|| "Input sum overflowed u64".to_string())
        })?;

        let output_sum = self.outputs.iter().try_fold(0u64, |acc, output| {
            acc.checked_add(output.value)
                .ok_or_else(|| "Output sum overflowed u64".to_string())
        })?;

        if input_sum < output_sum {
            return Err(format!(
                "Insufficient funds: input {} < output {}",
                input_sum, output_sum
            ));
        }

        Ok(())
    }

    fn verify_input_signature(
        &self,
        input: &TxInput,
        message_hash: &[u8; 32],
    ) -> Result<(), String> {
        let signature =
            Signature::from_slice(&input.signature).map_err(|_| "Invalid signature format")?;

        let verifying_key = VerifyingKey::from_sec1_bytes(&input.public_key)
            .map_err(|_| "Invalid public key format")?;

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

    pub fn get_balance(&self, pubkey_hash: &[u8; 20]) -> u64 {
        self.utxos
            .values()
            .flatten()
            .filter(|(_, output)| output.address == *pubkey_hash)
            .map(|(_, output)| output.value)
            .sum()
    }
}
