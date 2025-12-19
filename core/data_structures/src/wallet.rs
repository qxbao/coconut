use k256::{
    PublicKey, SecretKey,
    ecdsa::{Signature, SigningKey, signature::SignerMut},
    elliptic_curve::{rand_core::OsRng, sec1::ToEncodedPoint},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct WalletAddress {
    pub pubkey_hash: [u8; 20],
    pub network: constant::BitcoinNetwork,
}

impl WalletAddress {
    pub fn to_string(&self) -> String {
        crypto::pubkey_hash_to_address(&self.pubkey_hash, &self.network)
    }
}

pub struct Wallet {
    pub private_key: SecretKey,
    pub public_key: PublicKey,
    pub address: WalletAddress,
}

impl Wallet {
    pub fn new() -> Self {
        let private_key = SecretKey::random(&mut OsRng);

        let public_key = PublicKey::from_secret_scalar(&private_key.to_nonzero_scalar());
        let pubkey_bytes = public_key.to_encoded_point(true);

        let pubkey_hash = crypto::hash_public_key(pubkey_bytes.as_bytes());

        Self {
            private_key,
            public_key,
            address: WalletAddress {
                pubkey_hash,
                network: constant::NETWORK_TYPE,
            },
        }
    }

    pub fn sign(&self, message_hash: &[u8; 32]) -> Result<Vec<u8>, String> {
        let mut signing_key = SigningKey::from(&self.private_key);

        let signature: Signature = signing_key.sign(message_hash);

        Ok(signature.to_vec())
    }

    pub fn get_public_key_fixed(&self) -> [u8; 33] {
        let point = self.public_key.to_encoded_point(true);
        let mut pub_bytes = [0u8; 33];
        pub_bytes.copy_from_slice(point.as_bytes());
        pub_bytes
    }
}

#[cfg(test)]
mod wallet_test {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = Wallet::new();

        let addr_str = wallet.address.to_string();
        println!("Address: {}", addr_str);
        println!("Public Key: {}", hex::encode(wallet.get_public_key_fixed()));
        println!(
            "Private Key: {}",
            hex::encode(wallet.private_key.to_bytes())
        );

        assert_eq!(wallet.address.pubkey_hash.len(), 20);
        assert!(
            addr_str.starts_with('1') || addr_str.starts_with('m') || addr_str.starts_with('n')
        );
    }

    #[test]
    fn test_keypair_matching() {
        let wallet = Wallet::new();
        let derived_pubkey = PublicKey::from_secret_scalar(&wallet.private_key.to_nonzero_scalar());
        assert_eq!(
            derived_pubkey, wallet.public_key,
            "Derived public key does not match stored public key"
        );
    }
}

impl std::fmt::Display for WalletAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}
