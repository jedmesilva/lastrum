//! Certificate and asset models
//! 
//! This module defines the data structures for certificates and assets.

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::errors::LastrumError;

/// Types of assets that can be certified
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AssetType {
    Gold,
    Silver,
    Platinum,
    Palladium,
    Other
}

impl AssetType {
    /// Convert the asset type to a string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetType::Gold => "GOLD",
            AssetType::Silver => "SILVER",
            AssetType::Platinum => "PLATINUM",
            AssetType::Palladium => "PALLADIUM",
            AssetType::Other => "OTHER",
        }
    }
    
    /// Convert from a string to an AssetType
    pub fn from_str(s: &str) -> Result<Self, LastrumError> {
        match s.to_uppercase().as_str() {
            "GOLD" => Ok(AssetType::Gold),
            "SILVER" => Ok(AssetType::Silver),
            "PLATINUM" => Ok(AssetType::Platinum),
            "PALLADIUM" => Ok(AssetType::Palladium),
            "OTHER" => Ok(AssetType::Other),
            _ => Err(LastrumError::InvalidAssetType(s.to_string())),
        }
    }
}

/// Represents a physical asset being certified
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Asset {
    /// Type of asset (gold, silver, etc.)
    pub asset_type: AssetType,
    /// Weight of the asset in grams
    pub weight: f64,
    /// Purity of the asset (0.0-1.0)
    pub purity: f64,
    /// Serial number or other unique identifier for the asset
    pub serial: String,
}

impl Asset {
    /// Create a new asset
    pub fn new(asset_type: AssetType, weight: f64, purity: f64, serial: String) -> Self {
        Self {
            asset_type,
            weight,
            purity,
            serial,
        }
    }
    
    /// Get the asset type as a string
    pub fn asset_type_str(&self) -> &'static str {
        self.asset_type.as_str()
    }
    
    /// Validate that the asset data is correct
    pub fn validate(&self) -> Result<(), LastrumError> {
        if self.weight <= 0.0 {
            return Err(LastrumError::InvalidAsset("Weight must be greater than zero".into()));
        }
        
        if self.purity <= 0.0 || self.purity > 1.0 {
            return Err(LastrumError::InvalidAsset("Purity must be between 0 and 1".into()));
        }
        
        if self.serial.is_empty() {
            return Err(LastrumError::InvalidAsset("Serial number cannot be empty".into()));
        }
        
        Ok(())
    }
}

/// A certificate for a physical asset
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Certificate {
    /// Unique identifier for this certificate
    pub id: String,
    /// Name of the issuing custody house
    pub issuer: String,
    /// Public key of the issuer (hex encoded)
    pub issuer_public_key: String,
    /// The asset being certified
    pub asset: Asset,
    /// When the certificate was issued
    pub issued_at: DateTime<Utc>,
    /// Digital signature of the certificate (hex encoded)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl Certificate {
    /// Create a new certificate
    pub fn new(
        issuer: String,
        issuer_public_key: String,
        asset: Asset,
    ) -> Result<Self, LastrumError> {
        // Validate the asset data
        asset.validate()?;
        
        Ok(Self {
            id: Uuid::new_v4().to_string(),
            issuer,
            issuer_public_key,
            asset,
            issued_at: Utc::now(),
            signature: None,
        })
    }
    
    /// Get the certificate data as bytes for signing
    /// This excludes the signature field
    pub fn to_signable_bytes(&self) -> Result<Vec<u8>, LastrumError> {
        // Create a temporary copy without the signature
        let temp = Self {
            id: self.id.clone(),
            issuer: self.issuer.clone(),
            issuer_public_key: self.issuer_public_key.clone(),
            asset: self.asset.clone(),
            issued_at: self.issued_at,
            signature: None,
        };
        
        // Serialize to JSON
        serde_json::to_vec(&temp)
            .map_err(|e| LastrumError::SerializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_asset_type_conversion() {
        assert_eq!(AssetType::Gold.as_str(), "GOLD");
        assert_eq!(AssetType::from_str("GOLD").unwrap(), AssetType::Gold);
        assert_eq!(AssetType::from_str("gold").unwrap(), AssetType::Gold);
    }
    
    #[test]
    fn test_asset_validation() {
        // Valid asset
        let valid = Asset::new(AssetType::Gold, 100.0, 0.999, "SERIAL123".to_string());
        assert!(valid.validate().is_ok());
        
        // Invalid weight
        let invalid_weight = Asset::new(AssetType::Gold, 0.0, 0.999, "SERIAL123".to_string());
        assert!(invalid_weight.validate().is_err());
        
        // Invalid purity
        let invalid_purity = Asset::new(AssetType::Gold, 100.0, 1.5, "SERIAL123".to_string());
        assert!(invalid_purity.validate().is_err());
        
        // Invalid serial
        let invalid_serial = Asset::new(AssetType::Gold, 100.0, 0.999, "".to_string());
        assert!(invalid_serial.validate().is_err());
    }
    
    #[test]
    fn test_certificate_creation() {
        let asset = Asset::new(AssetType::Gold, 100.0, 0.999, "SERIAL123".to_string());
        let cert = Certificate::new(
            "Test Custody".to_string(),
            "0123456789abcdef".to_string(),
            asset.clone(),
        ).unwrap();
        
        assert_eq!(cert.issuer, "Test Custody");
        assert_eq!(cert.issuer_public_key, "0123456789abcdef");
        assert_eq!(cert.asset, asset);
        assert!(cert.signature.is_none());
    }
    
    #[test]
    fn test_signable_bytes() {
        let asset = Asset::new(AssetType::Gold, 100.0, 0.999, "SERIAL123".to_string());
        let mut cert = Certificate::new(
            "Test Custody".to_string(),
            "0123456789abcdef".to_string(),
            asset,
        ).unwrap();
        
        // Add a signature
        cert.signature = Some("test-signature".to_string());
        
        // Get signable bytes
        let bytes = cert.to_signable_bytes().unwrap();
        
        // Deserialize back to test that signature is not included
        let deserialized: Certificate = serde_json::from_slice(&bytes).unwrap();
        assert!(deserialized.signature.is_none());
    }
}
