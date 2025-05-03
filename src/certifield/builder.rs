//! Certificate builder for creating certificates
//! 
//! This module provides a builder pattern for creating certificates.

use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::certifield::model::{Certificate, Asset, RequestType};
use crate::errors::LastrumError;

/// Builder for creating certificates
pub struct CertificateBuilder {
    id: Option<String>,
    custody_house_id: Option<String>,
    custody_house_hash: Option<String>,
    authorized_wallets: Option<Vec<String>>,
    asset: Option<Asset>,
    description: Option<String>,
    expires_at: Option<DateTime<Utc>>,
    certificate_private_key: Option<String>,
}

impl CertificateBuilder {
    /// Create a new certificate builder
    pub fn new() -> Self {
        Self {
            id: None,
            custody_house_id: None,
            custody_house_hash: None,
            authorized_wallets: None,
            asset: None,
            description: None,
            expires_at: None,
            certificate_private_key: None,
        }
    }
    
    /// Set the certificate ID
    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }
    
    /// Set the custody house ID (name)
    pub fn with_custody_house_id(mut self, custody_house_id: String) -> Self {
        self.custody_house_id = Some(custody_house_id);
        self
    }
    
    /// Set the custody house hash (public key)
    pub fn with_custody_house_hash(mut self, custody_house_hash: String) -> Self {
        self.custody_house_hash = Some(custody_house_hash);
        self
    }
    
    /// Set the authorized wallets
    pub fn with_authorized_wallets(mut self, authorized_wallets: Vec<String>) -> Self {
        if !authorized_wallets.is_empty() {
            self.authorized_wallets = Some(authorized_wallets);
        }
        self
    }
    
    /// Add an authorized wallet
    pub fn add_authorized_wallet(mut self, wallet: String) -> Self {
        match &mut self.authorized_wallets {
            Some(wallets) => wallets.push(wallet),
            None => self.authorized_wallets = Some(vec![wallet]),
        }
        self
    }
    
    /// Set the asset
    pub fn with_asset(mut self, asset: Asset) -> Self {
        self.asset = Some(asset);
        self
    }
    
    /// Set the description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
    
    /// Set the expiration date
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    /// Set the certificate private key
    pub fn with_certificate_private_key(mut self, private_key: String) -> Self {
        self.certificate_private_key = Some(private_key);
        self
    }
    
    /// Build the certificate
    pub fn build(self) -> Result<Certificate, LastrumError> {
        // Get the custody house ID or return an error
        let custody_house_id = self.custody_house_id
            .ok_or_else(|| LastrumError::BuilderError("Custody house ID is required".into()))?;
        
        // Get the custody house hash or return an error
        let custody_house_hash = self.custody_house_hash
            .ok_or_else(|| LastrumError::BuilderError("Custody house hash is required".into()))?;
        
        // Get the asset or return an error
        let asset = self.asset
            .ok_or_else(|| LastrumError::BuilderError("Asset is required".into()))?;
        
        // Get the description or return an error
        let description = self.description
            .ok_or_else(|| LastrumError::BuilderError("Description is required".into()))?;
        
        // Validate the asset
        asset.validate()?;
        
        // Create the certificate
        Ok(Certificate {
            request_type: RequestType::EmitCertificate,
            id: self.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            custody_house_id,
            custody_house_hash,
            authorized_wallets: self.authorized_wallets,
            asset,
            description,
            issued_at: Utc::now(),
            expires_at: self.expires_at,
            certificate_private_key: self.certificate_private_key,
            signature: None,
            token_ledgers: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certifield::model::{Asset, AssetType};
    
    #[test]
    fn test_builder_with_all_fields() {
        let asset = Asset::new_with_purity(
            AssetType::Gold, 
            100.0, 
            "g".to_string(), 
            0.999, 
            "SERIAL123".to_string()
        );
        let id = Uuid::new_v4().to_string();
        let description = "Certificação de 100g de ouro 999 sob custódia.".to_string();
        
        let certificate = CertificateBuilder::new()
            .with_id(id.clone())
            .with_custody_house_id("Test Custody".to_string())
            .with_custody_house_hash("0123456789abcdef".to_string())
            .with_asset(asset.clone())
            .with_description(description.clone())
            .with_authorized_wallets(vec!["wallet1".to_string(), "wallet2".to_string()])
            .with_certificate_private_key("private_key_hex".to_string())
            .build()
            .unwrap();
        
        assert_eq!(certificate.id, id);
        assert_eq!(certificate.custody_house_id, "Test Custody");
        assert_eq!(certificate.custody_house_hash, "0123456789abcdef");
        assert_eq!(certificate.asset, asset);
        assert_eq!(certificate.description, description);
        assert_eq!(certificate.authorized_wallets.as_ref().unwrap().len(), 2);
        assert_eq!(certificate.certificate_private_key.unwrap(), "private_key_hex");
        assert!(certificate.signature.is_none());
        assert!(certificate.token_ledgers.is_empty());
    }
    
    #[test]
    fn test_builder_without_id() {
        let asset = Asset::new_with_purity(
            AssetType::Gold, 
            100.0, 
            "g".to_string(), 
            0.999, 
            "SERIAL123".to_string()
        );
        let description = "Certificação de ouro sob custódia.".to_string();
        
        let certificate = CertificateBuilder::new()
            .with_custody_house_id("Test Custody".to_string())
            .with_custody_house_hash("0123456789abcdef".to_string())
            .with_asset(asset.clone())
            .with_description(description.clone())
            .build()
            .unwrap();
        
        // ID should be auto-generated
        assert!(!certificate.id.is_empty());
        assert_eq!(certificate.custody_house_id, "Test Custody");
        assert_eq!(certificate.custody_house_hash, "0123456789abcdef");
        assert_eq!(certificate.asset, asset);
        assert_eq!(certificate.description, description);
    }
    
    #[test]
    fn test_builder_with_expiration() {
        let asset = Asset::new(
            AssetType::BiogasMetano, 
            275.5, 
            "m3".to_string(), 
            "BIO-001".to_string()
        );
        let description = "Biogás metano certificado.".to_string();
        let expires_at = Utc::now() + chrono::Duration::days(365);
        
        let certificate = CertificateBuilder::new()
            .with_custody_house_id("BioCustódia".to_string())
            .with_custody_house_hash("0123456789abcdef".to_string())
            .with_asset(asset)
            .with_description(description)
            .with_expiration(expires_at)
            .build()
            .unwrap();
        
        assert_eq!(certificate.expires_at.unwrap(), expires_at);
        assert!(!certificate.is_expired());
    }
    
    #[test]
    fn test_builder_missing_fields() {
        let asset = Asset::new_with_purity(
            AssetType::Gold, 
            100.0, 
            "g".to_string(), 
            0.999, 
            "SERIAL123".to_string()
        );
        let description = "Certificação de ouro sob custódia.".to_string();
        
        // Missing custody house ID
        let result = CertificateBuilder::new()
            .with_custody_house_hash("0123456789abcdef".to_string())
            .with_asset(asset.clone())
            .with_description(description.clone())
            .build();
        assert!(result.is_err());
        
        // Missing custody house hash
        let result = CertificateBuilder::new()
            .with_custody_house_id("Test Custody".to_string())
            .with_asset(asset.clone())
            .with_description(description.clone())
            .build();
        assert!(result.is_err());
        
        // Missing asset
        let result = CertificateBuilder::new()
            .with_custody_house_id("Test Custody".to_string())
            .with_custody_house_hash("0123456789abcdef".to_string())
            .with_description(description.clone())
            .build();
        assert!(result.is_err());
        
        // Missing description
        let result = CertificateBuilder::new()
            .with_custody_house_id("Test Custody".to_string())
            .with_custody_house_hash("0123456789abcdef".to_string())
            .with_asset(asset.clone())
            .build();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_builder_add_authorized_wallet() {
        let asset = Asset::new(
            AssetType::EnergiaSolar, 
            1000.0, 
            "kWh".to_string(), 
            "SOLAR-001".to_string()
        );
        let description = "Energia solar certificada.".to_string();
        
        let certificate = CertificateBuilder::new()
            .with_custody_house_id("EnergiaCustódia".to_string())
            .with_custody_house_hash("0123456789abcdef".to_string())
            .with_asset(asset)
            .with_description(description)
            .add_authorized_wallet("wallet1".to_string())
            .add_authorized_wallet("wallet2".to_string())
            .build()
            .unwrap();
        
        assert_eq!(certificate.authorized_wallets.as_ref().unwrap().len(), 2);
        assert_eq!(certificate.authorized_wallets.as_ref().unwrap()[0], "wallet1");
        assert_eq!(certificate.authorized_wallets.as_ref().unwrap()[1], "wallet2");
    }
}
