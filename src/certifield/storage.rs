//! Certificate storage
//! 
//! This module handles storage and retrieval of certificates
//! from local storage.

use std::path::PathBuf;
use rusqlite::{params, Connection};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::certifield::model::{Asset, AssetType, Certificate, RequestType};
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
    
    /// Create a certificate storage manager with an in-memory database for testing
    #[cfg(test)]
    pub fn new_in_memory() -> Result<Self, LastrumError> {
        let conn = Connection::open_in_memory()
            .map_err(|e| LastrumError::DatabaseError(format!("Failed to open in-memory database: {}", e)))?;
        
        // Create tables
        conn.execute(
            "CREATE TABLE certificates (
                id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        ).map_err(|e| LastrumError::DatabaseError(format!("Failed to create table: {}", e)))?;
        
        Ok(Self {
            db_path: PathBuf::from(":memory:"),
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
        
        // Tenta deserializar diretamente
        match serde_json::from_str::<Certificate>(&json) {
            Ok(certificate) => Ok(certificate),
            Err(e) => {
                log::warn!("Erro ao deserializar certificado {}: {}. Tentando migrar...", id, e);
                
                // Se falhar, tenta migrar do formato legado
                match self.migrate_legacy_certificate(id, &json) {
                    Ok(migrated) => {
                        // Se a migração for bem-sucedida, salva o certificado atualizado
                        log::info!("Atualizando certificado {} para o novo formato no armazenamento", id);
                        let _ = self.conn.execute(
                            "UPDATE certificates SET data = ? WHERE id = ?",
                            params![
                                serde_json::to_string(&migrated).map_err(|e| 
                                    LastrumError::SerializationError(format!("Falha ao serializar certificado migrado: {}", e))
                                )?,
                                id
                            ]
                        ).map_err(|e| 
                            LastrumError::DatabaseError(format!("Falha ao atualizar certificado migrado: {}", e))
                        )?;
                        
                        Ok(migrated)
                    },
                    Err(e) => {
                        log::error!("Falha na migração do certificado {}: {}", id, e);
                        Err(e)
                    }
                }
            }
        }
    }
    
    /// List all certificates
    pub fn list_all(&self) -> Result<Vec<Certificate>, LastrumError> {
        let mut stmt = self.conn.prepare("SELECT id, data FROM certificates ORDER BY created_at DESC")
            .map_err(|e| LastrumError::DatabaseError(format!("Failed to prepare query: {}", e)))?;
        
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let json: String = row.get(1)?;
            
            // Tentativa de desserialização
            let cert_result = serde_json::from_str::<Certificate>(&json);
            
            // Lidando com erro de desserialização (certificados antigos)
            match cert_result {
                Ok(certificate) => Ok(certificate),
                Err(e) => {
                    log::warn!("Erro ao deserializar certificado {}: {}. Tentando migrar...", id, e);
                    
                    // Tenta extrair dados-chave do JSON (como uma migração)
                    self.migrate_legacy_certificate(&id, &json)
                        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))
                }
            }
        }).map_err(|e| LastrumError::DatabaseError(format!("Failed to query certificates: {}", e)))?;
        
        let mut certificates = Vec::new();
        for result in rows {
            match result {
                Ok(cert) => certificates.push(cert),
                Err(e) => {
                    log::error!("Erro processando certificado: {}", e);
                    // Continua o processamento mesmo com erro em um certificado
                }
            }
        }
        
        Ok(certificates)
    }
    
    /// Tenta migrar um certificado legado para o novo formato
    fn migrate_legacy_certificate(&self, id: &str, json: &str) -> Result<Certificate, LastrumError> {
        use serde_json::Value;
        
        // Loga o JSON para depuração
        log::info!("Migrando certificado legado: {}", json);
        
        // Analisa o JSON como Value genérico
        let value: Value = serde_json::from_str(json)
            .map_err(|e| LastrumError::DeserializationError(format!("Falha ao analisar JSON legado: {}", e)))?;
        
        // Extrai campos comuns
        let obj = value.as_object()
            .ok_or_else(|| LastrumError::DeserializationError("JSON não é um objeto".to_string()))?;
        
        // Lê campos necessários (pode ajustar conforme seu modelo exato)
        let issuer = obj.get("issuer")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LastrumError::DeserializationError("Campo 'issuer' ausente ou inválido".to_string()))?;
        
        let signature = obj.get("signature")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LastrumError::DeserializationError("Campo 'signature' ausente ou inválido".to_string()))?;
        
        let issued_at = obj.get("issued_at")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LastrumError::DeserializationError("Campo 'issued_at' ausente ou inválido".to_string()))?;
        
        let issuer_public_key = obj.get("issuer_public_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LastrumError::DeserializationError("Campo 'issuer_public_key' ausente ou inválido".to_string()))?;
        
        // Extrai informações do asset (se existir)
        let asset_obj = obj.get("asset")
            .and_then(|v| v.as_object())
            .ok_or_else(|| LastrumError::DeserializationError("Campo 'asset' ausente ou inválido".to_string()))?;
        
        // Loga o asset_obj para depuração
        log::info!("Asset object: {:?}", asset_obj);
        
        let asset_type = asset_obj.get("asset_type")
            .and_then(|v| {
                log::info!("Asset type value: {:?}", v);
                v.as_str()
            })
            .and_then(|s| match s {
                // Formato antigo (com primeira letra maiúscula, resto minúsculo)
                "Gold" => Some(AssetType::Gold),
                "Silver" => Some(AssetType::Silver),
                "Platinum" => Some(AssetType::Platinum),
                "Palladium" => Some(AssetType::Palladium),
                // Formato novo (tudo maiúsculo)
                "GOLD" => Some(AssetType::Gold),
                "SILVER" => Some(AssetType::Silver),
                "PLATINUM" => Some(AssetType::Platinum),
                "PALLADIUM" => Some(AssetType::Palladium),
                // Energia renovável
                "BIOGAS_METANO" => Some(AssetType::BiogasMetano),
                "ENERGIA_SOLAR" => Some(AssetType::EnergiaSolar),
                "ENERGIA_EOLICA" => Some(AssetType::EnergiaEolica),
                "ENERGIA_HIDRICA" => Some(AssetType::EnergiaHidrica),
                "HIDROGENIO_VERDE" => Some(AssetType::HidrogenioVerde),
                _ => {
                    log::warn!("Tipo de ativo desconhecido: {}", s);
                    None
                },
            })
            .ok_or_else(|| LastrumError::DeserializationError(format!("Campo 'asset_type' ausente ou inválido: {:?}", asset_obj.get("asset_type"))))?;
        
        let weight = asset_obj.get("weight")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| LastrumError::DeserializationError("Campo 'weight' ausente ou inválido".to_string()))?;
        
        let purity = asset_obj.get("purity")
            .and_then(|v| v.as_f64());
        
        let serial = asset_obj.get("serial")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LastrumError::DeserializationError("Campo 'serial' ausente ou inválido".to_string()))?;
        
        // Criar o novo certificado
        let unit = match asset_type {
            AssetType::Gold | AssetType::Silver | AssetType::Platinum | AssetType::Palladium => "g".to_string(),
            _ => "unit".to_string(),
        };
        
        // Criar asset com o novo formato
        let asset = if let Some(p) = purity {
            Asset::new_with_purity(asset_type, weight, unit, p, serial.to_string())
        } else {
            Asset::new(asset_type, weight, unit.to_string(), serial.to_string())
        };
        
        // Parse da data de emissão
        let parsed_issued_at = chrono::DateTime::parse_from_rfc3339(issued_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .or_else(|_| {
                // Tenta outro formato se não for RFC3339
                chrono::NaiveDateTime::parse_from_str(issued_at, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| chrono::DateTime::<chrono::Utc>::from_utc(ndt, chrono::Utc))
            })
            .map_err(|e| LastrumError::DeserializationError(format!("Erro ao parsear data de emissão: {}", e)))?;
        
        // Criar o certificado migrado
        let mut certificate = Certificate {
            request_type: RequestType::EmitCertificate,
            id: id.to_string(),
            custody_house_id: issuer.to_string(),
            custody_house_hash: issuer_public_key.to_string(),
            authorized_wallets: None,
            asset,
            issued_at: parsed_issued_at,
            expires_at: None,
            description: format!("Certificado migrado de formato legado"),
            signature: Some(signature.to_string()),
            certificate_private_key: None,
            token_ledgers: Vec::new(),
        };
        
        // Loga a migração
        log::info!("Certificado {} migrado com sucesso do formato legado", id);
        
        Ok(certificate)
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
    // tempdir não é mais necessário com new_in_memory
    
    /// Helper to create a test certificate
    fn create_test_certificate() -> Certificate {
        let asset = Asset::new_with_purity(AssetType::Gold, 100.0, "g".to_string(), 0.999, "SERIAL123".to_string());
        
        let now = Utc::now();
        Certificate {
            request_type: RequestType::EmitCertificate,
            id: uuid::Uuid::new_v4().to_string(),
            custody_house_id: "Test Custody".to_string(),
            custody_house_hash: "0123456789abcdef".to_string(),
            authorized_wallets: None,
            asset,
            issued_at: now,
            expires_at: None,
            description: "Test certificate".to_string(),
            signature: Some("test_signature".to_string()),
            certificate_private_key: None,
            token_ledgers: Vec::new(),
        }
    }
    
    #[test]
    fn test_store_and_load() {
        // Create an in-memory database for testing
        let storage = CertificateStorage::new_in_memory().unwrap();
        
        let certificate = create_test_certificate();
        let id = storage.store(&certificate).unwrap();
        
        let loaded = storage.load_by_id(&id).unwrap();
        assert_eq!(loaded.id, certificate.id);
        assert_eq!(loaded.custody_house_id, certificate.custody_house_id);
        assert_eq!(loaded.asset.asset_type, certificate.asset.asset_type);
    }
    
    #[test]
    fn test_list_all() {
        // Create an in-memory database for testing
        let storage = CertificateStorage::new_in_memory().unwrap();
        
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
        let storage = CertificateStorage::new_in_memory().unwrap();
        
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
