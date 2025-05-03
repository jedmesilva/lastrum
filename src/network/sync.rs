//! Sincronização do ledger para Lastrum Certifield
//! 
//! Este módulo implementa a sincronização do ledger entre os nós da rede,
//! permitindo que todos os nós tenham uma cópia completa dos certificados emitidos.

use std::collections::HashMap;
use crate::certifield::model::Certificate;
use crate::certifield::storage::CertificateStorage;
use crate::errors::LastrumError;
use crate::network::peer::{Peer, PeerManager};
use crate::network::protocol::{Message, MessageType};
use crate::network::consensus::ConsensusManager;

/// Status de sincronização do ledger
#[derive(Debug, Clone, PartialEq)]
pub enum SyncStatus {
    /// Sincronização em progresso
    InProgress,
    /// Sincronização concluída
    Complete,
    /// Erro durante a sincronização
    Error(String),
}

/// Gerenciador de sincronização do ledger
pub struct LedgerSyncManager {
    /// Armazenamento local de certificados
    storage: CertificateStorage,
    /// Gerenciador de peers
    peer_manager: PeerManager,
    /// Gerenciador de consenso
    consensus_manager: ConsensusManager,
    /// Estado atual da sincronização
    sync_state: SyncStatus,
    /// Mapa de certificados pendentes para sincronização
    pending_certificates: HashMap<String, Certificate>,
}

impl LedgerSyncManager {
    /// Cria um novo gerenciador de sincronização
    pub fn new(peer_manager: PeerManager, consensus_manager: ConsensusManager) -> Result<Self, LastrumError> {
        let storage = CertificateStorage::new()?;
        
        Ok(Self {
            storage,
            peer_manager,
            consensus_manager,
            sync_state: SyncStatus::Complete,
            pending_certificates: HashMap::new(),
        })
    }
    
    /// Inicia o processo de sincronização
    pub fn start_sync(&mut self) -> Result<(), LastrumError> {
        // Marca o início da sincronização
        self.sync_state = SyncStatus::InProgress;
        
        // Em uma implementação real, isso enviaria mensagens de solicitação
        // de certificados para os peers conhecidos
        log::info!("Iniciando sincronização do ledger com {} peers", 
            self.peer_manager.get_peers().len());
        
        // Solicita a lista de certificados de cada peer
        self.request_certificates_from_peers()?;
        
        Ok(())
    }
    
    /// Solicita a lista de certificados dos peers
    fn request_certificates_from_peers(&self) -> Result<(), LastrumError> {
        for peer in self.peer_manager.get_peers() {
            log::info!("Solicitando certificados do peer: {}", peer.name);
            
            // Em uma implementação real, isso enviaria uma mensagem
            // ao peer solicitando sua lista de certificados
        }
        
        Ok(())
    }
    
    /// Processa uma nova mensagem recebida de um peer
    pub fn process_message(&mut self, message: Message) -> Result<(), LastrumError> {
        match message.msg_type {
            MessageType::Certificate => {
                // Extrai o certificado da mensagem
                if let Some(payload) = message.payload {
                    let certificate: Certificate = serde_json::from_str(&payload)
                        .map_err(|e| LastrumError::DeserializationError(e.to_string()))?;
                    
                    // Adiciona à lista de certificados pendentes
                    self.pending_certificates.insert(certificate.id.clone(), certificate.clone());
                    
                    // Inicia o processo de validação
                    self.validate_pending_certificates()?;
                }
            },
            _ => {
                // Outros tipos de mensagens não são relevantes para a sincronização
                log::debug!("Mensagem não relacionada à sincronização: {:?}", message.msg_type);
            }
        }
        
        Ok(())
    }
    
    /// Valida os certificados pendentes através do consenso
    fn validate_pending_certificates(&mut self) -> Result<(), LastrumError> {
        let mut validated_certificates = Vec::new();
        
        // Itera sobre todos os certificados pendentes
        for (id, certificate) in &self.pending_certificates {
            // Valida o certificado através do consenso
            match self.consensus_manager.validate_certificate(certificate)? {
                crate::network::consensus::ConsensusResult::Approved => {
                    log::info!("Certificado aprovado por consenso: {}", id);
                    
                    // Adiciona à lista de certificados validados
                    validated_certificates.push(id.clone());
                    
                    // Armazena no ledger local
                    self.storage.store(certificate)?;
                },
                crate::network::consensus::ConsensusResult::Rejected(reason) => {
                    log::warn!("Certificado rejeitado pelo consenso: {}. Motivo: {}", id, reason);
                    
                    // Remove da lista de pendentes
                    validated_certificates.push(id.clone());
                },
                crate::network::consensus::ConsensusResult::Error(error) => {
                    log::error!("Erro durante validação do certificado {}: {}", id, error);
                }
            }
        }
        
        // Remove os certificados já processados da lista de pendentes
        for id in validated_certificates {
            self.pending_certificates.remove(&id);
        }
        
        // Se não houver mais certificados pendentes, marca a sincronização como concluída
        if self.pending_certificates.is_empty() {
            self.sync_state = SyncStatus::Complete;
            log::info!("Sincronização do ledger concluída");
        }
        
        Ok(())
    }
    
    /// Propaga um novo certificado para a rede
    pub fn propagate_certificate(&mut self, certificate: &Certificate) -> Result<(), LastrumError> {
        // Primeiro, valida o certificado localmente
        match self.consensus_manager.validate_certificate(certificate)? {
            crate::network::consensus::ConsensusResult::Approved => {
                // Em uma implementação real, isso enviaria o certificado para os peers
                log::info!("Propagando certificado para {} peers", 
                    self.peer_manager.get_peers().len());
                
                // Cria a mensagem de broadcast
                let payload = serde_json::to_string(certificate)
                    .map_err(|e| LastrumError::SerializationError(e.to_string()))?;
                
                let _message = Message {
                    msg_type: MessageType::Certificate,
                    sender: format!("self"), // Em uma implementação real, seria o ID do nó atual
                    recipient: None, // Broadcast para todos os peers
                    payload: Some(payload),
                    timestamp: chrono::Utc::now(),
                };
                
                // Simula o envio para todos os peers
                for peer in self.peer_manager.get_peers() {
                    log::debug!("Enviando certificado para peer: {}", peer.name);
                    // Em uma implementação real, isso enviaria a mensagem ao peer
                }
                
                Ok(())
            },
            crate::network::consensus::ConsensusResult::Rejected(reason) => {
                Err(LastrumError::ValidationError(
                    format!("Certificado rejeitado pelo consenso local: {}", reason)
                ))
            },
            crate::network::consensus::ConsensusResult::Error(error) => {
                Err(LastrumError::ValidationError(
                    format!("Erro durante validação do certificado: {}", error)
                ))
            }
        }
    }
    
    /// Obtém o status atual da sincronização
    pub fn get_sync_status(&self) -> &SyncStatus {
        &self.sync_state
    }
}