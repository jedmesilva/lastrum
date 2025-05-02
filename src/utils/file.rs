//! File utility functions
//! 
//! This module provides utility functions for working with
//! files and directories.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Check if a file exists
pub fn file_exists(path: &Path) -> bool {
    path.exists() && path.is_file()
}

/// Create a directory and any parent directories
pub fn create_dir_if_not_exists(path: &Path) -> io::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Get all files in a directory with a specific extension
pub fn list_files_with_extension(dir: &Path, ext: &str) -> io::Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    
    if !dir.exists() || !dir.is_dir() {
        return Ok(result);
    }
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |e| e == ext) {
            result.push(path);
        }
    }
    
    Ok(result)
}

/// Read a JSON file and deserialize it
pub fn read_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let data = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse JSON: {}", e))
}

/// Write a serializable object to a JSON file
pub fn write_json_file<T: serde::Serialize>(path: &Path, data: &T) -> Result<(), String> {
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialize to JSON: {}", e))?;
    
    fs::write(path, json)
        .map_err(|e| format!("Failed to write file: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, Deserialize};
    use tempfile::tempdir;
    
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestStruct {
        name: String,
        value: i32,
    }
    
    #[test]
    fn test_file_exists() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        
        assert!(!file_exists(&file_path));
        
        fs::write(&file_path, "test").unwrap();
        assert!(file_exists(&file_path));
    }
    
    #[test]
    fn test_create_dir_if_not_exists() {
        let dir = tempdir().unwrap();
        let new_dir = dir.path().join("new_dir");
        
        assert!(!new_dir.exists());
        create_dir_if_not_exists(&new_dir).unwrap();
        assert!(new_dir.exists());
        assert!(new_dir.is_dir());
    }
    
    #[test]
    fn test_list_files_with_extension() {
        let dir = tempdir().unwrap();
        
        // Create some test files
        fs::write(dir.path().join("test1.txt"), "test").unwrap();
        fs::write(dir.path().join("test2.txt"), "test").unwrap();
        fs::write(dir.path().join("test.json"), "test").unwrap();
        
        let txt_files = list_files_with_extension(dir.path(), "txt").unwrap();
        assert_eq!(txt_files.len(), 2);
        
        let json_files = list_files_with_extension(dir.path(), "json").unwrap();
        assert_eq!(json_files.len(), 1);
    }
    
    #[test]
    fn test_json_read_write() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.json");
        
        let test_data = TestStruct {
            name: "Test".to_string(),
            value: 42,
        };
        
        write_json_file(&file_path, &test_data).unwrap();
        let loaded: TestStruct = read_json_file(&file_path).unwrap();
        
        assert_eq!(test_data, loaded);
    }
}
