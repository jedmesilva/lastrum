//! Configuration module for Lastrum Certifield
//! 
//! This module handles configuration loading, environment variables,
//! and global settings for the application.

use crate::errors::LastrumError;
use directories::ProjectDirs;
use std::path::PathBuf;

pub struct Config {
    data_dir: PathBuf,
    identity_dir: PathBuf,
    certificates_dir: PathBuf,
}

impl Config {
    /// Initialize a new configuration
    pub fn new() -> Result<Self, LastrumError> {
        let proj_dirs = ProjectDirs::from("com", "lastrum", "certifield")
            .ok_or_else(|| LastrumError::ConfigError("Failed to determine project directories".into()))?;
        
        let data_dir = proj_dirs.data_dir().to_path_buf();
        let identity_dir = data_dir.join("identities");
        let certificates_dir = data_dir.join("certificates");
        
        // Create directories if they don't exist
        for dir in &[&data_dir, &identity_dir, &certificates_dir] {
            if !dir.exists() {
                std::fs::create_dir_all(dir)
                    .map_err(|e| LastrumError::ConfigError(format!("Failed to create directory: {}", e)))?;
            }
        }
        
        Ok(Self {
            data_dir,
            identity_dir,
            certificates_dir,
        })
    }
    
    /// Get the base data directory
    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }
    
    /// Get the identity files directory
    pub fn identity_dir(&self) -> &PathBuf {
        &self.identity_dir
    }
    
    /// Get the certificates directory
    pub fn certificates_dir(&self) -> &PathBuf {
        &self.certificates_dir
    }
}
