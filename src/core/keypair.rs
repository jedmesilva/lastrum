//! Keypair management for cryptographic operations
//! 
//! This module handles the creation and management of
//! cryptographic keypairs used for signing and verification.

use ed25519_dalek::{Keypair as DalekKeypair, PublicKey, SecretKey, PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH};
// Use the version of rand that ed25519-dalek expects (0.7.x)
use rand_7 as rand;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use base64::{encode, decode};

use crate::errors::LastrumError;

/// Represents a cryptographic keypair (public and private keys)
pub struct KeyPair {
    /// The underlying ed25519 keypair
    keypair: DalekKeypair,
}

// Manually implement Clone since ed25519_dalek::Keypair doesn't implement Clone
impl Clone for KeyPair {
    fn clone(&self) -> Self {
        // Create a new keypair from the secret key
        let secret_bytes = self.keypair.secret.to_bytes();
        KeyPair::from_secret_bytes(&secret_bytes).unwrap()
    }
}

impl KeyPair {
    /// Generate a new random keypair
    pub fn generate() -> Result<Self, LastrumError> {
        let mut csprng = rand_7::rngs::OsRng{};
        let keypair = DalekKeypair::generate(&mut csprng);
        
        Ok(Self { keypair })
    }
    
    /// Create a keypair from existing secret key bytes
    pub fn from_secret_bytes(bytes: &[u8]) -> Result<Self, LastrumError> {
        if bytes.len() != SECRET_KEY_LENGTH {
            return Err(LastrumError::KeyError("Invalid secret key length".into()));
        }
        
        let secret_key = SecretKey::from_bytes(bytes)
            .map_err(|e| LastrumError::KeyError(format!("Failed to create secret key: {}", e)))?;
        
        let public_key = PublicKey::from(&secret_key);
        let keypair = DalekKeypair { secret: secret_key, public: public_key };
        
        Ok(Self { keypair })
    }
    
    /// Get the public key as bytes
    pub fn public_key_bytes(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        self.keypair.public.to_bytes()
    }
    
    /// Get the public key as a hex string
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.keypair.public.to_bytes())
    }
    
    /// Get the secret key as bytes
    pub fn secret_key_bytes(&self) -> [u8; SECRET_KEY_LENGTH] {
        self.keypair.secret.to_bytes()
    }
    
    /// Convert from a hex string to a public key
    pub fn public_key_from_hex(hex_str: &str) -> Result<PublicKey, LastrumError> {
        let bytes = hex::decode(hex_str)
            .map_err(|e| LastrumError::KeyError(format!("Invalid hex string: {}", e)))?;
        
        if bytes.len() != PUBLIC_KEY_LENGTH {
            return Err(LastrumError::KeyError("Invalid public key length".into()));
        }
        
        let mut public_key_bytes = [0u8; PUBLIC_KEY_LENGTH];
        public_key_bytes.copy_from_slice(&bytes);
        
        PublicKey::from_bytes(&public_key_bytes)
            .map_err(|e| LastrumError::KeyError(format!("Failed to create public key: {}", e)))
    }
    
    /// Get a reference to the underlying keypair
    pub fn inner(&self) -> &DalekKeypair {
        &self.keypair
    }
}

// Custom serialization for KeyPair
impl Serialize for KeyPair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let public_key = self.keypair.public.to_bytes();
        let secret_key = self.keypair.secret.to_bytes();
        
        let mut combined = Vec::with_capacity(PUBLIC_KEY_LENGTH + SECRET_KEY_LENGTH);
        combined.extend_from_slice(&public_key);
        combined.extend_from_slice(&secret_key);
        
        let encoded = encode(&combined);
        serializer.serialize_str(&encoded)
    }
}

// Custom deserialization for KeyPair
impl<'de> Deserialize<'de> for KeyPair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        
        let encoded = String::deserialize(deserializer)?;
        let combined = decode(&encoded).map_err(|e| Error::custom(format!("Base64 decode error: {}", e)))?;
        
        if combined.len() != PUBLIC_KEY_LENGTH + SECRET_KEY_LENGTH {
            return Err(Error::custom("Invalid keypair data length"));
        }
        
        let mut secret_key_bytes = [0u8; SECRET_KEY_LENGTH];
        secret_key_bytes.copy_from_slice(&combined[PUBLIC_KEY_LENGTH..]);
        
        let secret_key = SecretKey::from_bytes(&secret_key_bytes)
            .map_err(|e| Error::custom(format!("Invalid secret key: {}", e)))?;
        
        let public_key = PublicKey::from(&secret_key);
        let keypair = DalekKeypair { secret: secret_key, public: public_key };
        
        Ok(KeyPair { keypair })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_keypair_generation() {
        let keypair = KeyPair::generate().unwrap();
        assert_eq!(keypair.public_key_bytes().len(), PUBLIC_KEY_LENGTH);
        assert_eq!(keypair.secret_key_bytes().len(), SECRET_KEY_LENGTH);
    }
    
    #[test]
    fn test_keypair_from_secret() {
        let original = KeyPair::generate().unwrap();
        let copied = KeyPair::from_secret_bytes(&original.secret_key_bytes()).unwrap();
        
        assert_eq!(original.public_key_hex(), copied.public_key_hex());
    }
    
    #[test]
    fn test_public_key_from_hex() {
        let keypair = KeyPair::generate().unwrap();
        let hex_key = keypair.public_key_hex();
        let parsed_key = KeyPair::public_key_from_hex(&hex_key).unwrap();
        
        assert_eq!(keypair.keypair.public.to_bytes(), parsed_key.to_bytes());
    }
    
    #[test]
    fn test_serialization_deserialization() {
        let original = KeyPair::generate().unwrap();
        
        // Serialize to JSON
        let serialized = serde_json::to_string(&original).unwrap();
        
        // Deserialize back
        let deserialized: KeyPair = serde_json::from_str(&serialized).unwrap();
        
        // Compare public keys
        assert_eq!(original.public_key_hex(), deserialized.public_key_hex());
    }
}
