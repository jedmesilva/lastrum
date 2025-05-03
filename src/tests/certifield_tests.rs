//! Integration tests for certificate functionality
//! 
//! Tests for creating, storing, and validating certificates.

use crate::certifield::{
    model::{Asset, AssetType},
    builder::CertificateBuilder,
    storage::CertificateStorage,
};
use crate::core::{
    identity::Identity,
    signer::Signer,
    validator::Validator,
};
// Removido import desnecessário de Connection (usando new_in_memory)

#[test]
fn test_certificate_full_lifecycle() {
    // Create an in-memory database for this test
    let storage = CertificateStorage::new_in_memory().unwrap();
    
    // Create an identity
    let identity = Identity::new("Test Custody House".to_string()).unwrap();
    
    // Create an asset
    let asset = Asset::new(
        AssetType::Gold,
        100.0,
        0.999,
        "SERIAL123".to_string(),
    );
    
    // Create a certificate
    let certificate = CertificateBuilder::new()
        .with_issuer(identity.name().to_string())
        .with_issuer_public_key(identity.keypair().public_key_hex())
        .with_asset(asset)
        .build()
        .unwrap();
    
    // Sign the certificate
    let signer = Signer::new(identity.keypair().clone());
    let signed_certificate = signer.sign_certificate(certificate).unwrap();
    
    // Store the certificate
    let id = storage.store(&signed_certificate).unwrap();
    
    // Load the certificate
    let loaded_certificate = storage.load_by_id(&id).unwrap();
    
    // Verify that the loaded certificate matches the original
    assert_eq!(loaded_certificate.id, signed_certificate.id);
    assert_eq!(loaded_certificate.issuer, signed_certificate.issuer);
    assert_eq!(loaded_certificate.asset.asset_type, signed_certificate.asset.asset_type);
    assert_eq!(loaded_certificate.asset.weight, signed_certificate.asset.weight);
    assert_eq!(loaded_certificate.asset.purity, signed_certificate.asset.purity);
    assert_eq!(loaded_certificate.asset.serial, signed_certificate.asset.serial);
    
    // Verify the loaded certificate's signature
    let validator = Validator::new();
    let is_valid = validator.verify_certificate(&loaded_certificate).unwrap();
    
    assert!(is_valid, "Loaded certificate's signature should be valid");
    
    // List all certificates
    let all_certificates = storage.list_all().unwrap();
    assert_eq!(all_certificates.len(), 1);
    assert_eq!(all_certificates[0].id, signed_certificate.id);
    
    // Delete the certificate
    storage.delete(&id).unwrap();
    
    // Verify that it's gone
    let result = storage.load_by_id(&id);
    assert!(result.is_err());
}
