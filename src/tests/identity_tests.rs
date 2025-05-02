//! Integration tests for identity functionality
//! 
//! Tests for identity creation, saving, and loading.

use crate::core::{
    identity::Identity,
    validator::Validator,
    signer::Signer,
};
use tempfile::tempdir;

#[test]
fn test_identity_full_lifecycle() {
    // Create a temporary directory for the test
    let dir = tempdir().unwrap();
    let dir_path = dir.path().to_owned();
    
    // Create a new identity
    let identity = Identity::new("Test Custody House".to_string()).unwrap();
    
    // Get the keypair and node hash
    let keypair = identity.keypair();
    let node_hash = identity.node_hash();
    
    // Create a test message and sign it
    let message = b"Test message for signing";
    let signer = Signer::new(keypair.clone());
    let signature = signer.sign(message).unwrap();
    
    // Verify the signature
    let validator = Validator::new();
    let public_key = ed25519_dalek::PublicKey::from_bytes(&keypair.public_key_bytes()).unwrap();
    let is_valid = validator.verify(message, &signature, &public_key).unwrap();
    
    assert!(is_valid, "Signature verification should succeed");
    
    // Create a serialized version to test with
    let json = serde_json::to_string_pretty(&identity).unwrap();
    let file_path = dir_path.join(format!("{}.test.json", node_hash));
    std::fs::write(&file_path, json).unwrap();
    
    // Load the identity from the file
    let loaded = Identity::load(file_path.to_str().unwrap()).unwrap();
    
    // Verify it's the same identity
    assert_eq!(loaded.name(), identity.name());
    assert_eq!(loaded.node_hash(), identity.node_hash());
    assert_eq!(loaded.keypair().public_key_hex(), identity.keypair().public_key_hex());
    
    // Sign again with the loaded identity
    let loaded_signer = Signer::new(loaded.keypair().clone());
    let loaded_signature = loaded_signer.sign(message).unwrap();
    
    // Verify the new signature
    let loaded_public_key = ed25519_dalek::PublicKey::from_bytes(&loaded.keypair().public_key_bytes()).unwrap();
    let loaded_is_valid = validator.verify(message, &loaded_signature, &loaded_public_key).unwrap();
    
    assert!(loaded_is_valid, "Signature verification with loaded identity should succeed");
}
