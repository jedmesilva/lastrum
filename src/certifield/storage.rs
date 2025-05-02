//! Certificate storage
//! 
//! This module handles storage and retrieval of certificates
//! from local storage.

use std::fs;
use std::path::PathBuf;
use rusqlite::{params, Connection, Result as SqliteResult};

use crate::certifield::model::Certificate;
use crate::config::Config;
use crate::errors::LastrumError;

/// A storage manager for certificates
pub struct CertificateStorage {
    db_path: PathBuf,
    conn: Connection,
}

impl CertificateStorage {
    /// Create a new certificate storage manager
    pub fn new() -> Result<Self, LastrumError> {
        let config = Config::new()?;
        let db_path = config.data_dir().join("certificates.db");
        
        let conn = Connection::open(&db_path)
            .map_err(|e| LastrumError::DatabaseError(format!("Failed to open database: {}", e)))?;
        
        // Create tables if they don't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS certificates (
                id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        ).map_err(|e| LastrumError::DatabaseError(format!("Failed to create table: {}", e)))?;
        
        Ok(Self {
            db_path,
            conn,
        })
    }
    
    /// Store a certificate
    pub fn store(&self, certificate: &Certificate) -> Result<String, LastrumError> {
        let json = serde_json::to_string(certificate)
            .map_err(|e| LastrumError::SerializationError(e.to_string()))?;
        
        let now = chrono::Utc::now().to_rfc3339();
        
        self.conn.execute(
            "INSERT OR REPLACE INTO certificates (id, data, created_at) VALUES (?, ?, ?)",
            params![&certificate.id, &json, &now],
        ).map_err(|e| LastrumError::DatabaseError(format!("Failed to store certificate: {}", e)))?;
        
        Ok(certificate.id.clone())
    }
    
    /// Load a certificate by ID
    pub fn load_by_id(&self, id: &str) -> Result<Certificate, LastrumError> {
        let mut stmt = self.conn.prepare("SELECT data FROM certificates WHERE id = ?")
            .map_err(|e| LastrumError::DatabaseError(format!("Failed to prepare query: {}", e)))?;
        
        let json: String = stmt.query_row(params![id], |row| row.get(0))
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => LastrumError::NotFound(format!("Certificate not found: {}", id)),
                _ => LastrumError::DatabaseError(format!("Database error: {}", e)),
            })?;
        
        let certificate: Certificate = serde_json::from_str(&json)
            .map_err(|e| LastrumError::DeserializationError(e.to_string()))?;
        
        Ok(certificate)
    }
    
    /// List all certificates
    pub fn list_all(&self) -> Result<Vec<Certificate>, LastrumError> {
        let mut stmt = self.conn.prepare("SELECT data FROM certificates ORDER BY created_at DESC")
            .map_err(|e| LastrumError::DatabaseError(format!("Failed to prepare query: {}", e)))?;
        
        let rows = stmt.query_map([], |row| {
            let json: String = row.get(0)?;
            let certificate: Certificate = serde_json::from_str(&json)
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
            Ok(certificate)
        }).map_err(|e| LastrumError::DatabaseError(format!("Failed to query certificates: {}", e)))?;
        
        let mut certificates = Vec::new();
        for result in rows {
            match result {
                Ok(cert) => certificates.push(cert),
                Err(e) => return Err(LastrumError::DatabaseError(format!("Error parsing certificate: {}", e))),
            }
        }
        
        Ok(certificates)
    }
    
    /// Delete a certificate by ID
    pub fn delete(&self, id: &str) -> Result<(), LastrumError> {
        self.conn.execute("DELETE FROM certificates WHERE id = ?", params![id])
            .map_err(|e| LastrumError::DatabaseError(format!("Failed to delete certificate: {}", e)))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certifield::model::{Asset, AssetType};
    use crate::certifield::builder::CertificateBuilder;
    use tempfile::tempdir;
    
    /// Helper to create a test certificate
    fn create_test_certificate() -> Certificate {
        let asset = Asset::new(AssetType::Gold, 100.0, 0.999, "SERIAL123".to_string());
        
        CertificateBuilder::new()
            .with_issuer("Test Custody".to_string())
            .with_issuer_public_key("0123456789abcdef".to_string())
            .with_asset(asset)
            .build()
            .unwrap()
    }
    
    #[test]
    fn test_store_and_load() {
        // Create an in-memory database for testing
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE certificates (
                id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        ).unwrap();
        
        let storage = CertificateStorage {
            db_path: PathBuf::from(":memory:"),
            conn,
        };
        
        let certificate = create_test_certificate();
        let id = storage.store(&certificate).unwrap();
        
        let loaded = storage.load_by_id(&id).unwrap();
        assert_eq!(loaded.id, certificate.id);
        assert_eq!(loaded.issuer, certificate.issuer);
        assert_eq!(loaded.asset.asset_type, certificate.asset.asset_type);
    }
    
    #[test]
    fn test_list_all() {
        // Create an in-memory database for testing
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE certificates (
                id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        ).unwrap();
        
        let storage = CertificateStorage {
            db_path: PathBuf::from(":memory:"),
            conn,
        };
        
        // Store multiple certificates
        let cert1 = create_test_certificate();
        let cert2 = create_test_certificate();
        
        storage.store(&cert1).unwrap();
        storage.store(&cert2).unwrap();
        
        let all_certs = storage.list_all().unwrap();
        assert_eq!(all_certs.len(), 2);
    }
    
    #[test]
    fn test_delete() {
        // Create an in-memory database for testing
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE certificates (
                id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        ).unwrap();
        
        let storage = CertificateStorage {
            db_path: PathBuf::from(":memory:"),
            conn,
        };
        
        let certificate = create_test_certificate();
        let id = storage.store(&certificate).unwrap();
        
        // Verify it exists
        assert!(storage.load_by_id(&id).is_ok());
        
        // Delete it
        storage.delete(&id).unwrap();
        
        // Verify it's gone
        assert!(storage.load_by_id(&id).is_err());
    }
}
