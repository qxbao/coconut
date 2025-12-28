use std::fmt;

#[derive(Debug, Clone)]
pub enum BlockchainError {
    MiningFailed,
    InvalidBlock(String),
    InvalidTransaction(String),
    UTXONotFound,
    InsufficientFunds,
    InvalidSignature,
    DatabaseError(String),
    SerializationError(String),
    DuplicateBlock,
    OrphanBlock,
}

impl fmt::Display for BlockchainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BlockchainError::MiningFailed => write!(f, "Failed to mine block"),
            BlockchainError::InvalidBlock(msg) => write!(f, "Invalid block: {}", msg),
            BlockchainError::InvalidTransaction(msg) => write!(f, "Invalid transaction: {}", msg),
            BlockchainError::UTXONotFound => write!(f, "UTXO not found"),
            BlockchainError::InsufficientFunds => write!(f, "Insufficient funds"),
            BlockchainError::InvalidSignature => write!(f, "Invalid signature"),
            BlockchainError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            BlockchainError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            BlockchainError::DuplicateBlock => write!(f, "Duplicate block"),
            BlockchainError::OrphanBlock => write!(f, "Orphan block"),
        }
    }
}

impl std::error::Error for BlockchainError {}

pub type Result<T> = std::result::Result<T, BlockchainError>;