//! Certificate builder for creating certificates
//! 
//! This module provides a builder pattern for creating certificates.

use uuid::Uuid;
use chrono::Utc;

use crate::certifield::model::{Certificate, Asset};
use crate::errors::LastrumError;

/// Builder for creating certificates
pub struct CertificateBuilder {
    id: Option<String>,
    issuer: Option<String>,
    issuer_public_key: Option<String>,
    asset: Option<Asset>,
}

impl CertificateBuilder {
    /// Create a new certificate builder
    pub fn new() -> Self {
        Self {
            id: None,
            issuer: None,
            issuer_public_key: None,
            asset: None,
        }
    }
    
    /// Set the certificate ID
    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }
    
    /// Set the issuer name
    pub fn with_issuer(mut self, issuer: String) -> Self {
        self.issuer = Some(issuer);
        self
    }
    
    /// Set the issuer public key
    pub fn with_issuer_public_key(mut self, issuer_public_key: String) -> Self {
        self.issuer_public_key = Some(issuer_public_key);
        self
    }
    
    /// Set the asset
    pub fn with_asset(mut self, asset: Asset) -> Self {
        self.asset = Some(asset);
        self
    }
    
    /// Build the certificate
    pub fn build(self) -> Result<Certificate, LastrumError> {
        // Get the issuer or return an error
        let issuer = self.issuer
            .ok_or_else(|| LastrumError::BuilderError("Issuer is required".into()))?;
        
        // Get the issuer public key or return an error
        let issuer_public_key = self.issuer_public_key
            .ok_or_else(|| LastrumError::BuilderError("Issuer public key is required".into()))?;
        
        // Get the asset or return an error
        let asset = self.asset
            .ok_or_else(|| LastrumError::BuilderError("Asset is required".into()))?;
        
        // Validate the asset
        asset.validate()?;
        
        // Create the certificate
        Ok(Certificate {
            id: self.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            issuer,
            issuer_public_key,
            asset,
            issued_at: Utc::now(),
            signature: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certifield::model::{Asset, AssetType};
    
    #[test]
    fn test_builder_with_all_fields() {
        let asset = Asset::new(AssetType::Gold, 100.0, 0.999, "SERIAL123".to_string());
        let id = Uuid::new_v4().to_string();
        
        let certificate = CertificateBuilder::new()
            .with_id(id.clone())
            .with_issuer("Test Custody".to_string())
            .with_issuer_public_key("0123456789abcdef".to_string())
            .with_asset(asset.clone())
            .build()
            .unwrap();
        
        assert_eq!(certificate.id, id);
        assert_eq!(certificate.issuer, "Test Custody");
        assert_eq!(certificate.issuer_public_key, "0123456789abcdef");
        assert_eq!(certificate.asset, asset);
        assert!(certificate.signature.is_none());
    }
    
    #[test]
    fn test_builder_without_id() {
        let asset = Asset::new(AssetType::Gold, 100.0, 0.999, "SERIAL123".to_string());
        
        let certificate = CertificateBuilder::new()
            .with_issuer("Test Custody".to_string())
            .with_issuer_public_key("0123456789abcdef".to_string())
            .with_asset(asset.clone())
            .build()
            .unwrap();
        
        // ID should be auto-generated
        assert!(!certificate.id.is_empty());
        assert_eq!(certificate.issuer, "Test Custody");
        assert_eq!(certificate.issuer_public_key, "0123456789abcdef");
        assert_eq!(certificate.asset, asset);
    }
    
    #[test]
    fn test_builder_missing_fields() {
        let asset = Asset::new(AssetType::Gold, 100.0, 0.999, "SERIAL123".to_string());
        
        // Missing issuer
        let result = CertificateBuilder::new()
            .with_issuer_public_key("0123456789abcdef".to_string())
            .with_asset(asset.clone())
            .build();
        assert!(result.is_err());
        
        // Missing issuer public key
        let result = CertificateBuilder::new()
            .with_issuer("Test Custody".to_string())
            .with_asset(asset.clone())
            .build();
        assert!(result.is_err());
        
        // Missing asset
        let result = CertificateBuilder::new()
            .with_issuer("Test Custody".to_string())
            .with_issuer_public_key("0123456789abcdef".to_string())
            .build();
        assert!(result.is_err());
    }
}
