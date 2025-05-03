//! Módulo de broadcasting para Lastrum Certifield
//! 
//! Este módulo gerencia a transmissão de certificados para a rede,
//! garantindo que os certificados emitidos sejam propagados para todos os nós.

use std::sync::{Arc, Mutex};
use crate::certifield::model::Certificate;
use crate::certifield::storage::CertificateStorage;
use crate::errors::LastrumError;
use crate::network::protocol::{Message, MessageType};

/// Gerenciador de broadcasting para certificados
pub struct Broadcaster {
    /// ID do nó local
    node_id: String,
    /// Armazenamento local de certificados
    storage: Option<CertificateStorage>,
}

impl Broadcaster {
    /// Cria um novo gerenciador de broadcasting
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            storage: None,
        }
    }
    
    /// Cria um novo gerenciador de broadcasting com armazenamento
    pub fn with_storage(node_id: String) -> Result<Self, LastrumError> {
        let storage = CertificateStorage::new()?;
        
        Ok(Self {
            node_id,
            storage: Some(storage),
        })
    }
    
    /// Broadcast um certificado para a rede
    pub fn broadcast_certificate(&self, certificate: &Certificate) -> Result<(), LastrumError> {
        // Cria uma mensagem com o certificado completo
        let certificate_message = Message::certificate(self.node_id.clone(), certificate)
            .map_err(|e| LastrumError::SerializationError(e.to_string()))?;
        
        // Em uma implementação real, isso seria enviado para todos os peers conectados
        log::info!("Broadcasting certificado: {}", certificate.id);
        log::debug!("Emissor: {}, Asset: {} - {}g, pureza: {}", 
            certificate.issuer, 
            certificate.asset.asset_type_str(),
            certificate.asset.weight,
            certificate.asset.purity);
        
        // Simula o envio para peers
        log::debug!("Mensagem criada: {:?}", certificate_message.msg_type);
        
        Ok(())
    }
    
    /// Cria uma mensagem de anúncio para um certificado (apenas ID)
    pub fn create_announcement(&self, certificate_id: &str) -> Message {
        // Cria uma mensagem que anuncia apenas o ID do certificado
        // (peers interessados podem solicitar o certificado completo)
        Message::new(
            MessageType::NewCertificate,
            self.node_id.clone(),
            None, // Broadcast para todos
            Some(certificate_id.to_string()),
        )
    }
    
    /// Propaga todos os certificados locais para a rede
    pub fn broadcast_all_certificates(&self) -> Result<(), LastrumError> {
        // Verifica se o storage está disponível
        if let Some(storage) = &self.storage {
            // Carrega todos os certificados do armazenamento local
            let certificates = storage.list_all()?;
            
            if certificates.is_empty() {
                log::info!("Nenhum certificado para propagar");
                return Ok(());
            }
            
            log::info!("Propagando {} certificados para a rede", certificates.len());
            
            // Propaga cada certificado
            for certificate in certificates {
                self.broadcast_certificate(&certificate)?;
            }
            
            Ok(())
        } else {
            Err(LastrumError::ConfigError("Storage não configurado para o broadcaster".to_string()))
        }
    }
}
