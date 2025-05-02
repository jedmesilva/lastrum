//! Hash utility functions for Lastrum Certifield
//! 
//! This module provides hash-related utility functions for
//! creating node identifiers and other hash operations.

use sha2::{Digest, Sha256};
use hex;
use crate::errors::LastrumError;

/// Generate a SHA-256 hash from input data
pub fn sha256_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

/// Generate a unique node hash from a name and a public key
pub fn generate_node_hash(name: &str, public_key: &str) -> String {
    let combined = format!("{}:{}", name, public_key);
    sha256_hash(combined.as_bytes())
}

/// Validate if a hash is a valid SHA-256 hash string
pub fn validate_hash(hash: &str) -> Result<(), LastrumError> {
    // Check if it's a valid hex string of the right length (64 chars = 32 bytes)
    if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(LastrumError::InvalidHash);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sha256_hash() {
        let data = b"test data";
        let hash = sha256_hash(data);
        assert_eq!(hash.len(), 64); // SHA-256 is 32 bytes = 64 hex chars
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
    
    #[test]
    fn test_generate_node_hash() {
        let name = "TestCustody";
        let pubkey = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let hash = generate_node_hash(name, pubkey);
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
    
    #[test]
    fn test_validate_hash() {
        let valid_hash = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        assert!(validate_hash(valid_hash).is_ok());
        
        let short_hash = "0123456789abcdef";
        assert!(validate_hash(short_hash).is_err());
        
        let invalid_hash = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcde$";
        assert!(validate_hash(invalid_hash).is_err());
    }
}
