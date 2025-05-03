//! Identity management for custody houses
//! 
//! This module handles the creation and management of
//! identity for custody houses in the Lastrum network.

use serde::{Serialize, Deserialize};
use std::fs;
// PathBuf utilizado apenas em funções comentadas
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::config::Config;
use crate::core::hash_util;
use crate::core::keypair::KeyPair;
use crate::errors::LastrumError;

/// Represents the identity of a custody house in the Lastrum network
#[derive(Serialize, Deserialize, Clone)]
pub struct Identity {
    /// Unique identifier for this identity
    id: String,
    /// Name of the custody house
    name: String,
    /// Keypair (public and private keys)
    keypair: KeyPair,
    /// Hash identifier for this node in the network
    node_hash: String,
    /// Date and time when this identity was registered
    registered_at: DateTime<Utc>,
}

impl Identity {
    /// Create a new identity with the given name
    pub fn new(name: String) -> Result<Self, LastrumError> {
        let keypair = KeyPair::generate()?;
        let node_hash = hash_util::generate_node_hash(&name, &keypair.public_key_hex());
        let id = Uuid::new_v4().to_string();
        let registered_at = Utc::now();
        
        Ok(Self {
            id,
            name,
            keypair,
            node_hash,
            registered_at,
        })
    }
    
    /// Save the identity to a file
    pub fn save(&self) -> Result<String, LastrumError> {
        let config = Config::new()?;
        let file_name = format!("{}.identity.json", self.node_hash);
        let file_path = config.identity_dir().join(file_name);
        
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| LastrumError::SerializationError(e.to_string()))?;
        
        fs::write(&file_path, json)
            .map_err(|e| LastrumError::IoError(e.to_string()))?;
        
        Ok(file_path.to_string_lossy().to_string())
    }
    
    /// Load an identity from a file
    pub fn load(path: &str) -> Result<Self, LastrumError> {
        let json = fs::read_to_string(path)
            .map_err(|e| LastrumError::IoError(format!("Failed to read identity file: {}", e)))?;
        
        let identity: Identity = serde_json::from_str(&json)
            .map_err(|e| LastrumError::DeserializationError(e.to_string()))?;
        
        Ok(identity)
    }
    
    /// Get all available identities from the identity directory
    pub fn list_all() -> Result<Vec<Identity>, LastrumError> {
        let config = Config::new()?;
        let dir = config.identity_dir();
        
        let mut identities = Vec::new();
        
        for entry in fs::read_dir(dir).map_err(|e| LastrumError::IoError(e.to_string()))? {
            let entry = entry.map_err(|e| LastrumError::IoError(e.to_string()))?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(identity) = Identity::load(&path.to_string_lossy()) {
                    identities.push(identity);
                }
            }
        }
        
        Ok(identities)
    }
    
    // Getters
    
    /// Get the identity ID
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// Get the custody house name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get the node hash
    pub fn node_hash(&self) -> &str {
        &self.node_hash
    }
    
    /// Get the keypair
    pub fn keypair(&self) -> &KeyPair {
        &self.keypair
    }
    
    /// Get the registration date and time
    pub fn registered_at(&self) -> &DateTime<Utc> {
        &self.registered_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_identity_creation() {
        let identity = Identity::new("TestCustody".to_string()).unwrap();
        assert_eq!(identity.name(), "TestCustody");
        assert!(!identity.node_hash().is_empty());
        assert!(!identity.keypair().public_key_hex().is_empty());
        // Verifica se a data de registro foi definida (deve ser próxima ao tempo atual)
        let now = Utc::now();
        let diff = now.signed_duration_since(*identity.registered_at());
        assert!(diff.num_seconds() < 10, "A data de registro deve ser próxima ao tempo atual");
    }
    
    #[test]
    fn test_identity_save_load() {
        // Create a temporary directory for testing
        let dir = tempdir().unwrap();
        
        // Create a mock Config struct with our temp directory
        let identity = Identity::new("TestCustody".to_string()).unwrap();
        let file_path = dir.path().join(format!("{}.identity.json", identity.node_hash()));
        
        // Manually serialize and save
        let json = serde_json::to_string_pretty(&identity).unwrap();
        fs::write(&file_path, json).unwrap();
        
        // Load the identity
        let loaded = Identity::load(&file_path.to_string_lossy()).unwrap();
        
        // Verify the loaded identity matches the original
        assert_eq!(loaded.name(), identity.name());
        assert_eq!(loaded.node_hash(), identity.node_hash());
        assert_eq!(loaded.keypair().public_key_hex(), identity.keypair().public_key_hex());
        assert_eq!(loaded.registered_at(), identity.registered_at());
    }
}
