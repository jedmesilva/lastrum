//! Certificate validation functionality
//! 
//! This module provides functions for validating certificates
//! and their digital signatures.

use ed25519_dalek::{PublicKey, Verifier};

use crate::certifield::model::Certificate;
use crate::core::keypair::KeyPair;
use crate::errors::LastrumError;

/// A validator for verifying digital signatures
pub struct Validator;

impl Validator {
    /// Create a new validator
    pub fn new() -> Self {
        Self {}
    }
    
    /// Verify that a signature is valid for the given data and public key
    pub fn verify(&self, data: &[u8], signature: &[u8], public_key: &PublicKey) -> Result<bool, LastrumError> {
        // Convert signature bytes to a Signature
        let sig = ed25519_dalek::Signature::from_bytes(signature)
            .map_err(|e| LastrumError::SignatureError(format!("Invalid signature format: {}", e)))?;
        
        // Verify the signature
        match public_key.verify(data, &sig) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    /// Verify a certificate's signature
    pub fn verify_certificate(&self, certificate: &Certificate) -> Result<bool, LastrumError> {
        // If there's no signature, the certificate can't be verified
        let signature = match &certificate.signature {
            Some(sig) => sig,
            None => return Err(LastrumError::SignatureError("Certificate has no signature".into())),
        };
        
        // Decode the signature from hex
        let signature_bytes = hex::decode(signature)
            .map_err(|e| LastrumError::SignatureError(format!("Invalid signature hex: {}", e)))?;
        
        // Get the certificate's raw data without the signature
        let certificate_data = certificate.to_signable_bytes()?;
        
        // Parse the public key from the certificate
        let public_key = KeyPair::public_key_from_hex(&certificate.issuer_public_key)?;
        
        // Verify the signature
        self.verify(&certificate_data, &signature_bytes, &public_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certifield::{
        model::{Asset, AssetType},
        builder::CertificateBuilder,
    };
    use crate::core::{keypair::KeyPair, signer::Signer};
    
    #[test]
    fn test_verify_valid_signature() {
        let keypair = KeyPair::generate().unwrap();
        let signer = Signer::new(keypair.clone());
        let validator = Validator::new();
        
        let data = b"test data";
        let signature = signer.sign(data).unwrap();
        
        let public_key = ed25519_dalek::PublicKey::from_bytes(&keypair.public_key_bytes()).unwrap();
        let result = validator.verify(data, &signature, &public_key).unwrap();
        
        assert!(result);
    }
    
    #[test]
    fn test_verify_invalid_signature() {
        let keypair = KeyPair::generate().unwrap();
        let keypair2 = KeyPair::generate().unwrap();
        let signer = Signer::new(keypair.clone());
        let validator = Validator::new();
        
        let data = b"test data";
        let signature = signer.sign(data).unwrap();
        
        // Use a different public key for verification
        let public_key = ed25519_dalek::PublicKey::from_bytes(&keypair2.public_key_bytes()).unwrap();
        let result = validator.verify(data, &signature, &public_key).unwrap();
        
        assert!(!result);
    }
    
    #[test]
    fn test_verify_certificate() {
        let keypair = KeyPair::generate().unwrap();
        let signer = Signer::new(keypair.clone());
        let validator = Validator::new();
        
        let asset = Asset::new(
            AssetType::Gold,
            100.0,
            0.999,
            "SERIAL123".to_string(),
        );
        
        let certificate = CertificateBuilder::new()
            .with_issuer("Test Custody".to_string())
            .with_issuer_public_key(keypair.public_key_hex())
            .with_asset(asset)
            .build()
            .unwrap();
        
        // Sign the certificate
        let signed = signer.sign_certificate(certificate).unwrap();
        
        // Verify the certificate
        let result = validator.verify_certificate(&signed).unwrap();
        assert!(result);
        
        // Modify the certificate and verify it fails
        let mut tampered = signed.clone();
        tampered.asset.weight = 200.0; // Change the weight
        
        // This should fail verification
        let result = validator.verify_certificate(&tampered).unwrap();
        assert!(!result);
    }
}
