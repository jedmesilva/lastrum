//! Digital signature functionality
//! 
//! This module provides functions for signing data using
//! the ed25519 signature algorithm.

use ed25519_dalek::{Signer as DalekSigner};

use crate::certifield::model::Certificate;
use crate::core::keypair::KeyPair;
use crate::errors::LastrumError;

/// A signer for creating digital signatures
pub struct Signer {
    keypair: KeyPair,
}

impl Signer {
    /// Create a new signer with the given keypair
    pub fn new(keypair: KeyPair) -> Self {
        Self { keypair }
    }
    
    /// Sign arbitrary data
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, LastrumError> {
        let signature = self.keypair.inner().sign(data);
        Ok(signature.to_bytes().to_vec())
    }
    
    /// Sign a certificate, adding the signature to it
    pub fn sign_certificate(&self, mut certificate: Certificate) -> Result<Certificate, LastrumError> {
        // Create a serialized representation of the certificate without the signature
        let certificate_data = certificate.to_signable_bytes()?;
        
        // Sign the certificate data
        let signature = self.sign(&certificate_data)?;
        let signature_hex = hex::encode(&signature);
        
        // Add the signature to the certificate
        certificate.signature = Some(signature_hex);
        
        Ok(certificate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certifield::{
        model::{Asset, AssetType},
        builder::CertificateBuilder,
    };
    
    #[test]
    fn test_sign_data() {
        let keypair = KeyPair::generate().unwrap();
        let signer = Signer::new(keypair);
        
        let data = b"test data";
        let signature = signer.sign(data).unwrap();
        
        assert_eq!(signature.len(), 64); // ed25519 signatures are 64 bytes
    }
    
    #[test]
    fn test_sign_certificate() {
        let keypair = KeyPair::generate().unwrap();
        let signer = Signer::new(keypair.clone());
        
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
        
        // Certificate should not have a signature yet
        assert!(certificate.signature.is_none());
        
        // Sign the certificate
        let signed = signer.sign_certificate(certificate).unwrap();
        
        // Now it should have a signature
        assert!(signed.signature.is_some());
        assert_eq!(signed.signature.as_ref().unwrap().len(), 128); // 64 bytes as hex = 128 chars
    }
}
