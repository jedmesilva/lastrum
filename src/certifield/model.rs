//! Certificate and asset models
//! 
//! This module defines the data structures for certificates and assets.

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::collections::HashMap;

use crate::errors::LastrumError;

/// Types of assets that can be certified
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AssetType {
    // Metais preciosos
    Gold,
    Silver,
    Platinum,
    Palladium,
    
    // Energia renovável
    BiogasMetano,
    EnergiaSolar,
    EnergiaEolica,
    EnergiaHidrica,
    HidrogenioVerde,
    
    // Outros tipos
    Other
}

impl AssetType {
    /// Convert the asset type to a string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            // Metais preciosos
            AssetType::Gold => "GOLD",
            AssetType::Silver => "SILVER",
            AssetType::Platinum => "PLATINUM",
            AssetType::Palladium => "PALLADIUM",
            
            // Energia renovável
            AssetType::BiogasMetano => "BIOGAS_METANO",
            AssetType::EnergiaSolar => "ENERGIA_SOLAR",
            AssetType::EnergiaEolica => "ENERGIA_EOLICA",
            AssetType::EnergiaHidrica => "ENERGIA_HIDRICA",
            AssetType::HidrogenioVerde => "HIDROGENIO_VERDE",
            
            // Outros tipos
            AssetType::Other => "OTHER",
        }
    }
    
    /// Convert from a string to an AssetType
    pub fn from_str(s: &str) -> Result<Self, LastrumError> {
        match s.to_uppercase().as_str() {
            // Metais preciosos
            "GOLD" => Ok(AssetType::Gold),
            "SILVER" => Ok(AssetType::Silver),
            "PLATINUM" => Ok(AssetType::Platinum),
            "PALLADIUM" => Ok(AssetType::Palladium),
            
            // Energia renovável
            "BIOGAS_METANO" => Ok(AssetType::BiogasMetano),
            "ENERGIA_SOLAR" => Ok(AssetType::EnergiaSolar),
            "ENERGIA_EOLICA" => Ok(AssetType::EnergiaEolica),
            "ENERGIA_HIDRICA" => Ok(AssetType::EnergiaHidrica),
            "HIDROGENIO_VERDE" => Ok(AssetType::HidrogenioVerde),
            
            // Outros tipos
            "OTHER" => Ok(AssetType::Other),
            _ => Err(LastrumError::InvalidAssetType(s.to_string())),
        }
    }
}

/// Represents a physical asset being certified
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Asset {
    /// Type of asset (gold, biogas, renewable energy, etc.)
    pub asset_type: AssetType,
    
    /// Quantity of the asset (weight for metals, volume for gas, etc.)
    pub quantity: f64,
    
    /// Unit of measurement for the asset (g, m3, kWh, etc.)
    pub unit: String,
    
    /// Quality or purity of the asset (0.0-1.0 for metals, other metrics for other assets)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purity: Option<f64>,
    
    /// Serial number or other unique identifier for the asset
    pub serial: String,
}

impl Asset {
    /// Create a new asset
    pub fn new(asset_type: AssetType, quantity: f64, unit: String, serial: String) -> Self {
        Self {
            asset_type,
            quantity,
            unit,
            purity: None,
            serial,
        }
    }
    
    /// Create a new asset with purity information (for metals)
    pub fn new_with_purity(asset_type: AssetType, quantity: f64, unit: String, purity: f64, serial: String) -> Self {
        Self {
            asset_type,
            quantity,
            unit,
            purity: Some(purity),
            serial,
        }
    }
    
    /// Get the asset type as a string
    pub fn asset_type_str(&self) -> &'static str {
        self.asset_type.as_str()
    }
    
    /// Validate that the asset data is correct
    pub fn validate(&self) -> Result<(), LastrumError> {
        if self.quantity <= 0.0 {
            return Err(LastrumError::InvalidAsset("Quantity must be greater than zero".into()));
        }
        
        if let Some(purity) = self.purity {
            if purity <= 0.0 || purity > 1.0 {
                return Err(LastrumError::InvalidAsset("Purity must be between 0 and 1".into()));
            }
        }
        
        if self.unit.is_empty() {
            return Err(LastrumError::InvalidAsset("Unit of measurement cannot be empty".into()));
        }
        
        if self.serial.is_empty() {
            return Err(LastrumError::InvalidAsset("Serial number cannot be empty".into()));
        }
        
        Ok(())
    }
}

/// Request type for certificate operations
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RequestType {
    EmitCertificate,
    RevokeCertificate,
    TransferCertificate,
}

impl RequestType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RequestType::EmitCertificate => "emit_certificate",
            RequestType::RevokeCertificate => "revoke_certificate",
            RequestType::TransferCertificate => "transfer_certificate",
        }
    }
    
    pub fn from_str(s: &str) -> Result<Self, LastrumError> {
        match s.to_lowercase().as_str() {
            "emit_certificate" => Ok(RequestType::EmitCertificate),
            "revoke_certificate" => Ok(RequestType::RevokeCertificate),
            "transfer_certificate" => Ok(RequestType::TransferCertificate),
            _ => Err(LastrumError::InvalidRequestType(s.to_string())),
        }
    }
}

/// Representa o registro de um token emitido baseado no certificado
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenLedger {
    /// Hash do token
    pub token_hash: String,
    /// Carteira que emitiu o token
    pub issuer_wallet: String,
    /// Data de emissão
    pub issued_at: DateTime<Utc>,
    /// Data de uso/consumo (se aplicável)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumed_at: Option<DateTime<Utc>>,
    /// Carteira que usou/consumiu o token (se aplicável)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumer_wallet: Option<String>,
    /// Informação adicional sobre o uso/consumo do token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumption_data: Option<String>,
}

/// A certificate for a physical asset
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Certificate {
    /// Tipo da requisição (emit_certificate, revoke_certificate, etc)
    pub request_type: RequestType,
    
    /// Unique identifier for this certificate
    pub id: String,
    
    /// Custody House Information
    /// -------------------------
    /// Nome amigável da casa de custódia
    pub custody_house_id: String,
    
    /// Hash da chave pública da casa de custódia
    pub custody_house_hash: String,
    
    /// Lista de carteiras autorizadas a emitir tokens usando este certificado
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_wallets: Option<Vec<String>>,
    
    /// Asset Information
    /// ----------------
    /// The asset being certified
    pub asset: Asset,
    
    /// Descrição textual sobre o certificado
    pub description: String,
    
    /// Timestamps
    /// ----------
    /// When the certificate was issued
    pub issued_at: DateTime<Utc>,
    
    /// Data de expiração do certificado (opcional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    
    /// Security
    /// --------
    /// Chave privada do certificado (usada para autorizar a emissão de tokens)
    /// Este campo deve ser protegido e apenas compartilhado com partes autorizadas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_private_key: Option<String>,
    
    /// Digital signature of the certificate (hex encoded)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    
    /// Token Tracking
    /// -------------
    /// Ledger de todos os tokens emitidos com base neste certificado
    #[serde(default)]
    pub token_ledgers: Vec<TokenLedger>,
}

impl Certificate {
    /// Create a new certificate
    pub fn new(
        custody_house_id: String,
        custody_house_hash: String,
        asset: Asset,
        description: String,
    ) -> Result<Self, LastrumError> {
        // Validate the asset data
        asset.validate()?;
        
        Ok(Self {
            request_type: RequestType::EmitCertificate,
            id: Uuid::new_v4().to_string(),
            custody_house_id,
            custody_house_hash,
            authorized_wallets: None,
            asset,
            description,
            issued_at: Utc::now(),
            expires_at: None,
            certificate_private_key: None,
            signature: None,
            token_ledgers: Vec::new(),
        })
    }
    
    /// Adiciona uma carteira autorizada à lista de carteiras que podem emitir tokens
    pub fn add_authorized_wallet(&mut self, wallet_hash: String) {
        if let Some(wallets) = &mut self.authorized_wallets {
            wallets.push(wallet_hash);
        } else {
            self.authorized_wallets = Some(vec![wallet_hash]);
        }
    }
    
    /// Define uma data de expiração para o certificado
    pub fn set_expiration_date(&mut self, expires_at: DateTime<Utc>) {
        self.expires_at = Some(expires_at);
    }
    
    /// Define a chave privada do certificado
    pub fn set_certificate_private_key(&mut self, private_key: String) {
        self.certificate_private_key = Some(private_key);
    }
    
    /// Registra um novo token emitido com base neste certificado
    pub fn register_token(&mut self, token_hash: String, issuer_wallet: String) {
        let ledger = TokenLedger {
            token_hash,
            issuer_wallet,
            issued_at: Utc::now(),
            consumed_at: None,
            consumer_wallet: None,
            consumption_data: None,
        };
        
        self.token_ledgers.push(ledger);
    }
    
    /// Registra o consumo de um token
    pub fn register_token_consumption(&mut self, 
                                     token_hash: &str, 
                                     consumer_wallet: String, 
                                     consumption_data: Option<String>) -> Result<(), LastrumError> {
        // Encontra o token no ledger
        for ledger in &mut self.token_ledgers {
            if ledger.token_hash == token_hash {
                // Registra o consumo
                ledger.consumed_at = Some(Utc::now());
                ledger.consumer_wallet = Some(consumer_wallet);
                ledger.consumption_data = consumption_data;
                return Ok(());
            }
        }
        
        // Token não encontrado
        Err(LastrumError::TokenNotFound(token_hash.to_string()))
    }
    
    /// Verifica se o certificado está expirado
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }
    
    /// Get the certificate data as bytes for signing
    /// This excludes the signature field
    pub fn to_signable_bytes(&self) -> Result<Vec<u8>, LastrumError> {
        // Create a temporary copy without the signature
        let mut temp = self.clone();
        temp.signature = None;
        
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
        
        // Test new asset types
        assert_eq!(AssetType::BiogasMetano.as_str(), "BIOGAS_METANO");
        assert_eq!(AssetType::from_str("BIOGAS_METANO").unwrap(), AssetType::BiogasMetano);
    }
    
    #[test]
    fn test_request_type_conversion() {
        assert_eq!(RequestType::EmitCertificate.as_str(), "emit_certificate");
        assert_eq!(RequestType::from_str("emit_certificate").unwrap(), RequestType::EmitCertificate);
        assert_eq!(RequestType::from_str("EMIT_CERTIFICATE").unwrap(), RequestType::EmitCertificate);
    }
    
    #[test]
    fn test_asset_validation() {
        // Valid asset for gold with purity
        let valid_gold = Asset::new_with_purity(
            AssetType::Gold, 
            100.0, 
            "g".to_string(), 
            0.999, 
            "SERIAL123".to_string()
        );
        assert!(valid_gold.validate().is_ok());
        
        // Valid asset for biogas without purity
        let valid_biogas = Asset::new(
            AssetType::BiogasMetano,
            275.5,
            "m3".to_string(),
            "LOT-2025-05-001".to_string()
        );
        assert!(valid_biogas.validate().is_ok());
        
        // Invalid quantity
        let invalid_quantity = Asset::new(
            AssetType::EnergiaSolar, 
            0.0, 
            "kWh".to_string(), 
            "SOLAR-001".to_string()
        );
        assert!(invalid_quantity.validate().is_err());
        
        // Invalid unit
        let invalid_unit = Asset::new(
            AssetType::BiogasMetano, 
            100.0, 
            "".to_string(), 
            "BIO-001".to_string()
        );
        assert!(invalid_unit.validate().is_err());
        
        // Invalid purity
        let invalid_purity = Asset::new_with_purity(
            AssetType::Gold, 
            100.0, 
            "g".to_string(), 
            1.5, 
            "GOLD-001".to_string()
        );
        assert!(invalid_purity.validate().is_err());
        
        // Invalid serial
        let invalid_serial = Asset::new(
            AssetType::Gold, 
            100.0, 
            "g".to_string(), 
            "".to_string()
        );
        assert!(invalid_serial.validate().is_err());
    }
    
    #[test]
    fn test_certificate_creation() {
        let asset = Asset::new_with_purity(
            AssetType::Gold, 
            100.0, 
            "g".to_string(), 
            0.999, 
            "SERIAL123".to_string()
        );
        
        let description = "Certificação de 100g de ouro 999 sob custódia.".to_string();
        
        let cert = Certificate::new(
            "Casa de Custódia Exemplo".to_string(),
            "0123456789abcdef".to_string(),
            asset.clone(),
            description.clone(),
        ).unwrap();
        
        assert_eq!(cert.request_type, RequestType::EmitCertificate);
        assert_eq!(cert.custody_house_id, "Casa de Custódia Exemplo");
        assert_eq!(cert.custody_house_hash, "0123456789abcdef");
        assert_eq!(cert.asset, asset);
        assert_eq!(cert.description, description);
        assert!(cert.signature.is_none());
        assert!(cert.token_ledgers.is_empty());
    }
    
    #[test]
    fn test_certificate_authorized_wallets() {
        let asset = Asset::new(AssetType::BiogasMetano, 275.5, "m3".to_string(), "BIO-001".to_string());
        
        let mut cert = Certificate::new(
            "BioCustódia".to_string(),
            "0123456789abcdef".to_string(),
            asset,
            "Biogás metano sob custódia.".to_string(),
        ).unwrap();
        
        // Inicialmente não tem carteiras autorizadas
        assert!(cert.authorized_wallets.is_none());
        
        // Adiciona carteiras autorizadas
        cert.add_authorized_wallet("wallet1".to_string());
        cert.add_authorized_wallet("wallet2".to_string());
        
        // Verifica se as carteiras foram adicionadas
        assert_eq!(cert.authorized_wallets.as_ref().unwrap().len(), 2);
        assert_eq!(cert.authorized_wallets.as_ref().unwrap()[0], "wallet1");
        assert_eq!(cert.authorized_wallets.as_ref().unwrap()[1], "wallet2");
    }
    
    #[test]
    fn test_certificate_token_ledger() {
        let asset = Asset::new(AssetType::EnergiaEolica, 1000.0, "kWh".to_string(), "EOLICA-001".to_string());
        
        let mut cert = Certificate::new(
            "EnergiaCustódia".to_string(),
            "0123456789abcdef".to_string(),
            asset,
            "Energia eólica certificada.".to_string(),
        ).unwrap();
        
        // Inicialmente não tem tokens no ledger
        assert!(cert.token_ledgers.is_empty());
        
        // Registra um token
        cert.register_token("token1".to_string(), "wallet1".to_string());
        
        // Verifica se o token foi registrado
        assert_eq!(cert.token_ledgers.len(), 1);
        assert_eq!(cert.token_ledgers[0].token_hash, "token1");
        assert_eq!(cert.token_ledgers[0].issuer_wallet, "wallet1");
        assert!(cert.token_ledgers[0].consumed_at.is_none());
        
        // Registra o consumo do token
        cert.register_token_consumption(
            "token1", 
            "consumer_wallet".to_string(), 
            Some("Consumo para projeto verde".to_string())
        ).unwrap();
        
        // Verifica se o consumo foi registrado
        assert!(cert.token_ledgers[0].consumed_at.is_some());
        assert_eq!(cert.token_ledgers[0].consumer_wallet.unwrap(), "consumer_wallet");
    }
    
    #[test]
    fn test_certificate_expiration() {
        let asset = Asset::new(AssetType::Silver, 500.0, "g".to_string(), "SILVER-001".to_string());
        
        let mut cert = Certificate::new(
            "SilverCustódia".to_string(),
            "0123456789abcdef".to_string(),
            asset,
            "Prata sob custódia.".to_string(),
        ).unwrap();
        
        // Inicialmente não tem data de expiração
        assert!(cert.expires_at.is_none());
        assert!(!cert.is_expired());
        
        // Define uma data de expiração no passado
        let past_date = Utc::now() - chrono::Duration::days(1);
        cert.set_expiration_date(past_date);
        
        // Verifica se o certificado está expirado
        assert!(cert.is_expired());
        
        // Define uma data de expiração no futuro
        let future_date = Utc::now() + chrono::Duration::days(365);
        cert.set_expiration_date(future_date);
        
        // Verifica que o certificado não está expirado
        assert!(!cert.is_expired());
    }
    
    #[test]
    fn test_signable_bytes() {
        let asset = Asset::new(AssetType::Gold, 100.0, "g".to_string(), "SERIAL123".to_string());
        
        let mut cert = Certificate::new(
            "Test Custody".to_string(),
            "0123456789abcdef".to_string(),
            asset,
            "Gold certificate".to_string(),
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
