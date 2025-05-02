//! Integration tests for signing functionality
//! 
//! Tests for creating and verifying signatures.

use crate::certifield::{
    model::{Asset, AssetType, Certificate},
    builder::CertificateBuilder,
};
use crate::core::{
    keypair::KeyPair,
    signer::Signer,
    validator::Validator,
};

#[test]
fn test_certificate_signing_and_verification() {
    // Create a keypair for the test
    let keypair = KeyPair::generate().unwrap();
    let public_key_hex = keypair.public_key_hex();
    
    // Create an asset
    let asset = Asset::new(
        AssetType::Gold,
        100.0,
        0.999,
        "SERIAL123".to_string(),
    );
    
    // Create a certificate
    let certificate = CertificateBuilder::new()
        .with_issuer("Test Custody House".to_string())
        .with_issuer_public_key(public_key_hex)
        .with_asset(asset)
        .build()
        .unwrap();
    
    // Assert that the certificate doesn't have a signature yet
    assert!(certificate.signature.is_none());
    
    // Sign the certificate
    let signer = Signer::new(keypair);
    let signed_certificate = signer.sign_certificate(certificate).unwrap();
    
    // Assert that the certificate now has a signature
    assert!(signed_certificate.signature.is_some());
    
    // Verify the signature
    let validator = Validator::new();
    let is_valid = validator.verify_certificate(&signed_certificate).unwrap();
    
    assert!(is_valid, "Certificate signature should be valid");
    
    // Create a tampered certificate
    let mut tampered_certificate = signed_certificate.clone();
    tampered_certificate.asset.weight = 200.0; // Change the weight
    
    // Verify that the tampered certificate's signature is invalid
    let is_tampered_valid = validator.verify_certificate(&tampered_certificate).unwrap();
    
    assert!(!is_tampered_valid, "Tampered certificate signature should be invalid");
}
