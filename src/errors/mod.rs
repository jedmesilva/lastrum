//! Error types for Lastrum Certifield
//! 
//! This module defines error types used throughout the application.

use thiserror::Error;

/// Error types for the Lastrum Certifield application
#[derive(Error, Debug)]
pub enum LastrumError {
    /// Errors related to I/O operations
    #[error("I/O error: {0}")]
    IoError(String),
    
    /// Errors related to configuration
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    /// Errors related to cryptographic keys
    #[error("Key error: {0}")]
    KeyError(String),
    
    /// Errors related to digital signatures
    #[error("Signature error: {0}")]
    SignatureError(String),
    
    /// Errors related to invalid hash values
    #[error("Invalid hash value")]
    InvalidHash,
    
    /// Errors related to serialization
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Errors related to deserialization
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    /// Errors related to certificate building
    #[error("Certificate builder error: {0}")]
    BuilderError(String),
    
    /// Errors related to invalid assets
    #[error("Invalid asset: {0}")]
    InvalidAsset(String),
    
    /// Errors related to invalid asset types
    #[error("Invalid asset type: {0}")]
    InvalidAssetType(String),
    
    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),
    
    /// Database errors
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    /// Network errors
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// Validation errors
    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Convert from rusqlite errors to LastrumError
impl From<rusqlite::Error> for LastrumError {
    fn from(error: rusqlite::Error) -> Self {
        LastrumError::DatabaseError(error.to_string())
    }
}

/// Convert from serde_json errors to LastrumError
impl From<serde_json::Error> for LastrumError {
    fn from(error: serde_json::Error) -> Self {
        LastrumError::SerializationError(error.to_string())
    }
}

/// Convert from std::io::Error to LastrumError
impl From<std::io::Error> for LastrumError {
    fn from(error: std::io::Error) -> Self {
        LastrumError::IoError(error.to_string())
    }
}
